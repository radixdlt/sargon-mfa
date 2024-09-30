use crate::prelude::*;

impl HierarchicalDeterministicFactorInstance {
    pub fn satisfies(&self, request: UnquantifiedUnindexDerivationRequest) -> bool {
        self.derivation_path().satisfies(request.clone())
            && request.factor_source_id == self.factor_source_id
    }
}

impl DerivationPath {
    #[allow(clippy::nonminimal_bool)]
    fn satisfies(&self, request: impl Into<UnquantifiedUnindexDerivationRequest>) -> bool {
        let request = request.into();
        request.entity_kind == self.entity_kind
            && request.network_id == self.network_id
            && request.entity_kind == self.entity_kind
            && request.key_kind == self.key_kind
            && request.key_space == self.index.key_space()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Satisfaction {
    FullyWithoutSpare,
    FullyWithSpare,
    Partial,
    Empty,
}

use std::cmp::Ordering;
impl QuantifiedUnindexDerivationRequest {
    pub fn satisfaction(&self, instances: &dyn IsFactorInstanceCollectionBase) -> Satisfaction {
        let satisfied = instances
            .filter_satisfied_requests(UnquantifiedUnindexDerivationRequest::from(self.clone()));

        if satisfied.is_empty() {
            return Satisfaction::Empty;
        }
        match satisfied.len().cmp(&self.requested_quantity()) {
            Ordering::Equal => Satisfaction::FullyWithoutSpare,
            Ordering::Greater => Satisfaction::FullyWithSpare,
            Ordering::Less => Satisfaction::Partial,
        }
    }
}

pub trait IsFactorInstanceCollectionBase {
    fn factor_instances(&self) -> IndexSet<HierarchicalDeterministicFactorInstance>;

    fn filter_satisfied_requests(
        &self,
        request: UnquantifiedUnindexDerivationRequest,
    ) -> FactorInstances {
        self.factor_instances()
            .into_iter()
            .filter(|instance| instance.satisfies(request.clone()))
            .collect()
    }
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

impl FromIterator<HierarchicalDeterministicFactorInstance> for ProbablyFreeFactorInstances {
    fn from_iter<I: IntoIterator<Item = HierarchicalDeterministicFactorInstance>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl IntoIterator for ProbablyFreeFactorInstances {
    type Item = HierarchicalDeterministicFactorInstance;
    type IntoIter = <IndexSet<HierarchicalDeterministicFactorInstance> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.factor_instances().into_iter()
    }
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
