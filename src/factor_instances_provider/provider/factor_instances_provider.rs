use std::{
    ops::Add,
    sync::{Arc, RwLock},
};

use crate::prelude::*;

pub struct FactorInstancesProvider {
    /// We only derive factor instances for one network at a time, currently, this
    /// can be expanded in the future if we want to, but most users only care
    /// about mainnet.
    network_id: NetworkID,

    /// A Clone of a cache, the caller MUST commit the changes to the
    /// original cache if they want to persist them.
    #[allow(dead_code)]
    cache: RwLock<Cache>,

    next_entity_index_assigner: NextDerivationEntityIndexAssigner,

    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
}

impl FactorInstancesProvider {
    /// `Profile` is optional since None in case of Onboarding Account Recovery Scan
    /// No need to pass Profile as mut, since we just need to read it for the
    /// next derivation entity indices.
    fn new(
        network_id: NetworkID,
        cache: Cache,
        profile: impl Into<Option<Profile>>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        Self {
            network_id,
            cache: RwLock::new(cache),
            next_entity_index_assigner: NextDerivationEntityIndexAssigner::new(
                network_id,
                profile.into(),
            ),
            derivation_interactors,
        }
    }
}

impl FactorInstancesProvider {
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

    /// Supports loading many account vecis OR account mfa OR identity vecis OR identity mfa
    /// at once, does NOT support loading a mix of these. We COULD, but that would
    /// make the code more complex and we don't need it.
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
        // `pf` is short for `Per FactorSource`
        let mut pf_found_in_cache = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();
        let factor_source_ids = index_agnostic_path_and_quantity_per_factor_source
            .keys()
            .cloned()
            .collect::<IndexSet<_>>();

        // used to filter out factor instances to use directly from the newly derived, based on
        // `index_agnostic_path_and_quantity_per_factor_source`
        let index_agnostic_paths_originally_requested =
            index_agnostic_path_and_quantity_per_factor_source
                .values()
                .cloned()
                .map(|q| IndexAgnosticPath::from((network_id, q.agnostic_path)))
                .collect::<IndexSet<_>>();

        // For every factor source found in this map, we derive the remaining
        // quantity as to satisfy the request PLUS we are deriving to fill the
        // cache since we are deriving anyway, i.e. derive for all `IndexAgnosticPath`
        // "presets" (Account Veci, Identity Veci, Account MFA, Identity MFA).
        let mut pf_quantity_remaining_not_satisfied_by_cache =
            IndexMap::<FactorSourceIDFromHash, QuantifiedNetworkIndexAgnosticPath>::new();

        let mut pf_to_use_directly = IndexMap::<
            FactorSourceIDFromHash,
            IndexSet<HierarchicalDeterministicFactorInstance>,
        >::new();

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
                    from_cache = FactorInstances::default();
                    unsatisfied_quantity = quantity;
                }
                QuantityOutcome::Partial {
                    instances,
                    remaining,
                } => {
                    from_cache = instances;
                    unsatisfied_quantity = remaining;
                }
                QuantityOutcome::Full { instances } => {
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
                pf_to_use_directly.insert(*factor_source_id, from_cache.factor_instances());
            }
        }

        let mut pf_quantified_network_agnostic_paths_for_derivation = IndexMap::<
            FactorSourceIDFromHash,
            IndexSet<QuantifiedToCacheToUseNetworkIndexAgnosticPath>,
        >::new();

        for factor_source_id in factor_source_ids.iter() {
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
                    .unwrap_or(QuantifiedToCacheToUseNetworkIndexAgnosticPath {
                        quantity: QuantityToCacheToUseDirectly::OnlyCacheFilling {
                            fill_cache: CACHE_FILLING_QUANTITY,
                        },
                        agnostic_path: preset,
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
        assert!(pf_quantified_network_agnostic_paths_for_derivation
            .iter()
            .all(|x| x.1.len() == NetworkIndexAgnosticPath::all_presets().len()));
        println!(
            "🦄 pf_quantified_network_agnostic_paths_for_derivation: {:?}",
            pf_quantified_network_agnostic_paths_for_derivation
        );

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
                                let index = next_index_assigner
                                    .next(f, quantified_agnostic_path.network_agnostic());
                                println!("🦄 index: {:?}", index);
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
                pf_to_use_directly.insert(f, to_use_directly);
            }
        }

        let outcome = FactorInstancesProviderOutcome::transpose(
            pf_to_cache,
            pf_to_use_directly
                .into_iter()
                .map(|(k, v)| (k, FactorInstances::from(v)))
                .collect(),
            pf_found_in_cache,
            pf_newly_derived,
        );
        Ok(outcome)
    }
}

impl HDPathComponent {
    pub fn next(&self) -> Self {
        self.add_one()
    }
}

#[cfg(test)]
impl Cache {
    pub fn is_full(&self, network_id: NetworkID, factor_source_id: FactorSourceIDFromHash) -> bool {
        let count: usize = self
            .values
            .get(&factor_source_id)
            .and_then(|c| {
                c.values()
                    .map(|xs| {
                        xs.factor_instances()
                            .iter()
                            .filter(|x| x.agnostic_path().network_id == network_id)
                            .collect_vec()
                            .len()
                    })
                    .reduce(Add::add)
            })
            .unwrap_or(0);
        count == NetworkIndexAgnosticPath::all_presets().len() * CACHE_FILLING_QUANTITY
    }
    pub fn assert_is_full(&self, network_id: NetworkID, factor_source_id: FactorSourceIDFromHash) {
        assert!(self.is_full(network_id, factor_source_id));
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    type Sut = FactorInstancesProvider;

    #[actix_rt::test]
    async fn cache_is_always_filled_account_veci() {
        //      let cache = Arc::new(RwLock::new(FactorInstancesForEachNetworkCache::default()));

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

        // let outcome = Sut::for_account_mfa(
        //     cache.clone(),
        //     network,
        //     profile,
        //     ,
        //     InstancesQuery::AccountVeci {
        //         factor_source: bdfs.clone(),
        //     },
        // )
        // .await
        // .unwrap();

        // assert_eq!(outcome.statistics.number_of_instances_newly_derived, 1);
        let per_factor = outcome.per_factor;
        assert_eq!(per_factor.len(), 1);
        let outcome = per_factor.get(&bdfs.factor_source_id()).unwrap().clone();
        println!("🦄 outcome: {:?}", outcome);
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

        assert_eq!(outcome.to_use_directly.len(), 1);

        /*
        assert_eq!(outcome.statistics.number_of_instances_loaded_from_cache, 0);
        assert_eq!(
            outcome.statistics.number_of_instances_saved_into_cache,
            CACHE_SIZE * DerivationTemplate::all().len()
        );

        let instances_used_directly = outcome.factor_instances_to_use_directly;

        assert_eq!(
            instances_used_directly
                .account_veci()
                .unwrap()
                .derivation_entity_index(),
            HDPathComponent::Hardened(HDPathComponentHardened::Unsecurified(
                UnsecurifiedIndex::unsecurified_hardening_base_index(0)
            ))
        );

        cache
            .try_read()
            .unwrap()
            .assert_is_full(network, bdfs.factor_source_id());

        let cached = cache
            .try_read()
            .unwrap()
            .clone_for_network_or_empty(network)
            .peek_all_instances_of_factor_source(bdfs.factor_source_id())
            .unwrap();

        let account_vecis = cached
            .clone()
            .account_veci
            .into_iter()
            .map(|f| f.derivation_entity_index())
            .collect_vec();

        assert_eq!(
            account_vecis.first().unwrap().clone(),
            HDPathComponent::unsecurified_hardening_base_index(1)
        );

        assert_eq!(
            account_vecis.last().unwrap().clone(),
            HDPathComponent::unsecurified_hardening_base_index(30)
        );

        let account_mfas = cached
            .clone()
            .account_mfa
            .into_iter()
            .map(|f| f.derivation_entity_index())
            .collect_vec();

        assert_eq!(
            account_mfas.first().unwrap().clone(),
            HDPathComponent::securifying_base_index(0)
        );

        assert_eq!(
            account_mfas.last().unwrap().clone(),
            HDPathComponent::securifying_base_index(29)
        );
        assert!(cached
            .clone()
            .identity_mfa
            .into_iter()
            .all(|f| f.derivation_path().entity_kind == CAP26EntityKind::Identity));

        let identity_mfas = cached
            .identity_mfa
            .into_iter()
            .map(|f| f.derivation_entity_index())
            .collect_vec();

        assert_eq!(
            identity_mfas.first().unwrap().clone(),
            HDPathComponent::securifying_base_index(0)
        );

        assert_eq!(
            identity_mfas.last().unwrap().clone(),
            HDPathComponent::securifying_base_index(29)
        );
        */
    }
}
