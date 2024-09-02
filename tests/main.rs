use use_factors::prelude::*;

#[cfg(test)]
mod integration_test_signing {
    use core::panic;
    use std::sync::Arc;

    use indexmap::IndexSet;

    use super::*;

    struct TestSignInteractors;
    struct TestSignInteractor;

    #[async_trait::async_trait]
    impl PolyFactorSignInteractor for TestSignInteractor {
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
                        panic!("this test does not support neglecting factors")
                    }
                }
            }
            SignWithFactorsOutcome::signed(SignResponse::with_signatures(signatures))
        }
    }

    #[async_trait::async_trait]
    impl MonoFactorSignInteractor for TestSignInteractor {
        async fn sign(&self, request: MonoFactorSignRequest) -> SignWithFactorsOutcome {
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

    impl SignInteractors for TestSignInteractors {
        fn interactor_for(&self, kind: FactorSourceKind) -> SignInteractor {
            match kind {
                FactorSourceKind::Device => SignInteractor::mono(Arc::new(TestSignInteractor)),
                _ => SignInteractor::poly(Arc::new(TestSignInteractor)),
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
                    FI::mainnet_tx_account(i, f0.factor_source_id()),
                    FI::mainnet_tx_account(i, f1.factor_source_id()),
                    FI::mainnet_tx_account(i, f2.factor_source_id()),
                ],
                3,
            )
        });

        let bob = Account::securified_mainnet(1, "Bob", |i| {
            MatrixOfFactorInstances::threshold_only(
                [FI::mainnet_tx_account(i, f3.factor_source_id())],
                3,
            )
        });

        let satoshi = Persona::unsecurified_mainnet(2, "Satoshi", f4.factor_source_id());

        let tx0 = TransactionIntent::new([alice.entity_address()], []);
        let tx1 = TransactionIntent::new([alice.entity_address(), bob.entity_address()], []);
        let profile = Profile::new(IndexSet::from_iter([f0, f1, f2, f3]), [&alice, &non], []);
        let transactions = [tx0];
        let collector = SignaturesCollector::new(
            SigningFinishEarlyStrategy::default(),
            transactions,
            Arc::new(TestSignInteractors),
            &profile,
        )
        .unwrap();

        let outcome = collector.collect_signatures().await;

        assert!(outcome.successful());
    }
}
