use std::sync::{Arc, RwLock};

use itertools::cloned;

use crate::prelude::*;

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
        // This is hacky! We are using `account_veci` as agnostic_path, we could
        // have used any other value... we are not going to use any instances directly
        // at all, why we specify `0` here, we piggyback on the rest of the logic
        // to derive more... We should most definitely switch to `DerivationTemplate` enum
        let quantity_of_instances_to_use_directly = IndexMap::kv(
            factor_source.factor_source_id(),
            QuantifiedDerivationPresets {
                quantity: 0,                                      // HACKY
                derivation_preset: DerivationPreset::AccountVeci, // HACKY
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
            cache,
            IndexSet::just(factor_source.clone()),
            IndexMap::kv(
                factor_source.factor_source_id(),
                QuantifiedDerivationPresets {
                    quantity: 1,
                    derivation_preset: DerivationPreset::veci_entity_kind(entity_kind),
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
            cache,
            factor_sources,
            factor_sources_to_use
                .into_iter()
                .map(|f| {
                    (
                        f.factor_source_id(),
                        QuantifiedDerivationPresets {
                            quantity: addresses_of_entities.len(),
                            derivation_preset,
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

struct TakeFromCacheResult {
    pf_found_in_cache: IndexMap<FactorSourceIDFromHash, FactorInstances>,
    pf_quantity_remaining_not_satisfied_by_cache:
        IndexMap<FactorSourceIDFromHash, QuantifiedDerivationPresets>,
    need_to_derive_more_instances: bool,
}

struct SplitFactorInstances {
    pf_to_cache: IndexMap<FactorSourceIDFromHash, FactorInstances>,
    pf_to_use_directly: IndexMap<FactorSourceIDFromHash, FactorInstances>,
    /// "unsplit"
    pf_newly_derived: IndexMap<FactorSourceIDFromHash, FactorInstances>,
}

impl FactorInstancesProvider {
    async fn with(
        network_id: NetworkID,
        cache: &mut FactorInstancesCache,
        factor_sources: IndexSet<HDFactorSource>,
        index_agnostic_path_and_quantity_per_factor_source: IndexMap<
            FactorSourceIDFromHash,
            QuantifiedDerivationPresets,
        >,
        next_index_assigner: &NextDerivationEntityIndexAssigner,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<InternalFactorInstancesProviderOutcome> {
        // clone cache so that we do not mutate the cache itself, later, if
        // derivation is successful, we will write back the changes made to
        // this cloned cache, on top of which we will save the newly derived
        // instances.
        let mut cloned_cache = cache.clone();

        // take (consume) the cache and derive more instances if necessary
        let outcome = Self::with_copy_of_cache(
            network_id,
            &mut cloned_cache,
            factor_sources,
            index_agnostic_path_and_quantity_per_factor_source,
            next_index_assigner,
            interactors,
        )
        .await?;

        // derivation was successful, safe to write back the changes to the cache
        *cache = cloned_cache;

        // and now lets save all `to_cache` (newly derived minus enough instances
        // to satisfy the initial request) into the cache.
        cache.insert_all(
            outcome
                .per_factor
                .clone()
                .into_iter()
                .map(|(k, v)| {
                    // We are only saving the instances `to_cache` into the cache,
                    // the other instances should be used directly (if any).
                    let to_cache = v.to_cache;
                    (k, to_cache)
                })
                .collect::<IndexMap<_, _>>(),
        )?;

        Ok(outcome)
    }

    /// You should pass this a clone of the cache and not the cache itself.
    /// since this mutates the cache.
    #[allow(clippy::nonminimal_bool)]
    async fn with_copy_of_cache(
        network_id: NetworkID,
        cache: &mut FactorInstancesCache,
        factor_sources: IndexSet<HDFactorSource>,
        index_agnostic_path_and_quantity_per_factor_source: IndexMap<
            FactorSourceIDFromHash,
            QuantifiedDerivationPresets,
        >,
        next_index_assigner: &NextDerivationEntityIndexAssigner,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<InternalFactorInstancesProviderOutcome> {
        let pf_cache_result = Self::take_from_cache(
            network_id,
            cache,
            &index_agnostic_path_and_quantity_per_factor_source,
        )?;

        if !pf_cache_result.need_to_derive_more_instances {
            // `need_to_derive_more_instances` was never set to true, so we
            // can satisfy the request with what we found in the cache.
            return Ok(InternalFactorInstancesProviderOutcome::satisfied_by_cache(
                pf_cache_result.pf_found_in_cache,
            ));
        }

        // We need to derive more instances, since we could not satisfy the request
        // We will derive more and save them into the cache, and return
        // the concatenation of what was found in cache (if any) with the remaining
        // quantity from newly derived to satisfy the request.
        Self::derive_more_instances(
            network_id,
            cache,
            next_index_assigner,
            interactors,
            factor_sources,
            index_agnostic_path_and_quantity_per_factor_source,
            pf_cache_result.pf_quantity_remaining_not_satisfied_by_cache,
            pf_cache_result.pf_found_in_cache,
        )
        .await
    }

    fn take_from_cache(
        network_id: NetworkID,
        cache: &mut FactorInstancesCache,
        index_agnostic_path_and_quantity_per_factor_source: &IndexMap<
            FactorSourceIDFromHash,
            QuantifiedDerivationPresets,
        >,
    ) -> Result<TakeFromCacheResult> {
        // `pf` is short for `Per FactorSource`
        let mut pf_found_in_cache = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();

        // For every factor source found in this map, we derive the remaining
        // quantity as to satisfy the request PLUS we are deriving to fill the
        // cache since we are deriving anyway, i.e. derive for all `IndexAgnosticPath`
        // "presets" (Account Veci, Identity Veci, Account MFA, Identity MFA).
        let mut pf_quantity_remaining_not_satisfied_by_cache =
            IndexMap::<FactorSourceIDFromHash, QuantifiedDerivationPresets>::new();

        // if false we will not derive any more instances, we could satisfy the request
        // with what we found in the cache.
        let mut need_to_derive_more_instances: bool = false;

        for (factor_source_id, quantified_agnostic_path) in
            index_agnostic_path_and_quantity_per_factor_source.iter()
        {
            let from_cache: FactorInstances;
            let unsatisfied_quantity: usize;
            let cache_key =
                IndexAgnosticPath::from((network_id, quantified_agnostic_path.derivation_preset));

            // the quantity of factor instances needed to satisfy the request
            // this will be `0` in case of PRE_DERIVE_KEYS_FOR_NEW_FACTOR_SOURCE (hacky).
            // this will be `accounts.len()` in case of `for_account_mfa` (and analog for identities) and will
            // be `1` for account_veci / identity_veci.
            let quantity = quantified_agnostic_path.quantity;

            // we are mutating the cache, reading out `quantity` OR LESS instances.
            // we must check how many we got
            match cache.remove(factor_source_id, &cache_key, quantity) {
                // Found nothing in the cache
                QuantityOutcome::Empty => {
                    // need to derive more since cache was empty
                    need_to_derive_more_instances = true;
                    // nothing found in the cache, use empty...
                    from_cache = FactorInstances::default();
                    // ALL `quantity` many instances are "unsatisfied".
                    unsatisfied_quantity = quantity;
                }
                // Found some instances in the cache, but `remaining` many are still needed
                QuantityOutcome::Partial {
                    instances,
                    remaining,
                } => {
                    // we need to derive more since cache could only partially satisfy the request
                    need_to_derive_more_instances = true;
                    // use all found (and we will need to derive more)
                    from_cache = instances;
                    // `remaining` many instances are "unsatisfied", for this factor source
                    unsatisfied_quantity = remaining;
                }
                // Found all instances needed in the cache
                QuantityOutcome::Full { instances } => {
                    // we do not set `need_to_derive_more_instances` to `false`
                    // since an earlier iteration might have set it to true (for another factor source).
                    // so we do not change it.

                    // use all found (and we will not need to derive more for this factor source)
                    from_cache = instances;
                    // none unsatisfied for this factor source.
                    unsatisfied_quantity = 0;
                }
            }

            if unsatisfied_quantity > 0 {
                // need to save which IndexAgnosticPath we need to derive more for
                // and how many instances we need to derive to satisfy the request,
                // we might in fact derive even more than `unsatisfied_quantity` for
                // this unsatisfied `IndexAgnosticPath` so that the cache is filled
                // even after the request has been satisfied.
                pf_quantity_remaining_not_satisfied_by_cache.insert(
                    *factor_source_id,
                    QuantifiedDerivationPresets {
                        quantity: unsatisfied_quantity,
                        derivation_preset: quantified_agnostic_path.derivation_preset,
                    },
                );
            }

            // We don't wanna insert empty instances into `pf_found_in_cache`
            if !from_cache.is_empty() {
                pf_found_in_cache.insert(*factor_source_id, from_cache.clone());
            }
        }

        Ok(TakeFromCacheResult {
            need_to_derive_more_instances,
            pf_found_in_cache,
            pf_quantity_remaining_not_satisfied_by_cache,
        })
    }

    /// Derives more instances for the factor sources in `factor_sources`
    /// uses `pf_quantity_remaining_not_satisfied_by_cache` to split the newly
    /// derived ones in half, for each `IndexAgnosticPath`, for each factor sources
    /// and uses some of the newly derived to satisfy the request and the rest
    /// is saved them into the cache.
    ///
    /// We are also not deriving only for the `IndexAgnosticPath` that was unsatisfied,
    /// but rather we are deriving for all `IndexAgnosticPath` "presets" (account veci,
    /// identity veci, account mfa, identity mfa) for each factor source, i.e.
    /// we are filling the cache.
    #[allow(clippy::too_many_arguments)]
    async fn derive_more_instances(
        network_id: NetworkID,
        cache: &mut FactorInstancesCache,
        next_index_assigner: &NextDerivationEntityIndexAssigner,
        interactors: Arc<dyn KeysDerivationInteractors>,
        factor_sources: IndexSet<HDFactorSource>,
        index_agnostic_path_and_quantity_per_factor_source: IndexMap<
            FactorSourceIDFromHash,
            QuantifiedDerivationPresets,
        >,
        pf_quantity_remaining_not_satisfied_by_cache: IndexMap<
            FactorSourceIDFromHash,
            QuantifiedDerivationPresets,
        >,
        pf_found_in_cache: IndexMap<FactorSourceIDFromHash, FactorInstances>,
    ) -> Result<InternalFactorInstancesProviderOutcome> {
        // Based on contents of cache and the original request, calculate derivation paths
        // for instances to be derived.
        let paths = Self::calculate_derivation_paths(
            network_id,
            cache,
            next_index_assigner,
            &pf_found_in_cache,
            &index_agnostic_path_and_quantity_per_factor_source,
            &pf_quantity_remaining_not_satisfied_by_cache,
        )?;

        // Actually derive more factor instances.
        let keys_collector = KeysCollector::new(factor_sources, paths, interactors)?;
        let newly_derived_instances = keys_collector.collect_keys().await;

        // Split instances into `(to_use_directly, to_cache)`, per factor source.
        // We include the "unsplit" `pf_newly_derived` just for debug/test purposes.
        let split_factor_instances = Self::split(
            network_id,
            newly_derived_instances,
            index_agnostic_path_and_quantity_per_factor_source,
            &pf_quantity_remaining_not_satisfied_by_cache,
            &pf_found_in_cache,
        )?;

        // Build of the "collection" of FactorInstances:
        // * to_cache
        // * to_use_directly
        // * (for tests/debug) found in cache
        // * (for tests/debug) newly derived
        //
        // And "transpose" them, into one collection per FactorSource.
        let outcome = InternalFactorInstancesProviderOutcome::transpose(
            split_factor_instances.pf_to_cache,
            split_factor_instances
                .pf_to_use_directly
                .into_iter()
                .map(|(k, v)| (k, v.clone()))
                .collect(),
            pf_found_in_cache,
            split_factor_instances.pf_newly_derived,
        );
        Ok(outcome)
    }

    fn split(
        network_id: NetworkID,
        newly_derived: KeyDerivationOutcome,
        index_agnostic_path_and_quantity_per_factor_source: IndexMap<
            FactorSourceIDFromHash,
            QuantifiedDerivationPresets,
        >,
        pf_quantity_remaining_not_satisfied_by_cache: &IndexMap<
            FactorSourceIDFromHash,
            QuantifiedDerivationPresets,
        >,
        pf_found_in_cache: &IndexMap<FactorSourceIDFromHash, FactorInstances>,
    ) -> Result<SplitFactorInstances> {
        // used to filter out factor instances to use directly from the newly derived, based on
        // `index_agnostic_path_and_quantity_per_factor_source` we map
        // from: `IndexMap::<FactorSourceIDFromHash, (DerivationPreset, usize)>`
        //    to `IndexSet::<IndexAgnosticPath>`
        //
        // If any `IndexAgnosticPath` is in `index_agnostic_paths_originally_requested`,
        // it means we should not cache instances of that IndexAgnosticPath unconditionally,
        // rather we need to perform a split, to see which `remaining` quantity should
        // be used directly, and which instances should be cached.
        let index_agnostic_paths_originally_requested =
            index_agnostic_path_and_quantity_per_factor_source
                .values()
                .cloned()
                .map(|q| IndexAgnosticPath::from((network_id, q.derivation_preset)))
                .collect::<IndexSet<_>>();

        let mut pf_to_cache = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();

        let mut pf_newly_derived = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();

        // we will use directly what was found in clone, but later when
        // we derive more, we will add those to `pf_to_use_directly`, but
        // not to `pf_found_in_cache`, but we will include `pf_found_in_cache` for
        // unit tests.
        let mut pf_to_use_directly = pf_found_in_cache.clone();

        // Now split the newly derived FactorInstances, per factor source, into
        // `to_cache` and into `to_use_directly`.
        for (f, instances) in newly_derived.factors_by_source.into_iter() {
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
                // `instance_matches_original_request` should be `false` if we used
                // the `FactorInstancesProvider` for purpose "PRE_DERIVE_KEYS_FOR_NEW_FACTOR_SOURCE",

                let instance_matches_original_request = index_agnostic_paths_originally_requested
                    .contains(&instance.derivation_path().agnostic());

                if instance_matches_original_request {
                    // Here we ensure to only use `remaining` many
                    // instances for `to_use_directly`, the rest
                    // should be cached!
                    if to_use_directly.len() < remaining {
                        to_use_directly.insert(instance);
                    } else {
                        to_cache.insert(instance);
                    }
                } else {
                    // Does not match original request, cache all!
                    to_cache.insert(instance);
                }
            }

            pf_to_cache.insert(f, to_cache.into());

            if let Some(existing_to_use_directly) = pf_to_use_directly.get_mut(&f) {
                // We already have inserted some FactorInstances to use directly for this
                // FactorSource, this is possible we have made a "composite" request
                // loading some AccountMFA FactorInstances **and** a ROLA key for example.
                existing_to_use_directly.extend(to_use_directly.into_iter());
            } else {
                pf_to_use_directly.insert(f, FactorInstances::from(to_use_directly));
            }
        }

        Ok(SplitFactorInstances {
            pf_to_cache,
            pf_to_use_directly,
            pf_newly_derived,
        })
    }

    fn calculate_derivation_paths(
        network_id: NetworkID,
        cache: &FactorInstancesCache, // not mutated
        next_index_assigner: &NextDerivationEntityIndexAssigner,
        pf_found_in_cache: &IndexMap<FactorSourceIDFromHash, FactorInstances>,
        index_agnostic_path_and_quantity_per_factor_source: &IndexMap<
            FactorSourceIDFromHash,
            QuantifiedDerivationPresets,
        >,
        pf_quantity_remaining_not_satisfied_by_cache: &IndexMap<
            FactorSourceIDFromHash,
            QuantifiedDerivationPresets,
        >,
    ) -> Result<IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>> {
        // Per FactorSource a set of NetworkIndexAgnostic Paths ("presets") to derive for
        // and the quantity to derive, will be built up using `DerivationPreset::all()`
        // and the originally requested in `index_agnostic_path_and_quantity_per_factor_source`.
        let mut pf_quantified_network_agnostic_paths_for_derivation = IndexMap::<
            FactorSourceIDFromHash,
            IndexSet<QuantifiedToCacheToUseDerivationPresets>,
        >::new();

        // Lets build up `pf_quantified_network_agnostic_paths_for_derivation`, which
        // contains
        // `index_agnostic_path_and_quantity_per_factor_source` contains the QuantifiedDerivationPresets for
        // each FactorSource originally requested, we are gonna fill it with
        // `DerivationPreset::all()` and for each gonna look up
        // how many instances we need to derive to fill the cache, but first
        // we are gonna check if any of the `DerivationPreset::all()` was
        // matches the IndexAgnosticPath of the original request.
        for factor_source_id in index_agnostic_path_and_quantity_per_factor_source.keys() {
            let partial = pf_quantity_remaining_not_satisfied_by_cache
                .get(factor_source_id)
                .cloned();
            for derivation_preset in DerivationPreset::all() {
                let to_derive = partial
                    .and_then(|p| {
                        if p.derivation_preset == derivation_preset {
                            Some(QuantifiedToCacheToUseDerivationPresets {
                                quantity: QuantityToCacheToUseDirectly::ToCacheToUseDirectly {
                                    remaining: p.quantity,
                                    extra_to_fill_cache: CACHE_FILLING_QUANTITY,
                                },
                                derivation_preset: p.derivation_preset,
                            })
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| {
                        let cache_key =
                            derivation_preset.index_agnostic_path_on_network(network_id);

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

                        QuantifiedToCacheToUseDerivationPresets {
                            quantity: QuantityToCacheToUseDirectly::OnlyCacheFilling {
                                fill_cache: number_of_instances_to_derive_to_fill_cache,
                                instance_with_max_index,
                            },
                            derivation_preset,
                        }
                    });

                if let Some(existing) =
                    pf_quantified_network_agnostic_paths_for_derivation.get_mut(factor_source_id)
                {
                    existing.insert(to_derive);
                } else {
                    pf_quantified_network_agnostic_paths_for_derivation
                        .insert(*factor_source_id, IndexSet::just(to_derive));
                }
            }
        }

        // Map `NetworkAgnostic -> IndexAgnosticPath`, by using `network_id`.
        let pf_quantified_index_agnostic_paths_for_derivation =
            pf_quantified_network_agnostic_paths_for_derivation
                .into_iter()
                .map(|(factor_source_id, quantified_network_agnostic_paths)| {
                    let index_agnostic_paths = quantified_network_agnostic_paths
                        .into_iter()
                        .map(|q| QuantifiedToCacheToUseIndexAgnosticPath {
                            agnostic_path: q
                                .derivation_preset
                                .index_agnostic_path_on_network(network_id),
                            quantity: q.quantity,
                        })
                        .collect::<IndexSet<_>>();
                    (factor_source_id, index_agnostic_paths)
                })
                .collect::<IndexMap<_, _>>();

        // Now map from IndexAgnostic paths to index aware paths, a.k.a. DerivationPath
        // but ALSO we need to retain the information of how many factor instances of
        // the newly derived to append to the factor instances to use directly, and how many to cache.
        let paths: Result<IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>> =
            pf_quantified_index_agnostic_paths_for_derivation
                .clone()
                .into_iter()
                .map(|(f, agnostic_paths)| {
                    let paths: Vec<IndexSet<DerivationPath>> = agnostic_paths
                        .clone()
                        .into_iter()
                        .map(|quantified_agnostic_path| {
                            // IMPORTANT! We are not mapping one `IndexAgnosticPath` to one `DerivationPath`, but
                            // rather we are mapping one `IndexAgnosticPath` to **MANY** `DerivationPath`s! Equal to
                            // the same number as the specified quantity!
                            (0..quantified_agnostic_path.quantity.total_quantity_to_derive())
                                .map(|_| {
                                    let index_agnostic_path =
                                        quantified_agnostic_path.agnostic_path;

                                    let index = next_index_assigner.next(
                                        f,
                                        index_agnostic_path,
                                        // Must also use cache based offsets, checking max between cache and
                                        // profile. Since profile might not contain the highest entity
                                        // derivation index, the cache might!
                                        quantified_agnostic_path
                                            .quantity
                                            .max_index()
                                            .map(|max_index| OffsetFromCache::KnownMax {
                                                instance: max_index,
                                            })
                                            .unwrap_or(OffsetFromCache::FindMaxInRemoved {
                                                pf_found_in_cache: pf_found_in_cache.clone(),
                                            }),
                                    )?;
                                    Ok(DerivationPath::from((
                                        quantified_agnostic_path.agnostic_path,
                                        index,
                                    )))
                                })
                                .collect::<Result<IndexSet<_>>>()
                        })
                        .collect::<Result<Vec<IndexSet<_>>>>()?;

                    let paths = paths.into_iter().flatten().collect::<IndexSet<_>>();

                    Ok((f, paths))
                })
                .collect::<Result<IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>>>();

        paths
    }
}
