use std::sync::{Arc, RwLock};

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FillCacheStrategy {
    /// Fill only the originally requested derivation templates, i.e. if an
    /// account veci is requested, and cache could not satisfy the request with
    /// spare instances, and we derive more, we ONLY derive Account vecis, not
    /// e.g. MFA instances, or Identity vecis.
    OnlyRequestedTemplates,

    /// Fill cache with all templates, i.e. if an account veci is requested, and
    /// cache could not satisfy the request with spare instances, and we derive more,
    /// we derive all kinds of instances, e.g. MFA instances, Identity vecis etc.
    AllTemplates,
}
impl Default for FillCacheStrategy {
    fn default() -> Self {
        Self::AllTemplates
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactorInstancesProviderOutcome {
    /// FactorInstances that have not been saved into the cache, which are meant
    /// to be used directly. Can be empty. In case of Account veci this will
    /// be a single FactorInstance.
    pub factor_instances_to_use_directly: InstancesToUseDirectly,

    /// Statistics
    pub statistics: FactorInstancesProviderStatistics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactorInstancesProviderStatistics {
    pub number_of_instances_newly_derived: usize,
    pub number_of_instances_loaded_from_cache: usize,
    pub number_of_instances_saved_into_cache: usize,
}

pub struct FactorInstancesProvider {
    fill_cache_strategy: FillCacheStrategy,

    /// We only derive factor instances for one network at a time, currently, this
    /// can be expanded in the future if we want to, but most users only care
    /// about mainnet.
    network_id: NetworkID,

    /// A Clone of a cache, the caller MUST commit the changes to the
    /// original cache if they want to persist them.
    #[allow(dead_code)]
    cache: RwLock<FactorInstancesForSpecificNetworkCache>,

    next_entity_index_assigner: NextDerivationEntityIndexAssigner,

    derivation_interactors: Arc<dyn KeysDerivationInteractors>,

    query: InstancesQuery,
}

impl FactorInstancesProvider {
    /// `Profile` is optional since None in case of Onboarding Account Recovery Scan
    /// No need to pass Profile as mut, since we just need to read it for the
    /// next derivation entity indices.
    fn new(
        fill_cache_strategy: FillCacheStrategy,
        cache_on_network: FactorInstancesForSpecificNetworkCache,
        profile: impl Into<Option<Profile>>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        query: InstancesQuery,
    ) -> Self {
        let network_id = cache_on_network.network_id;
        Self {
            fill_cache_strategy,
            network_id,
            cache: RwLock::new(cache_on_network),
            next_entity_index_assigner: NextDerivationEntityIndexAssigner::new(
                network_id,
                profile.into(),
            ),
            derivation_interactors,
            query,
        }
    }

    pub async fn provide(
        fill_cache_strategy: FillCacheStrategy,
        cache: Arc<RwLock<FactorInstancesForEachNetworkCache>>,
        network_id: NetworkID,
        profile: impl Into<Option<Profile>>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        query: InstancesQuery,
    ) -> Result<FactorInstancesProviderOutcome> {
        let cloned_cache: FactorInstancesForSpecificNetworkCache =
            cache.read().unwrap().clone_for_network_or_empty(network_id);
        let provider = Self::new(
            fill_cache_strategy,
            cloned_cache,
            profile,
            derivation_interactors,
            query,
        );
        let provided = provider._provide().await?;
        cache.write().unwrap().merge(provided.cache_to_persist)?;
        Ok(FactorInstancesProviderOutcome {
            factor_instances_to_use_directly: provided.instances_to_be_used,
            statistics: provided.statistics,
        })
    }

    async fn _provide(self) -> Result<ProvidedInstances> {
        match self.query.clone() {
            InstancesQuery::AccountMfa {
                number_of_instances_per_factor_source,
                factor_sources,
            } => {
                self.provide_accounts_mfa(number_of_instances_per_factor_source, factor_sources)
                    .await
            }
            InstancesQuery::AccountVeci { factor_source } => {
                self.provide_account_veci(factor_source).await
            }
        }
    }
}

impl FactorInstancesProvider {
    fn paths(
        &self,
        indices_of_original_request: FillCacheIndicesPerFactor,
        quantities: QuantitiesPerFactor,
    ) -> DerivationPathPerFactorSource {
        println!(
            "🤖 indices_of_original_request: {:?}",
            indices_of_original_request
        );
        println!("🤖 quantities: {:?}", quantities);
        assert_eq!(
            indices_of_original_request
                .per_factor_source
                .keys()
                .collect::<HashSet<_>>(),
            quantities.per_factor_source.keys().collect::<HashSet<_>>(),
            "Discrepancy, every index needs a quantity, and vice versa."
        );
        let network_id = self.network_id;

        let mut paths_per_template_per_factor = IndexMap::<
            FactorSourceIDFromHash,
            IndexMap<DerivationTemplate, IndexSet<DerivationPath>>,
        >::new();

        let indices = match self.fill_cache_strategy {
            FillCacheStrategy::OnlyRequestedTemplates => {
                indices_of_original_request.per_factor_source
            }
            FillCacheStrategy::AllTemplates => {
                indices_of_original_request.merge_filling_cache(&self.next_entity_index_assigner)
            }
        };

        for (factor_source_id, indices) in indices.into_iter() {
            let quantities_for_factor =
                quantities.per_factor_source.get(&factor_source_id).unwrap();
            let mut paths_per_template_for_factor =
                IndexMap::<DerivationTemplate, IndexSet<DerivationPath>>::new();
            for (derivation_template, index) in indices.indices.into_iter() {
                println!("🤖 derivation_template: {:?}", derivation_template);

                let quantity = quantities_for_factor.quantity_for_template(derivation_template);
                let start_index = index.base_index();
                let end_index = start_index + quantity.value as u32;
                let range = start_index..end_index;
                let paths_for_template = range
                    .map(|i| {
                        DerivationPath::new(
                            network_id,
                            derivation_template.entity_kind(),
                            derivation_template.key_kind(),
                            HDPathComponent::new_with_key_space_and_index(
                                derivation_template.key_space(),
                                i,
                            )
                            .unwrap(),
                        )
                    })
                    .collect::<IndexSet<DerivationPath>>();
                paths_per_template_for_factor.insert(derivation_template, paths_for_template);
            }
            paths_per_template_per_factor.insert(factor_source_id, paths_per_template_for_factor);
        }
        println!(
            "🤖 paths_per_template_per_factor: {:?}",
            paths_per_template_per_factor
        );
        DerivationPathPerFactorSource {
            paths_per_template_per_factor,
        }
    }

    async fn derive(&self, paths: DerivationPathPerFactorSource) -> Result<KeyDerivationOutcome> {
        let factor_sources = self.query.factor_sources();
        let derivation_paths = paths.flatten();
        let keys_collector = KeysCollector::new(
            factor_sources,
            derivation_paths,
            self.derivation_interactors.clone(),
        )?;
        let outcome = keys_collector.collect_keys().await;
        Ok(outcome)
    }

    fn split_with(
        network_id: NetworkID,
        query: InstancesQuery,
        from_cache: Option<HierarchicalDeterministicFactorInstance>,
        derived: KeyDerivationOutcome,
    ) -> (InstancesToUseDirectly, InstancesToCache) {
        let derived = derived.factors_by_source;
        match query {
            InstancesQuery::AccountMfa {
                number_of_instances_per_factor_source: _,
                factor_sources: _,
            } => todo!(),
            InstancesQuery::AccountVeci { factor_source } => {
                let derived = derived
                    .get(&factor_source.factor_source_id())
                    .unwrap()
                    .clone();
                if let Some(from_cache) = from_cache {
                    (
                        InstancesToUseDirectly::just(from_cache),
                        InstancesToCache::from((
                            network_id,
                            IndexMap::kv(factor_source.factor_source_id(), derived),
                        )),
                    )
                } else {
                    let derived = derived.into_iter().collect_vec();
                    let (head, tail) = derived.split_at(1);
                    assert_eq!(head.len(), 1);
                    assert!(!tail.is_empty());
                    let head = head.first().unwrap().clone();
                    let tail = tail.iter().cloned().collect::<IndexSet<_>>();
                    (
                        InstancesToUseDirectly::just(head),
                        InstancesToCache::from((
                            network_id,
                            IndexMap::kv(factor_source.factor_source_id(), tail),
                        )),
                    )
                }
            }
        }
    }

    fn split(
        &self,
        from_cache: Option<HierarchicalDeterministicFactorInstance>,
        derived: KeyDerivationOutcome,
        _triple: QuantitiesTripleForFactor,
    ) -> (InstancesToUseDirectly, InstancesToCache) {
        Self::split_with(self.network_id, self.query.clone(), from_cache, derived)
    }
}

impl HDPathComponent {
    pub fn next(&self) -> Self {
        self.add_one()
    }
}

impl FactorInstancesProvider {
    async fn provide_account_veci(
        self,
        factor_source: HDFactorSource,
    ) -> Result<ProvidedInstances> {
        let template = DerivationTemplate::AccountVeci;
        let factor_source_id = factor_source.factor_source_id();

        let maybe_cached = self
            .cache
            .write()
            .unwrap()
            .consume_account_veci(factor_source_id);

        let mut maybe_next_index_for_derivation: Option<HDPathComponent> = None;

        let mut veci: Option<HierarchicalDeterministicFactorInstance> = None;

        let mut to_cache: Option<CollectionsOfFactorInstances> = None;

        let mut number_of_instances_loaded_from_cache = 0;
        let mut number_of_instances_saved_into_cache = 0;

        if let Some(cached) = maybe_cached {
            veci = Some(cached.instance.clone());
            number_of_instances_loaded_from_cache = 1;
            if cached.was_last_used {
                // TODO: Should we introduce a check to see if `next` is in fact free??? Check against Profile that is...
                // lets try skipping it for now
                maybe_next_index_for_derivation =
                    Some(cached.instance.derivation_path().index.next()); // expected to be UnsecurifiedIndex
            }
        } else {
            let next = self
                .next_entity_index_assigner
                .next_account_veci(factor_source_id);
            info!(
                "Cache empty of Account VECIs, using next index assigner for derivation: {:?}",
                next
            );
            maybe_next_index_for_derivation = Some(next)
        }

        assert!(
            !(veci.is_none() && maybe_next_index_for_derivation.is_none()),
            "Programmer error, both 'veci' and 'maybe_next_index_for_derivation' cannot be none."
        );

        if let Some(next_index_for_derivation) = maybe_next_index_for_derivation {
            let fill_cache_quantities_upper_bound = QuantitiesForFactor::fill(factor_source_id);

            println!(
                "🎃 fill_cache_quantities_upper_bound: {:?}",
                fill_cache_quantities_upper_bound
            );

            println!(
                "🎃 fill_cache_quantities_upper_bound: {:?}",
                fill_cache_quantities_upper_bound
            );

            let triple = {
                // TODO: For MFA we will do this
                // FOR EACH factor source FOR EACH derivation template
                // (or FOR EACH derivation template FOR EACH factor source)
                let in_cache = self
                    .cache
                    .read()
                    .unwrap()
                    .peek_all_instances_of_factor_source(factor_source.factor_source_id());

                fill_cache_quantities_upper_bound
                    .calculate_quantities_for_factor(Quantities::only(1, template), in_cache)
            };

            println!("🎃 triple: {:?}", triple);

            let paths = self.paths(
                FillCacheIndicesPerFactor::just(
                    factor_source_id,
                    template,
                    next_index_for_derivation,
                ),
                QuantitiesPerFactor::just(QuantitiesForFactor::new(
                    triple.clone().factor_source_id,
                    triple.clone().to_derive,
                )),
            );

            println!("🎃 paths: {:?}", paths);

            let derived = self.derive(paths).await?;
            let (split_to_use_directly, split_to_cache) = self.split(veci, derived, triple);
            assert_eq!(split_to_cache.0.len(), 1, "expected single factor");
            let split_to_cache = split_to_cache.0.values().last().unwrap().clone();
            // unconditionally set `veci`, since `split` should handle logic of it
            // being `None` or not.
            veci = Some(split_to_use_directly.account_veci()?.instance());
            to_cache = Some(split_to_cache);
        }
        let veci = veci.ok_or(CommonError::ExpectedVeci)?;
        if let Some(to_cache) = to_cache {
            number_of_instances_saved_into_cache = to_cache.total_number_of_instances();
            self.cache
                .write()
                .unwrap()
                .append_for_factor(factor_source_id, to_cache)?;
        }
        let cache = self.cache.into_inner().unwrap();
        let statistics = FactorInstancesProviderStatistics {
            number_of_instances_newly_derived: 1,
            number_of_instances_loaded_from_cache,
            number_of_instances_saved_into_cache,
        };
        Ok(ProvidedInstances::for_account_veci(cache, veci, statistics))
    }

    async fn provide_accounts_mfa(
        &self,
        _number_of_instances_per_factor_source: usize,
        _factor_sources: IndexSet<HDFactorSource>,
    ) -> Result<ProvidedInstances> {
        todo!()
    }
}

#[cfg(test)]
impl FactorInstancesForEachNetworkCache {
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
        let cache = Arc::new(RwLock::new(FactorInstancesForEachNetworkCache::default()));

        let network = NetworkID::Mainnet;
        let profile = Profile::default();
        let bdfs = HDFactorSource::sample();

        let outcome = Sut::provide(
            FillCacheStrategy::AllTemplates,
            cache.clone(),
            network,
            profile,
            Arc::new(TestDerivationInteractors::default()),
            InstancesQuery::AccountVeci {
                factor_source: bdfs.clone(),
            },
        )
        .await
        .unwrap();

        assert_eq!(outcome.statistics.number_of_instances_newly_derived, 1);
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
    }
}
