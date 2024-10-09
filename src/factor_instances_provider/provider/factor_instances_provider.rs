use std::sync::{Arc, RwLock};

use itertools::cloned;

use crate::prelude::*;

pub struct FactorInstancesProvider;

impl FactorInstancesProvider {
    pub async fn for_new_factor_source(
        cache: &mut Cache,
        profile: Option<Profile>,
        factor_source: HDFactorSource,
        network_id: NetworkID, // typically mainnet
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcome> {
        Self::with(
            network_id,
            cache,
            IndexSet::just(factor_source.clone()),
            IndexMap::kv(
                factor_source.factor_source_id(),
                QuantifiedNetworkIndexAgnosticPath {
                    quantity: 0,                                             // HACKY
                    agnostic_path: NetworkIndexAgnosticPath::account_veci(), // ANY really, important here is quantity `0`. This is HACKY, we really SHOULD switch to `DerivationTemplate` enum...
                },
            ),
            &NextDerivationEntityIndexAssigner::new(network_id, profile),
            interactors,
        )
        .await
    }

    pub async fn for_account_veci(
        cache: &mut Cache,
        profile: Option<Profile>,
        factor_source: HDFactorSource,
        network_id: NetworkID,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcome> {
        Self::with(
            network_id,
            cache,
            IndexSet::just(factor_source.clone()),
            IndexMap::kv(
                factor_source.factor_source_id(),
                QuantifiedNetworkIndexAgnosticPath {
                    quantity: 1,
                    agnostic_path: NetworkIndexAgnosticPath::account_veci(),
                },
            ),
            &NextDerivationEntityIndexAssigner::new(network_id, profile),
            interactors,
        )
        .await
    }

    pub async fn for_account_mfa(
        cache: &mut Cache,
        matrix_of_factor_sources: MatrixOfFactorSources,
        profile: Profile,
        accounts: IndexSet<AccountAddress>,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcome> {
        let factor_sources_to_use = matrix_of_factor_sources.all_factors();
        let factor_sources = profile.factor_sources.clone();
        assert!(
            factor_sources.is_superset(&factor_sources_to_use),
            "Missing FactorSources"
        );
        assert!(!accounts.is_empty(), "No accounts");
        assert!(
            accounts.iter().all(|a| profile.contains_account(a.clone())),
            "unknown account"
        );
        let network_id = accounts.first().unwrap().network_id();
        assert!(
            accounts.iter().all(|a| a.network_id() == network_id),
            "wrong network"
        );

        let entity_kind = CAP26EntityKind::Account;
        let key_kind = CAP26KeyKind::TransactionSigning;
        let key_space = KeySpace::Securified;

        let agnostic_path = NetworkIndexAgnosticPath {
            entity_kind,
            key_kind,
            key_space,
        };

        Self::with(
            network_id,
            cache,
            factor_sources,
            factor_sources_to_use
                .into_iter()
                .map(|f| {
                    (
                        f.factor_source_id(),
                        QuantifiedNetworkIndexAgnosticPath {
                            quantity: accounts.len(),
                            agnostic_path,
                        },
                    )
                })
                .collect(),
            &NextDerivationEntityIndexAssigner::new(network_id, Some(profile)),
            interactors,
        )
        .await
    }
}

impl FactorInstancesProvider {
    async fn with(
        network_id: NetworkID,
        cache: &mut Cache,
        factor_sources: IndexSet<HDFactorSource>,
        index_agnostic_path_and_quantity_per_factor_source: IndexMap<
            FactorSourceIDFromHash,
            QuantifiedNetworkIndexAgnosticPath,
        >,
        next_index_assigner: &NextDerivationEntityIndexAssigner,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcome> {
        let mut cloned_cache = cache.clone();

        let outcome = Self::with_copy_of_cache(
            network_id,
            &mut cloned_cache,
            factor_sources,
            index_agnostic_path_and_quantity_per_factor_source,
            next_index_assigner,
            interactors,
        )
        .await?;

        *cache = cloned_cache;

        for (f, stats) in outcome.per_factor.clone() {
            println!("🛡️ about to cache for factor {}", f);

            let mfa_left_in_cache = cache
                .peek_all_instances_of_factor_source(f)
                .unwrap_or_default()
                .get(&NetworkIndexAgnosticPath::account_mfa().on_network(NetworkID::Mainnet))
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|f| f.derivation_entity_index())
                .collect_vec();
            println!("🛡️ LEFT in cache {:?}", mfa_left_in_cache);
            println!(
                "🛡️ found_in_cache {:?}",
                stats
                    .found_in_cache
                    .into_iter()
                    .map(|f| f.derivation_entity_index())
                    .collect_vec()
            );
            println!(
                "🛡️ to_use_directly {:?}",
                stats
                    .to_use_directly
                    .into_iter()
                    .map(|f| f.derivation_entity_index())
                    .collect_vec()
            );
            println!(
                "🛡️ newly_derived {:?}",
                stats
                    .newly_derived
                    .into_iter()
                    .map(|f| f.derivation_entity_index())
                    .collect_vec()
            );
            println!(
                "🛡️ to_cache {:?}",
                stats
                    .to_cache
                    .into_iter()
                    .map(|f| f.derivation_entity_index())
                    .collect_vec()
            );
        }

        cache.insert_all(
            outcome
                .per_factor
                .clone()
                .into_iter()
                .map(|(k, v)| (k, v.to_cache))
                .collect::<IndexMap<_, _>>(),
        )?;

        Ok(outcome)
    }

    /// Supports loading many account vecis OR account mfa OR identity vecis OR identity mfa
    /// at once, does NOT support loading a mix of these. We COULD, but that would
    /// make the code more complex and we don't need it.
    #[allow(clippy::nonminimal_bool)]
    async fn with_copy_of_cache(
        network_id: NetworkID,
        cache: &mut Cache,
        factor_sources: IndexSet<HDFactorSource>,
        index_agnostic_path_and_quantity_per_factor_source: IndexMap<
            FactorSourceIDFromHash,
            QuantifiedNetworkIndexAgnosticPath,
        >,
        next_index_assigner: &NextDerivationEntityIndexAssigner,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcome> {
        // `pf` is short for `Per FactorSource`
        let mut pf_found_in_cache = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();

        // For every factor source found in this map, we derive the remaining
        // quantity as to satisfy the request PLUS we are deriving to fill the
        // cache since we are deriving anyway, i.e. derive for all `IndexAgnosticPath`
        // "presets" (Account Veci, Identity Veci, Account MFA, Identity MFA).
        let mut pf_quantity_remaining_not_satisfied_by_cache =
            IndexMap::<FactorSourceIDFromHash, QuantifiedNetworkIndexAgnosticPath>::new();

        let mut need_to_derive_more_instances: bool = false;

        for (factor_source_id, quantified_agnostic_path) in
            index_agnostic_path_and_quantity_per_factor_source.iter()
        {
            let from_cache: FactorInstances;
            let unsatisfied_quantity: usize;
            let cache_key =
                IndexAgnosticPath::from((network_id, quantified_agnostic_path.agnostic_path));
            let quantity = quantified_agnostic_path.quantity;
            match cache.remove(factor_source_id, &cache_key, quantity) {
                QuantityOutcome::Empty => {
                    need_to_derive_more_instances = true;
                    from_cache = FactorInstances::default();
                    unsatisfied_quantity = quantity;
                }
                QuantityOutcome::Partial {
                    instances,
                    remaining,
                } => {
                    need_to_derive_more_instances = true;
                    from_cache = instances;
                    unsatisfied_quantity = remaining;
                }
                QuantityOutcome::Full { instances } => {
                    need_to_derive_more_instances = false || need_to_derive_more_instances;
                    from_cache = instances;
                    unsatisfied_quantity = 0;
                }
            }
            if unsatisfied_quantity > 0 {
                pf_quantity_remaining_not_satisfied_by_cache.insert(
                    *factor_source_id,
                    QuantifiedNetworkIndexAgnosticPath {
                        quantity: unsatisfied_quantity,
                        agnostic_path: quantified_agnostic_path.agnostic_path,
                    },
                );
            }
            if !from_cache.is_empty() {
                pf_found_in_cache.insert(*factor_source_id, from_cache.clone());
            }
        }

        if !need_to_derive_more_instances {
            return Ok(FactorInstancesProviderOutcome::satisfied_by_cache(
                pf_found_in_cache,
            ));
        }

        Self::derive_more_instances(
            network_id,
            cache,
            next_index_assigner,
            interactors,
            factor_sources,
            index_agnostic_path_and_quantity_per_factor_source,
            pf_quantity_remaining_not_satisfied_by_cache,
            pf_found_in_cache,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn derive_more_instances(
        network_id: NetworkID,
        cache: &mut Cache,
        next_index_assigner: &NextDerivationEntityIndexAssigner,
        interactors: Arc<dyn KeysDerivationInteractors>,
        factor_sources: IndexSet<HDFactorSource>,
        index_agnostic_path_and_quantity_per_factor_source: IndexMap<
            FactorSourceIDFromHash,
            QuantifiedNetworkIndexAgnosticPath,
        >,
        pf_quantity_remaining_not_satisfied_by_cache: IndexMap<
            FactorSourceIDFromHash,
            QuantifiedNetworkIndexAgnosticPath,
        >,
        pf_found_in_cache: IndexMap<FactorSourceIDFromHash, FactorInstances>,
    ) -> Result<FactorInstancesProviderOutcome> {
        let mut pf_quantified_network_agnostic_paths_for_derivation = IndexMap::<
            FactorSourceIDFromHash,
            IndexSet<QuantifiedToCacheToUseNetworkIndexAgnosticPath>,
        >::new();

        // we will use directly what was found in clone, but later when
        // we derive more, we will add those to `pf_to_use_directly`, but
        // not to `pf_found_in_cache`, but we will include `pf_found_in_cache` for
        // unit tests.
        let mut pf_to_use_directly = pf_found_in_cache.clone();

        // used to filter out factor instances to use directly from the newly derived, based on
        // `index_agnostic_path_and_quantity_per_factor_source`
        let index_agnostic_paths_originally_requested =
            index_agnostic_path_and_quantity_per_factor_source
                .values()
                .cloned()
                .map(|q| IndexAgnosticPath::from((network_id, q.agnostic_path)))
                .collect::<IndexSet<_>>();

        for factor_source_id in index_agnostic_path_and_quantity_per_factor_source.keys() {
            let partial = pf_quantity_remaining_not_satisfied_by_cache
                .get(factor_source_id)
                .cloned();
            for preset in NetworkIndexAgnosticPath::all_presets() {
                let to_insert = partial
                    .and_then(|p| {
                        if p.agnostic_path == preset {
                            Some(QuantifiedToCacheToUseNetworkIndexAgnosticPath {
                                quantity: QuantityToCacheToUseDirectly::ToCacheToUseDirectly {
                                    remaining: p.quantity,
                                    extra_to_fill_cache: CACHE_FILLING_QUANTITY,
                                },
                                agnostic_path: p.agnostic_path,
                            })
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| {
                        let cache_key = preset.on_network(network_id);

                        let instances_in_cache = cache
                            .peek_all_instances_of_factor_source(*factor_source_id)
                            .and_then(|c| c.get(&cache_key).cloned())
                            .unwrap_or_default();

                        let number_of_instances_in_cache = instances_in_cache.len();
                        let instance_with_max_index =
                            instances_in_cache.into_iter().max_by(|a, b| {
                                a.derivation_entity_index()
                                    .cmp(&b.derivation_entity_index())
                            });
                        let number_of_instances_to_derive_to_fill_cache =
                            CACHE_FILLING_QUANTITY - number_of_instances_in_cache;

                        QuantifiedToCacheToUseNetworkIndexAgnosticPath {
                            quantity: QuantityToCacheToUseDirectly::OnlyCacheFilling {
                                fill_cache: number_of_instances_to_derive_to_fill_cache,
                                instance_with_max_index,
                            },
                            agnostic_path: preset,
                        }
                    });

                if let Some(existing) =
                    pf_quantified_network_agnostic_paths_for_derivation.get_mut(factor_source_id)
                {
                    existing.insert(to_insert);
                } else {
                    pf_quantified_network_agnostic_paths_for_derivation
                        .insert(*factor_source_id, IndexSet::just(to_insert));
                }
            }
        }

        // Map `NetworkAgnostic -> IndexAgnosticPath`
        let pf_quantified_index_agnostic_paths_for_derivation =
            pf_quantified_network_agnostic_paths_for_derivation
                .into_iter()
                .map(|(factor_source_id, quantified_network_agnostic_paths)| {
                    let index_agnostic_paths = quantified_network_agnostic_paths
                        .into_iter()
                        .map(|q| QuantifiedToCacheToUseIndexAgnosticPath {
                            agnostic_path: IndexAgnosticPath::from((network_id, q.agnostic_path)),
                            quantity: q.quantity,
                        })
                        .collect::<IndexSet<_>>();
                    (factor_source_id, index_agnostic_paths)
                })
                .collect::<IndexMap<_, _>>();

        // Now map from IndexAgnostic paths to index aware paths, a.k.a. DerivationPath
        // but ALSO we need to retain the information of how many factor instances of
        // the newly derived to append to the factor instances to use directly, and how many to cache.
        let paths = pf_quantified_index_agnostic_paths_for_derivation
            .clone()
            .into_iter()
            .map(|(f, agnostic_paths)| {
                let paths = agnostic_paths
                    .clone()
                    .into_iter()
                    .flat_map(|quantified_agnostic_path| {
                        // IMPORTANT! We are not mapping one `IndexAgnosticPath` to one `DerivationPath`, but
                        // rather we are mapping one `IndexAgnosticPath` to **MANY** `DerivationPath`s! Equal to
                        // the same number as the specified quantity!
                        (0..quantified_agnostic_path.quantity.total_quantity_to_derive())
                            .map(|_| {
                                let index_agnostic_path = quantified_agnostic_path.agnostic_path;

                                let index = next_index_assigner.next(
                                    f,
                                    index_agnostic_path,
                                    quantified_agnostic_path
                                        .quantity
                                        .max_index()
                                        .map(|max_index| OffsetFromCache::KnownMax {
                                            instance: max_index,
                                        })
                                        .unwrap_or(OffsetFromCache::FindMaxInRemoved {
                                            pf_found_in_cache: pf_found_in_cache.clone(),
                                        }),
                                );
                                let path = DerivationPath::from((
                                    quantified_agnostic_path.agnostic_path,
                                    index,
                                ));
                                println!("🎭 created path: {}", path);
                                path
                            })
                            .collect::<IndexSet<_>>()
                    })
                    .collect::<IndexSet<_>>();
                (f, paths)
            })
            .collect::<IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>>();

        let mut pf_to_cache = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();

        let mut pf_newly_derived = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();

        let keys_collector = KeysCollector::new(factor_sources, paths, interactors)?;
        let outcome = keys_collector.collect_keys().await;

        for (f, instances) in outcome.factors_by_source.into_iter() {
            pf_newly_derived.insert(f, instances.clone().into());
            let instances: Vec<HierarchicalDeterministicFactorInstance> =
                instances.into_iter().collect_vec();
            let mut to_use_directly = IndexSet::<HierarchicalDeterministicFactorInstance>::new();

            // to use directly
            let remaining = pf_quantity_remaining_not_satisfied_by_cache
                .get(&f)
                .map(|q| q.quantity)
                .unwrap_or(0);

            let mut to_cache = IndexSet::<HierarchicalDeterministicFactorInstance>::new();
            for instance in instances {
                // IMPORTANT: `instance_matches_original_request` SHOULD ALWAYS be
                // `false` if we used the `FactorInstancesProvider` for purpose "PRE_DERIVE_KEYS_FOR_NEW_FACTOR_SOURCE",
                // for which we don't want to use any factor instance directly.
                // By "original request" we mean, if we used the `FactorInstancesProvider` for purpose
                // "account_veci", then we check here that the derivation path of `instance` matches
                // that of `NetworkIndexAgnostic::account_veci()`, if it does, then that instance
                // "matches the original request", but if it is an instances for "identity_veci" or
                // "account_mfa" or "identity_mfa", then it does not match the original request, and
                // it should not be used directly, rather be put into the cache.
                let instance_matches_original_request = index_agnostic_paths_originally_requested
                    .contains(&instance.derivation_path().agnostic());

                if instance_matches_original_request {
                    // we can get MULTIPLE hits here, since we are deriving multiple factors per
                    // agnostic path..

                    if to_use_directly.len() < remaining {
                        to_use_directly.insert(instance);
                    } else {
                        to_cache.insert(instance);
                    }
                } else {
                    to_cache.insert(instance);
                }
            }

            pf_to_cache.insert(f, to_cache.into());
            if let Some(existing_to_use_directly) = pf_to_use_directly.get_mut(&f) {
                existing_to_use_directly.extend(to_use_directly.into_iter());
            } else {
                pf_to_use_directly.insert(f, FactorInstances::from(to_use_directly));
            }
        }

        let outcome = FactorInstancesProviderOutcome::transpose(
            pf_to_cache,
            pf_to_use_directly
                .into_iter()
                .map(|(k, v)| (k, v.clone()))
                .collect(),
            pf_found_in_cache,
            pf_newly_derived,
        );
        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {

    use std::ops::Add;

    use super::*;

    type Sut = FactorInstancesProvider;

    #[actix_rt::test]
    async fn create_accounts_when_last_is_used_cache_is_fill_only_with_account_vecis_and_if_profile_is_used_a_new_account_is_created(
    ) {
        let (mut os, bdfs) = SargonOS::with_bdfs().await;
        for i in 0..CACHE_FILLING_QUANTITY {
            let name = format!("Acco {}", i);
            let (acco, stats) = os
                .new_mainnet_account_with_bdfs(name.clone())
                .await
                .unwrap();
            assert_eq!(acco.name, name);
            assert_eq!(stats.to_cache.len(), 0);
            assert_eq!(stats.newly_derived.len(), 0);
        }
        assert_eq!(
            os.profile_snapshot().get_accounts().len(),
            CACHE_FILLING_QUANTITY
        );

        let (acco, stats) = os
            .new_mainnet_account_with_bdfs("newly derive")
            .await
            .unwrap();

        assert_eq!(
            os.profile_snapshot().get_accounts().len(),
            CACHE_FILLING_QUANTITY + 1
        );

        assert_eq!(stats.to_cache.len(), CACHE_FILLING_QUANTITY);
        assert_eq!(stats.newly_derived.len(), CACHE_FILLING_QUANTITY + 1);

        assert_eq!(
            acco.as_unsecurified()
                .unwrap()
                .factor_instance()
                .derivation_entity_index(),
            HDPathComponent::unsecurified_hardening_base_index(30)
        );
        assert!(os
            .cache_snapshot()
            .is_full(NetworkID::Mainnet, bdfs.factor_source_id()));

        // and another one
        let (acco, stats) = os
            .new_mainnet_account_with_bdfs("newly derive 2")
            .await
            .unwrap();

        assert_eq!(
            os.profile_snapshot().get_accounts().len(),
            CACHE_FILLING_QUANTITY + 2
        );

        assert_eq!(stats.to_cache.len(), 0);
        assert_eq!(stats.newly_derived.len(), 0);

        assert_eq!(
            acco.as_unsecurified()
                .unwrap()
                .factor_instance()
                .derivation_entity_index(),
            HDPathComponent::unsecurified_hardening_base_index(31)
        );
        assert!(
            !os.cache_snapshot()
                .is_full(NetworkID::Mainnet, bdfs.factor_source_id()),
            "just consumed one, so not full"
        );
    }

    #[actix_rt::test]
    async fn cache_is_always_filled_account_veci_then_after_all_used_we_start_over_at_zero_if_no_profile_is_used(
    ) {
        let network = NetworkID::Mainnet;
        let bdfs = HDFactorSource::sample();
        let mut cache = Cache::default();

        let outcome = Sut::for_account_veci(
            &mut cache,
            None,
            bdfs.clone(),
            network,
            Arc::new(TestDerivationInteractors::default()),
        )
        .await
        .unwrap();

        let per_factor = outcome.per_factor;
        assert_eq!(per_factor.len(), 1);
        let outcome = per_factor.get(&bdfs.factor_source_id()).unwrap().clone();
        assert_eq!(outcome.factor_source_id, bdfs.factor_source_id());

        assert_eq!(outcome.found_in_cache.len(), 0);

        assert_eq!(
            outcome.to_cache.len(),
            NetworkIndexAgnosticPath::all_presets().len() * CACHE_FILLING_QUANTITY
        );

        assert_eq!(
            outcome.newly_derived.len(),
            NetworkIndexAgnosticPath::all_presets().len() * CACHE_FILLING_QUANTITY + 1
        );

        let instances_used_directly = outcome.to_use_directly.factor_instances();
        assert_eq!(instances_used_directly.len(), 1);
        let instances_used_directly = instances_used_directly.first().unwrap();

        assert_eq!(
            instances_used_directly.derivation_entity_index(),
            HDPathComponent::Hardened(HDPathComponentHardened::Unsecurified(
                UnsecurifiedIndex::unsecurified_hardening_base_index(0)
            ))
        );

        cache.assert_is_full(network, bdfs.factor_source_id());

        let cached = cache
            .peek_all_instances_of_factor_source(bdfs.factor_source_id())
            .unwrap();

        let account_veci_paths = cached
            .clone()
            .get(&NetworkIndexAgnosticPath::account_veci().on_network(network))
            .unwrap()
            .factor_instances()
            .into_iter()
            .map(|x| x.derivation_path())
            .collect_vec();

        assert_eq!(account_veci_paths.len(), CACHE_FILLING_QUANTITY);

        assert!(account_veci_paths
            .iter()
            .all(|x| x.entity_kind == CAP26EntityKind::Account
                && x.network_id == network
                && x.key_space() == KeySpace::Unsecurified
                && x.key_kind == CAP26KeyKind::TransactionSigning));

        let account_veci_indices = account_veci_paths
            .into_iter()
            .map(|x| x.index)
            .collect_vec();

        assert_eq!(
            account_veci_indices.first().unwrap().clone(),
            HDPathComponent::unsecurified_hardening_base_index(1)
        );

        assert_eq!(
            account_veci_indices.last().unwrap().clone(),
            HDPathComponent::unsecurified_hardening_base_index(30)
        );

        let account_mfa_paths = cached
            .clone()
            .get(&NetworkIndexAgnosticPath::account_mfa().on_network(network))
            .unwrap()
            .factor_instances()
            .into_iter()
            .map(|x| x.derivation_path())
            .collect_vec();

        assert!(account_mfa_paths
            .iter()
            .all(|x| x.entity_kind == CAP26EntityKind::Account
                && x.network_id == network
                && x.key_space() == KeySpace::Securified
                && x.key_kind == CAP26KeyKind::TransactionSigning));

        let account_mfa_indices = account_mfa_paths.into_iter().map(|x| x.index).collect_vec();

        assert_eq!(
            account_mfa_indices.first().unwrap().clone(),
            HDPathComponent::securifying_base_index(0)
        );

        assert_eq!(
            account_mfa_indices.last().unwrap().clone(),
            HDPathComponent::securifying_base_index(29)
        );

        let identity_mfa_paths = cached
            .clone()
            .get(&NetworkIndexAgnosticPath::identity_mfa().on_network(network))
            .unwrap()
            .factor_instances()
            .into_iter()
            .map(|x| x.derivation_path())
            .collect_vec();

        assert!(identity_mfa_paths
            .iter()
            .all(|x| x.entity_kind == CAP26EntityKind::Identity
                && x.network_id == network
                && x.key_space() == KeySpace::Securified
                && x.key_kind == CAP26KeyKind::TransactionSigning));

        let identity_mfa_indices = identity_mfa_paths
            .into_iter()
            .map(|x| x.index)
            .collect_vec();

        assert_eq!(
            identity_mfa_indices.first().unwrap().clone(),
            HDPathComponent::securifying_base_index(0)
        );

        assert_eq!(
            identity_mfa_indices.last().unwrap().clone(),
            HDPathComponent::securifying_base_index(29)
        );

        let identity_veci_paths = cached
            .clone()
            .get(&NetworkIndexAgnosticPath::identity_veci().on_network(network))
            .unwrap()
            .factor_instances()
            .into_iter()
            .map(|x| x.derivation_path())
            .collect_vec();

        assert!(identity_veci_paths
            .iter()
            .all(|x| x.entity_kind == CAP26EntityKind::Identity
                && x.network_id == network
                && x.key_space() == KeySpace::Unsecurified
                && x.key_kind == CAP26KeyKind::TransactionSigning));

        let identity_veci_indices = identity_veci_paths
            .into_iter()
            .map(|x| x.index)
            .collect_vec();

        assert_eq!(
            identity_veci_indices.first().unwrap().clone(),
            HDPathComponent::unsecurified_hardening_base_index(0)
        );

        assert_eq!(
            identity_veci_indices.last().unwrap().clone(),
            HDPathComponent::unsecurified_hardening_base_index(29)
        );

        // lets create another account (same network, same factor source)

        let outcome = Sut::for_account_veci(
            &mut cache,
            None,
            bdfs.clone(),
            network,
            Arc::new(TestDerivationInteractors::default()),
        )
        .await
        .unwrap();

        let per_factor = outcome.per_factor;
        assert_eq!(per_factor.len(), 1);
        let outcome = per_factor.get(&bdfs.factor_source_id()).unwrap().clone();
        assert_eq!(outcome.factor_source_id, bdfs.factor_source_id());

        assert_eq!(outcome.found_in_cache.len(), 1); // This time we found in cache

        assert_eq!(outcome.to_cache.len(), 0);

        assert_eq!(outcome.newly_derived.len(), 0);

        let instances_used_directly = outcome.to_use_directly.factor_instances();
        assert_eq!(instances_used_directly.len(), 1);
        let instances_used_directly = instances_used_directly.first().unwrap();

        assert_eq!(
            instances_used_directly.derivation_entity_index(),
            HDPathComponent::Hardened(HDPathComponentHardened::Unsecurified(
                UnsecurifiedIndex::unsecurified_hardening_base_index(1) // Next one!
            ))
        );

        assert!(!cache.is_full(network, bdfs.factor_source_id())); // not full anymore, since we just used a veci

        let cached = cache
            .peek_all_instances_of_factor_source(bdfs.factor_source_id())
            .unwrap();

        let account_veci_paths = cached
            .clone()
            .get(&NetworkIndexAgnosticPath::account_veci().on_network(network))
            .unwrap()
            .factor_instances()
            .into_iter()
            .map(|x| x.derivation_path())
            .collect_vec();

        assert_eq!(account_veci_paths.len(), CACHE_FILLING_QUANTITY - 1);

        assert!(account_veci_paths
            .iter()
            .all(|x| x.entity_kind == CAP26EntityKind::Account
                && x.network_id == network
                && x.key_space() == KeySpace::Unsecurified
                && x.key_kind == CAP26KeyKind::TransactionSigning));

        let account_veci_indices = account_veci_paths
            .into_iter()
            .map(|x| x.index)
            .collect_vec();

        assert_eq!(
            account_veci_indices.first().unwrap().clone(),
            HDPathComponent::unsecurified_hardening_base_index(2) // first is not `1` anymore
        );

        assert_eq!(
            account_veci_indices.last().unwrap().clone(),
            HDPathComponent::unsecurified_hardening_base_index(30)
        );

        // create 29 more accounts, then we should be able to crate one more which should ONLY derive
        // more instances for ACCOUNT VECI, and not Identity Veci, Identity MFA and Account MFA, since that is
        // not needed.
        for _ in 0..29 {
            let outcome = Sut::for_account_veci(
                &mut cache,
                None,
                bdfs.clone(),
                network,
                Arc::new(TestDerivationInteractors::default()),
            )
            .await
            .unwrap();

            let per_factor = outcome.per_factor;
            assert_eq!(per_factor.len(), 1);
            let outcome = per_factor.get(&bdfs.factor_source_id()).unwrap().clone();
            assert_eq!(outcome.factor_source_id, bdfs.factor_source_id());

            assert_eq!(outcome.found_in_cache.len(), 1);
            assert_eq!(outcome.to_cache.len(), 0);
            assert_eq!(outcome.newly_derived.len(), 0);
        }

        let cached = cache
            .peek_all_instances_of_factor_source(bdfs.factor_source_id())
            .unwrap();

        assert!(
            cached
                .get(&NetworkIndexAgnosticPath::account_veci().on_network(network))
                .is_none(),
            "should have used the last instance..."
        );

        // Great, now lets create one more account, and this time we should derive more instances for
        // it. We should derive 31 instances, 30 for account veci to cache and 1 to use directly.
        // we should NOT derive more instances for Identity Veci, Identity MFA and Account MFA, since
        // that cache is already full.
        let outcome = Sut::for_account_veci(
            &mut cache,
            None,
            bdfs.clone(),
            network,
            Arc::new(TestDerivationInteractors::default()),
        )
        .await
        .unwrap();

        let per_factor = outcome.per_factor;
        assert_eq!(per_factor.len(), 1);
        let outcome = per_factor.get(&bdfs.factor_source_id()).unwrap().clone();
        assert_eq!(outcome.factor_source_id, bdfs.factor_source_id());

        assert_eq!(outcome.found_in_cache.len(), 0);
        assert_eq!(outcome.to_cache.len(), CACHE_FILLING_QUANTITY); // ONLY 30, not 120...
        assert_eq!(outcome.newly_derived.len(), CACHE_FILLING_QUANTITY + 1);

        let instances_used_directly = outcome.to_use_directly.factor_instances();
        assert_eq!(instances_used_directly.len(), 1);
        let instances_used_directly = instances_used_directly.first().unwrap();

        assert_eq!(
            instances_used_directly.derivation_entity_index(),
            HDPathComponent::Hardened(HDPathComponentHardened::Unsecurified(
                UnsecurifiedIndex::unsecurified_hardening_base_index(0) // IMPORTANT! Index 0 is used again! Why?! Well because are not using a Profile here, and we are not eagerly filling cache just before we are using the last index.
            ))
        );
    }

    struct SargonOS {
        cache: Cache,
        profile: RwLock<Profile>,
    }

    impl SargonOS {
        pub fn profile_snapshot(&self) -> Profile {
            self.profile.try_read().unwrap().clone()
        }
        pub fn new() -> Self {
            Arc::new(TestDerivationInteractors::default());
            Self {
                cache: Cache::default(),
                profile: RwLock::new(Profile::default()),
            }
        }
        pub async fn with_bdfs() -> (Self, HDFactorSource) {
            let mut self_ = Self::new();
            let bdfs = HDFactorSource::device();
            self_.add_factor_source(bdfs.clone()).await.unwrap();
            (self_, bdfs)
        }

        pub fn cache_snapshot(&self) -> Cache {
            self.cache.clone()
        }

        pub fn clear_cache(&mut self) {
            println!("💣 CLEAR CACHE");
            self.cache = Cache::default()
        }

        pub async fn new_mainnet_account_with_bdfs(
            &mut self,
            name: impl AsRef<str>,
        ) -> Result<(Account, FactorInstancesProviderOutcomeForFactor)> {
            self.new_account_with_bdfs(NetworkID::Mainnet, name).await
        }

        pub async fn new_account_with_bdfs(
            &mut self,
            network: NetworkID,
            name: impl AsRef<str>,
        ) -> Result<(Account, FactorInstancesProviderOutcomeForFactor)> {
            let bdfs = self.profile_snapshot().bdfs();
            self.new_account(bdfs, network, name).await
        }

        pub async fn new_account(
            &mut self,
            factor_source: HDFactorSource,
            network: NetworkID,
            name: impl AsRef<str>,
        ) -> Result<(Account, FactorInstancesProviderOutcomeForFactor)> {
            let profile_snapshot = self.profile_snapshot();
            let outcome = Sut::for_account_veci(
                &mut self.cache,
                Some(profile_snapshot),
                factor_source.clone(),
                network,
                Arc::new(TestDerivationInteractors::default()),
            )
            .await
            .unwrap();

            let outcome_for_factor = outcome
                .per_factor
                .get(&factor_source.factor_source_id())
                .unwrap()
                .clone();

            let instances_to_use_directly = outcome_for_factor.to_use_directly.clone();

            assert_eq!(instances_to_use_directly.len(), 1);
            let instance = instances_to_use_directly.first().unwrap();

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
            self.profile
                .try_write()
                .unwrap()
                .add_account(&account)
                .unwrap();
            Ok((account, outcome_for_factor))
        }

        pub async fn securify(
            &mut self,
            accounts: Accounts,
            shield: MatrixOfFactorSources,
        ) -> Result<(SecurifiedAccounts, FactorInstancesProviderOutcome)> {
            println!(
                "🛡️ Securifying accounts: '{:?}'",
                accounts.clone().into_iter().map(|x| x.name()).collect_vec()
            );

            let profile_snapshot = self.profile_snapshot();

            let outcome = Sut::for_account_mfa(
                &mut self.cache,
                shield.clone(),
                profile_snapshot,
                accounts
                    .clone()
                    .into_iter()
                    .map(|a| a.entity_address())
                    .collect(),
                Arc::new(TestDerivationInteractors::default()),
            )
            .await
            .unwrap();

            let mut instance_per_factor = outcome
                .clone()
                .per_factor
                .into_iter()
                .map(|(k, outcome_per_factor)| (k, outcome_per_factor.to_use_directly))
                .collect::<IndexMap<FactorSourceIDFromHash, FactorInstances>>();

            println!(
                "🧵🎉 securifying: #{} accounts, got: #{} factor instances",
                accounts.len(),
                instance_per_factor
                    .values()
                    .map(|x| x.len())
                    .reduce(Add::add)
                    .unwrap_or_default()
            );

            // Now we need to map the flat set of instances into many MatrixOfFactorInstances, and assign
            // one to each account
            let updated_accounts = accounts
                        .clone()
                        .into_iter()
                        .map(|a| {
                            let matrix_of_instances =
                            MatrixOfFactorInstances::fulfilling_matrix_of_factor_sources_with_instances(
                                &mut instance_per_factor,
                                shield.clone(),
                            )
                            .unwrap();
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
                    .update_account(&account.account())
                    .unwrap();
            }
            assert!(
                instance_per_factor.values().all(|x| x.is_empty()),
                "should have used all instances, but have unused instances: {:?}",
                instance_per_factor
            );
            SecurifiedAccounts::new(accounts.network_id(), updated_accounts).map(|x| (x, outcome))
        }

        async fn add_factor_source(&mut self, factor_source: HDFactorSource) -> Result<()> {
            let profile_snapshot = self.profile_snapshot();
            assert!(
                !profile_snapshot
                    .factor_sources
                    .iter()
                    .any(|x| x.factor_source_id() == factor_source.factor_source_id()),
                "factor already in Profile"
            );
            let outcome = Sut::for_new_factor_source(
                &mut self.cache,
                Some(profile_snapshot),
                factor_source.clone(),
                NetworkID::Mainnet,
                Arc::new(TestDerivationInteractors::default()),
            )
            .await
            .unwrap();

            let per_factor = outcome.per_factor;
            let outcome = per_factor
                .get(&factor_source.factor_source_id())
                .unwrap()
                .clone();
            assert_eq!(outcome.factor_source_id, factor_source.factor_source_id());

            assert_eq!(outcome.found_in_cache.len(), 0);

            assert_eq!(
                outcome.to_cache.len(),
                NetworkIndexAgnosticPath::all_presets().len() * CACHE_FILLING_QUANTITY
            );

            assert_eq!(
                outcome.newly_derived.len(),
                NetworkIndexAgnosticPath::all_presets().len() * CACHE_FILLING_QUANTITY
            );

            self.profile
                .try_write()
                .unwrap()
                .add_factor_source(factor_source.clone())
                .unwrap();

            Ok(())
        }
    }

    #[actix_rt::test]
    async fn add_factor_source() {
        let mut os = SargonOS::new();
        assert_eq!(os.cache_snapshot().total_number_of_factor_instances(), 0);
        assert_eq!(os.profile_snapshot().factor_sources.len(), 0);
        let factor_source = HDFactorSource::sample();
        os.add_factor_source(factor_source.clone()).await.unwrap();
        assert!(
            os.cache_snapshot()
                .is_full(NetworkID::Mainnet, factor_source.factor_source_id()),
            "Should have put factors into the cache."
        );
        assert_eq!(
            os.profile_snapshot().factor_sources,
            IndexSet::just(factor_source)
        );
    }

    #[actix_rt::test]
    async fn adding_accounts_and_clearing_cache_in_between() {
        let (mut os, _) = SargonOS::with_bdfs().await;
        assert!(os.profile_snapshot().get_accounts().is_empty());
        let (alice, stats) = os.new_mainnet_account_with_bdfs("alice").await.unwrap();
        assert!(!stats.found_in_cache.is_empty());
        assert!(stats.to_cache.is_empty());
        assert!(stats.newly_derived.is_empty());
        os.clear_cache();

        let (bob, stats) = os.new_mainnet_account_with_bdfs("bob").await.unwrap();
        assert!(stats.found_in_cache.is_empty());
        assert!(!stats.to_cache.is_empty());
        assert!(!stats.newly_derived.is_empty());
        assert_ne!(alice, bob);

        assert_eq!(os.profile_snapshot().get_accounts().len(), 2);
    }

    #[actix_rt::test]
    async fn adding_accounts_different_networks_different_factor_sources() {
        let mut os = SargonOS::new();
        assert_eq!(os.cache_snapshot().total_number_of_factor_instances(), 0);

        let fs_device = HDFactorSource::device();
        let fs_arculus = HDFactorSource::arculus();
        let fs_ledger = HDFactorSource::ledger();

        os.add_factor_source(fs_device.clone()).await.unwrap();
        os.add_factor_source(fs_arculus.clone()).await.unwrap();
        os.add_factor_source(fs_ledger.clone()).await.unwrap();

        assert_eq!(
            os.cache_snapshot().total_number_of_factor_instances(),
            3 * 4 * CACHE_FILLING_QUANTITY
        );

        assert!(os.profile_snapshot().get_accounts().is_empty());
        assert_eq!(os.profile_snapshot().factor_sources.len(), 3);

        let (alice, stats) = os
            .new_account(fs_device.clone(), NetworkID::Mainnet, "Alice")
            .await
            .unwrap();
        assert!(stats.newly_derived.is_empty());

        let (bob, stats) = os
            .new_account(fs_device.clone(), NetworkID::Mainnet, "Bob")
            .await
            .unwrap();
        assert!(stats.newly_derived.is_empty());

        let (carol, stats) = os
            .new_account(fs_device.clone(), NetworkID::Stokenet, "Carol")
            .await
            .unwrap();
        assert!(
            !stats.newly_derived.is_empty(),
            "Should have derived more, since first time Stokenet is used!"
        );

        let (diana, stats) = os
            .new_account(fs_device.clone(), NetworkID::Stokenet, "Diana")
            .await
            .unwrap();
        assert!(stats.newly_derived.is_empty());

        let (erin, stats) = os
            .new_account(fs_arculus.clone(), NetworkID::Mainnet, "Erin")
            .await
            .unwrap();
        assert!(stats.newly_derived.is_empty());

        let (frank, stats) = os
            .new_account(fs_arculus.clone(), NetworkID::Mainnet, "Frank")
            .await
            .unwrap();
        assert!(stats.newly_derived.is_empty());

        let (grace, stats) = os
            .new_account(fs_arculus.clone(), NetworkID::Stokenet, "Grace")
            .await
            .unwrap();
        assert!(
            !stats.newly_derived.is_empty(),
            "Should have derived more, since first time Stokenet is used with the Arculus!"
        );

        let (helena, stats) = os
            .new_account(fs_arculus.clone(), NetworkID::Stokenet, "Helena")
            .await
            .unwrap();
        assert!(stats.newly_derived.is_empty());

        let (isabel, stats) = os
            .new_account(fs_ledger.clone(), NetworkID::Mainnet, "isabel")
            .await
            .unwrap();
        assert!(stats.newly_derived.is_empty());

        let (jenny, stats) = os
            .new_account(fs_ledger.clone(), NetworkID::Mainnet, "Jenny")
            .await
            .unwrap();
        assert!(stats.newly_derived.is_empty());

        let (klara, stats) = os
            .new_account(fs_ledger.clone(), NetworkID::Stokenet, "Klara")
            .await
            .unwrap();
        assert!(
            !stats.newly_derived.is_empty(),
            "Should have derived more, since first time Stokenet is used with the Ledger!"
        );

        let (lisa, stats) = os
            .new_account(fs_ledger.clone(), NetworkID::Stokenet, "Lisa")
            .await
            .unwrap();
        assert!(stats.newly_derived.is_empty());

        assert_eq!(os.profile_snapshot().get_accounts().len(), 12);

        let accounts = vec![
            alice, bob, carol, diana, erin, frank, grace, helena, isabel, jenny, klara, lisa,
        ];

        let factor_source_count = os.profile_snapshot().factor_sources.len();
        let network_count = os.profile_snapshot().networks.len();
        assert_eq!(
            os.cache_snapshot().total_number_of_factor_instances(),
            network_count
                * factor_source_count
                * NetworkIndexAgnosticPath::all_presets().len()
                * CACHE_FILLING_QUANTITY
                - accounts.len()
                + factor_source_count // we do `+ factor_source_count` since every time a factor source is used on a new network for the first time, we derive `CACHE_FILLING_QUANTITY + 1`
        );

        assert_eq!(
            os.profile_snapshot()
                .get_accounts()
                .into_iter()
                .map(|a| a.entity_address())
                .collect::<HashSet<AccountAddress>>(),
            accounts
                .into_iter()
                .map(|a| a.entity_address())
                .collect::<HashSet<AccountAddress>>()
        );
    }

    #[actix_rt::test]
    async fn securified_accounts() {
        let (mut os, bdfs) = SargonOS::with_bdfs().await;
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

        let (securified_accounts, stats) = os
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

        assert!(
            !stats.derived_any_new_instance_for_any_factor_source(),
            "should have used cache"
        );

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

        let (securified_accounts, stats) = os
            .securify(
                Accounts::just(carol.clone()),
                MatrixOfFactorSources::new([], 0, [yubikey.clone()]),
            )
            .await
            .unwrap();
        assert!(
            !stats.derived_any_new_instance_for_any_factor_source(),
            "should have used cache"
        );
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

        let (securified_accounts, stats) = os
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
        assert!(
            !stats.derived_any_new_instance_for_any_factor_source(),
            "should have used cache"
        );
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

    #[ignore]
    #[actix_rt::test]
    async fn securify_when_cache_is_half_full_single_factor_source() {
        let (mut os, bdfs) = SargonOS::with_bdfs().await;

        let factor_sources = os.profile_snapshot().factor_sources.clone();
        assert_eq!(
            factor_sources.clone().into_iter().collect_vec(),
            vec![bdfs.clone(),]
        );

        let n = CACHE_FILLING_QUANTITY / 2;

        for i in 0..3 * n {
            let _ = os
                .new_mainnet_account_with_bdfs(format!("Acco: {}", i))
                .await
                .unwrap();
        }

        let shield_0 = MatrixOfFactorSources::new([bdfs.clone()], 1, []);

        let all_accounts = os
            .profile_snapshot()
            .get_accounts()
            .into_iter()
            .collect_vec();

        let first_half_of_accounts = all_accounts.clone()[0..n]
            .iter()
            .cloned()
            .collect::<IndexSet<Account>>();

        let second_half_of_accounts = all_accounts.clone()[n..3 * n]
            .iter()
            .cloned()
            .collect::<IndexSet<Account>>();

        assert_eq!(
            first_half_of_accounts.len() + second_half_of_accounts.len(),
            3 * n
        );

        let (_first_half_securified_accounts, stats) = os
            .securify(
                Accounts::new(NetworkID::Mainnet, first_half_of_accounts).unwrap(),
                shield_0.clone(),
            )
            .await
            .unwrap();

        assert!(
            !stats.derived_any_new_instance_for_any_factor_source(),
            "should have used cache"
        );

        let (_second_half_securified_accounts, stats) = os
            .securify(
                Accounts::new(NetworkID::Mainnet, second_half_of_accounts).unwrap(),
                shield_0,
            )
            .await
            .unwrap();

        assert!(
            stats.derived_any_new_instance_for_any_factor_source(),
            "should have derived more"
        );

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

    #[ignore]
    #[actix_rt::test]
    async fn securify_when_cache_is_half_full_multiple_factor_sources() {
        let (mut os, bdfs) = SargonOS::with_bdfs().await;

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

        let n = CACHE_FILLING_QUANTITY / 2;

        for i in 0..3 * n {
            let (_account, _stats) = os
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
            .iter()
            .cloned()
            .collect::<IndexSet<Account>>();

        let second_half_of_accounts = all_accounts.clone()[n..3 * n]
            .iter()
            .cloned()
            .collect::<IndexSet<Account>>();

        assert_eq!(
            first_half_of_accounts.len() + second_half_of_accounts.len(),
            3 * n
        );

        let (_first_half_securified_accounts, stats) = os
            .securify(
                Accounts::new(NetworkID::Mainnet, first_half_of_accounts).unwrap(),
                shield_0.clone(),
            )
            .await
            .unwrap();

        assert!(
            !stats.derived_any_new_instance_for_any_factor_source(),
            "should have used cache"
        );

        let (_second_half_securified_accounts, stats) = os
            .securify(
                Accounts::new(NetworkID::Mainnet, second_half_of_accounts).unwrap(),
                shield_0,
            )
            .await
            .unwrap();

        assert!(
            stats.derived_any_new_instance_for_any_factor_source(),
            "should have derived more"
        );

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
