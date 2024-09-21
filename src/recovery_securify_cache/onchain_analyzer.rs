#![allow(unused)]

use crate::prelude::*;

pub struct OnChainAnalyzer {
    gateway: Arc<dyn Gateway>,
}
impl OnChainAnalyzer {
    pub fn new(gateway: Arc<dyn Gateway>) -> Self {
        Self { gateway }
    }
}

#[async_trait::async_trait]
impl IsIntermediaryDerivationAnalyzer for OnChainAnalyzer {
    async fn analyze(
        &self,
        factor_instances: &FactorInstances,
    ) -> Result<IntermediaryDerivationAnalysis> {
        warn!("Not implemented");
        let recovered_unsecurified_entities = RecoveredUnsecurifiedEntities::sample();

        let recovered_securified_entities = RecoveredSecurifiedEntities::sample();

        let unrecovered_securified_entities = UnrecoveredSecurifiedEntities::sample();

        let virtual_entity_creating_instances = VirtualEntityCreatingInstances::sample();

        let known_taken = KnownTakenInstances::new(
            recovered_unsecurified_entities,
            recovered_securified_entities,
            unrecovered_securified_entities,
            virtual_entity_creating_instances,
        );

        let probably_free = ProbablyFreeFactorInstances::sample();

        let analysis = IntermediaryDerivationAnalysis::new(probably_free, known_taken);

        Ok(analysis)
    }
}
