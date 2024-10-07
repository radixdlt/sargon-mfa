#[cfg(test)]
mod tests {
    use std::{
        borrow::{Borrow, BorrowMut},
        sync::RwLockReadGuard,
    };

    use crate::prelude::*;

    struct SargonOS {
        cache: Arc<RwLock<FactorInstancesForEachNetworkCache>>,
        gateway: RwLock<TestGateway>,
        profile: RwLock<Profile>,
    }

    impl SargonOS {
        pub fn profile_snapshot(&self) -> Profile {
            self.profile.try_read().unwrap().clone()
        }
        pub fn new() -> Self {
            Arc::new(TestDerivationInteractors::default());
            Self {
                cache: Arc::new(RwLock::new(FactorInstancesForEachNetworkCache::default())),
                gateway: RwLock::new(TestGateway::default()),
                profile: RwLock::new(Profile::default()),
            }
        }
        pub async fn with_bdfs() -> (Self, HDFactorSource) {
            let self_ = Self::new();
            let bdfs = HDFactorSource::device();
            self_.add_factor_source(bdfs.clone()).await.unwrap();
            (self_, bdfs)
        }

        fn _cache(&self) -> Arc<RwLock<FactorInstancesForEachNetworkCache>> {
            self.cache.clone()
        }

        pub fn cache_snapshot(&self) -> FactorInstancesForEachNetworkCache {
            self.cache.try_read().unwrap().clone_snapshot()
        }

        pub fn clear_cache(&self) {
            println!("💣 CLEAR CACHE");
            *self.cache.try_write().unwrap() = FactorInstancesForEachNetworkCache::default();
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
        ) -> Result<(Account, FactorInstancesProviderStatistics)> {
            let interactors: Arc<dyn KeysDerivationInteractors> =
                Arc::new(TestDerivationInteractors::default());

            let outcome = FactorInstancesProvider::provide(
                FillCacheStrategy::AllTemplates,
                self._cache(),
                network,
                self.profile_snapshot(),
                interactors,
                InstancesQuery::AccountVeci {
                    factor_source: factor_source.clone(),
                },
            )
            .await?;

            let instance = outcome
                .factor_instances_to_use_directly
                .account_veci()?
                .instance();

            println!(
                "🔮 Created account: '{}' with veci.index: {}",
                name.as_ref(),
                instance.derivation_entity_index()
            );

            let address = AccountAddress::new(network, instance.public_key_hash());
            let account = Account::new(
                name,
                address,
                EntitySecurityState::Unsecured(instance),
                ThirdPartyDepositPreference::default(),
            );
            self.profile.try_write().unwrap().add_account(&account);
            Ok((account, outcome.statistics))
        }

        /*
                pub async fn securify(
                    &self,
                    accounts: Accounts,
                    shield: MatrixOfFactorSources,
                ) -> Result<(SecurifiedAccounts, DidDeriveNewFactorInstances)> {
                    println!(
                        "🛡️ Securifying accounts: '{:?}'",
                        accounts.clone().into_iter().map(|x| x.name()).collect_vec()
                    );
                    let interactors: Arc<dyn KeysDerivationInteractors> =
                        Arc::new(TestDerivationInteractors::default());

                    let factor_instances_provider =
                        FactorInstancesProvider::update_or_set_security_shield_for_accounts(
                            accounts.clone(),
                            shield.clone(),
                            self._cache(),
                            self.profile_snapshot(),
                            interactors,
                        );

                    let (instances, did_derive_new) = factor_instances_provider
                        .get_factor_instances_outcome()
                        .await?;
                    let mut instances = instances.factor_instances();

                    println!(
                        "🧵🎉 securfying: #{} accounts, got: #{} factor instances",
                        accounts.len(),
                        instances.len()
                    );

                    // Now we need to map the flat set of instances into many MatrixOfFactorInstances, and assign
                    // one to each account
                    let updated_accounts = accounts
                        .clone()
                        .into_iter()
                        .map(|a| {
                            let matrix_of_instances =
                            MatrixOfFactorInstances::fulfilling_matrix_of_factor_sources_with_instances(
                                instances.clone(),
                                shield.clone(),
                            )
                            .unwrap();
                        println!("🦆👻 removing: #{} instances used by Matrix from instances: #{} => left is #{}", matrix_of_instances.all_factors().len(), instances.len(), instances.len() - matrix_of_instances.all_factors().len());
                            for used_instance in matrix_of_instances.all_factors() {
                                instances.shift_remove(&used_instance);
                            }
                            let access_controller = match a.security_state() {
                                EntitySecurityState::Unsecured(_) => {
                                    AccessController::from_unsecurified_address(a.entity_address())
                                }
                                EntitySecurityState::Securified(sec) => sec.access_controller.clone(),
                            };
                            let veci = match a.security_state() {
                                EntitySecurityState::Unsecured(veci) => Some(veci),
                                EntitySecurityState::Securified(sec) => sec.veci.clone(),
                            };
                            let sec =
                                SecurifiedEntityControl::new(matrix_of_instances, access_controller, veci);

                            SecurifiedAccount::new(
                                a.name(),
                                a.entity_address(),
                                sec,
                                a.third_party_deposit(),
                            )
                        })
                        .collect::<IndexSet<SecurifiedAccount>>();

                    for account in updated_accounts.iter() {
                        self.profile
                            .try_write()
                            .unwrap()
                            .update_account(&account.account());
                    }
                    assert!(
                        instances.is_empty(),
                        "should have used all instances, but have unused instances: {:?}",
                        instances
                    );
                    SecurifiedAccounts::new(accounts.network_id(), updated_accounts)
                        .map(|x| (x, did_derive_new))
                }
        */
        async fn add_factor_source(&self, factor_source: HDFactorSource) -> Result<()> {
            let interactors: Arc<dyn KeysDerivationInteractors> =
                Arc::new(TestDerivationInteractors::default());

            // let factor_instances_provider =
            //     FactorInstancesProvider::pre_derive_instance_for_new_factor_source(
            //         &factor_source,
            //         self._cache(),
            //         self.profile_snapshot(),
            //         interactors,
            //     );
            let network_id = NetworkID::Mainnet;
            let outcome = FactorInstancesProvider::provide(
                FillCacheStrategy::AllTemplates,
                self._cache(),
                network_id,
                self.profile_snapshot(),
                interactors,
                InstancesQuery::PreDeriveKeys {
                    factor_source: factor_source.clone(),
                },
            )
            .await?;

           
            // assert!(did_derive_new.0);

            // assert!(
            //     instances.is_empty(),
            //     "should be empty, since should have been put into the cache, not here."
            // );

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
                .filter(|x| x.satisfies(DerReq::new(
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
                .filter(|x| x.satisfies(DerReq::new(
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
                .filter(|x| x.satisfies(DerReq::new(
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
                .filter(|x| x.satisfies(DerReq::new(
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

    #[actix_rt::test]
    async fn securified_accounts() {
        let (os, bdfs) = SargonOS::with_bdfs().await;
        let alice = os
            .new_account_with_bdfs(NetworkID::Mainnet, "Alice")
            .await
            .unwrap()
            .0;

        let bob = os
            .new_account_with_bdfs(NetworkID::Mainnet, "Bob")
            .await
            .unwrap()
            .0;
        assert_ne!(alice.address(), bob.address());
        let ledger = HDFactorSource::ledger();
        let arculus = HDFactorSource::arculus();
        let yubikey = HDFactorSource::yubikey();
        os.add_factor_source(ledger.clone()).await.unwrap();
        os.add_factor_source(arculus.clone()).await.unwrap();
        os.add_factor_source(yubikey.clone()).await.unwrap();
        let shield_0 =
            MatrixOfFactorSources::new([bdfs.clone(), ledger.clone(), arculus.clone()], 2, []);
        let (securified_accounts, did_derive_new) = os
            .securify(
                Accounts::new(
                    NetworkID::Mainnet,
                    IndexSet::from_iter([alice.clone(), bob.clone()]),
                )
                .unwrap(),
                shield_0,
            )
            .await
            .unwrap();
        assert!(!did_derive_new.0, "should have used cache");
        let alice_sec = securified_accounts
            .clone()
            .into_iter()
            .find(|x| x.address() == alice.entity_address())
            .unwrap();

        assert_eq!(
            alice_sec.securified_entity_control().veci.unwrap().clone(),
            alice.as_unsecurified().unwrap().veci().factor_instance()
        );
        let alice_matrix = alice_sec.securified_entity_control().matrix.clone();
        assert_eq!(alice_matrix.threshold, 2);

        assert_eq!(
            alice_matrix
                .all_factors()
                .into_iter()
                .map(|f| f.factor_source_id())
                .collect_vec(),
            [
                bdfs.factor_source_id(),
                ledger.factor_source_id(),
                arculus.factor_source_id()
            ]
        );

        assert_eq!(
            alice_matrix
                .all_factors()
                .into_iter()
                .map(|f| f.derivation_entity_index())
                .collect_vec(),
            [
                HDPathComponent::securifying_base_index(0),
                HDPathComponent::securifying_base_index(0),
                HDPathComponent::securifying_base_index(0)
            ]
        );

        // assert bob

        let bob_sec = securified_accounts
            .clone()
            .into_iter()
            .find(|x| x.address() == bob.entity_address())
            .unwrap();

        assert_eq!(
            bob_sec.securified_entity_control().veci.unwrap().clone(),
            bob.as_unsecurified().unwrap().veci().factor_instance()
        );
        let bob_matrix = bob_sec.securified_entity_control().matrix.clone();
        assert_eq!(bob_matrix.threshold, 2);

        assert_eq!(
            bob_matrix
                .all_factors()
                .into_iter()
                .map(|f| f.factor_source_id())
                .collect_vec(),
            [
                bdfs.factor_source_id(),
                ledger.factor_source_id(),
                arculus.factor_source_id()
            ]
        );

        assert_eq!(
            bob_matrix
                .all_factors()
                .into_iter()
                .map(|f| f.derivation_entity_index())
                .collect_vec(),
            [
                HDPathComponent::securifying_base_index(1),
                HDPathComponent::securifying_base_index(1),
                HDPathComponent::securifying_base_index(1)
            ]
        );

        let carol = os
            .new_account(ledger.clone(), NetworkID::Mainnet, "Carol")
            .await
            .unwrap()
            .0;

        assert_eq!(
            carol
                .as_unsecurified()
                .unwrap()
                .veci()
                .factor_instance()
                .derivation_entity_index()
                .base_index(),
            0,
            "First account created with ledger, should have index 0, even though this ledger was used in the shield, since we are using two different KeySpaces for Securified and Unsecurified accounts."
        );

        let (securified_accounts, did_derive_new) = os
            .securify(
                Accounts::just(carol.clone()),
                MatrixOfFactorSources::new([], 0, [yubikey.clone()]),
            )
            .await
            .unwrap();
        assert!(!did_derive_new.0, "should have used cache");
        let carol_sec = securified_accounts
            .clone()
            .into_iter()
            .find(|x| x.address() == carol.entity_address())
            .unwrap();

        let carol_matrix = carol_sec.securified_entity_control().matrix.clone();

        assert_eq!(
            carol_matrix
                .all_factors()
                .into_iter()
                .map(|f| f.factor_source_id())
                .collect_vec(),
            [yubikey.factor_source_id()]
        );

        assert_eq!(
            carol_matrix
                .all_factors()
                .into_iter()
                .map(|f| f.derivation_entity_index())
                .collect_vec(),
            [HDPathComponent::securifying_base_index(0)]
        );

        // Update Alice's shield to only use YubiKey

        let (securified_accounts, did_derive_new) = os
            .securify(
                Accounts::new(
                    NetworkID::Mainnet,
                    IndexSet::from_iter([alice.clone(), bob.clone()]),
                )
                .unwrap(),
                MatrixOfFactorSources::new([], 0, [yubikey.clone()]),
            )
            .await
            .unwrap();
        assert!(!did_derive_new.0, "should have used cache");
        let alice_sec = securified_accounts
            .clone()
            .into_iter()
            .find(|x| x.address() == alice.entity_address())
            .unwrap();

        let alice_matrix = alice_sec.securified_entity_control().matrix.clone();

        assert_eq!(
            alice_matrix
                .all_factors()
                .into_iter()
                .map(|f| f.derivation_entity_index())
                .collect_vec(),
            [
                HDPathComponent::securifying_base_index(1) // Carol used `0`.
            ]
        );
    }

    #[actix_rt::test]
    async fn test_partial_sec() {
        let (os, bdfs) = SargonOS::with_bdfs().await;

        let ledger = HDFactorSource::ledger();
        let arculus = HDFactorSource::arculus();
        let yubikey = HDFactorSource::yubikey();
        os.add_factor_source(ledger.clone()).await.unwrap();
        os.add_factor_source(arculus.clone()).await.unwrap();
        os.add_factor_source(yubikey.clone()).await.unwrap();

        let factor_sources = os.profile_snapshot().factor_sources.clone();
        assert_eq!(
            factor_sources.clone().into_iter().collect_vec(),
            vec![
                bdfs.clone(),
                ledger.clone(),
                arculus.clone(),
                yubikey.clone(),
            ]
        );

        let n = DerivationRequestQuantitySelector::FILL_CACHE_QUANTITY / 2;

        for i in 0..3 * n {
            let (account, did_derive_new_factor_instances) = os
                .new_mainnet_account_with_bdfs(format!("Acco: {}", i))
                .await
                .unwrap();
        }

        let shield_0 =
            MatrixOfFactorSources::new([bdfs.clone(), ledger.clone(), arculus.clone()], 2, []);

        let all_accounts = os
            .profile_snapshot()
            .get_accounts()
            .into_iter()
            .collect_vec();

        let first_half_of_accounts = all_accounts.clone()[0..n]
            .into_iter()
            .cloned()
            .collect::<IndexSet<Account>>();

        let second_half_of_accounts = all_accounts.clone()[n..3 * n]
            .into_iter()
            .cloned()
            .collect::<IndexSet<Account>>();

        assert_eq!(
            first_half_of_accounts.len() + second_half_of_accounts.len(),
            3 * n
        );

        let (first_half_securified_accounts, did_derive_new) = os
            .securify(
                Accounts::new(NetworkID::Mainnet, first_half_of_accounts).unwrap(),
                shield_0.clone(),
            )
            .await
            .unwrap();

        assert!(!did_derive_new.0, "should have used cache");

        let (second_half_securified_accounts, did_derive_new) = os
            .securify(
                Accounts::new(NetworkID::Mainnet, second_half_of_accounts).unwrap(),
                shield_0,
            )
            .await
            .unwrap();

        assert!(did_derive_new.0, "should have derived more");

        // let alice_sec = securified_accounts
        //     .clone()
        //     .into_iter()
        //     .find(|x| x.address() == alice.entity_address())
        //     .unwrap();

        // assert_eq!(
        //     alice_sec.securified_entity_control().veci.unwrap().clone(),
        //     alice.as_unsecurified().unwrap().veci().factor_instance()
        // );
        // let alice_matrix = alice_sec.securified_entity_control().matrix.clone();
        // assert_eq!(alice_matrix.threshold, 2);

        // assert_eq!(
        //     alice_matrix
        //         .all_factors()
        //         .into_iter()
        //         .map(|f| f.factor_source_id())
        //         .collect_vec(),
        //     [
        //         bdfs.factor_source_id(),
        //         ledger.factor_source_id(),
        //         arculus.factor_source_id()
        //     ]
        // );

        // assert_eq!(
        //     alice_matrix
        //         .all_factors()
        //         .into_iter()
        //         .map(|f| f.derivation_entity_index())
        //         .collect_vec(),
        //     [
        //         HDPathComponent::securifying_base_index(0),
        //         HDPathComponent::securifying_base_index(0),
        //         HDPathComponent::securifying_base_index(0)
        //     ]
        // );
    }
}
