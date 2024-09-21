use crate::prelude::*;

async fn _account_recovery_scan(
    input: DeriveAndAnalyzeAccountRecoveryScanInput,
) -> Result<DerivationAndAnalysisAccountRecoveryScan> {
    let input = DeriveAndAnalyzeInput::from(input);
    let analysis = derive_and_analyze(input).await?;
    DerivationAndAnalysisAccountRecoveryScan::try_from(analysis)
}

pub async fn onboarding_account_recovery_scan(
    factor_sources: IndexSet<HDFactorSource>,
    gateway: Arc<dyn Gateway>,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    is_done_query: Arc<dyn IsDerivationDoneQuery>,
) -> Result<AccountRecoveryScanOutcome> {
    let analysis = _account_recovery_scan(
        DeriveAndAnalyzeAccountRecoveryScanInput::onboarding_account_recovery_scan(
            factor_sources,
            gateway,
            derivation_interactors,
            is_done_query,
        ),
    )
    .await?;
    Ok(AccountRecoveryScanOutcome::from(analysis))
}

pub async fn manual_account_recovery_scan(
    factor_sources: IndexSet<HDFactorSource>,
    gateway: Arc<dyn Gateway>,
    cache: impl Into<Option<PreDerivedKeysCache>>,
    profile: ProfileAnalyzer,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    is_done_query: Arc<dyn IsDerivationDoneQuery>,
) -> Result<AccountRecoveryScanOutcome> {
    let analysis = _account_recovery_scan(
        DeriveAndAnalyzeAccountRecoveryScanInput::manual_account_recovery_scan(
            factor_sources,
            gateway,
            cache,
            profile,
            derivation_interactors,
            is_done_query,
        ),
    )
    .await?;
    Ok(AccountRecoveryScanOutcome::from(analysis))
}
