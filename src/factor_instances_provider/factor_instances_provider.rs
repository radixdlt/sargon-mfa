#![allow(unused)]
#![allow(unused_variables)]

use crate::prelude::*;

pub struct NextDerivationBasedOnProfileIndexAnalyzer {
    profile_snapshot: Profile,
}

impl NextDerivationBasedOnProfileIndexAnalyzer {
    pub fn next(
        &self,
        unindexed_requests: UnquantifiedUnindexDerivationRequests,
    ) -> FullDerivationRequests {
        todo!()
    }
}

pub struct FactorInstancesRequestOutcome {
    /// The FactorInstances that was requested.
    pub requested: FactorInstances,

    /// If we did derive FactorInstances past those requested and put into the cache.
    pub did_derive_past_requested: bool,
}

/// With known start index and quantity
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QuantifiedDerivationRequestWithStartIndex {
    pub factor_source_id: FactorSourceIDFromHash,
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_kind: CAP26KeyKind,
    pub key_space: KeySpace,
    pub quantity: usize,
    pub start_base_index: HDPathValue,
}
impl QuantifiedDerivationRequestWithStartIndex {
    fn new(
        factor_source_id: FactorSourceIDFromHash,
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        key_kind: CAP26KeyKind,
        key_space: KeySpace,
        quantity: usize,
        start_base_index: HDPathValue,
    ) -> Self {
        Self {
            factor_source_id,
            network_id,
            entity_kind,
            key_kind,
            key_space,
            quantity,
            start_base_index,
        }
    }
}
impl From<(QuantifiedUnindexDerivationRequest, HDPathValue)>
    for QuantifiedDerivationRequestWithStartIndex
{
    fn from(value: (QuantifiedUnindexDerivationRequest, HDPathValue)) -> Self {
        let (q, i) = value;
        Self::new(
            q.factor_source_id,
            q.network_id,
            q.entity_kind,
            q.key_kind,
            q.key_space,
            q.requested_quantity(),
            i,
        )
    }
}

/// ==================
/// *** Public API ***
/// ==================
impl FactorInstancesProvider {
    async fn derive_more(
        &self,
        // unsatisfied: Option<UnsatisfiedUnindexedDerivationRequests>,
        // initially_requested: UnindexDerivationRequests,
    ) -> Result<FactorInstances> {
        todo!()
    }

    /// Does not return ALL derived FactorInstances, but only those that are
    /// related to the purpose of the request.
    ///
    /// Might derive MORE than requested, those will be put into the cache.
    pub async fn get_factor_instances_outcome(self) -> Result<FactorInstancesRequestOutcome> {
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
            let aggregatable = outcome.aggregatable();

            // might be empty
            satisfied_by_cache.extend(aggregatable.loaded.factor_instances());

            if let Some(derive_more) = aggregatable.derive_more {
                derive_more_requests.insert(derive_more);
            }
        }

        todo!()
    }
}

// impl FactorInstancesFromCache {
//     pub fn should_derive_more(&self) -> bool {
//         self.
//     }
// }

impl NextDerivationBasedOnProfileIndexAnalyzer {
    pub fn new(profile_snapshot: Profile) -> Self {
        Self { profile_snapshot }
    }
}
pub struct FactorInstancesProvider {
    purpose: FactorInstancesRequestPurpose,

    /// If no cache present, a new one is created and will be filled.
    cache: Arc<PreDerivedKeysCache>,
    next_derivation_based_on_profile_index_analyzer:
        Option<NextDerivationBasedOnProfileIndexAnalyzer>,

    /// GUI hook
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
}

/// ==================
/// *** CTOR ***
/// ==================

impl FactorInstancesProvider {
    fn new(
        purpose: FactorInstancesRequestPurpose,
        maybe_cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        maybe_profile_snapshot: impl Into<Option<Profile>>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        let maybe_cache = maybe_cache.into();
        let next_derivation_based_on_profile_index_analyzer = maybe_profile_snapshot
            .into()
            .map(|p| NextDerivationBasedOnProfileIndexAnalyzer::new(p));

        let cache = maybe_cache.unwrap_or_else(|| Arc::new(PreDerivedKeysCache::default()));

        Self {
            purpose,
            cache,
            next_derivation_based_on_profile_index_analyzer,
            derivation_interactors,
        }
    }
}

/// ==================
/// *** Purposes ***
/// ==================
impl FactorInstancesProvider {
    pub fn oars(
        factor_sources: &FactorSources,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        Self::new(
            FactorInstancesRequestPurpose::OARS {
                factor_sources: factor_sources.clone(),
            },
            None,
            None,
            derivation_interactors,
        )
    }

    pub fn mars(
        factor_source: &HDFactorSource,
        cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        profile_snapshot: Profile,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        Self::new(
            FactorInstancesRequestPurpose::MARS {
                factor_source: factor_source.clone(),
                network_id: profile_snapshot.current_network(),
            },
            cache,
            profile_snapshot,
            derivation_interactors,
        )
    }

    pub fn pre_derive_instance_for_new_factor_source(
        factor_source: &HDFactorSource,
        cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        profile_snapshot: Profile,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        Self::new(
            FactorInstancesRequestPurpose::PreDeriveInstancesForNewFactorSource {
                factor_source: factor_source.clone(),
            },
            cache,
            profile_snapshot,
            derivation_interactors,
        )
    }

    pub fn new_virtual_unsecurified_account(
        network_id: NetworkID,
        factor_source: &HDFactorSource,
        cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        profile_snapshot: Profile,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        Self::new(
            FactorInstancesRequestPurpose::NewVirtualUnsecurifiedAccount {
                network_id,
                factor_source: factor_source.clone(),
            },
            cache,
            profile_snapshot,
            derivation_interactors,
        )
    }

    /// Securify unsecurified Accounts
    ///
    /// # Panics
    /// Panics if `UnsecurifiedEntity` is not an account
    /// or if it is not present in `profile_snapshot`.
    pub fn securify_unsecurified_accounts(
        unsecurified_accounts: UnsecurifiedAccounts,
        matrix_of_factor_sources: MatrixOfFactorSources,
        cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        profile_snapshot: Profile,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        assert!(profile_snapshot.contains_accounts(unsecurified_accounts.clone()));

        Self::new(
            FactorInstancesRequestPurpose::SecurifyUnsecurifiedAccounts {
                unsecurified_accounts,
                matrix_of_factor_sources,
            },
            cache,
            profile_snapshot,
            derivation_interactors,
        )
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
