use core::num;
use std::sync::RwLock;

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

// ===== ********** =====
// ===== PRIVATE API =====
// ===== ********** =====
impl FactorInstanceProvider {
    async fn derive_new_and_fill_cache<'p>(
        &self,
        profile: &'p Profile,
        unfulfillable_requests: &UnfulfillableRequests,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<()> {
        let factor_sources = profile.factor_sources.clone();
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

        let mut derivation_paths =
            IndexMap::<FactorSourceIDFromHash, IndexSet<DerivationPath>>::new();

        let mut add = |key: FactorSourceIDFromHash, path: DerivationPath| {
            if let Some(paths) = derivation_paths.get_mut(&key) {
                paths.insert(path);
            } else {
                derivation_paths.insert(key, IndexSet::just(path));
            }
        };

        for unfulfillable_request_because_last in unfulfillable_requests_because_last {
            // This is VERY EASY
            let request = unfulfillable_request_because_last.request;

            let index_range = {
                let last_index = unfulfillable_request_because_last
                    .reason
                    .into_last()
                    .unwrap();
                let next_index = last_index.add_one();
                next_index..next_index.add_n(DERIVATION_INDEX_BATCH_SIZE)
            };

            for index in index_range {
                let path = DerivationPath::from((request, index));
                add(request.factor_source_id, path);
            }
        }

        for securified_space_request in securified_space_requests_because_empty {
            // This is not as easy, but not hard.
            let request = securified_space_request.request;

            let index_range = {
                let last_index: Option<HDPathComponent> = {
                    let all_securified_in_profile = profile
                        .get_securified_entities_of_kind_on_network(
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

                next_index..next_index.add_n(DERIVATION_INDEX_BATCH_SIZE)
            };

            for index in index_range {
                let path = DerivationPath::from((request, index));
                add(request.factor_source_id, path);
            }
        }

        for unsecurified_space_request in unsecurified_space_requests_because_empty {
            let request = unsecurified_space_request.request;

            let index_range = {
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

                next_index..next_index.add_n(DERIVATION_INDEX_BATCH_SIZE)
            };

            for index in index_range {
                let path = DerivationPath::from((request, index));
                add(request.factor_source_id, path);
            }
        }

        let keys_collector =
            KeysCollector::new(factor_sources, derivation_paths, derivation_interactors)?;

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
            PreDeriveKeysCacheKey,
            IndexSet<HierarchicalDeterministicFactorInstance>,
        > = IndexMap::new();

        for derived_factor in free {
            let key = PreDeriveKeysCacheKey::from(derived_factor.clone());
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
        unfulfillable: UnfulfillableRequests,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<IndexMap<DerivationRequest, HierarchicalDeterministicFactorInstance>> {
        self.derive_new_and_fill_cache(profile, &unfulfillable, derivation_interactors)
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
