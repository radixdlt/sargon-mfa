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
    fn from(value: DeriveAndAnalyzeAccountRecoveryScanInput) -> Self {
        let next_derivation_entity_index_assigner = NextDerivationEntityIndexAssigner::ars();

        let analyze_factor_instances = IntermediaryDerivationAnalyzer::ars(value.gateway);

        Self::new(
            value.factor_sources.clone(),
            value
                .factor_sources
                .into_iter()
                .map(|f| f.factor_source_id())
                .collect(),
            next_derivation_entity_index_assigner,
            analyze_factor_instances,
        )
    }
}

impl NextDerivationEntityIndexAssigner {
    pub fn ars() -> Self {
        todo!()
    }
}

impl IntermediaryDerivationAnalyzer {
    pub fn ars(gateway: Arc<dyn Gateway>) -> Self {
        todo!()
    }
}
