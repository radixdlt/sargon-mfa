#![allow(unused)]
#![allow(unused_variables)]

use crate::prelude::*;

/// A provider of FactorInstances, reading them from the cache if present,
/// else if missing derives many instances in Abundance and caches the
/// not requested ones, and returns one matching the requested ones.
pub struct FactorInstancesProvider {
    /// The purpose of the requested FactorInstances.
    purpose: FactorInstancesRequestPurpose,

    /// If no cache present, a new one is created and will be filled.
    cache: Arc<PreDerivedKeysCache>,

    /// If we did not find any cached keys at all, use the next index
    /// analyser to get the "next index" - which uses the Profile
    /// if present.
    next_index_assigner: NextIndexAssignerWithEphemeralLocalOffsets,

    /// GUI hook used for KeysCollector if we need to derive more
    /// factor instances.
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
}

/// ==================
/// *** CTOR ***
/// ==================

impl FactorInstancesProvider {
    pub fn new(
        purpose: FactorInstancesRequestPurpose,
        maybe_cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        maybe_profile_snapshot: impl Into<Option<Profile>>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        let maybe_cache = maybe_cache.into();

        /// Important we create `NextIndexAssignerWithEphemeralLocalOffsets`
        /// inside this ctor and that we do not pass it as a parameter, since
        /// it cannot be reused, since it has ephemeral local offsets.
        let next_index_assigner =
            NextIndexAssignerWithEphemeralLocalOffsets::new(maybe_profile_snapshot);

        let cache = maybe_cache.unwrap_or_else(|| Arc::new(PreDerivedKeysCache::default()));

        Self {
            purpose,
            cache,
            next_index_assigner,
            derivation_interactors,
        }
    }
}

/// ==================
/// *** Private API ***
/// ==================
impl FactorInstancesProvider {
    async fn derive_more(&self, requests: IndexSet<DeriveMore>) -> Result<IndexSet<NewlyDerived>> {
        let with_ranges = requests
            .clone()
            .into_iter()
            .map(|x| match x {
                DeriveMore::WithKnownStartIndex {
                    with_start_index, ..
                } => with_start_index,
                DeriveMore::WithoutKnownLastIndex(ref partial) => {
                    let next = self.next_index_assigner.next(partial.clone().into());
                    DerivationRequestWithRange::from((partial.clone(), next))
                }
            })
            .collect::<IndexSet<DerivationRequestWithRange>>();

        let mut derivation_paths = with_ranges
            .into_iter()
            .into_group_map_by(|x| x.factor_source_id)
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    v.iter()
                        .flat_map(|x| x.derivation_paths())
                        .collect::<IndexSet<DerivationPath>>(),
                )
            })
            .collect::<IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>>();

        let keys_collector = KeysCollector::new(
            self.purpose.factor_sources(),
            derivation_paths,
            self.derivation_interactors.clone(),
        )?;

        let derivation_outcome = keys_collector.collect_keys().await;

        let factor_instances = derivation_outcome.all_factors();

        let out = factor_instances
            .into_iter()
            .into_group_map_by(|x| UnquantifiedUnindexDerivationRequest::from(x.clone()))
            .into_iter()
            .map(|(k, v)| {
                let original = requests.iter().find(|x| x.unquantified() == k).unwrap();
                if let Some(number_of_instances_needed_to_fully_satisfy_request) =
                    original.number_of_instances_needed_to_fully_satisfy_request()
                {
                    let (to_use_directly, to_cache) =
                        v.split_at(number_of_instances_needed_to_fully_satisfy_request);
                    NewlyDerived::some_to_use_directly(
                        k,
                        FactorInstances::from(to_cache),
                        FactorInstances::from(to_use_directly),
                    )
                } else {
                    NewlyDerived::cache_all(k, v.into_iter().collect())
                }
            })
            .collect::<IndexSet<NewlyDerived>>();

        Ok(out)
    }
}

/// ==================
/// *** Public API ***
/// ==================
impl FactorInstancesProvider {
    /// Does not return ALL derived FactorInstances, but only those that are
    /// related to the purpose of the request.
    ///
    /// Might derive MORE than requested, those will be put into the cache.
    pub async fn get_factor_instances_outcome(self) -> Result<FactorInstances> {
        let factor_sources = self.purpose.factor_sources();

        // Form requests untied to any FactorSources
        let unfactored = self.purpose.requests();

        // Form requests tied to FactorSources, but without indices, unquantified
        let unquantified = unfactored.for_each_factor_source(factor_sources);

        let quantity = self.purpose.quantity();
        let requested = unquantified
            .into_iter()
            .map(|x| QuantifiedUnindexDerivationRequest::quantifying(x, quantity))
            .collect::<QuantifiedUnindexDerivationRequests>();

        // Form quantified requests
        // Important we load from cache with requests without indices, since the cache
        // should know which are the next free indices to fulfill the requests.
        let take_from_cache_outcome = self.cache.take(&requested)?;

        let mut derive_more_requests = IndexSet::<DeriveMore>::new();
        let mut satisfied_by_cache = IndexSet::<HierarchicalDeterministicFactorInstance>::new();

        for outcome in take_from_cache_outcome.outcomes().into_iter() {
            match outcome.action() {
                Action::FullySatisfiedWithSpare(factor_instances) => {
                    satisfied_by_cache.extend(factor_instances);
                }
                Action::FullySatisfiedWithoutSpare(factor_instances, with_start_index) => {
                    satisfied_by_cache.extend(factor_instances);

                    derive_more_requests.insert(DeriveMore::WithKnownStartIndex {
                        with_start_index,
                        number_of_instances_needed_to_fully_satisfy_request: None,
                    });
                }
                Action::PartiallySatisfied {
                    partial_from_cache,
                    derive_more,
                    number_of_instances_needed_to_fully_satisfy_request,
                } => {
                    satisfied_by_cache.extend(partial_from_cache);
                    derive_more_requests.insert(DeriveMore::WithKnownStartIndex {
                        with_start_index: derive_more,
                        number_of_instances_needed_to_fully_satisfy_request: Some(
                            number_of_instances_needed_to_fully_satisfy_request,
                        ),
                    });
                }
                Action::CacheIsEmpty => {
                    derive_more_requests.insert(DeriveMore::WithoutKnownLastIndex(outcome.request));
                }
            }
        }

        let mut factor_instances_to_use_directly = satisfied_by_cache;

        if !derive_more_requests.is_empty() {
            let mut newly_derived_to_be_used_directly =
                IndexSet::<HierarchicalDeterministicFactorInstance>::new();

            let newly_derived = self.derive_more(derive_more_requests).await?;

            for _newly_derived in newly_derived.into_iter() {
                let (key, to_cache) = _newly_derived.key_value_for_cache();
                self.cache.put(key, to_cache)?;

                newly_derived_to_be_used_directly.extend(_newly_derived.to_use_directly);
            }

            factor_instances_to_use_directly.extend(newly_derived_to_be_used_directly);
        }

        Ok(FactorInstances::from(factor_instances_to_use_directly))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::{Borrow, BorrowMut},
        sync::RwLockReadGuard,
    };

    use super::*;

    struct SargonOS {
        cache: Arc<RwLock<PreDerivedKeysCache>>,
        gateway: RwLock<TestGateway>,
        profile: RwLock<Profile>,
        interactors: Arc<dyn KeysDerivationInteractors>,
    }

    impl SargonOS {
        pub fn profile_snapshot(&self) -> Profile {
            self.profile.try_read().unwrap().clone()
        }
        pub fn new() -> Self {
            let interactors: Arc<dyn KeysDerivationInteractors> =
                Arc::new(TestDerivationInteractors::default());
            Self {
                cache: Arc::new(RwLock::new(PreDerivedKeysCache::default())),
                gateway: RwLock::new(TestGateway::default()),
                profile: RwLock::new(Profile::default()),
                interactors,
            }
        }

        async fn add_factor_source(&self, factor_source: HDFactorSource) -> Result<()> {
            // let interactors: Arc<dyn KeysDerivationInteractors> =
            //     Arc::new(TestDerivationInteractors::default());

            // let cache: Arc<PreDerivedKeysCache> = Arc::new(self.cache.try_write().unwrap().clone());

            // let factor_instances_provider =
            //     FactorInstancesProvider::pre_derive_instance_for_new_factor_source(
            //         &factor_source,
            //         cache,
            //         self.profile_snapshot(),
            //         interactors,
            //     );

            // factor_instances_provider.get_factor_instances().await?;

            Ok(())
        }
    }

    #[actix_rt::test]
    async fn test() {
        let os = SargonOS::new();
        // assert_eq!(os.profile_snapshot().factor_sources.len(), 0);
        // os.add_factor_source(HDFactorSource::sample())
        //     .await
        //     .unwrap();
        // assert_eq!(os.profile_snapshot().factor_sources.len(), 1);
    }
}
