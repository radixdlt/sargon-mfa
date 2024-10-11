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
        let outcome = Self::with(
            network_id,
            IndexSet::just(factor_source.clone()),
            // This is hacky! We are using `account_veci` as agnostic_path, we could
            // have used any other value... we are not going to use any instances directly
            // at all, why we specify `0` here, we piggyback on the rest of the logic
            // to derive more... We should most definitely switch to `DerivationTemplate` enum
            QuantifiedDerivationPresets {
                quantity: 0,                                      // HACKY
                derivation_preset: DerivationPreset::AccountVeci, // HACKY
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

impl FactorInstancesCache {
    fn get(
        &self,
        factor_source_idss: IndexSet<FactorSourceIDFromHash>,
        quantified_index_agnostic_path: QuantifiedIndexAgnosticPath,
    ) -> Result<IndexMap<FactorSourceIDFromHash, FactorInstances>> {
        todo!()
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
        let quantified_index_agnostic_path = QuantifiedIndexAgnosticPath {
            agnostic_path: index_agnostic_path,
            quantity: originally_requested_quantity,
        };
        let next_index_assigner =
            NextDerivationEntityIndexAssigner::new(network_id, profile, cache.clone());
        // "pf" short for "Per FactorSource"
        let pf_from_cache = next_index_assigner
            .cache()
            .get(factor_source_ids, quantified_index_agnostic_path)?;

        let mut pf_quantity_missing_from_cache = IndexMap::<FactorSourceIDFromHash, usize>::new();
        let pf_quantity_to_derive = pf_from_cache
            .iter()
            .filter_map(|(factor_source_id, found_in_cache)| {
                let qty_missing_from_cache = originally_requested_quantity - found_in_cache.len();
                if qty_missing_from_cache <= 0 {
                    // no instances missing, cache can fully satisfy request amount
                    None
                } else {
                    // We must retain how many were missing from cache so that we can know how many of
                    // the newly derived we should use directly and how many to cache
                    pf_quantity_missing_from_cache
                        .insert(*factor_source_id, qty_missing_from_cache);

                    // If we are gonna derive anyway, lets derive so that we can fulfill the `number_of_accounts`
                    // originally request + have `CACHE_FILLING_SIZE` many more left after, a.k.a. full cache!
                    let qty_to_derive = CACHE_FILLING_QUANTITY + originally_requested_quantity
                        - found_in_cache.len();

                    Some((*factor_source_id, qty_to_derive))
                }
            })
            .collect::<IndexMap<FactorSourceIDFromHash, usize>>();

        let pf_newly_derived = Self::derive_more(
            pf_quantity_to_derive,
            network_id,
            originally_requested_derivation_preset,
            &next_index_assigner,
        )
        .await?;
        /*

         let pf_mixed = matrix_of_factor_sources.all().iter().map(|f|) {
             let mut merged = IndexSet::new();
             let from_cache = pf_from_cache.get(f).unwrap_or_default();
             let newly_derived = pf_newly_derived.get(f).unwrap_or_default();
             merged.extend(from_cache); // from cache first
             merged.extend(newly_derived);

             (f, merged)
         }.collect::<IndexMap<FactorSourceID, Index<HDFactorInstance>>>();

        let mut pf_to_cache = IndexMap::new();
        let mut pf_to_use_directly = IndexMap::new();
        for (factor_source_id, factor_instances) in pf_mixed {
            let instances_by_derivation_preset = factor_instances
                    .into_iter()
                    .into_group_map_by(|f| {
                        DerivationPreset::try_from(f.derivation_path()).expect("Only valid Presets")
                    })
                    .into_iter()
                    .collect::<IndexMap<DerivationPreset, IndexSet<HDFactorInstance>>>>();
            for (derivation_preset, instances) in instances_by_derivation_preset {
                if derivation_preset == originally_requested_derivation_preset {
                    // Must apply split logic
                    let quantity_missing_from_cache = pf_quantity_missing_from_cache.get(factor_source_id)
                        .expect("Programmer error, should have saved how many instances remains to fulfill");
                    let (to_use_directly, to_cache) = instances.split_at(quantity_missing_from_cache);
                    pf_to_use_directly.append_or_insert(factor_source_id, to_use_directly);
                    pf_to_cache.append_or_insert(factor_source_id, to_cache);
                } else {
                    // Extra derived to fill cache, can simply cache all!
                    pf_to_cache.append_or_insert(factor_source_id, instances);
                }
            }
        }

        next_index_assigner.cache.delete(pf_from_cache);
        next_index_assigner.cache.insert(pf_to_cache);

        Ok(pf_to_use_directly)
        */
        todo!()
    }

    async fn derive_more(
        pf_quantity_to_derive: IndexMap<FactorSourceIDFromHash, usize>,
        network_id: NetworkID,
        originally_requested_derivation_preset: DerivationPreset,
        next_index_assigner: &NextDerivationEntityIndexAssigner,
    ) -> Result<IndexMap<FactorSourceIDFromHash, FactorInstances>> {
        /*
         let pf_paths = pf_quantity_to_derive.into_iter().map(|(factor_source_id, qty)| {
            // `qty` many paths
            let originally_requested_paths_for_factor = (0..qty).map(|_| {
                let query = InstanceQuery {
                    network_id,
                    factor_source_id,
                    derivation_preset
                };
                let index =  next_index_assigner.next(query);
                DerivationPath::from((query, index))
            }).collect::<IndexSet<DerivationPath>>();
           let cache_filling_paths = DerivationPreset::all()
               .excluding(originally_requested_derivation_preset)
               .into_iter()
               .flat_map(|derivation_preset| {
                   let cache = next_index_assigner.cache;

                   let single_factor_from_cache = cache
                       .get_for_single_factor_source(
                           factor_source_id,
                           qty,
                           derivation_preset,
                           network_id
                       );
                   let qty_to_be_full = CACHE_FILLING_SIZE - single_factor_from_cache.len();
                   // `qty_to_be_full` is `0` if cache full, and the map below => empty
                   (0..qty_to_be_full).map(|_| {
                       let query = InstanceQuery {
                           network_id,
                           factor_source_id,
                           derivation_preset
                       };
                       let index =  next_index_assigner.next(query);
                       DerivationPath::from((query, index))
                   }).collect::<IndexSet<DerivationPath>>()
               })
               .collect::<IndexSet<DerivationPath>>();

           let mut paths = IndexSet::new();
           paths.extend(originally_requested_paths_for_factor);
           paths.extend(cache_filling_paths);
            (f, paths)
         }).collect::<IndexMap<FactorSourceID, IndexSet<DerivationPath>>>();

        KeysCollector::new(pf_paths, ...).collect().await
        */
        todo!()
    }
}
