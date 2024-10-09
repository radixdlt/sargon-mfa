use std::sync::{Arc, RwLock};

use itertools::cloned;

use crate::prelude::*;

pub struct FactorInstancesProvider;

impl FactorInstancesProvider {
    /// Use this to fill the cache with FactorInstances for a new FactorSource.
    /// Saves FactorInstances into the mutable `cache` parameter and returns a
    /// copy of the instances.
    pub async fn for_new_factor_source(
        cache: &mut Cache,
        profile: Option<Profile>,
        factor_source: HDFactorSource,
        network_id: NetworkID, // typically mainnet
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcomeForFactorFinal> {
        // This is hacky! We are using `account_veci` as agnostic_path, we could
        // have used any other value... we are not going to use any instances directly
        // at all, why we specify `0` here, we piggyback on the rest of the logic
        // to derive more... We should most definitely switch to `DerivationTemplate` enum
        let quantity_of_instances_to_use_directly = IndexMap::kv(
            factor_source.factor_source_id(),
            QuantifiedNetworkIndexAgnosticPath {
                quantity: 0,                                             // HACKY
                agnostic_path: NetworkIndexAgnosticPath::account_veci(), // HACKY
            },
        );

        let outcome = Self::with(
            network_id,
            cache,
            IndexSet::just(factor_source.clone()),
            quantity_of_instances_to_use_directly,
            &NextDerivationEntityIndexAssigner::new(network_id, profile),
            interactors,
        )
        .await?;

        let outcome = outcome
            .per_factor
            .get(&factor_source.factor_source_id())
            .cloned()
            .expect("Expected to have instances for the (new) factor source");

        assert!(
            outcome.to_use_directly.is_empty(),
            "Programmer error, expected to return an empty list of instances to use directly"
        );

        Ok(outcome.into())
    }

    /// Reads FactorInstances for `factor_source` on `network_id` of kind `account_veci`,
    /// meaning `(EntityKind::Account, KeyKind::TransactionSigning, KeySpace::Unsecurified)`,
    /// from cache, if any, otherwise derives more of that kind AND other kinds:
    /// identity_veci, account_mfa, identity_mfa
    /// and saves into the cache and returns a collection of instances, split into
    /// factor instance to use directly and factor instances which was cached, into
    /// the mutable `cache` parameter.
    pub async fn for_account_veci(
        cache: &mut Cache,
        profile: Option<Profile>,
        factor_source: HDFactorSource,
        network_id: NetworkID,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcomeForFactorFinal> {
        let outcome = Self::with(
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
        .await?;

        let outcome = outcome
            .per_factor
            .get(&factor_source.factor_source_id())
            .cloned()
            .expect("Expected to have instances for the factor source");

        Ok(outcome.into())
    }

    pub async fn for_account_mfa(
        cache: &mut Cache,
        matrix_of_factor_sources: MatrixOfFactorSources,
        profile: Profile,
        accounts: IndexSet<AccountAddress>,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcomeFinal> {
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

        let outcome = Self::with(
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
        .await?;

        Ok(outcome.into())
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
    ) -> Result<FactorInstancesProviderOutcomeNonFinal> {
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
    ) -> Result<FactorInstancesProviderOutcomeNonFinal> {
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
            return Ok(FactorInstancesProviderOutcomeNonFinal::satisfied_by_cache(
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
    ) -> Result<FactorInstancesProviderOutcomeNonFinal> {
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
                                DerivationPath::from((
                                    quantified_agnostic_path.agnostic_path,
                                    index,
                                ))
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

        let outcome = FactorInstancesProviderOutcomeNonFinal::transpose(
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
