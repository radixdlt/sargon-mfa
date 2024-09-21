#![allow(unused)]

use crate::prelude::*;

#[derive(Debug)]
pub struct ProfileAnalyzer {
    profile: Profile,
}

impl ProfileAnalyzer {
    pub fn new(profile: Profile) -> Self {
        Self { profile }
    }
}

#[async_trait::async_trait]
impl IsIntermediaryDerivationAnalyzer for ProfileAnalyzer {
    async fn analyze(
        &self,
        factor_instances: &FactorInstances,
    ) -> Result<IntermediaryDerivationAnalysis> {
        warn!("Not implemented");
        let recovered_unsecurified_entities = RecoveredUnsecurifiedEntities::sample_other();

        let recovered_securified_entities = RecoveredSecurifiedEntities::sample_other();

        let unrecovered_securified_entities = UnrecoveredSecurifiedEntities::sample_other();

        let virtual_entity_creating_instances = VirtualEntityCreatingInstances::sample_other();

        let known_taken = KnownTakenInstances::new(
            recovered_unsecurified_entities,
            recovered_securified_entities,
            unrecovered_securified_entities,
            virtual_entity_creating_instances,
        );

        let probably_free = ProbablyFreeFactorInstances::sample_other();

        let analysis = IntermediaryDerivationAnalysis::new(probably_free, known_taken);

        Ok(analysis)
    }
}
