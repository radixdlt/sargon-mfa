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
        let derived = Self::derive_more(
            IndexSet::just(factor_source.clone()),
            IndexMap::kv(
                factor_source.factor_source_id(),
                DerivationPreset::all()
                    .into_iter()
                    .map(|dp| (dp, CACHE_FILLING_QUANTITY))
                    .collect::<IndexMap<DerivationPreset, usize>>(),
            ),
            network_id,
            profile,
            cache,
            interactors,
        )
        .await?;
        cache.insert(&derived);

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
    pub async fn for_entity_veci(
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
            QuantifiedDerivationPreset::new(DerivationPreset::veci_entity_kind(entity_kind), 1),
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
            factor_sources_to_use,
            QuantifiedDerivationPreset::new(derivation_preset, addresses_of_entities.len()),
            profile,
            cache,
            interactors,
        )
        .await?;

        Ok(outcome.into())
    }
}

struct Split {
    pf_to_use_directly: IndexMap<FactorSourceIDFromHash, FactorInstances>,
    pf_to_cache: IndexMap<FactorSourceIDFromHash, FactorInstances>,
}

impl FactorInstancesProvider {
    async fn with(
        network_id: NetworkID,
        factor_sources: IndexSet<HDFactorSource>,
        originally_requested_quantified_derivation_preset: QuantifiedDerivationPreset,
        profile: impl Into<Option<Profile>>,
        cache: &mut FactorInstancesCache,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<InternalFactorInstancesProviderOutcome> {
        let cached = cache.get_poly_factor_with_quantities(
            &factor_sources
                .iter()
                .map(|f| f.factor_source_id())
                .collect(),
            &originally_requested_quantified_derivation_preset,
            network_id,
        )?;

        match cached {
            CachedInstancesWithQuantitiesOutcome::Satisfied(enough_instances) => {
                // Remove the instances which are going to be used from the cache
                // since we only peeked at them.
                cache.delete(&enough_instances);
                Ok(InternalFactorInstancesProviderOutcome::satisfied_by_cache(
                    enough_instances,
                ))
            }
            CachedInstancesWithQuantitiesOutcome::NotSatisfied {
                quantities_to_derive,
                partial_instances,
            } => {
                Self::derive_more_and_cache(
                    network_id,
                    factor_sources,
                    originally_requested_quantified_derivation_preset,
                    profile,
                    cache,
                    partial_instances,
                    quantities_to_derive,
                    interactors,
                )
                .await
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn derive_more_and_cache(
        network_id: NetworkID,
        factor_sources: IndexSet<HDFactorSource>,
        originally_requested_quantified_derivation_preset: QuantifiedDerivationPreset,
        profile: impl Into<Option<Profile>>,
        cache: &mut FactorInstancesCache,
        pf_found_in_cache_leq_requested: IndexMap<FactorSourceIDFromHash, FactorInstances>,
        pf_pdp_qty_to_derive: IndexMap<FactorSourceIDFromHash, IndexMap<DerivationPreset, usize>>,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<InternalFactorInstancesProviderOutcome> {
        let pf_newly_derived = Self::derive_more(
            factor_sources.clone(),
            pf_pdp_qty_to_derive,
            network_id,
            profile,
            cache,
            interactors.clone(),
        )
        .await?;

        let Split {
            pf_to_use_directly,
            pf_to_cache,
        } = Self::split(
            &originally_requested_quantified_derivation_preset,
            factor_sources
                .into_iter()
                .map(|f| f.factor_source_id())
                .collect(),
            &pf_found_in_cache_leq_requested,
            &pf_newly_derived,
        );

        cache.delete(&pf_found_in_cache_leq_requested);
        cache.insert(&pf_to_cache);

        let outcome = InternalFactorInstancesProviderOutcome::transpose(
            pf_to_cache,
            pf_to_use_directly,
            pf_found_in_cache_leq_requested,
            pf_newly_derived,
        );
        let outcome = outcome;
        Ok(outcome)
    }

    /// Per factor, split the instances into those to use directly and those to cache.
    /// based on the originally requested quantity.
    fn split(
        originally_requested_quantified_derivation_preset: &QuantifiedDerivationPreset,
        factor_source_ids: IndexSet<FactorSourceIDFromHash>,
        pf_found_in_cache_leq_requested: &IndexMap<FactorSourceIDFromHash, FactorInstances>,
        pf_newly_derived: &IndexMap<FactorSourceIDFromHash, FactorInstances>,
    ) -> Split {
        // Start by merging the instances found in cache and the newly derived instances,
        // into a single collection of instances per factor source, with the
        // instances from cache first in the list (per factor), and then the newly derived.
        // this is important so that we consume the instances from cache first.
        let pf_derived_appended_to_from_cache = factor_source_ids
            .into_iter()
            .map(|factor_source_id| {
                let mut merged = IndexSet::new();
                let from_cache = pf_found_in_cache_leq_requested
                    .get(&factor_source_id)
                    .cloned()
                    .unwrap_or_default();
                let newly_derived = pf_newly_derived
                    .get(&factor_source_id)
                    .cloned()
                    .unwrap_or_default();
                // IMPORTANT: Must put instances from cache **first**...
                merged.extend(from_cache);
                // ... and THEN the newly derived, so we consume the ones with
                // lower index from cache first.
                merged.extend(newly_derived);

                (factor_source_id, FactorInstances::from(merged))
            })
            .collect::<IndexMap<FactorSourceIDFromHash, FactorInstances>>();

        let mut pf_to_use_directly = IndexMap::new();
        let mut pf_to_cache = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();
        let quantity_originally_requested =
            originally_requested_quantified_derivation_preset.quantity;
        let preset_originally_requested =
            originally_requested_quantified_derivation_preset.derivation_preset;

        // Using the merged map, split the instances into those to use directly and those to cache.
        for (factor_source_id, instances) in pf_derived_appended_to_from_cache.clone().into_iter() {
            let mut instances_by_derivation_preset = InstancesByDerivationPreset::from(instances);

            if let Some(instances_relevant_to_use_directly_with_abundance) =
                instances_by_derivation_preset.remove(preset_originally_requested)
            {
                let (to_use_directly, to_cache) = instances_relevant_to_use_directly_with_abundance
                    .split_at(quantity_originally_requested);
                pf_to_use_directly.insert(factor_source_id, to_use_directly);
                pf_to_cache.insert(factor_source_id, to_cache);
            }

            pf_to_cache.append_or_insert_to(
                factor_source_id,
                instances_by_derivation_preset.all_instances(),
            );
        }

        Split {
            pf_to_use_directly,
            pf_to_cache,
        }
    }

    async fn derive_more(
        factor_sources: IndexSet<HDFactorSource>,
        pf_pdp_quantity_to_derive: IndexMap<
            FactorSourceIDFromHash,
            IndexMap<DerivationPreset, usize>,
        >,
        network_id: NetworkID,
        profile: impl Into<Option<Profile>>,
        cache: &FactorInstancesCache,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<IndexMap<FactorSourceIDFromHash, FactorInstances>> {
        let next_index_assigner =
            NextDerivationEntityIndexAssigner::new(network_id, profile, cache.clone());

        let pf_paths = pf_pdp_quantity_to_derive
            .into_iter()
            .map(|(factor_source_id, pdp_quantity_to_derive)| {
                let paths = pdp_quantity_to_derive
                    .into_iter()
                    .map(|(derivation_preset, qty)| {
                        // `qty` many paths
                        let paths = (0..qty)
                            .map(|_| {
                                let index_agnostic_path =
                                    derivation_preset.index_agnostic_path_on_network(network_id);
                                let index = next_index_assigner
                                    .next(factor_source_id, index_agnostic_path)?;
                                Ok(DerivationPath::from((index_agnostic_path, index)))
                            })
                            .collect::<Result<IndexSet<DerivationPath>>>()?;

                        Ok(paths)
                    })
                    .collect::<Result<Vec<IndexSet<DerivationPath>>>>()?;

                // flatten (I was unable to use `flat_map` above combined with `Result`...)
                let paths = paths.into_iter().flatten().collect::<IndexSet<_>>();

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
