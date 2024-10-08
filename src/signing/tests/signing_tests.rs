#![cfg(test)]
use crate::prelude::*;

mod tests {

    use super::*;

    mod multi_tx {

        use super::*;

        async fn multi_accounts_multi_personas_all_single_factor_controlled_with_sim_user(
            sim: SimulatedUser,
        ) {
            let factor_sources = &HDFactorSource::all();
            let a0 = &Account::a0();
            let a1 = &Account::a1();
            let a2 = &Account::a2();

            let p0 = &Persona::p0();
            let p1 = &Persona::p1();
            let p2 = &Persona::p2();

            let t0 = TransactionIntent::address_of([a0, a1], [p0, p1]);
            let t1 = TransactionIntent::address_of([a0, a1, a2], []);
            let t2 = TransactionIntent::address_of([], [p0, p1, p2]);

            let profile = Profile::new(factor_sources.clone(), [a0, a1, a2], [p0, p1, p2]);

            let collector = SignaturesCollector::new(
                SigningFinishEarlyStrategy::default(),
                IndexSet::<TransactionIntent>::from_iter([t0.clone(), t1.clone(), t2.clone()]),
                Arc::new(TestSignatureCollectingInteractors::new(sim)),
                &profile,
            )
            .unwrap();

            let outcome = collector.collect_signatures().await;
            assert!(outcome.successful());
            assert!(outcome.failed_transactions().is_empty());
            assert_eq!(outcome.signatures_of_successful_transactions().len(), 10);
            assert_eq!(
                outcome
                    .successful_transactions()
                    .into_iter()
                    .map(|t| t.intent_hash)
                    .collect::<HashSet<_>>(),
                HashSet::from_iter([
                    t0.clone().intent_hash,
                    t1.clone().intent_hash,
                    t2.clone().intent_hash,
                ])
            );
            let st0 = outcome
                .successful_transactions()
                .into_iter()
                .find(|st| st.intent_hash == t0.intent_hash)
                .unwrap();

            assert_eq!(
                st0.signatures
                    .clone()
                    .into_iter()
                    .map(|s| s.owned_factor_instance().owner.clone())
                    .collect::<HashSet<_>>(),
                HashSet::from_iter([a0.address(), a1.address(), p0.address(), p1.address()])
            );

            let st1 = outcome
                .successful_transactions()
                .into_iter()
                .find(|st| st.intent_hash == t1.intent_hash)
                .unwrap();

            assert_eq!(
                st1.signatures
                    .clone()
                    .into_iter()
                    .map(|s| s.owned_factor_instance().owner.clone())
                    .collect::<HashSet<_>>(),
                HashSet::from_iter([a0.address(), a1.address(), a2.address()])
            );

            let st2 = outcome
                .successful_transactions()
                .into_iter()
                .find(|st| st.intent_hash == t2.intent_hash)
                .unwrap();

            assert_eq!(
                st2.signatures
                    .clone()
                    .into_iter()
                    .map(|s| s.owned_factor_instance().owner.clone())
                    .collect::<HashSet<_>>(),
                HashSet::from_iter([p0.address(), p1.address(), p2.address()])
            );

            // Assert sorted in increasing "friction order".
            assert_eq!(
                outcome
                    .signatures_of_successful_transactions()
                    .iter()
                    .map(|f| { f.factor_source_id().kind })
                    .collect::<IndexSet::<FactorSourceKind>>(),
                IndexSet::<FactorSourceKind>::from_iter([
                    FactorSourceKind::Device,
                    FactorSourceKind::Ledger
                ])
            );
        }

        #[derive(Clone, Debug)]
        struct Vector {
            simulated_user: SimulatedUser,
            expected: Expected,
        }
        #[derive(Clone, Debug, PartialEq, Eq)]
        struct Expected {
            successful_txs_signature_count: usize,
            signed_factor_source_kinds: IndexSet<FactorSourceKind>,
            expected_neglected_factor_source_count: usize,
        }
        async fn multi_securified_entities_with_sim_user(vector: Vector) {
            let factor_sources = &HDFactorSource::all();

            let a4 = &Account::a4();
            let a5 = &Account::a5();
            let a6 = &Account::a6();

            let p4 = &Persona::p4();
            let p5 = &Persona::p5();
            let p6 = &Persona::p6();

            let t0 = TransactionIntent::address_of([a5], [p5]);
            let t1 = TransactionIntent::address_of([a4, a5, a6], []);
            let t2 = TransactionIntent::address_of([a4, a6], [p4, p6]);
            let t3 = TransactionIntent::address_of([], [p4, p5, p6]);

            let profile = Profile::new(factor_sources.clone(), [a4, a5, a6], [p4, p5, p6]);

            let collector = SignaturesCollector::new(
                SigningFinishEarlyStrategy::default(),
                IndexSet::<TransactionIntent>::from_iter([
                    t0.clone(),
                    t1.clone(),
                    t2.clone(),
                    t3.clone(),
                ]),
                Arc::new(TestSignatureCollectingInteractors::new(
                    vector.simulated_user,
                )),
                &profile,
            )
            .unwrap();

            let outcome = collector.collect_signatures().await;

            assert_eq!(
                outcome.neglected_factor_sources().len(),
                vector.expected.expected_neglected_factor_source_count
            );

            assert!(outcome.successful());
            assert!(outcome.failed_transactions().is_empty());
            assert_eq!(
                outcome.signatures_of_successful_transactions().len(),
                vector.expected.successful_txs_signature_count
            );
            assert_eq!(
                outcome
                    .successful_transactions()
                    .into_iter()
                    .map(|t| t.intent_hash)
                    .collect::<HashSet<_>>(),
                HashSet::from_iter([
                    t0.clone().intent_hash,
                    t1.clone().intent_hash,
                    t2.clone().intent_hash,
                    t3.clone().intent_hash,
                ])
            );

            // Assert sorted in increasing "friction order".
            assert_eq!(
                outcome
                    .signatures_of_successful_transactions()
                    .iter()
                    .map(|f| { f.factor_source_id().kind })
                    .collect::<IndexSet::<FactorSourceKind>>(),
                vector.expected.signed_factor_source_kinds
            );
        }

        mod with_failure {
            use std::rc::Rc;

            use super::*;

            #[actix_rt::test]
            async fn multi_securified_entities() {
                multi_securified_entities_with_sim_user(Vector {
                    simulated_user: SimulatedUser::prudent_with_failures(
                        SimulatedFailures::with_simulated_failures([FactorSourceIDFromHash::fs1()]),
                    ),
                    expected: Expected {
                        successful_txs_signature_count: 24,
                        // We always end early
                        // `Device` FactorSourceKind never got used since it
                        // we are done after YubiKey.
                        signed_factor_source_kinds: IndexSet::<FactorSourceKind>::from_iter([
                            FactorSourceKind::Arculus,
                            FactorSourceKind::Yubikey,
                        ]),
                        expected_neglected_factor_source_count: 1,
                    },
                })
                .await;
            }

            #[actix_rt::test]
            async fn failed_threshold_successful_override() {
                sensible_env_logger::safe_init!();
                let factor_sources = &HDFactorSource::all();
                let a9 = &Account::a9();
                let tx0 = TransactionIntent::address_of([a9], []);

                let all_transactions = [tx0.clone()];

                let profile = Profile::new(factor_sources.clone(), [a9], []);

                let collector = SignaturesCollector::new(
                    SigningFinishEarlyStrategy::default(),
                    all_transactions,
                    Arc::new(TestSignatureCollectingInteractors::new(
                        SimulatedUser::prudent_with_failures(
                            SimulatedFailures::with_simulated_failures([
                                FactorSourceIDFromHash::fs1(),
                            ]),
                        ),
                    )),
                    &profile,
                )
                .unwrap();

                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                assert_eq!(
                    outcome
                        .successful_transactions()
                        .into_iter()
                        .map(|t| t.intent_hash.clone())
                        .collect_vec(),
                    vec![tx0.clone().intent_hash]
                );
                assert_eq!(
                    outcome
                        .all_signatures()
                        .into_iter()
                        .map(|s| s.factor_source_id())
                        .collect_vec(),
                    vec![FactorSourceIDFromHash::fs8()]
                );
            }

            #[actix_rt::test]
            async fn many_failing_tx() {
                let factor_sources = &HDFactorSource::all();
                let a0 = &Account::a0();
                let p3 = &Persona::p3();
                let failing_transactions = (0..100)
                    .map(|_| TransactionIntent::address_of([a0], []))
                    .collect::<IndexSet<_>>();
                let tx = TransactionIntent::address_of([], [p3]);
                let mut all_transactions = failing_transactions.clone();
                all_transactions.insert(tx.clone());

                let profile = Profile::new(factor_sources.clone(), [a0], [p3]);

                let collector = SignaturesCollector::new(
                    SigningFinishEarlyStrategy::default(),
                    all_transactions,
                    Arc::new(TestSignatureCollectingInteractors::new(
                        SimulatedUser::prudent_with_failures(
                            SimulatedFailures::with_simulated_failures([
                                FactorSourceIDFromHash::fs0(),
                            ]),
                        ),
                    )),
                    &profile,
                )
                .unwrap();

                let outcome = collector.collect_signatures().await;
                assert!(!outcome.successful());
                assert_eq!(
                    outcome
                        .failed_transactions()
                        .iter()
                        .map(|t| t.intent_hash.clone())
                        .collect_vec(),
                    failing_transactions
                        .iter()
                        .map(|t| t.intent_hash.clone())
                        .collect_vec()
                );

                assert_eq!(
                    outcome
                        .ids_of_neglected_factor_sources_failed()
                        .into_iter()
                        .collect_vec(),
                    vec![FactorSourceIDFromHash::fs0()]
                );

                assert!(outcome
                    .ids_of_neglected_factor_sources_skipped_by_user()
                    .is_empty());

                assert_eq!(
                    outcome
                        .successful_transactions()
                        .into_iter()
                        .map(|t| t.intent_hash)
                        .collect_vec(),
                    vec![tx.intent_hash]
                )
            }

            #[actix_rt::test]
            async fn same_tx_is_not_shown_to_user_in_case_of_already_failure() {
                sensible_env_logger::safe_init!();
                let factor_sources = HDFactorSource::all();

                let a7 = Account::a7();
                let a0 = Account::a0();

                let tx0 = TransactionIntent::new([a7.entity_address(), a0.entity_address()], []);
                let tx1 = TransactionIntent::new([a0.entity_address()], []);

                let profile = Profile::new(factor_sources.clone(), [&a7, &a0], []);

                type Tuple = (FactorSourceKind, IndexSet<InvalidTransactionIfNeglected>);
                type Tuples = Vec<Tuple>;
                let tuples = Rc::<RefCell<Tuples>>::new(RefCell::new(Tuples::default()));
                let tuples_clone = tuples.clone();
                let collector = SignaturesCollector::new(
                    SigningFinishEarlyStrategy::default(),
                    [tx0.clone(), tx1.clone()],
                    Arc::new(TestSignatureCollectingInteractors::new(
                        SimulatedUser::with_spy(
                            move |kind, invalid| {
                                let tuple = (kind, invalid);
                                let mut x = RefCell::borrow_mut(&tuples_clone);
                                x.push(tuple)
                            },
                            SimulatedUserMode::Prudent,
                            SimulatedFailures::with_simulated_failures([
                                FactorSourceIDFromHash::fs2(), // will cause any TX with a7 to fail
                            ]),
                        ),
                    )),
                    &profile,
                )
                .unwrap();

                let outcome = collector.collect_signatures().await;

                let tuples = tuples.borrow().clone();
                assert_eq!(
                    tuples,
                    vec![
                        (
                            FactorSourceKind::Ledger,
                            IndexSet::just(InvalidTransactionIfNeglected::new(
                                tx0.clone().intent_hash,
                                [a7.address()]
                            ))
                        ),
                        // Important that we do NOT display any mentioning of `tx0` here again!
                        (
                            FactorSourceKind::Device,
                            IndexSet::just(InvalidTransactionIfNeglected::new(
                                tx1.clone().intent_hash,
                                [a0.address()]
                            ))
                        ),
                    ]
                );

                assert!(!outcome.successful());
                assert_eq!(
                    outcome.ids_of_neglected_factor_sources_failed(),
                    IndexSet::<FactorSourceIDFromHash>::just(FactorSourceIDFromHash::fs2())
                );
                assert_eq!(
                    outcome.ids_of_neglected_factor_sources_irrelevant(),
                    IndexSet::<FactorSourceIDFromHash>::from_iter([
                        FactorSourceIDFromHash::fs6(),
                        FactorSourceIDFromHash::fs7(),
                        FactorSourceIDFromHash::fs8(),
                        FactorSourceIDFromHash::fs9()
                    ])
                );
                assert_eq!(
                    outcome
                        .successful_transactions()
                        .into_iter()
                        .map(|t| t.intent_hash)
                        .collect_vec(),
                    vec![tx1.intent_hash.clone()]
                );

                assert_eq!(
                    outcome
                        .failed_transactions()
                        .into_iter()
                        .map(|t| t.intent_hash)
                        .collect_vec(),
                    vec![tx0.intent_hash.clone()]
                );

                assert_eq!(outcome.all_signatures().len(), 1);

                assert!(outcome
                    .all_signatures()
                    .into_iter()
                    .map(|s| s.intent_hash().clone())
                    .all(|i| i == tx1.intent_hash));

                assert_eq!(
                    outcome
                        .all_signatures()
                        .into_iter()
                        .map(|s| s.derivation_path())
                        .collect_vec(),
                    vec![DerivationPath::new(
                        NetworkID::Mainnet,
                        CAP26EntityKind::Account,
                        CAP26KeyKind::TransactionSigning,
                        HDPathComponent::unsecurified_hardening_base_index(0)
                    )]
                )
            }
        }

        mod no_fail {
            use super::*;

            #[actix_rt::test]
            async fn multi_accounts_multi_personas_all_single_factor_controlled() {
                multi_accounts_multi_personas_all_single_factor_controlled_with_sim_user(
                    SimulatedUser::prudent_no_fail(),
                )
                .await;

                // Same result with lazy user, not able to skip without failures.
                multi_accounts_multi_personas_all_single_factor_controlled_with_sim_user(
                    SimulatedUser::lazy_sign_minimum([]),
                )
                .await
            }

            #[actix_rt::test]
            async fn multi_securified_entities() {
                multi_securified_entities_with_sim_user(Vector {
                    simulated_user: SimulatedUser::prudent_no_fail(),
                    expected: Expected {
                        successful_txs_signature_count: 32,
                        // We always end early
                        // `Device` FactorSourceKind never got used since it
                        // we are done after YubiKey.
                        signed_factor_source_kinds: IndexSet::<FactorSourceKind>::from_iter([
                            FactorSourceKind::Ledger,
                            FactorSourceKind::Arculus,
                            FactorSourceKind::Yubikey,
                        ]),
                        expected_neglected_factor_source_count: 0,
                    },
                })
                .await;

                multi_securified_entities_with_sim_user(Vector {
                    simulated_user: SimulatedUser::lazy_sign_minimum([]),
                    expected: Expected {
                        successful_txs_signature_count: 24,
                        // We always end early, this lazy user was able to skip
                        // Ledger.
                        signed_factor_source_kinds: IndexSet::<FactorSourceKind>::from_iter([
                            FactorSourceKind::Arculus,
                            FactorSourceKind::Yubikey,
                            FactorSourceKind::Device,
                        ]),
                        expected_neglected_factor_source_count: 2,
                    },
                })
                .await;
            }
        }
    }

    mod single_tx {
        use super::*;

        mod multiple_entities {
            use super::*;

            #[actix_rt::test]
            async fn prudent_user_single_tx_two_accounts_same_factor_source() {
                let collector = SignaturesCollector::test_prudent([TXToSign::new([
                    Account::unsecurified_mainnet(
                        "A0",
                        HierarchicalDeterministicFactorInstance::mainnet_tx(
                            CAP26EntityKind::Account,
                            HDPathComponent::unsecurified_hardening_base_index(0),
                            FactorSourceIDFromHash::fs0(),
                        ),
                    ),
                    Account::unsecurified_mainnet(
                        "A1",
                        HierarchicalDeterministicFactorInstance::mainnet_tx(
                            CAP26EntityKind::Account,
                            HDPathComponent::unsecurified_hardening_base_index(1),
                            FactorSourceIDFromHash::fs0(),
                        ),
                    ),
                ])]);

                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 2);
                assert_eq!(
                    signatures
                        .into_iter()
                        .map(|s| s.derivation_path())
                        .collect::<HashSet<_>>(),
                    [
                        DerivationPath::account_tx(
                            NetworkID::Mainnet,
                            HDPathComponent::unsecurified_hardening_base_index(0)
                        ),
                        DerivationPath::account_tx(
                            NetworkID::Mainnet,
                            HDPathComponent::unsecurified_hardening_base_index(1)
                        ),
                    ]
                    .into_iter()
                    .collect::<HashSet<_>>()
                )
            }

            #[actix_rt::test]
            async fn prudent_user_single_tx_two_accounts_different_factor_sources() {
                let collector = SignaturesCollector::test_prudent([TXToSign::new([
                    Account::a0(),
                    Account::a1(),
                ])]);

                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 2);
            }
        }

        mod single_entity {

            use super::*;

            async fn prudent_user_single_tx_e0<E: IsEntity>() {
                let collector = SignaturesCollector::test_prudent([TXToSign::new([E::e0()])]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 1);
            }

            async fn prudent_user_single_tx_e0_assert_correct_intent_hash_is_signed<E: IsEntity>() {
                let tx = TXToSign::new([E::e0()]);
                let collector = SignaturesCollector::test_prudent([tx.clone()]);
                let signature = &collector.collect_signatures().await.all_signatures()[0];
                assert_eq!(signature.intent_hash(), &tx.intent_hash);
                assert_eq!(signature.derivation_path().entity_kind, E::kind());
            }

            async fn prudent_user_single_tx_e0_assert_correct_owner_has_signed<E: IsEntity>() {
                let entity = E::e0();
                let tx = TXToSign::new([entity.clone()]);
                let collector = SignaturesCollector::test_prudent([tx.clone()]);
                let signature = &collector.collect_signatures().await.all_signatures()[0];
                assert_eq!(signature.owned_factor_instance().owner, entity.address());
            }

            async fn prudent_user_single_tx_e0_assert_correct_owner_factor_instance_signed<
                E: IsEntity,
            >() {
                let entity = E::e0();
                let tx = TXToSign::new([entity.clone()]);
                let collector = SignaturesCollector::test_prudent([tx.clone()]);
                let signature = &collector.collect_signatures().await.all_signatures()[0];

                assert_eq!(
                    signature.owned_factor_instance().factor_instance(),
                    entity
                        .security_state()
                        .all_factor_instances()
                        .first()
                        .unwrap()
                );
            }

            async fn prudent_user_single_tx_e1<E: IsEntity>() {
                let collector = SignaturesCollector::test_prudent([TXToSign::new([E::e1()])]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 1);
            }

            async fn prudent_user_single_tx_e2<E: IsEntity>() {
                let collector = SignaturesCollector::test_prudent([TXToSign::new([E::e2()])]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 1);
            }

            async fn prudent_user_single_tx_e3<E: IsEntity>() {
                let collector = SignaturesCollector::test_prudent([TXToSign::new([E::e3()])]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 1);
            }

            async fn prudent_user_single_tx_e4<E: IsEntity>() {
                let collector = SignaturesCollector::test_prudent([TXToSign::new([E::e4()])]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 2);
            }

            async fn prudent_user_single_tx_e5<E: IsEntity>() {
                let collector = SignaturesCollector::test_prudent([TXToSign::new([E::e5()])]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 1);
            }

            async fn prudent_user_single_tx_e6<E: IsEntity>() {
                let collector = SignaturesCollector::test_prudent([TXToSign::new([E::e6()])]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 1);
            }

            async fn prudent_user_single_tx_e7<E: IsEntity>() {
                let collector = SignaturesCollector::test_prudent([TXToSign::new([E::e7()])]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();

                assert_eq!(signatures.len(), 5);
            }

            async fn lazy_sign_minimum_user_single_tx_e0<E: IsEntity>() {
                let collector = SignaturesCollector::test_lazy_sign_minimum_no_failures([
                    TXToSign::new([E::e0()]),
                ]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 1);
            }

            async fn lazy_sign_minimum_user_single_tx_e1<E: IsEntity>() {
                let collector = SignaturesCollector::test_lazy_sign_minimum_no_failures([
                    TXToSign::new([E::e1()]),
                ]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 1);
            }

            async fn lazy_sign_minimum_user_single_tx_e2<E: IsEntity>() {
                let collector = SignaturesCollector::test_lazy_sign_minimum_no_failures([
                    TXToSign::new([E::e2()]),
                ]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 1);
            }

            async fn lazy_sign_minimum_user_e3<E: IsEntity>() {
                let collector = SignaturesCollector::test_lazy_sign_minimum_no_failures([
                    TXToSign::new([E::e3()]),
                ]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 1);
            }

            async fn lazy_sign_minimum_user_e4<E: IsEntity>() {
                let collector = SignaturesCollector::test_lazy_sign_minimum_no_failures([
                    TXToSign::new([E::e4()]),
                ]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 2);
            }

            async fn lazy_sign_minimum_user_e5<E: IsEntity>() {
                let collector = SignaturesCollector::test_lazy_sign_minimum_no_failures([
                    TXToSign::new([E::e5()]),
                ]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 1);
            }

            async fn lazy_sign_minimum_user_e6<E: IsEntity>() {
                let collector = SignaturesCollector::test_lazy_sign_minimum_no_failures([
                    TXToSign::new([E::e6()]),
                ]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();

                assert_eq!(signatures.len(), 2);
            }

            async fn lazy_sign_minimum_user_e7<E: IsEntity>() {
                let collector = SignaturesCollector::test_lazy_sign_minimum_no_failures([
                    TXToSign::new([E::e7()]),
                ]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();

                assert_eq!(signatures.len(), 5);
            }

            async fn lazy_sign_minimum_user_e5_last_factor_used<E: IsEntity>() {
                let entity = E::e5();
                let collector = SignaturesCollector::test_lazy_sign_minimum_no_failures([
                    TXToSign::new([entity.clone()]),
                ]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 1);

                let signature = &signatures[0];

                assert_eq!(
                    signature
                        .owned_factor_instance()
                        .factor_instance()
                        .factor_source_id,
                    FactorSourceIDFromHash::fs4()
                );

                assert_eq!(
                    outcome.ids_of_neglected_factor_sources(),
                    IndexSet::just(FactorSourceIDFromHash::fs1())
                )
            }

            async fn lazy_sign_minimum_all_known_factors_used_as_override_factors_signed_with_device_for_entity<
                E: IsEntity,
            >() {
                let collector = SignaturesCollector::test_lazy_sign_minimum_no_failures([
                    TXToSign::new([E::securified_mainnet("Alice", E::Address::sample(), || {
                        let idx = HDPathComponent::securifying_base_index(0);
                        MatrixOfFactorInstances::override_only(
                            HDFactorSource::all().into_iter().map(|f| {
                                HierarchicalDeterministicFactorInstance::mainnet_tx_account(
                                    idx,
                                    f.factor_source_id(),
                                )
                            }),
                        )
                    })]),
                ]);
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                let signatures = outcome.all_signatures();
                assert_eq!(signatures.len(), 2);

                assert!(signatures
                    .into_iter()
                    .all(|s| s.factor_source_id().kind == FactorSourceKind::Device));
            }

            async fn lazy_always_skip_user_single_tx_e0<E: IsEntity>() {
                let collector =
                    SignaturesCollector::test_lazy_always_skip([TXToSign::new([E::e0()])]);
                let outcome = collector.collect_signatures().await;
                assert!(!outcome.successful());
                let signatures = outcome.all_signatures();
                assert!(signatures.is_empty());
            }

            async fn fail_get_neglected_e0<E: IsEntity>() {
                let failing = IndexSet::<_>::just(FactorSourceIDFromHash::fs0());
                let collector = SignaturesCollector::test_prudent_with_failures(
                    [TXToSign::new([E::e0()])],
                    SimulatedFailures::with_simulated_failures(failing.clone()),
                );
                let outcome = collector.collect_signatures().await;
                assert!(!outcome.successful());
                let neglected = outcome.ids_of_neglected_factor_sources();
                assert_eq!(neglected, failing);
            }

            async fn lazy_always_skip_user_single_tx_e1<E: IsEntity>() {
                let collector =
                    SignaturesCollector::test_lazy_always_skip([TXToSign::new([E::e1()])]);
                let outcome = collector.collect_signatures().await;
                assert!(!outcome.successful());
                let signatures = outcome.all_signatures();
                assert!(signatures.is_empty());
            }

            async fn lazy_always_skip_user_single_tx_e2<E: IsEntity>() {
                let collector =
                    SignaturesCollector::test_lazy_always_skip([TXToSign::new([E::e2()])]);
                let outcome = collector.collect_signatures().await;
                assert!(!outcome.successful());
                let signatures = outcome.all_signatures();
                assert!(signatures.is_empty());
            }

            async fn lazy_always_skip_user_e3<E: IsEntity>() {
                let collector =
                    SignaturesCollector::test_lazy_always_skip([TXToSign::new([E::e3()])]);
                let outcome = collector.collect_signatures().await;
                assert!(!outcome.successful());
                let signatures = outcome.all_signatures();
                assert!(signatures.is_empty());
            }

            async fn lazy_always_skip_user_e4<E: IsEntity>() {
                let collector =
                    SignaturesCollector::test_lazy_always_skip([TXToSign::new([E::e4()])]);
                let outcome = collector.collect_signatures().await;
                assert!(!outcome.successful());
                let signatures = outcome.all_signatures();
                assert!(signatures.is_empty());
            }

            async fn lazy_always_skip_user_e5<E: IsEntity>() {
                let collector =
                    SignaturesCollector::test_lazy_always_skip([TXToSign::new([E::e5()])]);
                let outcome = collector.collect_signatures().await;
                assert!(!outcome.successful());
                let signatures = outcome.all_signatures();
                assert!(signatures.is_empty());
            }

            async fn lazy_always_skip_user_e6<E: IsEntity>() {
                let collector =
                    SignaturesCollector::test_lazy_always_skip([TXToSign::new([E::e6()])]);
                let outcome = collector.collect_signatures().await;
                assert!(!outcome.successful());
                let signatures = outcome.all_signatures();
                assert!(signatures.is_empty());
            }

            async fn lazy_always_skip_user_e7<E: IsEntity>() {
                let collector =
                    SignaturesCollector::test_lazy_always_skip([TXToSign::new([E::e7()])]);
                let outcome = collector.collect_signatures().await;
                assert!(!outcome.successful());
                let signatures = outcome.all_signatures();
                assert!(signatures.is_empty());
            }

            async fn failure_e0<E: IsEntity>() {
                let collector = SignaturesCollector::test_prudent_with_failures(
                    [TXToSign::new([E::e0()])],
                    SimulatedFailures::with_simulated_failures([FactorSourceIDFromHash::fs0()]),
                );
                let outcome = collector.collect_signatures().await;
                assert!(!outcome.successful());
                assert_eq!(
                    outcome
                        .ids_of_neglected_factor_sources_failed()
                        .into_iter()
                        .collect_vec(),
                    vec![FactorSourceIDFromHash::fs0()]
                );
                assert!(outcome
                    .ids_of_neglected_factor_sources_skipped_by_user()
                    .is_empty())
            }

            async fn failure_e5<E: IsEntity>() {
                let collector = SignaturesCollector::new_test(
                    SigningFinishEarlyStrategy::r#continue(),
                    HDFactorSource::all(),
                    [TXToSign::new([E::e5()])],
                    SimulatedUser::prudent_with_failures(
                        SimulatedFailures::with_simulated_failures([FactorSourceIDFromHash::fs4()]),
                    ),
                );

                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                assert_eq!(
                    outcome
                        .ids_of_neglected_factor_sources_failed()
                        .into_iter()
                        .collect_vec(),
                    vec![FactorSourceIDFromHash::fs4()]
                );
                assert!(outcome
                    .ids_of_neglected_factor_sources_skipped_by_user()
                    .is_empty());
            }

            async fn building_can_succeed_even_if_one_factor_source_fails_assert_ids_of_successful_tx_e4<
                E: IsEntity,
            >() {
                let collector = SignaturesCollector::test_prudent_with_failures(
                    [TXToSign::new([E::e4()])],
                    SimulatedFailures::with_simulated_failures([FactorSourceIDFromHash::fs3()]),
                );
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                assert_eq!(
                    outcome
                        .signatures_of_successful_transactions()
                        .into_iter()
                        .map(|f| f.factor_source_id())
                        .collect::<IndexSet<_>>(),
                    IndexSet::<_>::from_iter([
                        FactorSourceIDFromHash::fs0(),
                        FactorSourceIDFromHash::fs5()
                    ])
                );
            }

            async fn building_can_succeed_even_if_one_factor_source_fails_assert_ids_of_failed_tx_e4<
                E: IsEntity,
            >() {
                let collector = SignaturesCollector::test_prudent_with_failures(
                    [TXToSign::new([E::e4()])],
                    SimulatedFailures::with_simulated_failures([FactorSourceIDFromHash::fs3()]),
                );
                let outcome = collector.collect_signatures().await;
                assert!(outcome.successful());
                assert_eq!(
                    outcome.ids_of_neglected_factor_sources(),
                    IndexSet::<_>::just(FactorSourceIDFromHash::fs3())
                );
            }

            mod account {
                use super::*;
                type E = Account;

                #[actix_rt::test]
                async fn prudent_user_single_tx_a0() {
                    prudent_user_single_tx_e0::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_a0_assert_correct_intent_hash_is_signed() {
                    prudent_user_single_tx_e0_assert_correct_intent_hash_is_signed::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_a0_assert_correct_owner_has_signed() {
                    prudent_user_single_tx_e0_assert_correct_owner_has_signed::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_a0_assert_correct_owner_factor_instance_signed() {
                    prudent_user_single_tx_e0_assert_correct_owner_factor_instance_signed::<E>()
                        .await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_a1() {
                    prudent_user_single_tx_e1::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_a2() {
                    prudent_user_single_tx_e2::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_a3() {
                    prudent_user_single_tx_e3::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_a4() {
                    prudent_user_single_tx_e4::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_a5() {
                    prudent_user_single_tx_e5::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_a6() {
                    prudent_user_single_tx_e6::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_a7() {
                    prudent_user_single_tx_e7::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_single_tx_a0() {
                    lazy_sign_minimum_user_single_tx_e0::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_single_tx_a1() {
                    lazy_sign_minimum_user_single_tx_e1::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_single_tx_a2() {
                    lazy_sign_minimum_user_single_tx_e2::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_a3() {
                    lazy_sign_minimum_user_e3::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_a4() {
                    lazy_sign_minimum_user_e4::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_a5() {
                    lazy_sign_minimum_user_e5::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_a6() {
                    lazy_sign_minimum_user_e6::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_a7() {
                    lazy_sign_minimum_user_e7::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_a5_last_factor_used() {
                    lazy_sign_minimum_user_e5_last_factor_used::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_all_known_factors_used_as_override_factors_signed_with_device_for_account(
                ) {
                    lazy_sign_minimum_all_known_factors_used_as_override_factors_signed_with_device_for_entity::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_single_tx_a0() {
                    lazy_always_skip_user_single_tx_e0::<E>().await
                }

                #[actix_rt::test]
                async fn fail_get_skipped_a0() {
                    fail_get_neglected_e0::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_single_tx_a1() {
                    lazy_always_skip_user_single_tx_e1::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_single_tx_a2() {
                    lazy_always_skip_user_single_tx_e2::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_a3() {
                    lazy_always_skip_user_e3::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_a4() {
                    lazy_always_skip_user_e4::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_a5() {
                    lazy_always_skip_user_e5::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_a6() {
                    lazy_always_skip_user_e6::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_a7() {
                    lazy_always_skip_user_e7::<E>().await
                }

                #[actix_rt::test]
                async fn failure_a0() {
                    failure_e0::<E>().await
                }

                #[actix_rt::test]
                async fn failure_a5() {
                    failure_e5::<E>().await
                }

                #[actix_rt::test]
                async fn building_can_succeed_even_if_one_factor_source_fails_assert_ids_of_successful_tx(
                ) {
                    building_can_succeed_even_if_one_factor_source_fails_assert_ids_of_successful_tx_e4::<E>()
                        .await
                }

                #[actix_rt::test]
                async fn building_can_succeed_even_if_one_factor_source_fails_assert_ids_of_failed_tx(
                ) {
                    building_can_succeed_even_if_one_factor_source_fails_assert_ids_of_failed_tx_e4::<E>().await
                }
            }

            mod persona {
                use super::*;
                type E = Persona;

                #[actix_rt::test]
                async fn prudent_user_single_tx_p0() {
                    prudent_user_single_tx_e0::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_p0_assert_correct_intent_hash_is_signed() {
                    prudent_user_single_tx_e0_assert_correct_intent_hash_is_signed::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_p0_assert_correct_owner_has_signed() {
                    prudent_user_single_tx_e0_assert_correct_owner_has_signed::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_p0_assert_correct_owner_factor_instance_signed() {
                    prudent_user_single_tx_e0_assert_correct_owner_factor_instance_signed::<E>()
                        .await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_p1() {
                    prudent_user_single_tx_e1::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_p2() {
                    prudent_user_single_tx_e2::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_p3() {
                    prudent_user_single_tx_e3::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_p4() {
                    prudent_user_single_tx_e4::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_p5() {
                    prudent_user_single_tx_e5::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_p6() {
                    prudent_user_single_tx_e6::<E>().await
                }

                #[actix_rt::test]
                async fn prudent_user_single_tx_p7() {
                    prudent_user_single_tx_e7::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_single_tx_p0() {
                    lazy_sign_minimum_user_single_tx_e0::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_single_tx_p1() {
                    lazy_sign_minimum_user_single_tx_e1::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_single_tx_p2() {
                    lazy_sign_minimum_user_single_tx_e2::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_p3() {
                    lazy_sign_minimum_user_e3::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_p4() {
                    lazy_sign_minimum_user_e4::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_p5() {
                    lazy_sign_minimum_user_e5::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_p6() {
                    lazy_sign_minimum_user_e6::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_p7() {
                    lazy_sign_minimum_user_e7::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_user_p5_last_factor_used() {
                    lazy_sign_minimum_user_e5_last_factor_used::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_sign_minimum_all_known_factors_used_as_override_factors_signed_with_device_for_account(
                ) {
                    lazy_sign_minimum_all_known_factors_used_as_override_factors_signed_with_device_for_entity::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_single_tx_p0() {
                    lazy_always_skip_user_single_tx_e0::<E>().await
                }

                #[actix_rt::test]
                async fn fail_get_skipped_p0() {
                    fail_get_neglected_e0::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_single_tx_p1() {
                    lazy_always_skip_user_single_tx_e1::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_single_tx_p2() {
                    lazy_always_skip_user_single_tx_e2::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_p3() {
                    lazy_always_skip_user_e3::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_p4() {
                    lazy_always_skip_user_e4::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_p5() {
                    lazy_always_skip_user_e5::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_p6() {
                    lazy_always_skip_user_e6::<E>().await
                }

                #[actix_rt::test]
                async fn lazy_always_skip_user_p7() {
                    lazy_always_skip_user_e7::<E>().await
                }

                #[actix_rt::test]
                async fn failure_p0() {
                    failure_e0::<E>().await
                }

                #[actix_rt::test]
                async fn failure_p5() {
                    failure_e5::<E>().await
                }

                #[actix_rt::test]
                async fn building_can_succeed_even_if_one_factor_source_fails_assert_ids_of_successful_tx(
                ) {
                    building_can_succeed_even_if_one_factor_source_fails_assert_ids_of_successful_tx_e4::<E>()
                        .await
                }

                #[actix_rt::test]
                async fn building_can_succeed_even_if_one_factor_source_fails_assert_ids_of_failed_tx(
                ) {
                    building_can_succeed_even_if_one_factor_source_fails_assert_ids_of_failed_tx_e4::<E>().await
                }
            }
        }
    }
}
