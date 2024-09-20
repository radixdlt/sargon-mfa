use crate::prelude::*;

async fn _account_recovery_scan(
    input: DeriveAndAnalyzeInputAccountRecoveryScan,
) -> Result<DerivationAndAnalysisAccountRecoveryScan> {
    let input = DeriveAndAnalyzeInput::from(input);
    let analysis = derive_and_analyze(input).await?;
    DerivationAndAnalysisAccountRecoveryScan::try_from(analysis)
}

pub async fn account_recovery_scan(
    factor_sources: IndexSet<HDFactorSource>,
    gateway: Arc<dyn Gateway>,
) -> Result<AccountRecoveryScanOutcome> {
    let analysis = _account_recovery_scan(DeriveAndAnalyzeInputAccountRecoveryScan::new(
        factor_sources,
        gateway,
    ))
    .await?;
    Ok(AccountRecoveryScanOutcome::from(analysis))
}
