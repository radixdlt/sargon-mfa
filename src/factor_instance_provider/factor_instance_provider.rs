use core::num;
use std::{ops::Range, sync::RwLock};

use derive_more::derive;
use rand::seq::index;

use crate::prelude::*;

/// A coordinator of sorts between `PreDeriveKeysCache`, `KeysCollector` and Gateway,
/// used to provide `HierarchicalDeterministicFactorInstance`s for a given `DerivationRequest`.
///
/// This FactorInstanceProvider is used when creating new entities or when
/// securing existing entities.  It is used to provide the "next"
/// `HierarchicalDeterministicFactorInstance` in both cases for the given request.
pub struct FactorInstanceProvider {
    pub gateway: Arc<dyn Gateway>,
    cache: Arc<dyn IsPreDerivedKeysCache>,
}

impl FactorInstanceProvider {
    pub fn new(gateway: Arc<dyn Gateway>, cache: Arc<dyn IsPreDerivedKeysCache>) -> Self {
        Self { gateway, cache }
    }
}

// ===== ********** =====
// ===== PUBLIC API =====
// ===== ********** =====
impl FactorInstanceProvider {
    pub async fn provide_genesis_factor_for<'p>(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        entity_kind: CAP26EntityKind,
        network_id: NetworkID,
        profile: &'p Profile,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<HierarchicalDeterministicFactorInstance> {
        let key_kind = CAP26KeyKind::TransactionSigning;

        let request = DerivationRequest::new(
            KeySpace::Unsecurified,
            entity_kind,
            key_kind,
            factor_source_id,
            network_id,
        );

        let derived_factors_map = self
            .provide_factor_instances(profile, IndexSet::just(request), derivation_interactors)
            .await?;

        let derived_factor = derived_factors_map
            .into_iter()
            .next()
            .ok_or(CommonError::InstanceProviderFailedToCreateGenesisFactor)?
            .1;

        Ok(derived_factor)
    }

    pub async fn provide_factor_instances<'p>(
        &self,
        profile: &'p Profile,
        requests: IndexSet<DerivationRequest>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<IndexMap<DerivationRequest, HierarchicalDeterministicFactorInstance>> {
        let peek_outcome = self.cache.peek(requests.clone()).await;

        match peek_outcome {
            NextDerivationPeekOutcome::Failure(e) => {
                error!("Failed to peek next derivation index: {:?}", e);
                Err(e)
            }
            NextDerivationPeekOutcome::Fulfillable => {
                self.cache.consume_next_factor_instances(requests).await
            }
            NextDerivationPeekOutcome::Unfulfillable(unfulfillable) => {
                let fulfillable = requests
                    .difference(&unfulfillable.requests())
                    .cloned()
                    .collect::<IndexSet<_>>();
                self.fulfill(profile, fulfillable, unfulfillable, derivation_interactors)
                    .await
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]

pub struct NextDerivationRange {
    start_index: HDPathComponent,

    /// we will add `derivation_size` to `start_index` to get the range
    derivation_size: HDPathValue,
}
impl NextDerivationRange {
    pub fn range(&self) -> Range<HDPathComponent> {
        self.start_index..self.start_index.add_n(self.derivation_size)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NextDerivations;
impl NextDerivations {
    /// Without Profile and without Cache....
    pub fn recovery_start(
        &self,
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        factor_sources: IndexSet<HDFactorSource>,
    ) -> IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>> {
        self.recovery_with_offsets(
            network_id,
            entity_kind,
            factor_sources
                .clone()
                .into_iter()
                .map(|f| {
                    (f.factor_source_id(), {
                        if f.is_olympia() {
                            HDPathComponent::Unhardened(UnhardenedIndex::new(0))
                        } else {
                            HDPathComponent::unsecurified_hardening_base_index(0)
                        }
                    })
                })
                .collect(),
            factor_sources
                .clone()
                .into_iter()
                .map(|f| {
                    (
                        f.factor_source_id(),
                        HDPathComponent::securifying_base_index(0),
                    )
                })
                .collect(),
        )
    }

    fn recovery_with_offsets(
        &self,
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        unsecurified_key_space_offsets_by_factor_source: IndexMap<
            FactorSourceIDFromHash,
            HDPathComponent,
        >,
        securified_key_space_offsets_by_factor_source: IndexMap<
            FactorSourceIDFromHash,
            HDPathComponent,
        >,
    ) -> IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>> {
        let mut map_paths = IndexMap::<FactorSourceIDFromHash, IndexSet<DerivationPath>>::new();
        let mut extend_paths = |ranges: IndexMap<FactorSourceIDFromHash, HDPathComponent>| -> () {
            for (key, value) in ranges.into_iter() {
                let path = DerivationPath::new(
                    network_id,
                    entity_kind,
                    CAP26KeyKind::TransactionSigning,
                    value,
                );
                if let Some(mut existing) = map_paths.get_mut(&key) {
                    existing.insert(path);
                } else {
                    map_paths.insert(key, IndexSet::just(path));
                }
            }
        };

        extend_paths(unsecurified_key_space_offsets_by_factor_source);
        extend_paths(securified_key_space_offsets_by_factor_source);

        map_paths
    }

    pub fn next_paths_analyzing_profile(
        &self,
        profile: &Profile,
        unfulfillable_requests: &DerivationRequestsUnfulfillableByCache,
    ) -> IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>> {
        let ranges = self.next_ranges_analyzing_profile(profile, unfulfillable_requests);
        ranges
            .into_iter()
            .map(|(k, v)| {
                let paths = v
                    .range()
                    .into_iter()
                    .map(|c| {
                        DerivationPath::new(
                            k.request.network_id,
                            k.request.entity_kind,
                            k.request.key_kind,
                            c,
                        )
                    })
                    .collect::<IndexSet<_>>();
                let factor_source_id = k.factor_source_id();
                assert!(
                    profile
                        .factor_sources
                        .iter()
                        .any(|f| f.factor_source_id() == factor_source_id),
                    "Discrepancy unknown factor source"
                );
                (factor_source_id, paths)
            })
            .collect::<IndexMap<_, _>>()
    }

    fn next_ranges_analyzing_profile(
        &self,
        profile: &Profile,
        unfulfillable_requests: &DerivationRequestsUnfulfillableByCache,
    ) -> IndexMap<DerivationRequestUnfulfillableByCache, NextDerivationRange> {
        let unfulfillable_requests = unfulfillable_requests.unfulfillable();

        let unfulfillable_requests_because_empty = unfulfillable_requests
            .iter()
            .filter(|ur| ur.is_reason_empty())
            .cloned()
            .collect::<IndexSet<_>>();

        let unfulfillable_requests_because_last = unfulfillable_requests
            .iter()
            .filter(|ur| ur.is_reason_last())
            .cloned()
            .collect::<IndexSet<_>>();

        drop(unfulfillable_requests);

        let securified_space_requests_because_empty = unfulfillable_requests_because_empty
            .iter()
            .filter(|ur| ur.request.key_space == KeySpace::Securified)
            .cloned()
            .collect::<IndexSet<_>>();

        let unsecurified_space_requests_because_empty = unfulfillable_requests_because_empty
            .iter()
            .filter(|ur| ur.request.key_space == KeySpace::Unsecurified)
            .cloned()
            .collect::<IndexSet<_>>();

        let mut next_derivation_indices =
            IndexMap::<DerivationRequestUnfulfillableByCache, NextDerivationRange>::new();

        let mut add = |key: DerivationRequestUnfulfillableByCache, start: HDPathComponent| {
            let request = key.request;

            let Some(derivation_size) = request.derivation_size() else {
                warn!(
                    "Skipping request since it has no derivation size: {:?}",
                    request
                );
                return;
            };

            let range = NextDerivationRange {
                start_index: start,
                derivation_size: derivation_size,
            };

            assert!(
                !next_derivation_indices.contains_key(&key),
                "Discrepancy, key already exists, this is a programmer error"
            );
            next_derivation_indices.insert(key, range);
        };

        for unfulfillable_request_because_last in unfulfillable_requests_because_last {
            // This is VERY EASY
            let last_index_from_cache_or_profile = match unfulfillable_request_because_last.reason {
                DerivationRequestUnfulfillableByCacheReason::Empty => {
                    let last_index_from_profile = unfulfillable_request_because_last
                        .clone()
                        .reason
                        .into_last()
                        .unwrap();

                    last_index_from_profile
                }
                DerivationRequestUnfulfillableByCacheReason::Last(last_from_cache) => {
                    last_from_cache
                }
            };

            let next_index = last_index_from_cache_or_profile.add_one();

            add(unfulfillable_request_because_last, next_index);
        }

        for securified_space_request in securified_space_requests_because_empty {
            // This is not as easy, but not hard.
            let request = securified_space_request.request.clone();

            let last_index: Option<HDPathComponent> = {
                let all_securified_in_profile = profile.get_securified_entities_of_kind_on_network(
                    request.entity_kind,
                    request.network_id,
                );

                all_securified_in_profile
                    .into_iter()
                    .flat_map(|e: SecurifiedEntity| e.highest_derivation_path_index(&request))
                    .max()
            };

            let next_index = last_index
                .map(|l| l.add_one())
                .unwrap_or(HDPathComponent::securifying_base_index(0));

            add(securified_space_request, next_index);
        }

        for unsecurified_space_request in unsecurified_space_requests_because_empty {
            let request = unsecurified_space_request.request;

            let last_index: Option<HDPathComponent> = {
                let all_unsecurified_in_profile = profile
                    .get_unsecurified_entities_of_kind_on_network(
                        request.entity_kind,
                        request.network_id,
                    );

                all_unsecurified_in_profile
                    .into_iter()
                    .map(|u| u.factor_instance)
                    .filter(|fi| fi.matches(&request))
                    .map(|fi| fi.derivation_path().index)
                    .max()
            };

            let next_index = last_index
                .map(|l| l.add_one())
                .unwrap_or(HDPathComponent::unsecurified_hardening_base_index(0));

            add(unsecurified_space_request, next_index);
        }

        next_derivation_indices
    }
}

// ===== ********** =====
// ===== PRIVATE API =====
// ===== ********** =====
impl FactorInstanceProvider {
    async fn derive_new_and_fill_cache<'p>(
        &self,
        next_derivations: NextDerivations,
        profile: &'p Profile,
        unfulfillable_requests: &DerivationRequestsUnfulfillableByCache,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<()> {
        let derivation_paths =
            next_derivations.next_paths_analyzing_profile(profile, unfulfillable_requests);

        let keys_collector = KeysCollector::new(
            profile.factor_sources.clone(),
            derivation_paths,
            derivation_interactors,
        )?;

        let derivation_outcome = keys_collector.collect_keys().await;
        let derived_factors = derivation_outcome.all_factors();

        let public_key_hash_to_factor_map = derived_factors
            .into_iter()
            .map(|f| (f.public_key_hash(), f))
            .collect::<IndexMap<_, _>>();

        let is_known_by_gateway_map = self
            .gateway
            .query_public_key_hash_is_known(
                public_key_hash_to_factor_map
                    .keys()
                    .cloned()
                    .collect::<IndexSet<_>>(),
            )
            .await?;

        // believed to be free by gateway
        let mut free = IndexSet::<HierarchicalDeterministicFactorInstance>::new();
        for (hash, public_key) in public_key_hash_to_factor_map.into_iter() {
            let is_known_by_gateway = is_known_by_gateway_map.get(&hash).unwrap();
            if !is_known_by_gateway {
                free.insert(public_key.clone());
            }
        }

        let mut to_insert: IndexMap<
            PreDerivedKeysCacheKey,
            IndexSet<HierarchicalDeterministicFactorInstance>,
        > = IndexMap::new();

        for derived_factor in free {
            let key = PreDerivedKeysCacheKey::from(derived_factor.clone());
            if let Some(existing) = to_insert.get_mut(&key) {
                existing.insert(derived_factor);
            } else {
                to_insert.insert(key, IndexSet::just(derived_factor));
            }
        }
        self.cache.insert(to_insert).await?;

        Ok(())
    }

    async fn fulfill<'p>(
        &self,
        profile: &'p Profile,
        fulfillable: IndexSet<DerivationRequest>,
        unfulfillable: DerivationRequestsUnfulfillableByCache,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<IndexMap<DerivationRequest, HierarchicalDeterministicFactorInstance>> {
        self.derive_new_and_fill_cache(
            NextDerivations::default(),
            profile,
            &unfulfillable,
            derivation_interactors,
        )
        .await?;
        let requests = fulfillable
            .union(&unfulfillable.requests())
            .cloned()
            .collect::<IndexSet<_>>();
        self.cache.consume_next_factor_instances(requests).await
    }
}

// ===== ********** =====
// =====  HELPERS   =====
// ===== ********** =====
impl From<(DerivationRequest, HDPathComponent)> for DerivationPath {
    fn from(value: (DerivationRequest, HDPathComponent)) -> Self {
        let (request, index) = value;
        DerivationPath::new(
            request.network_id,
            request.entity_kind,
            request.key_kind,
            index,
        )
    }
}

impl MatrixOfFactorInstances {
    fn highest_derivation_path_index(
        &self,
        request: &DerivationRequest,
    ) -> Option<HDPathComponent> {
        self.all_factors()
            .into_iter()
            .filter(|f| f.matches(request))
            .map(|f| f.derivation_path().index)
            .max()
    }
}
impl SecurifiedEntityControl {
    fn highest_derivation_path_index(
        &self,
        request: &DerivationRequest,
    ) -> Option<HDPathComponent> {
        self.matrix.highest_derivation_path_index(request)
    }
}
impl SecurifiedEntity {
    fn highest_derivation_path_index(
        &self,
        request: &DerivationRequest,
    ) -> Option<HDPathComponent> {
        self.control.highest_derivation_path_index(request)
    }
}

impl From<HierarchicalDeterministicFactorInstance> for DerivationRequest {
    fn from(value: HierarchicalDeterministicFactorInstance) -> Self {
        let key_space = value.derivation_path().index.key_space();
        let key_kind = value.derivation_path().key_kind;
        let entity_kind = value.derivation_path().entity_kind;
        let network_id = value.derivation_path().network_id;
        let factor_source_id = value.factor_source_id();

        Self::new(
            key_space,
            entity_kind,
            key_kind,
            factor_source_id,
            network_id,
        )
    }
}

#[cfg(test)]
mod securify_tests {

    use super::*;

    #[actix_rt::test]
    async fn derivation_path_is_never_same_after_securified() {
        let all_factors = HDFactorSource::all();
        let a = &Account::unsecurified_mainnet(
            "A0",
            HierarchicalDeterministicFactorInstance::mainnet_tx(
                CAP26EntityKind::Account,
                HDPathComponent::unsecurified_hardening_base_index(0),
                FactorSourceIDFromHash::fs0(),
            ),
        );
        let b = &Account::unsecurified_mainnet(
            "A1",
            HierarchicalDeterministicFactorInstance::mainnet_tx(
                CAP26EntityKind::Account,
                HDPathComponent::unsecurified_hardening_base_index(1),
                FactorSourceIDFromHash::fs0(),
            ),
        );

        let mut profile = Profile::new(all_factors.clone(), [a, b], []);
        let matrix = MatrixOfFactorSources::new([fs_at(0)], 1, []);

        let gateway = Arc::new(TestGateway::default());

        let factor_instance_provider = FactorInstanceProvider::new(
            gateway.clone(),
            Arc::new(InMemoryPreDerivedKeysCache::default()),
        );

        let interactors = Arc::new(TestDerivationInteractors::default());
        let b_sec = factor_instance_provider
            .securify(b, &matrix, &mut profile, interactors.clone())
            .await
            .unwrap();

        assert_eq!(
            b_sec
                .matrix
                .all_factors()
                .clone()
                .into_iter()
                .map(|f| f.derivation_path().index)
                .collect::<HashSet<_>>()
                .len(),
            1
        );

        assert_eq!(
            b_sec
                .matrix
                .all_factors()
                .clone()
                .into_iter()
                .map(|f| f.derivation_path().index)
                .collect::<HashSet<_>>(),
            HashSet::just(HDPathComponent::securifying_base_index(0))
        );

        let a_sec = factor_instance_provider
            .securify(a, &matrix, &mut profile, interactors.clone())
            .await
            .unwrap();

        assert_eq!(
            a_sec
                .matrix
                .all_factors()
                .clone()
                .into_iter()
                .map(|f| f.derivation_path().index)
                .collect::<HashSet<_>>(),
            HashSet::just(HDPathComponent::securifying_base_index(1))
        );
    }
}
