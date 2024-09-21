#![allow(unused)]
#![allow(unused_variables)]

use crate::{gateway, prelude::*};

/// Use by OARS (Onboarding Account Recovery Scan) and MARS
/// (Manual Account Recovery Scan).
pub struct DeriveAndAnalyzeAccountRecoveryScanInput {
    factor_sources: IndexSet<HDFactorSource>,
    gateway: Arc<dyn Gateway>,

    /// `None` for OARS, **maybe** `Some` for MARS
    cache: Option<PreDerivedKeysCache>,
    /// `None` for OARS, **guaranteed** `Some` for MARS
    profile: Option<ProfileAnalyzer>,

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
            cache: cache.into(),
            profile: profile.into(),
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
            value.factor_sources.clone(),
            value.derivation_interactors,
            value.cache,
            OnChainAnalyzer::new(value.gateway.clone()),
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
