use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct EntityRecoveryOutcome<E>
where
    E: IsEntity + std::hash::Hash + Eq,
{
    pub recovered_unsecurified: IndexSet<E>,
    pub recovered_securified: IndexSet<E>,
    pub unrecovered: Vec<UncoveredEntity>, // want `IndexSet` but is not `Hash`
}

impl<E: IsEntity + std::hash::Hash + Eq> EntityRecoveryOutcome<E> {
    pub fn new(
        recovered_unsecurified: impl IntoIterator<Item = E>,
        recovered_securified: impl IntoIterator<Item = E>,
        unrecovered: impl IntoIterator<Item = UncoveredEntity>,
    ) -> Self {
        Self {
            recovered_unsecurified: recovered_unsecurified.into_iter().collect(),
            recovered_securified: recovered_securified.into_iter().collect(),
            unrecovered: unrecovered.into_iter().collect(),
        }
    }
}
