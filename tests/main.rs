use use_factors::prelude::*;

#[cfg(test)]
mod integration_test_derivation {
    use std::sync::Arc;

    use indexmap::{IndexMap, IndexSet};

    use super::*;

    struct TestDerivationInteractors;
    struct TestDerivationInteractor;

    #[async_trait::async_trait]
    impl PolyFactorKeyDerivationInteractor for TestDerivationInteractor {
        async fn derive(
            &self,
            request: PolyFactorKeyDerivationRequest,
        ) -> Result<KeyDerivationResponse> {
            let mut keys = IndexMap::<
                FactorSourceIDFromHash,
                IndexSet<HierarchicalDeterministicFactorInstance>,
            >::new();
            for (f, req) in request.per_factor_source.into_iter() {
                let resp = <Self as MonoFactorKeyDerivationInteractor>::derive(self, req).await?;

                keys.insert(f, resp.per_factor_source.into_iter().next().unwrap().1);
            }
            Ok(KeyDerivationResponse::new(keys))
        }
    }

    #[async_trait::async_trait]
    impl MonoFactorKeyDerivationInteractor for TestDerivationInteractor {
        async fn derive(
            &self,
            request: MonoFactorKeyDerivationRequest,
        ) -> Result<KeyDerivationResponse> {
            let factor_source_id = request.clone().factor_source_id;
            Ok(KeyDerivationResponse::new(IndexMap::from_iter([(
                factor_source_id,
                request
                    .derivation_paths
                    .clone()
                    .into_iter()
                    .map(|p| {
                        HierarchicalDeterministicFactorInstance::mocked_with(p, &factor_source_id)
                    })
                    .collect(),
            )])))
        }
    }

    impl KeysDerivationInteractors for TestDerivationInteractors {
        fn interactor_for(&self, kind: FactorSourceKind) -> KeyDerivationInteractor {
            match kind {
                FactorSourceKind::Device => {
                    KeyDerivationInteractor::MonoFactor(Arc::new(TestDerivationInteractor))
                }
                _ => KeyDerivationInteractor::PolyFactor(Arc::new(TestDerivationInteractor)),
            }
        }
    }

    #[actix_rt::test]
    async fn valid() {
        let f0 = HDFactorSource::ledger();
        let f1 = HDFactorSource::device();
        let f2 = HDFactorSource::device();
        let f3 = HDFactorSource::arculus();

        let paths = IndexMap::<_, _>::from_iter([
            (
                f0.factor_source_id(),
                IndexSet::<_>::from_iter([
                    DerivationPath::account_tx(NetworkID::Mainnet, HDPathComponent::securified(0)),
                    DerivationPath::account_tx(NetworkID::Mainnet, HDPathComponent::securified(1)),
                    DerivationPath::account_tx(
                        NetworkID::Stokenet,
                        HDPathComponent::non_hardened(2),
                    ),
                ]),
            ),
            (
                f1.factor_source_id(),
                IndexSet::<_>::from_iter([DerivationPath::account_tx(
                    NetworkID::Stokenet,
                    HDPathComponent::non_hardened(3),
                )]),
            ),
            (
                f2.factor_source_id(),
                IndexSet::<_>::from_iter([DerivationPath::account_tx(
                    NetworkID::Mainnet,
                    HDPathComponent::non_hardened(4),
                )]),
            ),
            (
                f3.factor_source_id(),
                IndexSet::<_>::from_iter([DerivationPath::new(
                    NetworkID::Mainnet,
                    CAP26EntityKind::Identity,
                    CAP26KeyKind::Rola,
                    HDPathComponent::securified(5),
                )]),
            ),
        ]);

        let collector = KeysCollector::new(
            [f0, f1, f2, f3],
            paths.clone(),
            Arc::new(TestDerivationInteractors),
        )
        .unwrap();

        let outcome = collector.collect_keys().await;
        let factors = outcome.all_factors();
        assert_eq!(
            factors.len(),
            paths
                .clone()
                .into_iter()
                .flat_map(|(_, v)| v)
                .collect::<IndexSet<_>>()
                .len(),
        );
    }
}

#[cfg(test)]
mod integration_test_signing {
    use std::sync::Arc;

    use indexmap::IndexSet;

    use super::*;

    struct TestLazySignMinimumInteractors;
    struct TestLazySignMinimumInteractor;

    #[async_trait::async_trait]
    impl PolyFactorSignInteractor for TestLazySignMinimumInteractor {
        async fn sign(&self, request: PolyFactorSignRequest) -> SignWithFactorsOutcome {
            let mut signatures = IndexSet::<HDSignature>::new();
            for (_, req) in request.per_factor_source.iter() {
                let resp = <Self as MonoFactorSignInteractor>::sign(
                    self,
                    MonoFactorSignRequest::new(
                        req.clone(),
                        request.invalid_transactions_if_neglected.clone(),
                    ),
                )
                .await;

                match resp {
                    SignWithFactorsOutcome::Signed {
                        produced_signatures,
                    } => {
                        signatures.extend(
                            produced_signatures
                                .signatures
                                .into_iter()
                                .flat_map(|(_, xs)| xs)
                                .collect::<IndexSet<_>>(),
                        );
                    }
                    SignWithFactorsOutcome::Neglected(_) => {
                        return SignWithFactorsOutcome::Neglected(NeglectedFactors::new(
                            NeglectFactorReason::UserExplicitlySkipped,
                            request.factor_source_ids(),
                        ));
                    }
                }
            }
            SignWithFactorsOutcome::signed(SignResponse::with_signatures(signatures))
        }
    }

    #[async_trait::async_trait]
    impl MonoFactorSignInteractor for TestLazySignMinimumInteractor {
        async fn sign(&self, request: MonoFactorSignRequest) -> SignWithFactorsOutcome {
            if request.invalid_transactions_if_neglected.is_empty() {
                return SignWithFactorsOutcome::Neglected(NeglectedFactors::new(
                    NeglectFactorReason::UserExplicitlySkipped,
                    IndexSet::from_iter([request.input.factor_source_id]),
                ));
            }
            let signatures = request
                .input
                .per_transaction
                .into_iter()
                .flat_map(|r| {
                    r.signature_inputs()
                        .iter()
                        .map(|x| HDSignature::produced_signing_with_input(x.clone()))
                        .collect::<IndexSet<_>>()
                })
                .collect::<IndexSet<HDSignature>>();
            SignWithFactorsOutcome::Signed {
                produced_signatures: SignResponse::with_signatures(signatures),
            }
        }
    }

    impl SignInteractors for TestLazySignMinimumInteractors {
        fn interactor_for(&self, kind: FactorSourceKind) -> SignInteractor {
            match kind {
                FactorSourceKind::Device => {
                    SignInteractor::mono(Arc::new(TestLazySignMinimumInteractor))
                }
                _ => SignInteractor::poly(Arc::new(TestLazySignMinimumInteractor)),
            }
        }
    }

    #[actix_rt::test]
    async fn valid() {
        type FI = HierarchicalDeterministicFactorInstance;

        let f0 = HDFactorSource::ledger();
        let f1 = HDFactorSource::device();
        let f2 = HDFactorSource::device();
        let f3 = HDFactorSource::arculus();
        let f4 = HDFactorSource::off_device();

        let alice = Account::securified_mainnet(0, "Alice", |i| {
            MatrixOfFactorInstances::threshold_only(
                [
                    FI::mainnet_tx_account(i, f0.factor_source_id()), // SKIPPED
                    FI::mainnet_tx_account(i, f1.factor_source_id()),
                    FI::mainnet_tx_account(i, f2.factor_source_id()),
                ],
                2,
            )
        });

        let bob = Account::securified_mainnet(1, "Bob", |i| {
            MatrixOfFactorInstances::override_only([FI::mainnet_tx_account(
                i,
                f3.factor_source_id(),
            )])
        });

        let carol = Account::securified_mainnet(2, "Carol", |i| {
            MatrixOfFactorInstances::new(
                [FI::mainnet_tx_account(i, f2.factor_source_id())],
                1,
                [FI::mainnet_tx_account(i, f4.factor_source_id())],
            )
        });

        let satoshi = Persona::unsecurified_mainnet(1337, "Satoshi", f4.factor_source_id());

        let tx0 = TransactionIntent::new([alice.entity_address()], []);
        let tx1 = TransactionIntent::new(
            [
                alice.entity_address(),
                bob.entity_address(),
                carol.entity_address(),
            ],
            [satoshi.entity_address()],
        );
        let tx2 = TransactionIntent::new([bob.entity_address()], [satoshi.entity_address()]);

        let transactions = [tx0, tx1, tx2];

        let profile = Profile::new(
            IndexSet::from_iter([f0.clone(), f1, f2, f3, f4]),
            [&alice, &bob, &carol],
            [&satoshi],
        );

        let collector = SignaturesCollector::new(
            SigningFinishEarlyStrategy::default(),
            transactions,
            Arc::new(TestLazySignMinimumInteractors),
            &profile,
        )
        .unwrap();

        let outcome = collector.collect_signatures().await;

        assert!(outcome.successful());
        assert_eq!(outcome.signatures_of_successful_transactions().len(), 10);
        assert_eq!(
            outcome.ids_of_neglected_factor_sources(),
            IndexSet::<FactorSourceIDFromHash>::from_iter([f0.factor_source_id()])
        );
    }
}
