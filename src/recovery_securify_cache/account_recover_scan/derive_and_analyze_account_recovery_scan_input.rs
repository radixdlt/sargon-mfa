#![allow(unused)]
#![allow(unused_variables)]

use crate::{gateway, prelude::*};

/// Use by OARS (Onboarding Account Recovery Scan) and MARS
/// (Manual Account Recovery Scan).
pub struct DeriveAndAnalyzeAccountRecoveryScanInput {
    factor_sources: IndexSet<HDFactorSource>,
    gateway: Arc<dyn Gateway>,

    /// Empty for OARS, **maybe** not empty for MARS
    cache: PreDerivedKeysCache,
    /// Empty for OARS, **guaranteed** not empty for MARS
    profile: ProfileAnalyzer,

    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    is_done_query: Arc<dyn IsDerivationDoneQuery>,
}

impl DeriveAndAnalyzeAccountRecoveryScanInput {
    fn new(
        factor_sources: IndexSet<HDFactorSource>,
        gateway: Arc<dyn Gateway>,
        cache: impl Into<Option<PreDerivedKeysCache>>,
        profile: impl Into<Option<ProfileAnalyzer>>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        is_done_query: Arc<dyn IsDerivationDoneQuery>,
    ) -> Self {
        Self {
            factor_sources,
            gateway,
            cache: cache.into().unwrap_or_default(),
            profile: profile.into().unwrap_or_default(),
            derivation_interactors,
            is_done_query,
        }
    }

    /// OARS
    pub fn onboarding_account_recovery_scan(
        factor_sources: IndexSet<HDFactorSource>,
        gateway: Arc<dyn Gateway>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        is_done_query: Arc<dyn IsDerivationDoneQuery>,
    ) -> Self {
        Self::new(
            factor_sources,
            gateway,
            None,
            None,
            derivation_interactors,
            is_done_query,
        )
    }

    /// MARS
    pub fn manual_account_recovery_scan(
        factor_sources: IndexSet<HDFactorSource>,
        gateway: Arc<dyn Gateway>,
        cache: impl Into<Option<PreDerivedKeysCache>>,
        profile: ProfileAnalyzer,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        is_done_query: Arc<dyn IsDerivationDoneQuery>,
    ) -> Self {
        Self::new(
            factor_sources,
            gateway,
            cache,
            profile,
            derivation_interactors,
            is_done_query,
        )
    }
}

#[derive(Debug, Default)]
pub struct ProfileAnalyzer;

pub struct FactorInstancesProviderImpl {
    cache: PreDerivedKeysCache,
    gateway: Option<Arc<dyn Gateway>>,
    profile: ProfileAnalyzer,
}
impl FactorInstancesProviderImpl {
    /// # Panics
    /// Panics if all arguments are `None`.
    fn new(
        cache: impl Into<Option<PreDerivedKeysCache>>,
        gateway: impl Into<Option<Arc<dyn Gateway>>>,
        profile: impl Into<Option<ProfileAnalyzer>>,
    ) -> Self {
        let cache = cache.into();
        let gateway = gateway.into();
        let profile = profile.into();
        assert!(
            !(cache.is_none() && gateway.is_none() && profile.is_none()),
            "All arguments are None"
        );
        Self {
            cache: cache.unwrap_or_default(),
            gateway,
            profile: profile.unwrap_or_default(),
        }
    }

    /// OARS
    pub fn onboarding_account_recovery_scan(gateway: Arc<dyn Gateway>) -> Self {
        Self::new(None, gateway, None)
    }

    /// MARS
    pub fn manual_account_recovery_scan(
        cache: impl Into<Option<PreDerivedKeysCache>>,
        gateway: Arc<dyn Gateway>,
        profile: ProfileAnalyzer,
    ) -> Self {
        Self::new(cache, gateway, profile)
    }
}

#[async_trait::async_trait]
impl IsFactorInstancesProvider for FactorInstancesProviderImpl {
    async fn provide_instances(
        &self,
        derivation_requests: DerivationRequests,
    ) -> Result<FactorInstances> {
        let cache_outcome = self.cache.load(&derivation_requests).await?;
        todo!();
    }
}

#[async_trait::async_trait]
impl IsIntermediaryDerivationAnalyzer for FactorInstancesProviderImpl {
    async fn analyze(
        &self,
        factor_instances: FactorInstances,
    ) -> Result<IntermediaryDerivationAnalysis> {
        todo!()
    }
}

impl From<DeriveAndAnalyzeAccountRecoveryScanInput> for DeriveAndAnalyzeInput {
    #[allow(clippy::diverging_sub_expression)]
    fn from(value: DeriveAndAnalyzeAccountRecoveryScanInput) -> Self {
        let unfactored_derivation_requests = AnyFactorDerivationRequest::many_for_each_on(
            NetworkID::Mainnet,
            [CAP26EntityKind::Account],
            [CAP26KeyKind::TransactionSigning],
            [KeySpace::Securified, KeySpace::Unsecurified],
        );

        let initial_derivation_requests = value
            .factor_sources
            .clone()
            .into_iter()
            .flat_map(|f| {
                let factor_source_id = f.factor_source_id();
                unfactored_derivation_requests
                    .clone()
                    .into_iter()
                    .map(move |u| u.derivation_request_with_factor_source_id(factor_source_id))
            })
            .collect::<DerivationRequests>();

        let analyzing_factor_instance_provider = Arc::new(FactorInstancesProviderImpl::new(
            value.cache,
            value.gateway,
            value.profile,
        ));

        Self::new(
            value.factor_sources.clone(),
            value
                .factor_sources
                .into_iter()
                .map(|f| f.factor_source_id())
                .collect(),
            initial_derivation_requests,
            analyzing_factor_instance_provider.clone(),
            analyzing_factor_instance_provider.clone(),
            value.is_done_query,
        )
    }
}

pub struct UncachedFactorInstanceProvider {
    factor_sources: IndexSet<HDFactorSource>,
    derivation_index_ranges_start_values:
        IndexMap<FactorSourceIDFromHash, IndexMap<DerivationRequest, HDPathValue>>,
    interactors: Arc<dyn KeysDerivationInteractors>,
}

impl UncachedFactorInstanceProvider {
    fn derivation_paths_for_requests(
        &self,
        derivation_requests: DerivationRequests,
    ) -> IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>> {
        todo!()
    }
}

#[async_trait::async_trait]
impl IsFactorInstancesProvider for UncachedFactorInstanceProvider {
    async fn provide_instances(
        &self,
        derivation_requests: DerivationRequests,
    ) -> Result<FactorInstances> {
        let derivation_paths = self.derivation_paths_for_requests(derivation_requests);
        let keys_collector = KeysCollector::new(
            self.factor_sources.clone(),
            derivation_paths,
            self.interactors.clone(),
        )?;
        let derived = keys_collector.collect_keys().await;
        Ok(derived.all_factors())
    }
}
