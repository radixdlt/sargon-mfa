use std::sync::{Arc, RwLock};

use itertools::cloned;

use crate::{factor_instances_provider::next_index_assigner, prelude::*};

/// A coordinator between a cache, an optional profile and the KeysCollector.
///
/// We can ask this type to provide FactorInstances for some operation, either
/// creation of new virtual accounts or securifying accounts (or analogously for identities).
/// It will try to read instances from the cache, if any, and if there are not enough instances
/// in the cache, it will derive more instances and save them into the cache.
///
/// We are always reading from the beginning of each FactorInstance collection in the cache,
/// and we are always appending to the end.
///
/// Whenever we need to derive more, we always derive for all `IndexAgnosticPath` "presets",
/// i.e. we are not only filling the cache with factor instances relevant to the operation
/// but rather we are filling the cache with factor instances for all kinds of operations, i.e.
/// if we did not have `CACHE_FILLING_QUANTITY` instances for "account_mfa", when we tried
/// to read "account_veci" instances, we will derive more "account_mfa" instances as well,
/// so many that at the end of execution we will have `CACHE_FILLING_QUANTITY` instances for
/// both "account_veci" and "account_mfa" (and same for identities).
pub struct FactorInstancesProvider;

impl FactorInstancesProvider {
    /// Use this to fill the cache with FactorInstances for a new FactorSource.
    /// Saves FactorInstances into the mutable `cache` parameter and returns a
    /// copy of the instances.
    pub async fn for_new_factor_source(
        cache: &mut FactorInstancesCache,
        profile: Option<Profile>,
        factor_source: HDFactorSource,
        network_id: NetworkID, // typically mainnet
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcomeForFactor> {
        // let outcome = Self::with(
        //     network_id,
        //     IndexSet::just(factor_source.clone()),
        //     // This is hacky! We are using `account_veci` as agnostic_path, we could
        //     // have used any other value... we are not going to use any instances directly
        //     // at all, why we specify `0` here, we piggyback on the rest of the logic
        //     // to derive more... We should most definitely switch to `DerivationTemplate` enum
        //     QuantifiedDerivationPresets {
        //         quantity: 0,                                      // HACKY
        //         derivation_preset: DerivationPreset::AccountVeci, // HACKY
        //     },
        //     profile,
        //     cache,
        //     interactors,
        // )
        // .await?;
        let next_index_assigner =
            &NextDerivationEntityIndexAssigner::new(network_id, profile, cache.clone());

        let derived = Self::derive_more(
            IndexSet::just(factor_source.clone()),
            IndexMap::kv(factor_source.factor_source_id(), CACHE_FILLING_QUANTITY),
            DerivationPreset::AccountMfa,
            network_id,
            next_index_assigner,
            interactors,
        )
        .await?;
        cache.insert(derived.clone());

        let derived = derived
            .get(&factor_source.factor_source_id())
            .unwrap()
            .clone();
        let outcome = InternalFactorInstancesProviderOutcomeForFactor::new(
            factor_source.factor_source_id(),
            derived.clone(),
            FactorInstances::default(),
            FactorInstances::default(),
            derived,
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
    ///
    /// We are always reading from the beginning of each FactorInstance collection in the cache,
    /// and we are always appending to the end.
    pub async fn for_account_veci(
        cache: &mut FactorInstancesCache,
        profile: Option<Profile>,
        factor_source: HDFactorSource,
        network_id: NetworkID,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcomeForFactor> {
        Self::for_entity_veci(
            cache,
            CAP26EntityKind::Account,
            profile,
            factor_source,
            network_id,
            interactors,
        )
        .await
    }

    /// Reads FactorInstances for `factor_source` on `network_id` of kind `persona_veci`,
    /// meaning `(EntityKind::Identity, KeyKind::TransactionSigning, KeySpace::Unsecurified)`,
    /// from cache, if any, otherwise derives more of that kind AND other kinds:
    /// account_veci, account_mfa, identity_mfa
    /// and saves into the cache and returns a collection of instances, split into
    /// factor instance to use directly and factor instances which was cached, into
    /// the mutable `cache` parameter.
    ///
    /// We are always reading from the beginning of each FactorInstance collection in the cache,
    /// and we are always appending to the end.
    pub async fn for_persona_veci(
        cache: &mut FactorInstancesCache,
        profile: Option<Profile>,
        factor_source: HDFactorSource,
        network_id: NetworkID,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcomeForFactor> {
        Self::for_entity_veci(
            cache,
            CAP26EntityKind::Identity,
            profile,
            factor_source,
            network_id,
            interactors,
        )
        .await
    }

    /// Reads FactorInstances for `factor_source` on `network_id` of kind `account_veci`,
    /// meaning `(EntityKind::Account, KeyKind::TransactionSigning, KeySpace::Unsecurified)`,
    /// from cache, if any, otherwise derives more of that kind AND other kinds:
    /// identity_veci, account_mfa, identity_mfa
    /// and saves into the cache and returns a collection of instances, split into
    /// factor instance to use directly and factor instances which was cached, into
    /// the mutable `cache` parameter.
    ///
    /// We are always reading from the beginning of each FactorInstance collection in the cache,
    /// and we are always appending to the end.
    pub async fn for_entity_veci<'c>(
        cache: &mut FactorInstancesCache,
        entity_kind: CAP26EntityKind,
        profile: Option<Profile>,
        factor_source: HDFactorSource,
        network_id: NetworkID,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcomeForFactor> {
        let outcome = Self::with(
            network_id,
            IndexSet::just(factor_source.clone()),
            QuantifiedDerivationPresets {
                quantity: 1,
                derivation_preset: DerivationPreset::veci_entity_kind(entity_kind),
            },
            profile,
            cache,
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

    /// Reads FactorInstances for every `factor_source` in matrix_of_factor_sources
    /// on `network_id` of kind `account_mfa`,
    /// meaning `(EntityKind::Account, KeyKind::TransactionSigning, KeySpace::Securified)`,
    /// from cache, if any, otherwise derives more of that kind AND other kinds:
    /// identity_veci, account_veci, identity_mfa
    /// and saves into the cache and returns a collection of instances, per factor source,
    /// split into factor instance to use directly and factor instances which was cached, into
    /// the mutable `cache` parameter.
    ///
    /// We are always reading from the beginning of each FactorInstance collection in the cache,
    /// and we are always appending to the end.
    pub async fn for_account_mfa(
        cache: &mut FactorInstancesCache,
        matrix_of_factor_sources: MatrixOfFactorSources,
        profile: Profile,
        account_addresses: IndexSet<AccountAddress>,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcome> {
        Self::for_entity_mfa::<Account>(
            cache,
            matrix_of_factor_sources,
            profile,
            account_addresses,
            interactors,
        )
        .await
    }

    /// Reads FactorInstances for every `factor_source` in matrix_of_factor_sources
    /// on `network_id` of kind `account_mfa`,
    /// meaning `(EntityKind::Account, KeyKind::TransactionSigning, KeySpace::Securified)`,
    /// from cache, if any, otherwise derives more of that kind AND other kinds:
    /// identity_veci, account_veci, identity_mfa
    /// and saves into the cache and returns a collection of instances, per factor source,
    /// split into factor instance to use directly and factor instances which was cached, into
    /// the mutable `cache` parameter.
    ///
    /// We are always reading from the beginning of each FactorInstance collection in the cache,
    /// and we are always appending to the end.
    pub async fn for_persona_mfa(
        cache: &mut FactorInstancesCache,
        matrix_of_factor_sources: MatrixOfFactorSources,
        profile: Profile,
        persona_addresses: IndexSet<IdentityAddress>,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcome> {
        Self::for_entity_mfa::<Persona>(
            cache,
            matrix_of_factor_sources,
            profile,
            persona_addresses,
            interactors,
        )
        .await
    }

    /// Reads FactorInstances for every `factor_source` in matrix_of_factor_sources
    /// on `network_id` of kind `account_mfa` or `identity_mfa` depending on Entity kind,
    /// meaning `(EntityKind::_, KeyKind::TransactionSigning, KeySpace::Securified)`,
    /// from cache, if any, otherwise derives more of that kind AND other kinds:
    /// identity_veci, account_veci, identity_mfa/account_mfa
    /// and saves into the cache and returns a collection of instances, per factor source,
    /// split into factor instance to use directly and factor instances which was cached, into
    /// the mutable `cache` parameter.
    ///
    /// We are always reading from the beginning of each FactorInstance collection in the cache,
    /// and we are always appending to the end.
    pub async fn for_entity_mfa<E: IsEntity>(
        cache: &mut FactorInstancesCache,
        matrix_of_factor_sources: MatrixOfFactorSources,
        profile: Profile,
        addresses_of_entities: IndexSet<E::Address>,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<FactorInstancesProviderOutcome> {
        let factor_sources_to_use = matrix_of_factor_sources.all_factors();
        let factor_sources = profile.factor_sources.clone();
        assert!(
            factor_sources.is_superset(&factor_sources_to_use),
            "Missing FactorSources"
        );
        assert!(!addresses_of_entities.is_empty(), "No entities");
        assert!(
            addresses_of_entities
                .iter()
                .all(|e| profile.contains_entity_by_address::<E>(e)),
            "unknown entity"
        );
        let network_id = addresses_of_entities.first().unwrap().network_id();
        assert!(
            addresses_of_entities
                .iter()
                .all(|a| a.network_id() == network_id),
            "wrong network"
        );

        let entity_kind = E::kind();
        let derivation_preset = DerivationPreset::mfa_entity_kind(entity_kind);

        let outcome = Self::with(
            network_id,
            factor_sources,
            QuantifiedDerivationPresets {
                quantity: addresses_of_entities.len(),
                derivation_preset,
            },
            profile,
            cache,
            interactors,
        )
        .await?;

        Ok(outcome.into())
    }
}

impl FactorInstancesProvider {
    async fn with(
        network_id: NetworkID,
        factor_sources: IndexSet<HDFactorSource>,
        quantified_derivation_preset: QuantifiedDerivationPresets,
        profile: impl Into<Option<Profile>>,
        cache: &mut FactorInstancesCache,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<InternalFactorInstancesProviderOutcome> {
        let profile = profile.into();
        let factor_source_ids = factor_sources
            .iter()
            .map(|f| f.factor_source_id())
            .collect::<IndexSet<_>>();
        let originally_requested_derivation_preset = quantified_derivation_preset.derivation_preset;
        let index_agnostic_path =
            originally_requested_derivation_preset.index_agnostic_path_on_network(network_id);

        let originally_requested_quantity = quantified_derivation_preset.quantity;

        let next_index_assigner =
            NextDerivationEntityIndexAssigner::new(network_id, profile, cache.clone());
        // "pf" short for "Per FactorSource"
        let pf_found_in_cache = next_index_assigner
            .cache()
            .get_poly_factor(&factor_source_ids, &index_agnostic_path)?;

        let mut pf_quantity_missing_from_cache = IndexMap::<FactorSourceIDFromHash, usize>::new();
        let pf_quantity_to_derive = pf_found_in_cache
            .iter()
            .filter_map(|(factor_source_id, found_in_cache)| {
                let qty_missing_from_cache =
                    originally_requested_quantity as i8 - found_in_cache.len() as i8;
                    println!("🦆 found_in_cache.len(): {}, originally_requested_quantity: {} ==> qty_missing_from_cache: {}", found_in_cache.len(), originally_requested_quantity, qty_missing_from_cache);
                if qty_missing_from_cache <= 0 {
                    // no instances missing, cache can fully satisfy request amount
                    None
                } else {
                    // We must retain how many were missing from cache so that we can know how many of
                    // the newly derived we should use directly and how many to cache
                    pf_quantity_missing_from_cache
                        .insert(*factor_source_id, qty_missing_from_cache as usize);

                    // If we are gonna derive anyway, lets derive so that we can fulfill the `number_of_accounts`
                    // originally request + have `CACHE_FILLING_SIZE` many more left after, a.k.a. full cache!
                    let qty_to_derive = CACHE_FILLING_QUANTITY + originally_requested_quantity
                        - found_in_cache.len();

                    Some((*factor_source_id, qty_to_derive))
                }
            })
            .collect::<IndexMap<FactorSourceIDFromHash, usize>>();

        let pf_newly_derived = Self::derive_more(
            factor_sources,
            pf_quantity_to_derive,
            originally_requested_derivation_preset,
            network_id,
            &next_index_assigner,
            interactors.clone(),
        )
        .await?;

        let pf_mixed = factor_source_ids
            .iter()
            .map(|f| {
                let mut merged = IndexSet::new();
                let from_cache = pf_found_in_cache.get(f).cloned().unwrap_or_default();
                let newly_derived = pf_newly_derived.get(f).cloned().unwrap_or_default();
                merged.extend(from_cache); // from cache first, since it has lower indices
                merged.extend(newly_derived);

                (*f, FactorInstances::from(merged))
            })
            .collect::<IndexMap<FactorSourceIDFromHash, FactorInstances>>();

        let mut pf_to_cache = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();
        let mut pf_to_use_directly = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();
        for (factor_source_id, factor_instances) in pf_mixed {
            let instances_by_derivation_preset = factor_instances
                .into_iter()
                .into_group_map_by(|f| {
                    DerivationPreset::try_from(f.agnostic_path()).expect("Only valid Presets")
                })
                .into_iter()
                .collect::<IndexMap<DerivationPreset, Vec<HierarchicalDeterministicFactorInstance>>>();

            for (derivation_preset, instances) in instances_by_derivation_preset {
                if derivation_preset == originally_requested_derivation_preset {
                    // Must apply split logic
                    let qty_split: usize;
                    if let Some(quantity_missing_from_cache) =
                        pf_quantity_missing_from_cache.get(&factor_source_id)
                    {
                        qty_split = *quantity_missing_from_cache;
                    } else {
                        println!("🙅‍♀️ pf_quantity_missing_from_cache was NONE for: derivation_preset={:?}", derivation_preset);
                        qty_split = 0
                    };
                    let (to_use_directly, to_cache) = instances.split_at(qty_split);
                    let to_use_directly =
                        to_use_directly.iter().cloned().collect::<FactorInstances>();

                    let to_cache = to_cache.iter().cloned().collect::<FactorInstances>();

                    append_or_insert_to(
                        &mut pf_to_use_directly,
                        &factor_source_id,
                        to_use_directly,
                    );

                    append_or_insert_to(&mut pf_to_cache, &factor_source_id, to_cache);
                } else {
                    // Extra derived to fill cache, can simply cache all!
                    append_or_insert_to(&mut pf_to_cache, &factor_source_id, instances);
                }
            }
        }

        cache.delete(pf_found_in_cache.clone());
        cache.insert(pf_to_cache.clone());
        let outcome = InternalFactorInstancesProviderOutcome::transpose(
            pf_to_cache,
            pf_to_use_directly,
            pf_found_in_cache,
            pf_newly_derived,
        );
        let outcome = outcome.into();
        Ok(outcome)
    }

    async fn derive_more(
        factor_sources: IndexSet<HDFactorSource>,
        pf_quantity_to_derive: IndexMap<FactorSourceIDFromHash, usize>,
        originally_requested_derivation_preset: DerivationPreset,
        network_id: NetworkID,
        next_index_assigner: &NextDerivationEntityIndexAssigner,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<IndexMap<FactorSourceIDFromHash, FactorInstances>> {
        let pf_paths = pf_quantity_to_derive
            .into_iter()
            .map(|(factor_source_id, qty)| {
                // `qty` many paths
                let originally_requested_paths_for_factor = (0..qty)
                    .map(|_| {
                        let index_agnostic_path = originally_requested_derivation_preset
                            .index_agnostic_path_on_network(network_id);
                        let index =
                            next_index_assigner.next(factor_source_id, index_agnostic_path)?;
                        Ok(DerivationPath::from((index_agnostic_path, index)))
                    })
                    .collect::<Result<IndexSet<DerivationPath>>>()?;

                let cache_filling_paths = DerivationPreset::all()
                    .excluding(originally_requested_derivation_preset)
                    .into_iter()
                    .map(|derivation_preset| {
                        let cache = next_index_assigner.cache();
                        let index_agnostic_path =
                            derivation_preset.index_agnostic_path_on_network(network_id);
                        let single_factor_from_cache =
                            cache.get_mono_factor(&factor_source_id, &index_agnostic_path)?;
                        let qty_to_be_full =
                            CACHE_FILLING_QUANTITY - single_factor_from_cache.len();
                        // `qty_to_be_full` is `0` if cache full, and the map below => empty
                        (0..qty_to_be_full)
                            .map(|_| {
                                let index = next_index_assigner
                                    .next(factor_source_id, index_agnostic_path)?;
                                Ok(DerivationPath::from((index_agnostic_path, index)))
                            })
                            .collect::<Result<IndexSet<DerivationPath>>>()
                    })
                    .collect::<Result<Vec<IndexSet<DerivationPath>>>>()?;

                // flatten (I was unable to use `flat_map` above combined with `Result`...)
                let cache_filling_paths = cache_filling_paths
                    .into_iter()
                    .flat_map(|xs| xs)
                    .collect::<IndexSet<_>>();

                let mut paths = IndexSet::new();
                paths.extend(originally_requested_paths_for_factor);
                paths.extend(cache_filling_paths);
                Ok((factor_source_id, paths))
            })
            .collect::<Result<IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>>>()?;

        let keys_collector = KeysCollector::new(factor_sources, pf_paths, interactors)?;

        let pf_instances = keys_collector
            .collect_keys()
            .await
            .factors_by_source
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect::<FactorInstances>()))
            .collect::<IndexMap<_, _>>();

        Ok(pf_instances)
    }
}

pub trait Excluding {
    type Item;
    fn excluding(&self, item: Self::Item) -> Self;
}
impl Excluding for IndexSet<DerivationPreset> {
    type Item = DerivationPreset;
    fn excluding(&self, item: Self::Item) -> Self {
        self.iter().filter(|i| *i != &item).cloned().collect()
    }
}

pub trait AppendableCollection: FromIterator<Self::Element> {
    type Element: Eq + std::hash::Hash;
    fn append<T: IntoIterator<Item = Self::Element>>(&mut self, iter: T);
}
impl<V: Eq + std::hash::Hash> AppendableCollection for IndexSet<V> {
    type Element = V;

    fn append<T: IntoIterator<Item = Self::Element>>(&mut self, iter: T) {
        self.extend(iter)
    }
}

impl AppendableCollection for FactorInstances {
    type Element = HierarchicalDeterministicFactorInstance;

    fn append<T: IntoIterator<Item = Self::Element>>(&mut self, iter: T) {
        self.extend(iter)
    }
}

pub fn append_or_insert_to<K, V, I>(map: &mut IndexMap<K, V>, key: &K, items: I)
where
    I: IntoIterator<Item = V::Element>,
    K: Eq + std::hash::Hash + Clone,
    V: AppendableCollection,
{
    if let Some(existing) = map.get_mut(key) {
        existing.append(items);
    } else {
        map.insert(key.clone(), V::from_iter(items));
    }
}
