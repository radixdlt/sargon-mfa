use core::num;
use std::sync::RwLock;

use derive_more::derive;
use rand::seq::index;

use crate::prelude::*;

/// A cache for pre-derived keys, saved on file and which will derive more keys
/// if needed, using UI/UX via KeysCollector.
///
/// We must implement the `FactorInstanceProvider` in a way that it can handle
/// the case where the cache does not exist, which it does not for users before
/// Radix Wallet version 2.0.
///
/// The purpose of this cache is only to speed up the process of accessing  
/// FactorInstances.
#[async_trait::async_trait]
pub trait IsPreDerivedKeysCache {
    /// Inserts the `derived` keys into the cache, notice the asymmetry of this
    /// "save" vs the `consume_next_factor_instances` ("load") - this method accepts
    /// a set of factors per request, while the `consume_next_factor_instances`
    /// returns a single factor per request.
    ///
    /// The reason is that we are deriving many keys and caching them, per request,
    /// whereas the `consume_next_factor_instances` ("load") only ever cares about
    /// the next key to be consumed.
    async fn insert(
        &self,
        derived: IndexMap<PreDeriveKeysCacheKey, IndexSet<HierarchicalDeterministicFactorInstance>>,
    ) -> Result<()>;

    /// Must be async since might need to derive more keys if we are about
    /// to use the last, thus will require usage of KeysCollector - which is async.
    /// Also typically we cache to file - which itself is async
    async fn consume_next_factor_instances(
        &self,
        requests: IndexSet<DerivationRequest>,
    ) -> Result<IndexMap<DerivationRequest, HierarchicalDeterministicFactorInstance>>;

    /// Returns `NextDerivationPeekOutcome::WouldHaveAtLeastOneFactorLeftPerFulfilledRequests`
    /// if there would be **at least on key left** after we have consumed
    /// (deleted) keys fulfilling all `requests`. Otherwise returns
    ///`NextDerivationPeekOutcome::WouldConsumeLastFactorOfRequests(last)` where `indices` is a map of the last consumed indices
    /// for each request. By index we mean Derivation Entity Index (`HDPathComponent`).
    /// If there is any problem with the cache, returns `Err`.
    ///
    /// We **must** have one key/factor left fulfilling the request, so that we can
    /// derive the next keys based on that.
    /// This prevents us from a problem:
    /// 1. Account X with address `A` is created by FactorInstance `F` with
    /// `{ factor_source: L, key_space: Unsecurified, index: 0 }`
    /// 2. User securified account `X`, and `F = { factor_source: L, key_space: Unsecurified, index: 0 }`
    /// is now "free", since it is no longer found in the Profile.
    /// 3. User tries to create account `Y` with `L` and if we would have used
    /// Profile "static analysis" it would say that `F = { factor_source: L, key_space: Unsecurified, index: 0 }`
    /// is next/available.
    /// 4. Failure! Account `Y` was never created since it would have same
    /// address `A` as account `X`, since it would have used same FactorInstance.
    /// 5. This problem is we cannot do this simple static analysis of Profile
    /// to find next index we would actually need to form derivation paths and
    /// derive the keys and check if that public key has been used to create any
    /// of the addresses in profile.
    ///
    /// Eureka! Or we just ensure to not loose track of the fact that `0` has
    /// been used, by letting the cache contains (0...N) keys and **before** `N`
    /// is consumed, we derive the next `(N+1, N+N)` keys and cache them. This
    /// way we need only derive more keys when they are needed.
    async fn peek(&self, requests: IndexSet<DerivationRequest>) -> NextDerivationPeekOutcome;
}

/// Used as a map key in `InMemoryPreDerivedKeysCache`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PreDeriveKeysCacheKey {
    factor_source_id: FactorSourceIDFromHash,
    path_without_index: DerivationPathWithoutIndex,
}
impl From<HierarchicalDeterministicFactorInstance> for PreDeriveKeysCacheKey {
    fn from(value: HierarchicalDeterministicFactorInstance) -> Self {
        Self::new(
            value.factor_source_id(),
            DerivationPathWithoutIndex::from(value),
        )
    }
}
impl PreDeriveKeysCacheKey {
    pub fn new(
        factor_source_id: FactorSourceIDFromHash,
        path_without_index: DerivationPathWithoutIndex,
    ) -> Self {
        Self {
            factor_source_id,
            path_without_index,
        }
    }
}

/// Like a `DerivationPath` but without the last path component.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DerivationPathWithoutIndex {
    network_id: NetworkID,
    entity_kind: CAP26EntityKind,
    key_kind: CAP26KeyKind,
    key_space: KeySpace,
}
impl DerivationPathWithoutIndex {
    fn new(
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        key_kind: CAP26KeyKind,
        key_space: KeySpace,
    ) -> Self {
        Self {
            network_id,
            entity_kind,
            key_kind,
            key_space,
        }
    }
}
impl From<HierarchicalDeterministicFactorInstance> for DerivationPathWithoutIndex {
    fn from(value: HierarchicalDeterministicFactorInstance) -> Self {
        Self::new(
            value.derivation_path().network_id,
            value.derivation_path().entity_kind,
            value.derivation_path().key_kind,
            value.derivation_path().index.key_space(),
        )
    }
}
impl From<DerivationPath> for DerivationPathWithoutIndex {
    fn from(value: DerivationPath) -> Self {
        Self::new(
            value.network_id,
            value.entity_kind,
            value.key_kind,
            value.index.key_space(),
        )
    }
}

impl From<(DerivationPathWithoutIndex, HDPathComponent)> for DerivationPath {
    fn from(value: (DerivationPathWithoutIndex, HDPathComponent)) -> Self {
        let (without_index, index) = value;
        assert!(index.is_in_key_space(without_index.key_space));
        Self::new(
            without_index.network_id,
            without_index.entity_kind,
            without_index.key_kind,
            index,
        )
    }
}

#[cfg(test)]
/// A simple `IsPreDerivedKeysCache` which uses in-memory cache instead of on
/// file which the live implementation will use.
#[derive(Default)]
pub struct InMemoryPreDerivedKeysCache {
    cache:
        RwLock<HashMap<PreDeriveKeysCacheKey, IndexSet<HierarchicalDeterministicFactorInstance>>>,
}

impl From<DerivationRequest> for DerivationPathWithoutIndex {
    fn from(value: DerivationRequest) -> Self {
        Self::new(
            value.network_id,
            value.entity_kind,
            value.key_kind,
            value.key_space,
        )
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Tuple {
    request: DerivationRequest,
    path: DerivationPathWithoutIndex,
}
impl Tuple {
    #[allow(unused)]
    fn cache_key(&self) -> PreDeriveKeysCacheKey {
        PreDeriveKeysCacheKey::new(self.request.factor_source_id, self.path.clone())
    }
}
#[cfg(test)]
impl InMemoryPreDerivedKeysCache {
    fn tuples(requests: IndexSet<DerivationRequest>) -> IndexSet<Tuple> {
        requests
            .clone()
            .into_iter()
            .map(|request| Tuple {
                request,
                path: DerivationPathWithoutIndex::from(request),
            })
            .collect::<IndexSet<Tuple>>()
    }

    /// Internal helper for implementing `peek`.
    /// `peek` will will this and map:
    /// 1. `Err(e)` -> `NextDerivationPeekOutcome::Failure(e)`
    /// 1. `Ok(None)` -> `NextDerivationPeekOutcome::Fulfillable`
    /// 1. `Ok(requests)` -> `NextDerivationPeekOutcome::Unfulfillable(UnfulfillableRequests::new(requests))`
    async fn try_peek(
        &self,
        requests: IndexSet<DerivationRequest>,
    ) -> Result<Option<UnfulfillableRequests>> {
        let cached = self
            .cache
            .try_read()
            .map_err(|_| CommonError::KeysCacheReadGuard)?;

        let request_and_path_tuples = InMemoryPreDerivedKeysCache::tuples(requests.clone());

        let mut unfulfillable = IndexSet::<UnfulfillableRequest>::new();
        for tuple in request_and_path_tuples.iter() {
            let request = tuple.request;
            let Some(for_key) = cached.get(&tuple.cache_key()) else {
                unfulfillable.insert(UnfulfillableRequest::empty(request));
                continue;
            };

            let factors_left = for_key.len();
            if factors_left == 0 {
                unfulfillable.insert(UnfulfillableRequest::empty(request));
            } else if factors_left == 1 {
                let last_factor = for_key.last().expect("Just checked length.");
                unfulfillable.insert(UnfulfillableRequest::last(request, last_factor));
            } else {
                // all good
                continue;
            }
        }

        if unfulfillable.is_empty() {
            Ok(None)
        } else {
            Ok(Some(UnfulfillableRequests::new(unfulfillable)))
        }
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl IsPreDerivedKeysCache for InMemoryPreDerivedKeysCache {
    async fn insert(
        &self,
        derived_factors_map: IndexMap<
            PreDeriveKeysCacheKey,
            IndexSet<HierarchicalDeterministicFactorInstance>,
        >,
    ) -> Result<()> {
        for (key, values) in derived_factors_map.iter() {
            for value in values {
                assert_eq!(
                    value.factor_source_id(),
                    key.factor_source_id,
                    "Discrepancy! FactorSourceID mismatch, this is a developer error."
                );
            }
        }

        let mut write_guard = self
            .cache
            .try_write()
            .map_err(|_| CommonError::KeysCacheWriteGuard)?;

        for (key, derived_factors) in derived_factors_map {
            if let Some(existing_factors) = write_guard.get_mut(&key) {
                existing_factors.extend(derived_factors);
            } else {
                write_guard.insert(key, derived_factors);
            }
        }
        drop(write_guard);

        Ok(())
    }

    async fn consume_next_factor_instances(
        &self,
        requests: IndexSet<DerivationRequest>,
    ) -> Result<IndexMap<DerivationRequest, HierarchicalDeterministicFactorInstance>> {
        let mut cached = self
            .cache
            .try_write()
            .map_err(|_| CommonError::KeysCacheWriteGuard)?;

        let mut instances_read_from_cache =
            IndexMap::<DerivationRequest, HierarchicalDeterministicFactorInstance>::new();

        let request_and_path_tuples = InMemoryPreDerivedKeysCache::tuples(requests.clone());

        for tuple in request_and_path_tuples {
            let for_key = cached
                .get_mut(&tuple.cache_key())
                .ok_or(CommonError::KeysCacheUnknownKey)?;
            let read_from_cache = for_key
                .first()
                .ok_or(CommonError::KeysCacheEmptyForKey)?
                .clone();
            assert!(
                read_from_cache.matches(&tuple.request),
                "incorrect implementation"
            );
            for_key.shift_remove(&read_from_cache);
            instances_read_from_cache.insert(tuple.request, read_from_cache);
        }

        Ok(instances_read_from_cache)
    }

    async fn peek(&self, requests: IndexSet<DerivationRequest>) -> NextDerivationPeekOutcome {
        let outcome = self.try_peek(requests).await;
        match outcome {
            Ok(None) => NextDerivationPeekOutcome::Fulfillable,
            Ok(Some(unfulfillable)) => NextDerivationPeekOutcome::Unfulfillable(unfulfillable),
            Err(e) => NextDerivationPeekOutcome::Failure(e),
        }
    }
}

/// A coordinator of sorts between `PreDeriveKeysCache`, `KeysCollector` and Gateway,
/// used to provide `HierarchicalDeterministicFactorInstance`s for a given `DerivationRequest`.
///
/// This FactorInstanceProvider is used when creating new entities or when
/// securing existing entities.  It is used to provide the "next"
/// `HierarchicalDeterministicFactorInstance` in both cases for the given request.
pub struct FactorInstanceProvider {
    pub gateway: Arc<dyn Gateway>,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    cache: Arc<dyn IsPreDerivedKeysCache>,
}
impl FactorInstanceProvider {
    pub fn new(
        gateway: Arc<dyn Gateway>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        cache: Arc<dyn IsPreDerivedKeysCache>,
    ) -> Self {
        Self {
            gateway,
            derivation_interactors,
            cache,
        }
    }
}

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

impl FactorInstanceProvider {
    async fn derive_new_and_fill_cache<'p>(
        &self,
        profile: &'p Profile,
        unfulfillable_requests: &UnfulfillableRequests,
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

        let keys_collector = KeysCollector::new(
            factor_sources,
            derivation_paths,
            self.derivation_interactors.clone(),
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
    ) -> Result<IndexMap<DerivationRequest, HierarchicalDeterministicFactorInstance>> {
        self.derive_new_and_fill_cache(profile, &unfulfillable)
            .await?;
        let requests = fulfillable
            .union(&unfulfillable.requests())
            .cloned()
            .collect::<IndexSet<_>>();
        self.cache.consume_next_factor_instances(requests).await
    }
}

impl FactorInstanceProvider {
    pub async fn provide_genesis_factor_for<'p>(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        entity_kind: CAP26EntityKind,
        network_id: NetworkID,
        profile: &'p Profile,
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
            .provide_factor_instances(profile, IndexSet::just(request))
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
                self.fulfill(profile, fulfillable, unfulfillable).await
            }
        }
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

        let interactors = Arc::new(TestDerivationInteractors::default());
        let gateway = Arc::new(TestGateway::default());

        let factor_instance_provider = FactorInstanceProvider::new(
            gateway.clone(),
            interactors,
            Arc::new(InMemoryPreDerivedKeysCache::default()),
        );

        let b_sec = factor_instance_provider
            .securify(b, &matrix, &mut profile)
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
            .securify(a, &matrix, &mut profile)
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
