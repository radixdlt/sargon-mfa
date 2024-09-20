use crate::prelude::*;

use crate::prelude::*;

/// Securified Entities that were discovered and recovered part of
/// `derive_and_analyze` that we did not successfully recover due to
/// not enough matched FactorInstances.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnrecoveredSecurifiedEntities {
    entities: Vec<UnrecoveredSecurifiedEntity>,
}

impl UnrecoveredSecurifiedEntities {
    pub fn new(entities: IndexSet<UnrecoveredSecurifiedEntity>) -> Self {
        Self {
            entities: entities.into_iter().collect(),
        }
    }

    pub fn entities(&self) -> IndexSet<UnrecoveredSecurifiedEntity> {
        self.entities.clone().into_iter().collect()
    }
}

impl IsFactorInstanceCollectionBase for UnrecoveredSecurifiedEntities {
    fn factor_instances(&self) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        self.entities()
            .into_iter()
            .flat_map(|x| x.matched_factor_instances())
            .collect()
    }
}

impl HasSampleValues for UnrecoveredSecurifiedEntities {
    fn sample() -> Self {
        Self::new(IndexSet::from_iter([
            UnrecoveredSecurifiedEntity::sample(),
            UnrecoveredSecurifiedEntity::sample_other(),
        ]))
    }

    fn sample_other() -> Self {
        Self::new(IndexSet::just(UnrecoveredSecurifiedEntity::sample_other()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = UnrecoveredSecurifiedEntity;

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
