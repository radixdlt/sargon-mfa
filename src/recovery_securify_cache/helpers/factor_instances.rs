use crate::prelude::*;

/// A collection of factor instances.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct FactorInstances {
    factor_instances: Vec<HierarchicalDeterministicFactorInstance>,
}

impl From<IndexSet<HierarchicalDeterministicFactorInstance>> for FactorInstances {
    fn from(instances: IndexSet<HierarchicalDeterministicFactorInstance>) -> Self {
        Self::new(instances)
    }
}

impl FactorInstances {
    pub fn len(&self) -> usize {
        self.factor_instances.len()
    }
    pub fn filter_satisfying(
        &self,
        derivation_requests: &UnindexDerivationRequests,
    ) -> Result<Self> {
        if self.satisfies_all_requests(derivation_requests) {
            Ok(self.clone())
        } else {
            Err(CommonError::FactorInstancesDoesNotSatisfyDerivationRequests)
        }
    }
}

impl IntoIterator for FactorInstances {
    type Item = HierarchicalDeterministicFactorInstance;
    type IntoIter = <IndexSet<HierarchicalDeterministicFactorInstance> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.factor_instances().into_iter()
    }
}

impl FromIterator<HierarchicalDeterministicFactorInstance> for FactorInstances {
    fn from_iter<I: IntoIterator<Item = HierarchicalDeterministicFactorInstance>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl IsFactorInstanceCollectionBase for FactorInstances {
    fn factor_instances(&self) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        self.factor_instances.iter().cloned().collect()
    }
}
impl IsFactorInstanceCollection for FactorInstances {
    fn new(instances: IndexSet<HierarchicalDeterministicFactorInstance>) -> Self {
        Self {
            factor_instances: instances.into_iter().collect(),
        }
    }
}

impl HasSampleValues for FactorInstances {
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

    type Sut = FactorInstances;

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
