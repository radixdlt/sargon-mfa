use core::num;
use std::sync::RwLock;

use derive_more::derive;

use crate::prelude::*;

/// The reason why a request is unfulfillable, and if the reason is that the
/// last factor would be consumed, the value of that last factor is included,
/// to act as the range.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum UnfulfillableRequestReason {
    /// Users before Radix Wallet 2.0 does not have any cache.
    /// This will be kick of the cumbersome process of analyzing the Profile
    /// and deriving a broad range of keys to find out the "last used" key per
    /// factor source, and then use that to derive the next batch of keys and
    /// cache them.
    Empty,

    /// The request would consume the last factor, the `HDPathComponent` is
    /// the value of this last factor, which we can use as a base for the
    /// next index range to derive keys for, i.e. we will derive keys in the range
    /// `(last_index + 1, last_index + N)` where `N` is the batch size (e.g. 50).
    Last(HDPathComponent),
}

/// A request that cannot be fulfilled, and the reason why.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct UnfulfillableRequest {
    /// The request which cannot be fulfilled.
    request: DerivationRequest,

    /// The reason why `request` could not be fulfilled.
    reason: UnfulfillableRequestReason,
}
impl UnfulfillableRequest {
    pub fn empty(request: DerivationRequest) -> Self {
        Self {
            request,
            reason: UnfulfillableRequestReason::Empty,
        }
    }

    /// # Panics
    /// Panics if `last_factor` does not share same parameters as `request`
    pub fn last(
        request: DerivationRequest,
        last_factor: &HierarchicalDeterministicFactorInstance,
    ) -> Self {
        assert!(
            last_factor.matches(&request),
            "last_factor must match request"
        );
        Self {
            request,
            reason: UnfulfillableRequestReason::Last(last_factor.derivation_path().index),
        }
    }
}

impl HierarchicalDeterministicFactorInstance {
    fn matches(&self, request: &DerivationRequest) -> bool {
        self.factor_source_id() == request.factor_source_id
            && self.derivation_path().matches(request)
    }
}
impl DerivationPath {
    fn matches(&self, request: &DerivationRequest) -> bool {
        self.network_id == request.network_id
            && self.entity_kind == request.entity_kind
            && self.key_kind == request.key_kind
            && self.index.key_space() == request.key_space
    }
}

/// A non-empty collection of unfulfillable requests
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct UnfulfillableRequests {
    /// A non-empty collection of unfulfillable requests
    unfulfillable: Vec<UnfulfillableRequest>, // we want `Set` but `IndexSet` is not `Hash`
}
impl UnfulfillableRequests {
    /// # Panics
    /// Panics if `unfulfillable` is empty.
    pub fn new(unfulfillable: IndexSet<UnfulfillableRequest>) -> Self {
        assert!(!unfulfillable.is_empty(), "non_empty must not be empty");
        Self {
            unfulfillable: unfulfillable.into_iter().collect(),
        }
    }
    pub fn unfulfillable(&self) -> IndexSet<UnfulfillableRequest> {
        self.unfulfillable.clone().into_iter().collect()
    }

    pub fn requests(&self) -> IndexSet<DerivationRequest> {
        self.unfulfillable
            .clone()
            .into_iter()
            .map(|ur| ur.request)
            .collect()
    }
}

/// The outcome of peeking the next derivation index for a request.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NextDerivationPeekOutcome {
    /// We failed to peek the next derivation index for the request, probably
    /// an error while reading from cache.
    Failure(CommonError),

    /// All requests would have at least one factor left after they are fulfilled.
    Fulfillable,

    /// The `IndexMap` contains the last consumed index for each request.
    ///
    /// N.B. that if some request would not consume the last factor, it will not
    /// be present in this map.
    Unfulfillable(UnfulfillableRequests),
}

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
        derived: IndexMap<DerivationRequest, IndexSet<HierarchicalDeterministicFactorInstance>>,
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

/// Like a `DerivationPath` but without the last path component. Used as a
/// HashMap key in `InMemoryPreDerivedKeysCache`.
#[derive(Clone, PartialEq, Eq, Hash)]
struct DerivationPathWithoutIndex {
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
    cache: RwLock<
        HashMap<DerivationPathWithoutIndex, IndexSet<HierarchicalDeterministicFactorInstance>>,
    >,
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
        let cached = self.cache.try_read().map_err(|_| CommonError::Failure)?;

        let request_and_path_tuples = InMemoryPreDerivedKeysCache::tuples(requests.clone());

        let mut unfulfillable = IndexSet::<UnfulfillableRequest>::new();
        for tuple in request_and_path_tuples.iter() {
            let for_key = cached.get(&tuple.path).ok_or(CommonError::Failure)?;
            let factors_left = for_key.len();
            let request = tuple.request;
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
        derived_factors: IndexMap<
            DerivationRequest,
            IndexSet<HierarchicalDeterministicFactorInstance>,
        >,
    ) -> Result<()> {
        let mut write_guard = self.cache.try_write().map_err(|_| CommonError::Failure)?;

        for (request, derived_factor) in derived_factors {
            let key = DerivationPathWithoutIndex::from(request);
            if let Some(existing_factors) = write_guard.get_mut(&key) {
                existing_factors.extend(derived_factor);
            } else {
                write_guard.insert(key, derived_factor);
            }
        }

        Ok(())
    }

    async fn consume_next_factor_instances(
        &self,
        requests: IndexSet<DerivationRequest>,
    ) -> Result<IndexMap<DerivationRequest, HierarchicalDeterministicFactorInstance>> {
        let mut cached = self.cache.try_write().map_err(|_| CommonError::Failure)?;

        let mut instances_read_from_cache =
            IndexMap::<DerivationRequest, HierarchicalDeterministicFactorInstance>::new();

        let request_and_path_tuples = InMemoryPreDerivedKeysCache::tuples(requests.clone());

        for tuple in request_and_path_tuples {
            let for_key = cached.get_mut(&tuple.path).ok_or(CommonError::Failure)?;
            let read_from_cache = for_key.pop().ok_or(CommonError::Failure)?;
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
///
/// A `DerivationRequest` is a tuple of `(KeySpace, EntityKind, KeyKind, FactorSourceID, NetworkID)`.
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

impl FactorInstanceProvider {
    async fn derive_new_and_fill_cache<'p>(
        &self,
        profile: &'p Profile,
        unfulfillable_requests: &UnfulfillableRequests,
    ) -> Result<()> {
        let factor_sources = profile.factor_sources.clone();
        let unfulfillable_requests = unfulfillable_requests.unfulfillable();

        let derivation_paths = IndexMap::<FactorSourceIDFromHash, IndexSet<DerivationPath>>::new();

        for unfulfillable_request in unfulfillable_requests {
            // this is HARD. we might have to re-derive the public keys to re-discover
            // know which DerivationPath was used to derive key public keys of
            // securified accounts. The reason is that a securified account is
            // no longer referencing its "genesis" factor in `KeySpace::Unsecurified`,
            // and thus we cannot know which DerivationPath was used to derive that
            // public key.
            //
            // We might derive some kind of naive best effort guess and then use
            // gateway to see if the hash of this public key is "known". But this
            // might result in multiple "passes" of `KeysCollector`... which is
            // terrible UX.
            //
            // OR
            //
            // We might upload the derivation index of the genesis factor instance
            // for an account when it gets securified. That way we do not need to
            // try to re-derive the public keys of addresses of securified entities,
            // just to know the last used index, instead we can just read it from
            // gateway.
            //
            // ~~~ OR ~~~
            // Am I over-complicating things here? Can I supply rely on the index
            // of an account in the Profile to know the next index to use - filtered
            // adjusted of course based on the FactorSource used? This code does
            // not relate to recovery (which I've just worked on, so my mind might
            // be too used to the idea of not having access to the profile...).
            // But there might be "gaps" in indices used, but at the very least
            // we should be able to get an APPROXIMATE next index and then we can
            // "pad" with some indices before and then we will create a wide
            // range of indices to derive keys for - which SHOULD "hit" the public
            // key of the address of the securified entities. Thus we should know
            // the last index used, and we know which is the next factor.
            //
            // Right?
        }

        let keys_collector = KeysCollector::new(
            factor_sources,
            derivation_paths,
            self.derivation_interactors.clone(),
        )?;

        let derivation_outcome = keys_collector.collect_keys().await;
        let derived_factors = derivation_outcome.all_factors();

        let mut to_insert: IndexMap<
            DerivationRequest,
            IndexSet<HierarchicalDeterministicFactorInstance>,
        > = IndexMap::new();

        for derived_factor in derived_factors {
            // add `HierarchicalDeterministicFactorInstance` into `to_insert`...
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
        return self.cache.consume_next_factor_instances(requests).await;
    }
}

impl FactorInstanceProvider {
    pub async fn securify<E: IsEntity>(
        &self,
        entity: &E,
        matrix: &MatrixOfFactorSources,
        profile: &Profile,
    ) -> Result<MatrixOfFactorInstances> {
        let entity_kind = E::kind();
        let network_id = entity.address().network_id();
        let key_kind = CAP26KeyKind::TransactionSigning;

        let requests = matrix
            .clone()
            .all_factors()
            .into_iter()
            .map(|factor_source| {
                DerivationRequest::securify(
                    entity_kind,
                    key_kind,
                    factor_source.factor_source_id(),
                    network_id,
                )
            })
            .collect::<IndexSet<_>>();

        let derived_factors_map = self.provide_factor_instances(profile, requests).await?;
        let derived_factors = derived_factors_map
            .values()
            .into_iter()
            .cloned()
            .collect::<IndexSet<_>>();

        let matrix = MatrixOfFactorInstances::fulfilling_matrix_of_factor_sources_with_instances(
            derived_factors,
            matrix.clone(),
        )?;

        Ok(matrix)
    }

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
            .ok_or(CommonError::Failure)?
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
                return Err(e);
            }
            NextDerivationPeekOutcome::Fulfillable => {
                return self.cache.consume_next_factor_instances(requests).await;
            }
            NextDerivationPeekOutcome::Unfulfillable(unfulfillable) => {
                let fulfillable = requests
                    .difference(&unfulfillable.requests())
                    .cloned()
                    .collect::<IndexSet<_>>();
                return self.fulfill(profile, fulfillable, unfulfillable).await;
            }
        }
    }
}
