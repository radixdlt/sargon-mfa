use core::num;
use std::{f32::consts::E, sync::RwLock};

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

/// A non-empty collection of unsatisfied requests
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct UnfulfillableRequests {
    /// A non-empty collection of unsatisfied requests
    unsatisfied: Vec<UnfulfillableRequest>, // we want `Set` but `IndexSet` is not `Hash`
}
impl UnfulfillableRequests {
    /// # Panics
    /// Panics if `unsatisfied` is empty.
    pub fn new(unsatisfied: IndexSet<UnfulfillableRequest>) -> Self {
        assert!(!unsatisfied.is_empty(), "non_empty must not be empty");
        Self {
            unsatisfied: unsatisfied.into_iter().collect(),
        }
    }
    pub fn unsatisfied(&self) -> IndexSet<UnfulfillableRequest> {
        self.unsatisfied.clone().into_iter().collect()
    }

    pub fn requests(&self) -> IndexSet<DerivationRequest> {
        self.unsatisfied
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
#[async_trait::async_trait]
pub trait IsPreDerivedKeysCache {
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
            // if for_key.len() <= 1 {
            //     if for_key.is_empty() {
            //         warn!("Incorrect implementation of Cache! Should never be empty!");
            //     }
            //     unfulfillable.insert(tuple.request.clone());
            // }
            if factors_left == 0 {
                unfulfillable.insert(UnfulfillableRequest {
                    request: tuple.request.clone(),
                    reason: UnfulfillableRequestReason::Empty,
                });
            } else if factors_left == 1 {
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

#[async_trait::async_trait]
impl IsPreDerivedKeysCache for InMemoryPreDerivedKeysCache {
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
        unfulfillable: &UnfulfillableRequests,
    ) -> Result<()> {
        let factor_sources = profile.factor_sources.clone();

        // let request_by_factor: HashMap<FactorSourceIDFromHash, IndexSet<DerivationPath>> =
        //     unfulfillable
        //         .into_iter()
        //         .into_grouping_map_by(|x| x)
        //         .collect::<IndexSet<_>>();

        // let derivation_paths = request_by_factor
        //     .into_iter()
        //     .collect::<IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>>();
        // derivation_paths: IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>,

        // let keys_collector = KeysCollector::new(
        //     factor_sources,
        //     derivation_paths,
        //     self.derivation_interactors.clone(),
        // )?;

        // let derivation_outcome = keys_collector.collect_keys().await?;
        // let newly_derived = derivation_outcome.factor_instances;

        todo!()
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
