#![allow(unused)]
#![allow(unused_variables)]

use std::ops::Range;

use crate::prelude::*;

pub struct NextDerivationBasedOnProfileIndexAnalyzer {
    local_offsets: HashMap<UnquantifiedUnindexDerivationRequest, usize>,
    profile_snapshot: Profile,
}

impl NextDerivationBasedOnProfileIndexAnalyzer {
    pub fn next(&self, unindexed_request: UnquantifiedUnindexDerivationRequest) -> HDPathValue {
        todo!()
    }
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeriveMore {
    WithKnownStartIndex {
        with_start_index: QuantifiedDerivationRequestWithStartIndex,
        number_of_instances_needed_to_fully_satisfy_request: Option<usize>,
    },
    WithoutKnownLastIndex(QuantifiedUnindexDerivationRequest),
}
impl DeriveMore {
    pub fn requires_profile_index_assigner(&self) -> bool {
        match self {
            Self::WithKnownStartIndex { .. } => false,
            Self::WithoutKnownLastIndex(_) => true,
        }
    }
    /// `None` for `WithoutKnownLastIndex`, only `Some` for `WithKnownStartIndex`
    ///  where `if_partial_how_many_to_use_directly` is `Some`
    pub fn number_of_instances_needed_to_fully_satisfy_request(&self) -> Option<usize> {
        match self {
            Self::WithKnownStartIndex {
                number_of_instances_needed_to_fully_satisfy_request,
                ..
            } => *number_of_instances_needed_to_fully_satisfy_request,
            Self::WithoutKnownLastIndex(_) => None,
        }
    }
    pub fn unquantified(&self) -> UnquantifiedUnindexDerivationRequest {
        match self {
            Self::WithKnownStartIndex {
                with_start_index, ..
            } => with_start_index.clone().into(),
            Self::WithoutKnownLastIndex(request) => request.clone().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NewlyDerived {
    key: UnquantifiedUnindexDerivationRequest,
    /// never empty
    to_cache: FactorInstances,
    /// can be empty
    pub to_use_directly: FactorInstances,
}
impl NewlyDerived {
    pub fn cache_all(key: UnquantifiedUnindexDerivationRequest, to_cache: FactorInstances) -> Self {
        Self::new(key, to_cache, FactorInstances::default())
    }

    /// # Panics if `to_cache` or to `to_use_directly` is empty.
    pub fn some_to_use_directly(
        key: UnquantifiedUnindexDerivationRequest,
        to_cache: FactorInstances,
        to_use_directly: FactorInstances,
    ) -> Self {
        assert!(!to_use_directly.is_empty());
        Self::new(key, to_cache, to_use_directly)
    }
    /// # Panics
    /// Panics if `to_cache` is empty.
    /// Also panics if any FactorInstances does not match the key.
    fn new(
        key: UnquantifiedUnindexDerivationRequest,
        to_cache: FactorInstances,
        to_use_directly: FactorInstances,
    ) -> Self {
        assert!(to_cache
            .factor_instances()
            .iter()
            .all(|factor_instance| { factor_instance.satisfies(key.clone()) }));

        assert!(to_use_directly
            .factor_instances()
            .iter()
            .all(|factor_instance| { factor_instance.satisfies(key.clone()) }));

        Self {
            key,
            to_cache,
            to_use_directly,
        }
    }
    pub fn key_value_for_cache(&self) -> (UnquantifiedUnindexDerivationRequest, FactorInstances) {
        (self.key.clone(), self.to_cache.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DerivationRequestWithRange {
    pub factor_source_id: FactorSourceIDFromHash,
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_kind: CAP26KeyKind,
    pub key_space: KeySpace,
    pub range: Range<HDPathValue>,
}
impl HDPathComponent {
    pub fn with_base_index_in_keyspace(base_index: u32, key_space: KeySpace) -> Self {
        match key_space {
            KeySpace::Securified => Self::securifying_base_index(base_index),
            KeySpace::Unsecurified => Self::unsecurified_hardening_base_index(base_index),
        }
    }
}
impl DerivationRequestWithRange {
    pub fn derivation_paths(&self) -> IndexSet<DerivationPath> {
        let mut paths = IndexSet::<DerivationPath>::new();
        for i in self.range.clone() {
            paths.insert(DerivationPath::new(
                self.network_id,
                self.entity_kind,
                self.key_kind,
                HDPathComponent::with_base_index_in_keyspace(i, self.key_space),
            ));
        }
        paths
    }
}

/// ==================
/// *** Public API ***
/// ==================
impl FactorInstancesProvider {
    async fn derive_more(&self, requests: IndexSet<DeriveMore>) -> Result<IndexSet<NewlyDerived>> {
        if requests.iter().any(|x| x.requires_profile_index_assigner())
            && self
                .next_derivation_based_on_profile_index_analyzer
                .is_none()
        {
            return Err(CommonError::ProfileIndexAssignerNotPresent);
        }
        let with_proto_ranges = requests
            .clone()
            .into_iter()
            .map(|x| match x {
                DeriveMore::WithKnownStartIndex {
                    with_start_index, ..
                } => with_start_index,
                DeriveMore::WithoutKnownLastIndex(ref partial) => {
                    let next_index_assigner = self
                        .next_derivation_based_on_profile_index_analyzer
                        .as_ref()
                        .expect("should have been checked before");
                    let next = next_index_assigner.next(partial.clone().into());
                    QuantifiedDerivationRequestWithStartIndex::from((partial.clone(), next))
                }
            })
            .collect::<IndexSet<QuantifiedDerivationRequestWithStartIndex>>();

        let with_ranges = with_proto_ranges
            .into_iter()
            .map(|x| DerivationRequestWithRange {
                factor_source_id: x.factor_source_id,
                network_id: x.network_id,
                entity_kind: x.entity_kind,
                key_kind: x.key_kind,
                key_space: x.key_space,
                range: x.start_base_index..(x.start_base_index + x.quantity as u32),
            })
            .collect::<IndexSet<_>>();

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

impl NextDerivationBasedOnProfileIndexAnalyzer {
    pub fn new(profile_snapshot: Profile) -> Self {
        Self {
            profile_snapshot,
            local_offsets: HashMap::new(),
        }
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
            .map(NextDerivationBasedOnProfileIndexAnalyzer::new);

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
