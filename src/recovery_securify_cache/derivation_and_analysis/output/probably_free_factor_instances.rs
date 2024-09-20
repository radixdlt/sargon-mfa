use crate::prelude::*;

/// "Probably" since we might not have all the information to be sure, since
/// Gateway might not keep track of past FactorInstances, some of the FactorInstances
/// in KeySpace::Securified might in fact have been used in the past for some entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProbablyFreeFactorInstances {
    factor_instances: Vec<HierarchicalDeterministicFactorInstance>,
}

impl ProbablyFreeFactorInstances {
    pub fn new(instances: IndexSet<HierarchicalDeterministicFactorInstance>) -> Self {
        Self {
            factor_instances: instances.into_iter().collect(),
        }
    }
    pub fn merge(&self, other: &Self) -> Self {
        Self::new(
            self.instances()
                .union(&other.instances())
                .cloned()
                .collect(),
        )
    }
    pub fn instances(&self) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        self.factor_instances.iter().cloned().collect()
    }
}

impl HasSampleValues for ProbablyFreeFactorInstances {
    fn sample() -> Self {
        Self::new(IndexSet::from_iter([
            HierarchicalDeterministicFactorInstance::sample(),
            HierarchicalDeterministicFactorInstance::sample_other(),
        ]))
    }

    fn sample_other() -> Self {
        Self::new(IndexSet::just(
            HierarchicalDeterministicFactorInstance::sample_other(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = ProbablyFreeFactorInstances;

    #[test]
    fn equality() {
        assert_eq!(Sut::sample(), Sut::sample());
        assert_eq!(Sut::sample_other(), Sut::sample_other());
    }

    #[test]
    fn inequality() {
        assert_ne!(Sut::sample(), Sut::sample_other());
    }
}
