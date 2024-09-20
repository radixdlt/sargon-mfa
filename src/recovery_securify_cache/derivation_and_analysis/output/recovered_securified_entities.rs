use crate::prelude::*;

/// Securified Entities that were discovered and recovered part of
/// `derive_and_analyze`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecoveredSecurifiedEntities {
    securified_entities: Vec<SecurifiedEntity>,
}

impl RecoveredSecurifiedEntities {
    pub fn new(securified_entities: IndexSet<SecurifiedEntity>) -> Self {
        Self {
            securified_entities: securified_entities.into_iter().collect(),
        }
    }

    pub fn securified_entities(&self) -> IndexSet<SecurifiedEntity> {
        self.securified_entities.clone().into_iter().collect()
    }

    pub fn entities(&self) -> IndexSet<AccountOrPersona> {
        self.securified_entities()
            .into_iter()
            .map(AccountOrPersona::from)
            .collect()
    }
}

impl HasSampleValues for RecoveredSecurifiedEntities {
    fn sample() -> Self {
        Self::new(IndexSet::from_iter([
            SecurifiedEntity::sample(),
            SecurifiedEntity::sample_other(),
        ]))
    }

    fn sample_other() -> Self {
        Self::new(IndexSet::just(SecurifiedEntity::sample_other()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = RecoveredSecurifiedEntities;

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