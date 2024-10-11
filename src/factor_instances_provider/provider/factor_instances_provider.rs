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
            &cache,
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

#[derive(enum_as_inner::EnumAsInner)]
enum CachedInstancesWithQuantitiesOutcome {
    Satisfied(IndexMap<FactorSourceIDFromHash, FactorInstances>),
    NotSatisfied(IndexMap<FactorSourceIDFromHash, FactorInstances>),
}
pub struct CachedInstancesWithQuantities {
    originally_requested_quantified_derivation_preset: QuantifiedDerivationPresets,
    network_id: NetworkID,
    outcome: CachedInstancesWithQuantitiesOutcome,
}
impl CachedInstancesWithQuantities {
    fn satisfied(&self) -> Option<IndexMap<FactorSourceIDFromHash, FactorInstances>> {
        self.outcome.as_satisfied().cloned()
    }
    fn quantities_to_derive(
        &self,
    ) -> IndexMap<FactorSourceIDFromHash, IndexMap<DerivationPreset, usize>> {
        let instances = self._not_requested();
        todo!()
    }
    fn _not_requested(&self) -> IndexMap<FactorSourceIDFromHash, FactorInstances> {
        self.outcome
            .as_not_satisfied()
            .cloned()
            .expect("not satisfied")
    }
    fn get_requested(self) -> IndexMap<FactorSourceIDFromHash, FactorInstances> {
        self._not_requested()
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
        quantified_derivation_preset: QuantifiedDerivationPresets,
        profile: impl Into<Option<Profile>>,
        cache: &mut FactorInstancesCache,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<InternalFactorInstancesProviderOutcome> {
        let originally_requested_quantified_derivation_preset = quantified_derivation_preset;
        let profile = profile.into();
        let factor_source_ids = factor_sources
            .iter()
            .map(|f| f.factor_source_id())
            .collect::<IndexSet<_>>();

        let cached = cache.get_poly_factor_with_quantities(
            &factor_source_ids,
            &originally_requested_quantified_derivation_preset,
            network_id,
        )?;

        if let Some(satisfied_by_cache) = cached.satisfied() {
            let outcome = InternalFactorInstancesProviderOutcome::satisfied_by_cache(
                satisfied_by_cache.clone(),
            );
            // consume
            cache.delete(satisfied_by_cache);
            return Ok(outcome);
        }

        let pf_newly_derived = Self::derive_more(
            factor_sources,
            cached.quantities_to_derive(),
            network_id,
            profile,
            cache,
            interactors.clone(),
        )
        .await?;

        let pf_found_in_cache_leq_requested = cached.get_requested();

        let Split {
            pf_to_use_directly,
            pf_to_cache,
        } = Self::split(
            &originally_requested_quantified_derivation_preset,
            &pf_found_in_cache_leq_requested,
            &pf_newly_derived,
        );

        cache.delete(pf_found_in_cache_leq_requested.clone());
        cache.insert(pf_to_cache.clone());

        let outcome = InternalFactorInstancesProviderOutcome::transpose(
            pf_to_cache,
            pf_to_use_directly,
            pf_found_in_cache_leq_requested,
            pf_newly_derived,
        );
        let outcome = outcome.into();
        Ok(outcome)
    }

    fn split(
        originally_requested_quantified_derivation_preset: &QuantifiedDerivationPresets,
        pf_found_in_cache_leq_requested: &IndexMap<FactorSourceIDFromHash, FactorInstances>,
        newly_derived: &IndexMap<FactorSourceIDFromHash, FactorInstances>,
    ) -> Split {
        todo!()
    }

    async fn derive_more(
        factor_sources: IndexSet<HDFactorSource>,
        pf_pdp_quantity_to_derive: IndexMap<
            FactorSourceIDFromHash,
            IndexMap<DerivationPreset, usize>,
        >,
        network_id: NetworkID,
        profile: Option<Profile>,
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
                let paths = paths.into_iter().flat_map(|xs| xs).collect::<IndexSet<_>>();

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
