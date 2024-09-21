use crate::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct IntermediaryDerivationAnalysis {
    pub probably_free_instances: ProbablyFreeFactorInstances,
    pub known_taken: KnownTakenInstances,
}

impl IntermediaryDerivationAnalysis {
    /// # Panics
    /// Panics if the collections of factor instances are not disjoint
    pub fn new(
        probably_free_instances: ProbablyFreeFactorInstances,
        known_taken: KnownTakenInstances,
    ) -> Self {
        assert_are_factor_instance_collections_disjoint(vec![
            &probably_free_instances,
            &known_taken,
        ]);

        Self {
            probably_free_instances,
            known_taken,
        }
    }

    pub fn merge(self, other: IntermediaryDerivationAnalysis) -> Self {
        Self::new(
            self.probably_free_instances
                .merge(other.probably_free_instances),
            self.known_taken.merge(other.known_taken),
        )
    }
}
