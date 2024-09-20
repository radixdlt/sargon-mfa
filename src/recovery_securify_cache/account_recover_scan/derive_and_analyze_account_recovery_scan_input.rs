#![allow(unused)]
#![allow(unused_variables)]

use crate::prelude::*;

pub struct DeriveAndAnalyzeAccountRecoveryScanInput {
    factor_sources: IndexSet<HDFactorSource>,
    gateway: Arc<dyn Gateway>,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
}

impl DeriveAndAnalyzeAccountRecoveryScanInput {
    pub fn new(
        factor_sources: IndexSet<HDFactorSource>,
        gateway: Arc<dyn Gateway>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        Self {
            factor_sources,
            gateway,
            derivation_interactors,
        }
    }
}
impl From<DeriveAndAnalyzeAccountRecoveryScanInput> for DeriveAndAnalyzeInput {
    #[allow(clippy::diverging_sub_expression)]
    fn from(value: DeriveAndAnalyzeAccountRecoveryScanInput) -> Self {
        let initial_derivation_requests = IndexSet::<DerivationRequest>::new();
        let factor_instances_provider: Arc<dyn IsFactorInstancesProvider> = { unreachable!() };
        let analyze_factor_instances: Arc<dyn IsIntermediaryDerivationAnalyzer> =
            { unreachable!() };

        Self::new(
            value.factor_sources.clone(),
            value
                .factor_sources
                .into_iter()
                .map(|f| f.factor_source_id())
                .collect(),
            initial_derivation_requests,
            factor_instances_provider,
            analyze_factor_instances,
        )
    }
}
