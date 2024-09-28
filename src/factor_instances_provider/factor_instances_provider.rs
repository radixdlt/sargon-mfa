#![allow(unused)]
#![allow(unused_variables)]

use crate::prelude::*;

use super::split_cache_response::split_cache_response;

/// A provider of FactorInstances, reading them from the cache if present,
/// else if missing derives many instances in Abundance and caches the
/// not requested ones, and returns one matching the requested ones.
pub struct FactorInstancesProvider {
    /// The purpose of the requested FactorInstances.
    purpose: FactorInstancesRequestPurpose,

    /// If no cache present, a new one is created and will be filled.
    cache: Arc<RwLock<PreDerivedKeysCache>>,

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
        maybe_cache: Option<Arc<RwLock<PreDerivedKeysCache>>>,
        maybe_profile_snapshot: impl Into<Option<Profile>>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        /// Important we create `NextIndexAssignerWithEphemeralLocalOffsets`
        /// inside this ctor and that we do not pass it as a parameter, since
        /// it cannot be reused, since it has ephemeral local offsets.
        let next_index_assigner =
            NextIndexAssignerWithEphemeralLocalOffsets::new(maybe_profile_snapshot);

        let cache =
            maybe_cache.unwrap_or_else(|| Arc::new(RwLock::new(PreDerivedKeysCache::default())));

        Self {
            purpose,
            cache,
            next_index_assigner,
            derivation_interactors,
        }
    }
}

fn fill_cache_for_every_factor_source_in(
    purpose: FactorInstancesRequestPurpose,
) -> QuantifiedUnindexDerivationRequests {
    purpose
        .factor_sources()
        .into_iter()
        .flat_map(|factor_source| {
            FactorInstancesRequestPurpose::PreDeriveInstancesForNewFactorSource { factor_source }
                .requests()
                .requests()
        })
        .collect()
}

/// ==================
/// *** Private API ***
/// ==================
impl FactorInstancesProvider {
    fn paths_for_additional_derivation(
        next_index_assigner: &NextIndexAssignerWithEphemeralLocalOffsets,
        requests_to_satisfy_request: IndexSet<DeriveMore>,
        original_purpose: FactorInstancesRequestPurpose,
        cache: PreDerivedKeysCache, // check if already saturated
    ) -> IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>> {
        let fill_cache_for_all_referenced_factor_source =
            fill_cache_for_every_factor_source_in(original_purpose);

        let fill_cache = fill_cache_for_all_referenced_factor_source
            .into_iter()
            .filter(|x| {
                if cache.is_saturated_for(x) {
                    return false;
                }

                let original_contains_matching =
                    requests_to_satisfy_request
                        .iter()
                        .any(|haystack| match haystack {
                            DeriveMore::WithKnownStartIndex {
                                with_start_index, ..
                            } => {
                                let h = UnquantifiedUnindexDerivationRequest::from(
                                    with_start_index.clone(),
                                );
                                UnquantifiedUnindexDerivationRequest::from(x.clone()) == h
                            }
                            DeriveMore::WithoutKnownLastIndex { ref request, .. } => {
                                let h = UnquantifiedUnindexDerivationRequest::from(request.clone());
                                UnquantifiedUnindexDerivationRequest::from(x.clone()) == h
                            }
                        });
                /// we don't want to fill the cache with the same thing we are trying to satisfy,
                /// instead we need to retain the original request with possible known start index,
                /// and we are instead going to change the number of derived factors to
                /// match filling cache quantity.
                !original_contains_matching
            })
            .map(|x| {
                DerivationRequestWithRange::new(
                    x.factor_source_id,
                    x.network_id,
                    x.entity_kind,
                    x.key_kind,
                    x.key_space,
                    DerivationRequestQuantitySelector::FILL_CACHE_QUANTITY,
                    // safe to use next_index_assigner here, before we map
                    // `requests_to_satisfy_request`, since we have filtered out
                    // any matching request above, thus the ordering does not
                    // matter. MEANING: below with `requests_to_satisfy_request`,
                    // some of the requests have KNOWN start indices, and we would
                    // not wanna mess up the state of the `next_index_assigner`
                    // by using it before we have filtered out the matching requests.
                    next_index_assigner.next(x.clone().into()).base_index(),
                )
            })
            .collect::<IndexSet<DerivationRequestWithRange>>();

        let mut requests_to_satisfy_with_ranges = requests_to_satisfy_request
            .into_iter()
            .map(|x| {
                /// Always fill cache!
                let quantity = DerivationRequestQuantitySelector::FILL_CACHE_QUANTITY;

                match x {
                    DeriveMore::WithKnownStartIndex {
                        with_start_index: z,
                        ..
                    } => DerivationRequestWithRange::new(
                        z.factor_source_id,
                        z.network_id,
                        z.entity_kind,
                        z.key_kind,
                        z.key_space,
                        quantity,
                        z.start_base_index,
                    ),
                    DeriveMore::WithoutKnownLastIndex { ref request, .. } => {
                        let next = next_index_assigner.next(request.clone().into());
                        DerivationRequestWithRange::new(
                            request.factor_source_id,
                            request.network_id,
                            request.entity_kind,
                            request.key_kind,
                            request.key_space,
                            quantity,
                            next.base_index(),
                        )
                    }
                }
            })
            .collect::<IndexSet<DerivationRequestWithRange>>();

        let mut requests_with_ranges_with_abundance = IndexSet::<DerivationRequestWithRange>::new();

        requests_with_ranges_with_abundance.extend(requests_to_satisfy_with_ranges);
        requests_with_ranges_with_abundance.extend(fill_cache);

        requests_with_ranges_with_abundance
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
            .collect::<IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>>()
    }

    async fn derive_more(
        purpose: FactorInstancesRequestPurpose,
        next_index_assigner: &NextIndexAssignerWithEphemeralLocalOffsets,
        requests_to_satisfy_request: IndexSet<DeriveMore>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        cache: PreDerivedKeysCache, // check if already saturated
    ) -> Result<IndexSet<NewlyDerived>> {
        let derivation_paths = Self::paths_for_additional_derivation(
            next_index_assigner,
            requests_to_satisfy_request.clone(),
            purpose.clone(),
            cache,
        );

        let keys_collector = KeysCollector::new(
            purpose.factor_sources(),
            derivation_paths,
            derivation_interactors.clone(),
        )?;

        let derivation_outcome = keys_collector.collect_keys().await;

        let factor_instances = derivation_outcome.all_factors();

        let out = factor_instances
            .into_iter()
            .into_group_map_by(|x| UnquantifiedUnindexDerivationRequest::from(x.clone()))
            .into_iter()
            .map(|(k, v)| {
                if let Some(original) = requests_to_satisfy_request
                    .iter()
                    .find(|x| x.unquantified() == k)
                {
                    let number_of_instances_to_use_directly =
                        original.number_of_instances_to_use_directly(purpose.clone());
                    let (to_use_directly, to_cache) =
                        v.split_at(number_of_instances_to_use_directly);

                    let to_use_directly =
                        to_use_directly.iter().cloned().collect::<FactorInstances>();

                    let to_cache = to_cache.iter().cloned().collect::<FactorInstances>();
                    println!(
                        "🍭 to_cache: #{}, to_use_directly: #{}",
                        to_cache.len(),
                        to_use_directly.len()
                    );
                    NewlyDerived::maybe_some_to_use_directly(k, to_cache, to_use_directly)
                } else {
                    let to_cache = v.into_iter().collect::<FactorInstances>();
                    NewlyDerived::cache_all(k, to_cache)
                }
            })
            .collect::<IndexSet<NewlyDerived>>();

        Ok(out)
    }

    async fn _get_factor_instances_outcome(
        purpose: FactorInstancesRequestPurpose,
        cache: PreDerivedKeysCache,
        next_index_assigner: &NextIndexAssignerWithEphemeralLocalOffsets,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<(
        FactorInstances,
        PreDerivedKeysCache,
        DidDeriveNewFactorInstances,
    )> {
        let requested = purpose.requests();

        let from_cache = cache.take(&requested)?;
        let split = split_cache_response(from_cache);

        let mut factor_instances_to_use_directly = split.satisfied_by_cache();

        let mut did_derive_new_instances = false;
        if let Some(derive_more_requests) = split.derive_more_requests() {
            let mut newly_derived_to_be_used_directly =
                IndexSet::<HierarchicalDeterministicFactorInstance>::new();

            let newly_derived = Self::derive_more(
                purpose,
                next_index_assigner,
                derive_more_requests,
                derivation_interactors,
                cache.clone_snapshot(), // check if already saturated
            )
            .await?;

            for _newly_derived in newly_derived.into_iter() {
                let (key, to_cache) = _newly_derived.key_value_for_cache();
                cache.put(key, to_cache)?;

                newly_derived_to_be_used_directly.extend(_newly_derived.to_use_directly);
            }

            did_derive_new_instances = true;
            factor_instances_to_use_directly.extend(newly_derived_to_be_used_directly);
        }

        Ok((
            FactorInstances::from(factor_instances_to_use_directly),
            cache,
            DidDeriveNewFactorInstances(did_derive_new_instances),
        ))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct DidDeriveNewFactorInstances(bool);

/// ==================
/// *** Public API ***
/// ==================
impl FactorInstancesProvider {
    /// Does not return ALL derived FactorInstances, but only those that are
    /// related to the purpose of the request.
    ///
    /// Might derive MORE than requested, those will be put into the cache.
    pub async fn get_factor_instances_outcome(
        self,
    ) -> Result<(FactorInstances, DidDeriveNewFactorInstances)> {
        let copy_of_cache = self.cache.try_read().unwrap().clone_snapshot();

        let result = Self::_get_factor_instances_outcome(
            self.purpose,
            // Take a copy of the cache, so we can modify it without affecting the original,
            // important if this method fails, we do not want to rollback the cache.
            // Instead we update the cache with the returned one in case of success.
            copy_of_cache,
            &self.next_index_assigner,
            self.derivation_interactors,
        )
        .await;

        match result {
            Ok((outcome, updated_cache, did_derive_new_instances)) => {
                // Replace cache with updated one
                *self.cache.try_write().unwrap() = updated_cache;
                Ok((outcome, did_derive_new_instances))
            }
            Err(e) => {
                // No need to rollback the cache, we did not modify it, only a copy of it.
                Err(e)
            }
        }
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
        pub async fn with_bdfs() -> (Self, HDFactorSource) {
            let self_ = Self::new();
            let bdfs = HDFactorSource::device();
            self_.add_factor_source(bdfs.clone()).await.unwrap();
            (self_, bdfs)
        }

        fn _cache(&self) -> Option<Arc<RwLock<PreDerivedKeysCache>>> {
            Some(self.cache.clone())
        }

        pub fn cache_snapshot(&self) -> PreDerivedKeysCache {
            self.cache.try_read().unwrap().clone_snapshot()
        }

        pub fn clear_cache(&self) {
            println!("💣 CLEAR CACHE");
            *self.cache.try_write().unwrap() = PreDerivedKeysCache::default();
        }

        pub async fn new_mainnet_account_with_bdfs(
            &self,
            name: impl AsRef<str>,
        ) -> Result<(Account, DidDeriveNewFactorInstances)> {
            self.new_account_with_bdfs(NetworkID::Mainnet, name).await
        }

        pub async fn new_account_with_bdfs(
            &self,
            network: NetworkID,
            name: impl AsRef<str>,
        ) -> Result<(Account, DidDeriveNewFactorInstances)> {
            let bdfs = self.profile_snapshot().bdfs();
            self.new_account(bdfs, network, name).await
        }

        pub async fn new_account(
            &self,
            factor_source: HDFactorSource,
            network: NetworkID,
            name: impl AsRef<str>,
        ) -> Result<(Account, DidDeriveNewFactorInstances)> {
            println!("🔮 Creating account: '{}'", name.as_ref());
            let interactors: Arc<dyn KeysDerivationInteractors> =
                Arc::new(TestDerivationInteractors::default());

            let factor_instances_provider =
                FactorInstancesProvider::new_virtual_unsecurified_account(
                    network,
                    &factor_source,
                    self._cache(),
                    self.profile_snapshot(),
                    interactors,
                );

            let (instances, did_derive_new) = factor_instances_provider
                .get_factor_instances_outcome()
                .await?;

            assert_eq!(instances.len(), 1);
            let instance = instances.into_iter().next().unwrap();
            println!("🇸🇪🚀 instance: {:?}", instance);
            println!(
                "🇸🇪🚀 cache: {:?}",
                self.cache_snapshot()
                    .all_factor_instances()
                    .into_iter()
                    .map(|f| f.derivation_path())
                    .filter(|x| x.entity_kind == CAP26EntityKind::Account
                        && x.key_kind == CAP26KeyKind::TransactionSigning
                        && x.index.key_space() == KeySpace::Unsecurified)
                    .map(|x| x.index)
                    .collect_vec()
            );
            let address = AccountAddress::new(network, instance.public_key_hash());
            let account = Account::new(name, address, EntitySecurityState::Unsecured(instance));
            self.profile.try_write().unwrap().add_account(&account);
            Ok((account, did_derive_new))
        }

        async fn add_factor_source(&self, factor_source: HDFactorSource) -> Result<()> {
            let interactors: Arc<dyn KeysDerivationInteractors> =
                Arc::new(TestDerivationInteractors::default());

            let factor_instances_provider =
                FactorInstancesProvider::pre_derive_instance_for_new_factor_source(
                    &factor_source,
                    self._cache(),
                    self.profile_snapshot(),
                    interactors,
                );

            let (instances, did_derive_new) = factor_instances_provider
                .get_factor_instances_outcome()
                .await?;

            assert!(did_derive_new.0);

            assert!(
                instances.is_empty(),
                "should be empty, since should have been put into the cache, not here."
            );

            self.profile
                .try_write()
                .unwrap()
                .add_factor_source(factor_source.clone());

            Ok(())
        }
    }

    #[actix_rt::test]
    async fn add_factor_source() {
        let os = SargonOS::new();
        assert_eq!(os.cache_snapshot().total_number_of_factor_instances(), 0);
        assert_eq!(os.profile_snapshot().factor_sources.len(), 0);
        let factor_source = HDFactorSource::sample();
        os.add_factor_source(factor_source.clone()).await.unwrap();
        assert!(
            !os.cache_snapshot().all_factor_instances().is_empty(),
            "Should have put factors into the cache."
        );
        assert_eq!(
            os.profile_snapshot().factor_sources,
            IndexSet::just(factor_source)
        );
    }

    #[actix_rt::test]
    async fn create_account() {
        let (os, bdfs) = SargonOS::with_bdfs().await;
        let free_factor_instances_before_any_account_created =
            os.cache_snapshot().all_factor_instances();
        let number_of_free_factor_instances =
            free_factor_instances_before_any_account_created.len();
        assert!(
            number_of_free_factor_instances > 0,
            "should have many, for bdfs"
        );
        assert_eq!(
            os.profile_snapshot().factor_sources.len(),
            1,
            "should have bdfs"
        );
        assert_eq!(os.profile_snapshot().accounts.len(), 0, "no accounts");

        let network = NetworkID::Mainnet;
        let entity_kind = CAP26EntityKind::Account;
        let key_kind = CAP26KeyKind::TransactionSigning;
        let key_space = KeySpace::Unsecurified;

        let expected_path = DerivationPath::new(
            network,
            entity_kind,
            key_kind,
            HDPathComponent::unsecurified_hardening_base_index(0),
        );

        assert_eq!(
            free_factor_instances_before_any_account_created
                .clone()
                .into_iter()
                .filter(|x| x.satisfies(UnquantifiedUnindexDerivationRequest::new(
                    bdfs.factor_source_id(),
                    network,
                    entity_kind,
                    key_kind,
                    key_space
                )))
                .count(),
            DerivationRequestQuantitySelector::FILL_CACHE_QUANTITY
        );

        assert!(
            free_factor_instances_before_any_account_created
                .clone()
                .into_iter()
                .filter(|x| x.factor_source_id() == bdfs.factor_source_id())
                .filter(|x| x.derivation_path() == expected_path)
                .count()
                == 1
        );

        let (alice, did_derive_new_factor_instances) =
            os.new_mainnet_account_with_bdfs("Alice").await.unwrap();
        assert!(!did_derive_new_factor_instances.0, "should have used cache");
        assert_eq!(
            os.profile_snapshot().get_accounts(),
            IndexSet::just(alice.clone())
        );

        let free_factor_instances_after_account_creation =
            os.cache_snapshot().all_factor_instances();
        assert_eq!(
            free_factor_instances_after_account_creation.len(),
            number_of_free_factor_instances - 1
        );

        assert_eq!(
            free_factor_instances_after_account_creation
                .clone()
                .into_iter()
                .filter(|x| x.satisfies(UnquantifiedUnindexDerivationRequest::new(
                    bdfs.factor_source_id(),
                    network,
                    entity_kind,
                    key_kind,
                    key_space
                )))
                .count(),
            DerivationRequestQuantitySelector::FILL_CACHE_QUANTITY - 1
        );

        assert_eq!(
            alice
                .clone()
                .as_unsecurified()
                .unwrap()
                .factor_instance()
                .derivation_path(),
            expected_path
        );

        assert!(
            free_factor_instances_after_account_creation
                .clone()
                .into_iter()
                .filter(|x| x.factor_source_id() == bdfs.factor_source_id())
                .filter(|x| x.derivation_path() == expected_path)
                .count()
                == 0
        );

        let (bob, did_derive_new_factor_instances) =
            os.new_mainnet_account_with_bdfs("Bob").await.unwrap();

        assert!(!did_derive_new_factor_instances.0, "should have used cache");
        assert_ne!(alice.address(), bob.address());

        let free_factor_instances_after_account_creation =
            os.cache_snapshot().all_factor_instances();
        assert_eq!(
            free_factor_instances_after_account_creation.len(),
            number_of_free_factor_instances - 2
        );

        assert_eq!(
            free_factor_instances_after_account_creation
                .clone()
                .into_iter()
                .filter(|x| x.satisfies(UnquantifiedUnindexDerivationRequest::new(
                    bdfs.factor_source_id(),
                    network,
                    entity_kind,
                    key_kind,
                    key_space
                )))
                .count(),
            DerivationRequestQuantitySelector::FILL_CACHE_QUANTITY - 2
        );

        let bob_veci = bob.clone().as_unsecurified().unwrap().factor_instance();
        assert_eq!(
            bob_veci.derivation_path(),
            DerivationPath::new(
                NetworkID::Mainnet,
                CAP26EntityKind::Account,
                CAP26KeyKind::TransactionSigning,
                HDPathComponent::unsecurified_hardening_base_index(1),
            )
        );
        assert_eq!(bob_veci.factor_source_id, bdfs.factor_source_id());

        // NOW CLEAR CACHE and create 3rd account, should work thanks to the profile...
        os.clear_cache();
        assert_eq!(os.cache_snapshot().total_number_of_factor_instances(), 0);
        let (carol, did_derive_new_factor_instances) =
            os.new_mainnet_account_with_bdfs("Carol").await.unwrap();
        assert!(
            did_derive_new_factor_instances.0,
            "cache was cleared, so we should have derive more..."
        );
        assert_ne!(carol.address(), bob.address());

        let free_factor_instances_after_account_creation =
            os.cache_snapshot().all_factor_instances();

        assert_eq!(
                   free_factor_instances_after_account_creation.len(),
                   (DerivationRequestQuantitySelector::FILL_CACHE_QUANTITY * 6 ) - 1,
                   "BatchOfNew.count - 1, since we just cleared cache, derive many more, and consumed one."
               );

        assert_eq!(
            free_factor_instances_after_account_creation
                .clone()
                .into_iter()
                .filter(|x| x.satisfies(UnquantifiedUnindexDerivationRequest::new(
                    bdfs.factor_source_id(),
                    network,
                    entity_kind,
                    key_kind,
                    key_space
                )))
                .count(),
            DerivationRequestQuantitySelector::FILL_CACHE_QUANTITY - 1,
            "since we just cleared cache, derive many more, and consumed one."
        );
        let carol_veci = carol.clone().as_unsecurified().unwrap().factor_instance();
        assert_eq!(
            carol_veci.derivation_path(),
            DerivationPath::new(
                NetworkID::Mainnet,
                CAP26EntityKind::Account,
                CAP26KeyKind::TransactionSigning,
                HDPathComponent::unsecurified_hardening_base_index(2), // third account should have index 2
            )
        );
        assert_eq!(carol_veci.factor_source_id, bdfs.factor_source_id());

        // Should be possible to derive fourth account, using cache, and the derivation index should be 3

        let (diana, did_derive_new_factor_instances) =
            os.new_mainnet_account_with_bdfs("Diana").await.unwrap();

        assert!(!did_derive_new_factor_instances.0, "should have used cache");
        assert_ne!(diana.address(), carol.address());

        let free_factor_instances_after_account_creation =
            os.cache_snapshot().all_factor_instances();

        assert_eq!(
            free_factor_instances_after_account_creation.len(),
            (DerivationRequestQuantitySelector::FILL_CACHE_QUANTITY * 6 ) - 2,
            "BatchOfNew.count - 2, we cleared cached and then derived many and directly used one for Carol, and now one more for Diana, thus - 2"
        );

        let diana_veci = diana.clone().as_unsecurified().unwrap().factor_instance();
        assert_eq!(
            diana_veci.derivation_path(),
            DerivationPath::new(
                NetworkID::Mainnet,
                CAP26EntityKind::Account,
                CAP26KeyKind::TransactionSigning,
                HDPathComponent::unsecurified_hardening_base_index(3),
            )
        );
        assert_eq!(diana_veci.factor_source_id, bdfs.factor_source_id());

        // Now lets derive a bunch using only keys in cache but without using the last
        let expected_start = diana_veci.derivation_entity_base_index() + 1;
        assert_eq!(expected_start, 4); // Diana used 3, so next should be 4

        let left_in_cache = os
            .cache_snapshot()
            .all_factor_instances()
            .into_iter()
            .map(|x| x.derivation_path())
            .filter(|x| {
                x.entity_kind == CAP26EntityKind::Account
                    && x.key_kind == CAP26KeyKind::TransactionSigning
                    && x.index.key_space() == KeySpace::Unsecurified
            })
            .count();

        let count = (left_in_cache - 1) as u32; // -1 since if we were to use the last one the FactorInstancesProvider will
                                                // fill the cache, but we want to derive using all instances without filling the cache yet again
        let mut derivation_entity_indices = IndexSet::<HDPathComponent>::new();
        for i in expected_start..expected_start + count {
            let (account, did_derive_new_factor_instances) = os
                .new_mainnet_account_with_bdfs(format!("Acco: {}", i))
                .await
                .unwrap();
            assert!(
                !did_derive_new_factor_instances.0,
                "should have used the cache"
            );
            let derivation_entity_index = account
                .as_unsecurified()
                .unwrap()
                .veci()
                .factor_instance()
                .derivation_entity_index();

            assert_eq!(derivation_entity_index.base_index(), i);

            derivation_entity_indices.insert(derivation_entity_index);
        }
        assert_eq!(
            *derivation_entity_indices.first().unwrap(),
            HDPathComponent::unsecurified_hardening_base_index(expected_start)
        );
        assert_eq!(
            *derivation_entity_indices.last().unwrap(),
            HDPathComponent::unsecurified_hardening_base_index(expected_start + count - 1)
        );
        assert_eq!(derivation_entity_indices.last().unwrap().base_index(), 30);

        let (last_in_cache, did_use_cache) = os
            .new_mainnet_account_with_bdfs("Last of the...")
            .await
            .unwrap();
        assert!(did_use_cache.0, "should have use (last) in the cache");

        assert_eq!(
            last_in_cache
                .as_unsecurified()
                .unwrap()
                .veci()
                .factor_instance()
                .derivation_entity_index()
                .base_index(),
            31
        );
    }
}
