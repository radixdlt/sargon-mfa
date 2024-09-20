use crate::prelude::*;

/// Unsecurified Entities that were discovered and recovered part of
/// `derive_and_analyze`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecoveredUnsecurifiedEntities {
    entities: Vec<UnsecurifiedEntity>,
}

impl RecoveredUnsecurifiedEntities {
    pub fn new(entities: IndexSet<UnsecurifiedEntity>) -> Self {
        Self {
            entities: entities.into_iter().collect(),
        }
    }

    pub fn entities(&self) -> IndexSet<UnsecurifiedEntity> {
        self.entities.clone().into_iter().collect()
    }
}

impl HasSampleValues for RecoveredUnsecurifiedEntities {
    fn sample() -> Self {
        Self::new(IndexSet::from_iter([
            UnsecurifiedEntity::sample(),
            UnsecurifiedEntity::sample_other(),
        ]))
    }

    fn sample_other() -> Self {
        Self::new(IndexSet::just(UnsecurifiedEntity::sample_other()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = RecoveredUnsecurifiedEntities;

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
