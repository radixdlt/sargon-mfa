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

impl NextIndexAssignerWithEphemeralLocalOffsets {
    pub fn paths_for_additional_derivation(
        &self,
        requests: IndexSet<DeriveMoreToSatisfyOriginalRequest>,
    ) -> IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>> {
        let with_ranges = requests
            .clone()
            .into_iter()
            .map(|x| match x {
                DeriveMoreToSatisfyOriginalRequest::WithKnownStartIndex {
                    with_start_index,
                    ..
                } => with_start_index,
                DeriveMoreToSatisfyOriginalRequest::WithoutKnownLastIndex(ref partial) => {
                    let next = self.next(partial.clone().into());
                    DerivationRequestWithRange::from((partial.clone(), next))
                }
            })
            .collect::<IndexSet<DerivationRequestWithRange>>();

        with_ranges
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
}

/// ==================
/// *** Private API ***
/// ==================
impl FactorInstancesProvider {
    async fn derive_more(
        purpose: FactorInstancesRequestPurpose,
        next_index_assigner: &NextIndexAssignerWithEphemeralLocalOffsets,
        derive_more_with_abundance: DeriveMoreWithAbundance,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<IndexSet<NewlyDerived>> {
        let requests = derive_more_with_abundance.all_requests();
        let derivation_paths =
            Self::paths_for_additional_derivation(next_index_assigner, requests.clone());

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
        println!(
            "🎭 factor_instances_to_use_directly: #{}",
            factor_instances_to_use_directly.len()
        );
        if let Some(derive_more_requests) = split.derive_more_requests() {
            println!(
                "🎭 derive_more_requests: #{} - ‼️ might result in MANY factor instances right?",
                derive_more_requests.len()
            );
            let derive_more_with_abundance =
                DeriveMoreWithAbundance::new(derive_more_requests, &purpose, next_index_assigner);
            let mut newly_derived_to_be_used_directly =
                IndexSet::<HierarchicalDeterministicFactorInstance>::new();

            let newly_derived = Self::derive_more(
                purpose,
                next_index_assigner,
                derive_more_with_abundance,
                derivation_interactors,
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
        println!(
            "🍬🔮 cache passed to FactorInstancesProvider contains #{} instances",
            copy_of_cache.total_number_of_factor_instances()
        );
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
                println!(
                    "🍬✅ updating cache to a one with #{} instances",
                    updated_cache.total_number_of_factor_instances()
                );
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
            println!(
                "🌈 cache.total_number_of_factor_instances: {}",
                self.cache_snapshot().total_number_of_factor_instances()
            );
            assert_eq!(instances.len(), 1);
            let instance = instances.into_iter().next().unwrap();
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
            30
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
            29
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
            28
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
            179,
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
            29,
            "29, since we just cleared cache, derive many more, and consumed one."
        );

        let carol_veci = carol.clone().as_unsecurified().unwrap().factor_instance();
        assert_eq!(
            bob_veci.derivation_path(),
            DerivationPath::new(
                NetworkID::Mainnet,
                CAP26EntityKind::Account,
                CAP26KeyKind::TransactionSigning,
                HDPathComponent::unsecurified_hardening_base_index(2), // third account should have index 2
            )
        );
        assert_eq!(carol_veci.factor_source_id, bdfs.factor_source_id());
    }
}
