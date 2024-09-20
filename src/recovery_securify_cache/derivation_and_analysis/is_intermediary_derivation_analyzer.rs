use crate::prelude::*;

/// Check if there is any known entity associated with a given factor instance,
/// if so, some base info, if not, it is counted as "probably free".
#[async_trait::async_trait]
pub trait IsIntermediaryDerivationAnalyzer: Sync + Send {
    async fn analyze(
        &self,
        factor_instances: IndexSet<HierarchicalDeterministicFactorInstance>,
    ) -> Result<IntermediaryDerivationAnalysis>;
}
