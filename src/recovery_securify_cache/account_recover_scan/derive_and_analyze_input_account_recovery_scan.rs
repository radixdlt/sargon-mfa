use crate::prelude::*;

pub struct DeriveAndAnalyzeInputAccountRecoveryScan {
    factor_sources: IndexSet<HDFactorSource>,
    gateway: Arc<dyn Gateway>,
}
impl DeriveAndAnalyzeInputAccountRecoveryScan {
    pub fn new(factor_sources: IndexSet<HDFactorSource>, gateway: Arc<dyn Gateway>) -> Self {
        Self {
            factor_sources,
            gateway,
        }
    }
}
impl From<DeriveAndAnalyzeInputAccountRecoveryScan> for DeriveAndAnalyzeInput {
    fn from(value: DeriveAndAnalyzeInputAccountRecoveryScan) -> Self {
        Self::new(
            value.factor_sources.clone(),
            value
                .factor_sources
                .into_iter()
                .map(|f| f.factor_source_id())
                .collect(),
            None,
            value.gateway,
            true,
            None,
        )
    }
}
