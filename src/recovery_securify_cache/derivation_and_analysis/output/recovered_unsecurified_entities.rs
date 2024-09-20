use crate::prelude::*;

/// Unsecurified Entities that were discovered and recovered part of
/// `derive_and_analyze`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecoveredUnsecurifiedEntities {
    unsecurified_entities: Vec<UnsecurifiedEntity>,
}

impl RecoveredUnsecurifiedEntities {
    pub fn new(unsecurified_entities: IndexSet<UnsecurifiedEntity>) -> Self {
        Self {
            unsecurified_entities: unsecurified_entities.into_iter().collect(),
        }
    }

    pub fn unsecurified_entities(&self) -> IndexSet<UnsecurifiedEntity> {
        self.unsecurified_entities.clone().into_iter().collect()
    }

    pub fn entities(&self) -> IndexSet<AccountOrPersona> {
        self.unsecurified_entities()
            .into_iter()
            .map(AccountOrPersona::from)
            .collect()
    }
}

impl IsFactorInstanceCollectionBase for RecoveredUnsecurifiedEntities {
    fn factor_instances(&self) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        self.unsecurified_entities()
            .into_iter()
            .map(|x| x.factor_instance())
            .collect()
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

fn merge_recovered_entities(
    unsecurified: RecoveredUnsecurifiedEntities,
    securified: RecoveredSecurifiedEntities,
) -> IndexSet<AccountOrPersona> {
    let unsecurified = unsecurified.entities();
    let securified = securified.entities();
    let mut entities = IndexSet::new();
    entities.extend(unsecurified);
    entities.extend(securified);

    entities
}

impl RecoveredUnsecurifiedEntities {
    pub fn merge_with_securified(
        self,
        securified: RecoveredSecurifiedEntities,
    ) -> IndexSet<AccountOrPersona> {
        merge_recovered_entities(self, securified)
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
