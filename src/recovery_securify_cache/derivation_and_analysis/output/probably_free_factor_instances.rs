use crate::prelude::*;

pub trait IsFactorInstanceCollectionBase {
    fn factor_instances(&self) -> IndexSet<HierarchicalDeterministicFactorInstance>;
}
pub trait IsFactorInstanceCollection:
    IsFactorInstanceCollectionBase + HasSampleValues + Sized
{
    fn new(instances: IndexSet<HierarchicalDeterministicFactorInstance>) -> Self;

    fn merge<T: IsFactorInstanceCollection>(&self, other: T) -> Self {
        Self::new(
            self.factor_instances()
                .union(&other.factor_instances())
                .cloned()
                .collect(),
        )
    }
}

pub fn are_factor_instance_collections_disjoint(
    collections: Vec<&dyn IsFactorInstanceCollectionBase>,
) -> bool {
    let mut all_instances = IndexSet::new();
    for collection in collections {
        let instances = collection.factor_instances();
        if !instances.is_disjoint(&all_instances) {
            return false;
        }
        all_instances.extend(instances);
    }
    true
}

pub fn assert_are_factor_instance_collections_disjoint(
    collections: Vec<&dyn IsFactorInstanceCollectionBase>,
) {
    assert!(
        are_factor_instance_collections_disjoint(collections),
        "Discrepancy! FactorInstance found in multiple collections, this is a programmer error!"
    );
}

/// "Probably" since we might not have all the information to be sure, since
/// Gateway might not keep track of past FactorInstances, some of the FactorInstances
/// in KeySpace::Securified might in fact have been used in the past for some entity.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct ProbablyFreeFactorInstances {
    factor_instances: Vec<HierarchicalDeterministicFactorInstance>,
}

impl IsFactorInstanceCollectionBase for ProbablyFreeFactorInstances {
    fn factor_instances(&self) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        self.factor_instances.iter().cloned().collect()
    }
}
impl IsFactorInstanceCollection for ProbablyFreeFactorInstances {
    fn new(instances: IndexSet<HierarchicalDeterministicFactorInstance>) -> Self {
        Self {
            factor_instances: instances.into_iter().collect(),
        }
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
