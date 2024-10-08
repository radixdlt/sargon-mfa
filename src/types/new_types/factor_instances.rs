use crate::prelude::*;

/// A collection of factor instances.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct FactorInstances {
    factor_instances: Vec<HierarchicalDeterministicFactorInstance>,
}
impl FactorInstances {
    pub fn extend(
        &mut self,
        instances: impl IntoIterator<Item = HierarchicalDeterministicFactorInstance>,
    ) {
        let instances = instances.into_iter().collect::<IndexSet<_>>(); // remove duplicates
        self.factor_instances
            .extend(instances.into_iter().collect_vec());
    }
    pub fn first(&self) -> Option<HierarchicalDeterministicFactorInstance> {
        self.factor_instances.first().cloned()
    }
}

impl From<IndexSet<HierarchicalDeterministicFactorInstance>> for FactorInstances {
    fn from(instances: IndexSet<HierarchicalDeterministicFactorInstance>) -> Self {
        Self::new(instances)
    }
}

impl From<FactorInstances> for IndexSet<HierarchicalDeterministicFactorInstance> {
    fn from(value: FactorInstances) -> Self {
        value.factor_instances()
    }
}
impl FactorInstances {
    pub fn append(
        &mut self,
        instances: impl Into<IndexSet<HierarchicalDeterministicFactorInstance>>,
    ) {
        let to_append: IndexSet<_> = instances.into();
        let mut values = self.factor_instances();
        values.extend(to_append);
        self.factor_instances = values.into_iter().collect_vec()
    }
    pub fn is_empty(&self) -> bool {
        self.factor_instances.is_empty()
    }
    pub fn len(&self) -> usize {
        self.factor_instances.len()
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

impl FactorInstances {
    pub fn new(instances: IndexSet<HierarchicalDeterministicFactorInstance>) -> Self {
        Self {
            factor_instances: instances.into_iter().collect(),
        }
    }
    pub fn factor_instances(&self) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        let instances = self
            .factor_instances
            .iter()
            .cloned()
            .collect::<IndexSet<_>>();
        assert_eq!(
            instances.len(),
            self.factor_instances.len(),
            "DUPLICATE FOUND, this is programmer error",
        );
        instances
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
