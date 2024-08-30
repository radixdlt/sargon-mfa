#[cfg(test)]
use crate::prelude::*;

#[cfg(test)]
mod key_derivation_tests {

    use super::CAP26EntityKind::*;
    use super::CAP26KeyKind::*;
    use super::NetworkID::*;
    use super::*;

    #[actix_rt::test]
    async fn failure_unknown_factor() {
        let res = KeysCollector::new(
            IndexSet::new(),
            IndexMap::from_iter([(
                FactorSourceIDFromHash::fs0(),
                IndexSet::from_iter([DerivationPath::new(
                    Mainnet,
                    Account,
                    T9n,
                    HDPathComponent::securified(0),
                )]),
            )]),
            Arc::new(TestDerivationInteractors::default()),
        );
        assert!(matches!(res, Err(CommonError::UnknownFactorSource)));
    }

    #[actix_rt::test]
    async fn failure_from_interactor() {
        let factor_source = fs_at(0);
        let paths = [0, 1, 2]
            .into_iter()
            .map(|i| DerivationPath::at(Mainnet, Account, T9n, i))
            .collect::<IndexSet<_>>();
        let collector = KeysCollector::new(
            HDFactorSource::all(),
            [(factor_source.factor_source_id(), paths.clone())]
                .into_iter()
                .collect::<IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>>(),
            Arc::new(TestDerivationInteractors::fail()),
        )
        .unwrap();
        let outcome = collector.collect_keys().await;
        assert!(outcome.all_factors().is_empty())
    }

    mod multi_key {
        use super::*;

        #[actix_rt::test]
        async fn multi_keys_same_factor_source_different_indices() {
            let factor_source = fs_at(0);
            let paths = [0, 1, 2]
                .into_iter()
                .map(|i| DerivationPath::at(Mainnet, Account, T9n, i))
                .collect::<IndexSet<_>>();
            let collector =
                KeysCollector::new_test([(factor_source.factor_source_id(), paths.clone())]);
            let outcome = collector.collect_keys().await;
            assert_eq!(
                outcome
                    .all_factors()
                    .into_iter()
                    .map(|f| f.derivation_path())
                    .collect::<IndexSet<_>>(),
                paths
            );

            assert!(outcome
                .all_factors()
                .into_iter()
                .all(|f| f.factor_source_id == factor_source.factor_source_id()));
        }

        #[actix_rt::test]
        async fn multi_keys_multi_factor_sources_single_index_per() {
            let path = DerivationPath::account_tx(Mainnet, HDPathComponent::non_hardened(0));
            let paths = IndexSet::from_iter([path]);
            let factor_sources = HDFactorSource::all();

            let collector = KeysCollector::new_test(
                factor_sources
                    .iter()
                    .map(|f| (f.factor_source_id(), paths.clone()))
                    .collect_vec(),
            );
            let outcome = collector.collect_keys().await;
            assert_eq!(
                outcome
                    .all_factors()
                    .into_iter()
                    .map(|f| f.derivation_path())
                    .collect::<IndexSet<_>>(),
                paths
            );

            assert_eq!(
                outcome
                    .all_factors()
                    .into_iter()
                    .map(|f| f.factor_source_id)
                    .collect::<HashSet::<_>>(),
                factor_sources
                    .into_iter()
                    .map(|f| f.factor_source_id())
                    .collect::<HashSet::<_>>()
            );
        }

        #[actix_rt::test]
        async fn multi_keys_multi_factor_sources_multi_paths() {
            let paths = [0, 1, 2]
                .into_iter()
                .map(|i| DerivationPath::at(Mainnet, Account, T9n, i))
                .collect::<IndexSet<_>>();

            let factor_sources = HDFactorSource::all();

            let collector = KeysCollector::new_test(
                factor_sources
                    .iter()
                    .map(|f| (f.factor_source_id(), paths.clone()))
                    .collect_vec(),
            );
            let outcome = collector.collect_keys().await;

            assert_eq!(
                outcome
                    .all_factors()
                    .into_iter()
                    .map(|f| f.derivation_path())
                    .collect::<IndexSet<_>>(),
                paths
            );

            assert_eq!(
                outcome
                    .all_factors()
                    .into_iter()
                    .map(|f| f.factor_source_id)
                    .collect::<HashSet::<_>>(),
                factor_sources
                    .into_iter()
                    .map(|f| f.factor_source_id())
                    .collect::<HashSet::<_>>()
            );
        }

        #[actix_rt::test]
        async fn multi_keys_multi_factor_sources_multi_paths_complex() {
            let mut paths = IndexSet::new();

            paths.extend(
                [0, 1, 2]
                    .into_iter()
                    .map(|i| DerivationPath::at(Mainnet, Account, T9n, i)),
            );

            paths.extend(
                [0, 1, 2]
                    .into_iter()
                    .map(|i| DerivationPath::at(Stokenet, Account, T9n, i)),
            );

            paths.extend(
                [0, 1, 2]
                    .into_iter()
                    .map(|i| DerivationPath::at(Mainnet, Identity, T9n, i)),
            );

            paths.extend(
                [0, 1, 2]
                    .into_iter()
                    .map(|i| DerivationPath::at(Stokenet, Identity, T9n, i)),
            );

            paths.extend(
                [0, 1, 2]
                    .into_iter()
                    .map(|i| DerivationPath::at(Mainnet, Account, Rola, i)),
            );

            paths.extend(
                [0, 1, 2]
                    .into_iter()
                    .map(|i| DerivationPath::at(Stokenet, Account, Rola, i)),
            );

            paths.extend(
                [0, 1, 2]
                    .into_iter()
                    .map(|i| DerivationPath::at(Mainnet, Identity, Rola, i)),
            );

            paths.extend(
                [0, 1, 2]
                    .into_iter()
                    .map(|i| DerivationPath::at(Stokenet, Identity, Rola, i)),
            );

            paths.extend(
                [
                    0,
                    1,
                    2,
                    BIP32_SECURIFIED_HALF,
                    BIP32_SECURIFIED_HALF + 1,
                    BIP32_SECURIFIED_HALF + 2,
                ]
                .into_iter()
                .map(|i| DerivationPath::at(Mainnet, Account, T9n, i)),
            );

            paths.extend(
                [
                    0,
                    1,
                    2,
                    BIP32_SECURIFIED_HALF,
                    BIP32_SECURIFIED_HALF + 1,
                    BIP32_SECURIFIED_HALF + 2,
                ]
                .into_iter()
                .map(|i| DerivationPath::at(Stokenet, Account, T9n, i)),
            );

            paths.extend(
                [
                    0,
                    1,
                    2,
                    BIP32_SECURIFIED_HALF,
                    BIP32_SECURIFIED_HALF + 1,
                    BIP32_SECURIFIED_HALF + 2,
                ]
                .into_iter()
                .map(|i| DerivationPath::at(Mainnet, Identity, T9n, i)),
            );

            paths.extend(
                [
                    0,
                    1,
                    2,
                    BIP32_SECURIFIED_HALF,
                    BIP32_SECURIFIED_HALF + 1,
                    BIP32_SECURIFIED_HALF + 2,
                ]
                .into_iter()
                .map(|i| DerivationPath::at(Stokenet, Identity, T9n, i)),
            );

            paths.extend(
                [
                    0,
                    1,
                    2,
                    BIP32_SECURIFIED_HALF,
                    BIP32_SECURIFIED_HALF + 1,
                    BIP32_SECURIFIED_HALF + 2,
                ]
                .into_iter()
                .map(|i| DerivationPath::at(Mainnet, Account, Rola, i)),
            );

            paths.extend(
                [
                    0,
                    1,
                    2,
                    BIP32_SECURIFIED_HALF,
                    BIP32_SECURIFIED_HALF + 1,
                    BIP32_SECURIFIED_HALF + 2,
                ]
                .into_iter()
                .map(|i| DerivationPath::at(Stokenet, Account, Rola, i)),
            );

            paths.extend(
                [
                    0,
                    1,
                    2,
                    BIP32_SECURIFIED_HALF,
                    BIP32_SECURIFIED_HALF + 1,
                    BIP32_SECURIFIED_HALF + 2,
                ]
                .into_iter()
                .map(|i| DerivationPath::at(Mainnet, Identity, Rola, i)),
            );

            paths.extend(
                [
                    0,
                    1,
                    2,
                    BIP32_SECURIFIED_HALF,
                    BIP32_SECURIFIED_HALF + 1,
                    BIP32_SECURIFIED_HALF + 2,
                ]
                .into_iter()
                .map(|i| DerivationPath::at(Stokenet, Identity, Rola, i)),
            );

            let factor_sources = HDFactorSource::all();

            let collector = KeysCollector::new_test(
                factor_sources
                    .iter()
                    .map(|f| (f.factor_source_id(), paths.clone()))
                    .collect_vec(),
            );
            let outcome = collector.collect_keys().await;

            assert_eq!(
                outcome
                    .all_factors()
                    .into_iter()
                    .map(|f| f.derivation_path())
                    .collect::<IndexSet<_>>(),
                paths
            );

            assert!(outcome.all_factors().len() > 200);

            assert_eq!(
                outcome
                    .all_factors()
                    .into_iter()
                    .map(|f| f.factor_source_id)
                    .collect::<HashSet::<_>>(),
                factor_sources
                    .into_iter()
                    .map(|f| f.factor_source_id())
                    .collect::<HashSet::<_>>()
            );
        }
    }

    mod single_key {
        use super::*;

        struct Expected {
            index: HDPathComponent,
        }

        async fn do_test(
            key_space: KeySpace,
            factor_source: &HDFactorSource,
            network_id: NetworkID,
            entity_kind: CAP26EntityKind,
            key_kind: CAP26KeyKind,
            expected: Expected,
        ) {
            let collector =
                KeysCollector::with(factor_source, network_id, key_kind, entity_kind, key_space);

            let outcome = collector.collect_keys().await;
            let factors = outcome.all_factors();
            assert_eq!(factors.len(), 1);
            let factor = factors.first().unwrap();
            assert_eq!(
                factor.derivation_path(),
                DerivationPath::new(network_id, entity_kind, key_kind, expected.index)
            );
            assert_eq!(factor.factor_source_id, factor_source.factor_source_id());
        }

        mod securified {
            use super::*;

            async fn test(
                factor_source: &HDFactorSource,
                network_id: NetworkID,
                entity_kind: CAP26EntityKind,
                key_kind: CAP26KeyKind,
            ) {
                do_test(
                    KeySpace::Securified,
                    factor_source,
                    network_id,
                    entity_kind,
                    key_kind,
                    Expected {
                        index: HDPathComponent::non_hardened(BIP32_SECURIFIED_HALF),
                    },
                )
                .await
            }

            mod account {
                use super::*;

                async fn each_factor(network_id: NetworkID, key_kind: CAP26KeyKind) {
                    for factor_source in HDFactorSource::all().iter() {
                        test(factor_source, network_id, Account, key_kind).await
                    }
                }

                #[actix_rt::test]
                async fn single_first_account_mainnet_t9n() {
                    each_factor(Mainnet, T9n).await
                }
            }
        }

        mod unsecurified {
            use super::*;

            async fn test(
                factor_source: &HDFactorSource,
                network_id: NetworkID,
                entity_kind: CAP26EntityKind,
                key_kind: CAP26KeyKind,
            ) {
                do_test(
                    KeySpace::Unsecurified,
                    factor_source,
                    network_id,
                    entity_kind,
                    key_kind,
                    Expected {
                        index: HDPathComponent::non_hardened(0),
                    },
                )
                .await
            }

            mod account {
                use super::*;

                async fn each_factor(network_id: NetworkID, key_kind: CAP26KeyKind) {
                    for factor_source in HDFactorSource::all().iter() {
                        test(factor_source, network_id, Account, key_kind).await
                    }
                }

                #[actix_rt::test]
                async fn single_first_account_mainnet_t9n() {
                    each_factor(Mainnet, T9n).await
                }

                #[actix_rt::test]
                async fn single_first_account_stokenet_t9n() {
                    each_factor(Mainnet, T9n).await
                }

                #[actix_rt::test]
                async fn single_first_account_mainnet_rola() {
                    each_factor(Mainnet, Rola).await
                }

                #[actix_rt::test]
                async fn single_first_account_stokenet_rola() {
                    each_factor(Stokenet, Rola).await
                }
            }

            mod persona {
                use super::*;

                async fn each_factor(network_id: NetworkID, key_kind: CAP26KeyKind) {
                    for factor_source in HDFactorSource::all().iter() {
                        test(factor_source, network_id, Identity, key_kind).await
                    }
                }

                #[actix_rt::test]
                async fn single_first_persona_mainnet_t9n() {
                    each_factor(Mainnet, T9n).await
                }

                #[actix_rt::test]
                async fn single_first_persona_stokenet_t9n() {
                    each_factor(Mainnet, T9n).await
                }

                #[actix_rt::test]
                async fn single_first_persona_mainnet_rola() {
                    each_factor(Mainnet, Rola).await
                }

                #[actix_rt::test]
                async fn single_first_persona_stokenet_rola() {
                    each_factor(Stokenet, Rola).await
                }
            }
        }
    }
}
