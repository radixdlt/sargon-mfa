use crate::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct KeyDerivationOutcome {
    pub factors_by_source:
        IndexMap<FactorSourceIDFromHash, IndexSet<HierarchicalDeterministicFactorInstance>>,
}
impl KeyDerivationOutcome {
    pub fn new(
        factors_by_source: IndexMap<
            FactorSourceIDFromHash,
            IndexSet<HierarchicalDeterministicFactorInstance>,
        >,
    ) -> Self {
        Self { factors_by_source }
    }

    /// ALL factor instances derived by the KeysCollector
    pub fn all_factors(&self) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        self.factors_by_source
            .clone()
            .into_iter()
            .flat_map(|(_, v)| v)
            .collect()
    }
}
