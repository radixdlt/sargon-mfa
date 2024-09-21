use crate::prelude::*;

use super::onchain_analyzer;

pub struct FactorInstancesProviderImpl {
    factor_sources: IndexSet<HDFactorSource>,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    cache: Option<PreDerivedKeysCache>,
    onchain_analyzer: Option<OnChainAnalyzer>,
    profile_analyzer: Option<ProfileAnalyzer>,
}
impl FactorInstancesProviderImpl {
    /// # Panics
    /// Panics if all arguments are `None`.
    pub fn new(
        factor_sources: IndexSet<HDFactorSource>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        cache: impl Into<Option<PreDerivedKeysCache>>,
        onchain_analyzer: impl Into<Option<OnChainAnalyzer>>,
        profile_analyzer: impl Into<Option<ProfileAnalyzer>>,
    ) -> Self {
        let cache = cache.into();
        let onchain_analyzer = onchain_analyzer.into();
        let profile_analyzer = profile_analyzer.into();
        assert!(
            !(cache.is_none() && profile_analyzer.is_none() && profile_analyzer.is_none()),
            "All arguments are None"
        );
        Self {
            factor_sources,
            derivation_interactors,
            cache,
            onchain_analyzer,
            profile_analyzer,
        }
    }

    /// OARS
    pub fn onboarding_account_recovery_scan(
        factor_sources: IndexSet<HDFactorSource>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        onchain_analyzer: OnChainAnalyzer,
    ) -> Self {
        Self::new(
            factor_sources,
            derivation_interactors,
            None,
            onchain_analyzer,
            None,
        )
    }

    /// MARS
    pub fn manual_account_recovery_scan(
        factor_sources: IndexSet<HDFactorSource>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        cache: impl Into<Option<PreDerivedKeysCache>>,
        onchain_analyzer: OnChainAnalyzer,
        profile_analyzer: ProfileAnalyzer,
    ) -> Self {
        Self::new(
            factor_sources,
            derivation_interactors,
            cache,
            onchain_analyzer,
            profile_analyzer,
        )
    }
}

impl FactorInstancesProviderImpl {
    pub async fn derive_more(&self, next_requests: DerivationRequests) -> Result<FactorInstances> {
        let keys_collector = KeysCollector::new(
            self.factor_sources.clone(),
            next_requests.derivation_paths(),
            self.derivation_interactors.clone(),
        )?;

        let outcome = keys_collector.collect_keys().await;
        Ok(outcome.all_factors())
    }
}

impl FactorInstancesProviderImpl {
    async fn provide_instances_with_cache(
        &self,
        cache: &PreDerivedKeysCache,
        derivation_requests: DerivationRequests,
    ) -> Result<FactorInstances> {
        let CacheOutcome {
            fully_satisfied,
            has_spare_capacity,
            mut factor_instances,
            ..
        } = cache.load(&derivation_requests).await?;

        let derive_more = !fully_satisfied || !has_spare_capacity;
        if derive_more {
            let next_requests = derivation_requests.next();
            let more = self.derive_more(next_requests).await?;
            factor_instances = factor_instances.merge(more);
        }

        factor_instances.filter_satisfying(&derivation_requests)
    }
}

#[async_trait::async_trait]
impl IsFactorInstancesProvider for FactorInstancesProviderImpl {
    async fn provide_instances(
        &self,
        derivation_requests: DerivationRequests,
    ) -> Result<FactorInstances> {
        if let Some(ref cache) = self.cache {
            self.provide_instances_with_cache(cache, derivation_requests)
                .await
        } else {
            self.derive_more(derivation_requests).await
        }
    }
}

#[async_trait::async_trait]
impl IsIntermediaryDerivationAnalyzer for FactorInstancesProviderImpl {
    async fn analyze(
        &self,
        factor_instances: &FactorInstances,
    ) -> Result<IntermediaryDerivationAnalysis> {
        let mut analysis = IntermediaryDerivationAnalysis::default();

        if let Some(ref analyzer) = self.onchain_analyzer {
            let onchain_analysis = analyzer.analyze(factor_instances).await?;
            analysis = analysis.merge(onchain_analysis);
        }

        if let Some(ref analyzer) = self.profile_analyzer {
            let profile_analysis = analyzer.analyze(factor_instances).await?;
            analysis = analysis.merge(profile_analysis);
        }

        Ok(analysis)
    }
}
