use core::num;
use std::sync::RwLock;

use derive_more::derive;
use rand::seq::index;

use crate::prelude::*;

/// The reason why a request is unfulfillable, and if the reason is that the
/// last factor would be consumed, the value of that last factor is included,
/// to act as the range.
#[derive(Clone, PartialEq, Eq, Hash, Debug, EnumAsInner)]
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

    pub fn is_reason_empty(&self) -> bool {
        matches!(self.reason, UnfulfillableRequestReason::Empty)
    }

    pub fn is_reason_last(&self) -> bool {
        matches!(self.reason, UnfulfillableRequestReason::Last(_))
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
        let cached = self
            .cache
            .try_read()
            .map_err(|_| CommonError::KeysCacheReadGuard)?;

        let request_and_path_tuples = InMemoryPreDerivedKeysCache::tuples(requests.clone());

        let mut unfulfillable = IndexSet::<UnfulfillableRequest>::new();
        for tuple in request_and_path_tuples.iter() {
            let request = tuple.request;
            let Some(for_key) = cached.get(&tuple.path) else {
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
        derived_factors: IndexMap<
            DerivationRequest,
            IndexSet<HierarchicalDeterministicFactorInstance>,
        >,
    ) -> Result<()> {
        let mut write_guard = self
            .cache
            .try_write()
            .map_err(|_| CommonError::KeysCacheWriteGuard)?;

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
        let mut cached = self
            .cache
            .try_write()
            .map_err(|_| CommonError::KeysCacheWriteGuard)?;

        let mut instances_read_from_cache =
            IndexMap::<DerivationRequest, HierarchicalDeterministicFactorInstance>::new();

        let request_and_path_tuples = InMemoryPreDerivedKeysCache::tuples(requests.clone());

        for tuple in request_and_path_tuples {
            let for_key = cached
                .get_mut(&tuple.path)
                .ok_or(CommonError::KeysCacheUnknownKey)?;
            let read_from_cache = for_key.pop().ok_or(CommonError::KeysCacheEmptyForKey)?;
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
///
/// Gateway MUST be able find entities by PAST public key hashes, not only current ones, ponder this
/// scenario:
///
/// NOTATION:
/// Accounts are written as `A`, `B`, `C`, etc.
/// FactorSources are written as `L`, `M`, `N`, etc.
/// FactorInstances derived in:
/// * Unsecurified  Keyspace are written as: `🔴α` (alpha), `🔴β` (beta), `🔴γ` (gamma), `🔴δ` (delta).
/// * Securified    KeySpace are written as: `🔵π` (pi), `🔵ρ` (rho), `🔵σ` (sigma), `🔵τ` (tau), `🔵φ (Phi)`
/// Those FactorInstances are spelled out as:
/// `🔴α=(0', L)`, derived using FactorSource `L` at index `0'`.
/// `🔵π=(0^, M)`, derived using FactorSource `M` at index `0^`.
///
/// Derivation Entity Index: `0'` means 0 hardened, which is in the Unsecurified KeySpace.
/// Derivation Entity Index: `0^` means 0 hardened, which is in the Securified KeySpace.
/// The FactorInstance which was used to form the Address of an entity is called a
/// "genesis factor instance".
///
/// SCENARIO
/// 1. User creates account `A` with genesis FactorInstance `🔴α=(0', L)`
/// 2. User securifies account `A` with `{ override_factors: [🔵π=(0^, L), 🔵ρ=(0^, M)] }`
/// 3. User updates security shield of account `A` with `{ override_factors: [🔵ρ=(0^, M) }`
/// 4. If user tries to create a new account `🔴(❓, L) which value to use for index?
/// 5. Naive implementation will assign `index = 0'`, since that DerivationPath is
/// not referenced anywhere in Profile.
/// 6. FAILURE! We cannot create a new account with `0'`, since it will have same address as account `A`.
/// Solution: We must **retain** the genesis factor instance `🔴α=(0', L)` for account `A`
/// when it gets securified, and persist this in Profile.
///
/// Let us talk about recovery without Profile...
///
/// 7. Imagine user tosses this Profile, and performs recovery using only `M`, then securified account `A`
/// is found and recovered.
/// 8. Later user (re-)add FactorSource `L`, which SHOULD immediately trigger it to derive many keys
/// to be put in the cache. We MUST NOT put `🔴α = (0', L)` in the cache as the "next free" instance
/// to use, since we would have same problem as in step 3-5. Since  `🔴α=(0', L)` is in unsecurified
/// key space we would be able to form an Address from it and see that it is the same address of A.
/// However, we MUST NOT save 🔵π=(0^, L) as a free factor instance either. It has already been used by in
/// step 2, with account `A`.
///
/// The conclusion is this:
/// 💡🔮 Gateway MUST be able find entities by PAST public key hashes, not only current ones. 🔮💡
///
/// Using Gateway's lookup by past and present public key hashes, it will tell use that the hash of both
/// `🔴α =(0', L)` and 🔵π=(0^, L) have been used, so when `L` is saved into Profile and the derived instances
/// are saved into the cache, we will NOT save `🔴α=(0', L)` not `🔵π=(0^, L)`, the cached instances will be
/// `🔴β=(1', L)`, `🔴γ=(2', L)`, ...  and 🔵σ=(1^, L), `🔵τ`=(2^, L), ..., up to some batch size (e.g. `30`).
/// Next time user creates an unsecurified entity it will use FactorInstanceProvider which uses the cache,
/// to consume the next-never-used FactorInstance from the Unsecurified KeySpace. And the next time the user
/// securifies an entity it will consume the next-never-used FactorInstances from the Securified KeySpace - for
/// each FactorSource.
///
/// 9. Later user uses `L` to create a new account `B`, which using this `FactorInstanceProcider` will
/// find the "next" FactorInstance `🔴β=(1', L)`, success!
/// 10. User securifies account `B` with `{ override_factors: [🔵σ=(1^, L), 🔵φ=(1^, M)] }`, success!
/// 11. User know has two securified accounts, none of them share any public key hashes.
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
        println!("🌈 derive_new_and_fill_cache...");
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

        _ = drop(unfulfillable_requests);

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
                    .unwrap_or(HDPathComponent::securified(0));

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
                        .map(|ue| ue.factor_instance)
                        .filter(|fi| fi.matches(&request))
                        .map(|fi| fi.derivation_path().index)
                        .max()
                };

                let next_index = last_index
                    .map(|l| l.add_one())
                    .unwrap_or(HDPathComponent::unsecurified(0));

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

        let mut to_insert: IndexMap<
            DerivationRequest,
            IndexSet<HierarchicalDeterministicFactorInstance>,
        > = IndexMap::new();

        for derived_factor in derived_factors {
            let key = DerivationRequest::from(derived_factor.clone());
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
        return self.cache.consume_next_factor_instances(requests).await;
    }
}

impl FactorInstanceProvider {
    pub async fn securify_with_address<E: IsEntity + std::hash::Hash + std::cmp::Eq>(
        &self,
        address: &E::Address,
        matrix: MatrixOfFactorSources,
        profile: &mut Profile,
    ) -> Result<SecurifiedEntityControl> {
        let entity = profile.entity_by_address::<E>(address)?;
        self.securify(&entity, &matrix, profile).await
    }

    pub async fn securify<E: IsEntity>(
        &self,
        entity: &E,
        matrix: &MatrixOfFactorSources,
        profile: &mut Profile,
    ) -> Result<SecurifiedEntityControl> {
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

        let component_metadata = ComponentMetadata::new(matrix.clone());

        let securified_entity_control = SecurifiedEntityControl::new(
            matrix,
            AccessController {
                address: AccessControllerAddress::new(entity.entity_address()),
                metadata: component_metadata,
            },
        );

        profile.update_entity(E::new(
            entity.name(),
            entity.entity_address(),
            EntitySecurityState::Securified(securified_entity_control.clone()),
        ));

        let gateway = self.gateway.clone();
        gateway
            .set_securified_entity(securified_entity_control.clone(), entity.address())
            .await?;
        Ok(securified_entity_control)
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
