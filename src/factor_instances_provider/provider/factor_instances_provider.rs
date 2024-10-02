use std::sync::{Arc, RwLock};

use crate::prelude::*;

pub struct FactorInstancesProvider {
    /// A Clone of a cache, the caller MUST commit the changes to the
    /// original cache if they want to persist them.
    #[allow(dead_code)]
    cache: RwLock<FactorInstancesForSpecificNetworkCache>,

    query: InstancesQuery,

    next_entity_index_assigner: NextDerivationEntityIndexAssigner,
}

impl FactorInstancesProvider {
    /// `Profile` is optional since None in case of Onboarding Account Recovery Scan
    /// No need to pass Profile as mut, since we just need to read it for the
    /// next derivation entity indices.
    fn new(
        cache_on_network: FactorInstancesForSpecificNetworkCache,
        profile: impl Into<Option<Profile>>,
        query: InstancesQuery,
    ) -> Self {
        let network_id = cache_on_network.network_id;
        Self {
            cache: RwLock::new(cache_on_network),
            query,
            next_entity_index_assigner: NextDerivationEntityIndexAssigner::new(
                network_id,
                profile.into(),
            ),
        }
    }

    pub async fn provide(
        cache: Arc<RwLock<FactorInstancesForEachNetworkCache>>,
        network_id: NetworkID,
        profile: impl Into<Option<Profile>>,
        query: InstancesQuery,
    ) -> Result<ToUseDirectly> {
        let cloned_cache = cache.read().unwrap().clone_for_network_or_empty(network_id);
        let provider = Self::new(cloned_cache, profile, query);
        let provided = provider._provide().await?;
        cache.write().unwrap().merge(provided.cache_to_persist)?;
        Ok(provided.instances_to_be_used)
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
    fn paths_single_factor(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        known_indices_for_templates: IndexMap<DerivationTemplate, HDPathComponent>,
        fill_cache: FillCacheQuantitiesForFactor,
    ) -> DerivationPathPerFactorSource {
        todo!()
    }

    async fn derive(&self, paths: DerivationPathPerFactorSource) -> Result<KeyDerivationOutcome> {
        todo!()
    }
    fn split(
        &self,
        from_cache: Option<HierarchicalDeterministicFactorInstance>,
        derived: KeyDerivationOutcome,
    ) -> (ToUseDirectly, ToCache) {
        todo!()
    }
}

impl HDPathComponent {
    pub fn next(&self) -> Self {
        todo!()
    }
}

impl FactorInstancesProvider {
    async fn provide_account_veci(
        self,
        factor_source: HDFactorSource,
    ) -> Result<ProvidedInstances> {
        let factor_source_id = factor_source.factor_source_id();

        let maybe_cached = self
            .cache
            .write()
            .unwrap()
            .consume_account_veci(factor_source_id);
        let mut maybe_next_index_for_derivation: Option<HDPathComponent> = None;
        let mut veci: Option<HierarchicalDeterministicFactorInstance> = None;
        let mut to_cache: Option<ToCache> = None;
        if let Some(cached) = maybe_cached {
            veci = Some(cached.instance.clone());
            if cached.was_last_used {
                // TODO: Must we check if `next` is in fact free??? Check against Profile that is...
                // lets try skipping it for now
                maybe_next_index_for_derivation =
                    Some(cached.instance.derivation_path().index.next()); // expected to be UnsecurifiedIndex
            }
        } else {
            maybe_next_index_for_derivation = Some(
                self.next_entity_index_assigner
                    .next_account_veci(factor_source_id),
            )
        }
        assert!(
            !(veci.is_none() && maybe_next_index_for_derivation.is_none()),
            "Programmer error, both 'veci' and 'maybe_next_index_for_derivation' cannot be none."
        );
        if let Some(next_index_for_derivation) = maybe_next_index_for_derivation {
            // furthermore, since we are deriving ANYWAY, we should also derive to Fill The Cache....
            let fill_cache_maybe_over_estimated =
                FillCacheQuantitiesForFactor::fill(factor_source_id);

            let existing = self
                .cache
                .read()
                .unwrap()
                .peek_all_instances_for_factor_source(factor_source.factor_source_id());

            let fill_cache = fill_cache_maybe_over_estimated.subtracting_existing(existing);

            let paths = self.paths_single_factor(
                factor_source_id,
                IndexMap::from_iter([(DerivationTemplate::AccountVeci, next_index_for_derivation)]),
                fill_cache,
            );

            let derived = self.derive(paths).await?;
            let (split_to_use_directly, split_to_cache) = self.split(veci, derived);

            // unconditionally set `veci`, since `split` should handle logic of it
            // being `None` or not.
            veci = Some(split_to_use_directly.account_veci()?.instance());
            to_cache = Some(split_to_cache);
        }
        let veci = veci.ok_or(CommonError::ExpectedVeci)?;
        if let Some(to_cache) = to_cache {
            self.cache
                .write()
                .unwrap()
                .append_for_factor(factor_source_id, to_cache)?;
        }
        let cache = self.cache.into_inner().unwrap();
        Ok(ProvidedInstances::for_account_veci(cache, veci))
    }

    async fn provide_accounts_mfa(
        &self,
        number_of_instances_per_factor_source: usize,
        factor_sources: IndexSet<HDFactorSource>,
    ) -> Result<ProvidedInstances> {
        todo!()
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
            cache.clone(),
            network,
            profile,
            InstancesQuery::AccountVeci {
                factor_source: bdfs.clone(),
            },
        )
        .await
        .unwrap();

        assert!(cache
            .try_read()
            .unwrap()
            .clone_for_network(network)
            .unwrap()
            .peek_all_instances_for_factor_source(bdfs.factor_source_id())
            .unwrap()
            .is_full());

        assert_eq!(
            outcome.account_veci().unwrap().derivation_entity_index(),
            HDPathComponent::Hardened(HDPathComponentHardened::Unsecurified(
                UnsecurifiedIndex::unsecurified_hardening_base_index(0)
            ))
        );
    }
}
