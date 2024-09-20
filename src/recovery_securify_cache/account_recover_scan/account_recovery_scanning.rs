use crate::prelude::*;

async fn _account_recovery_scan(
    input: DeriveAndAnalyzeAccountRecoveryScanInput,
) -> Result<DerivationAndAnalysisAccountRecoveryScan> {
    let input = DeriveAndAnalyzeInput::from(input);
    let analysis = derive_and_analyze(input).await?;
    DerivationAndAnalysisAccountRecoveryScan::try_from(analysis)
}

pub async fn account_recovery_scan(
    factor_sources: IndexSet<HDFactorSource>,
    gateway: Arc<dyn Gateway>,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
) -> Result<AccountRecoveryScanOutcome> {
    let analysis = _account_recovery_scan(DeriveAndAnalyzeAccountRecoveryScanInput::new(
        factor_sources,
        gateway,
        derivation_interactors,
    ))
    .await?;
    Ok(AccountRecoveryScanOutcome::from(analysis))
}
