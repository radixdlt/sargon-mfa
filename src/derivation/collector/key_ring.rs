use crate::prelude::*;

/// A collection of `HierarchicalDeterministicFactorInstance` derived from a
/// factor source.
#[derive(Clone, Debug)]
pub(crate) struct Keyring {
    pub(crate) factor_source_id: FactorSourceIDFromHash,
    pub(crate) paths: IndexSet<DerivationPath>,
    derived: RefCell<IndexSet<HierarchicalDeterministicFactorInstance>>,
}

impl Keyring {
    pub(crate) fn new(
        factor_source_id: FactorSourceIDFromHash,
        paths: IndexSet<DerivationPath>,
    ) -> Self {
        Self {
            factor_source_id,
            paths,
            derived: RefCell::new(IndexSet::new()),
        }
    }
    pub(crate) fn factors(&self) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        self.derived.borrow().clone()
    }

    pub(crate) fn process_response(
        &self,
        response: IndexSet<HierarchicalDeterministicFactorInstance>,
    ) {
        assert!(response
            .iter()
            .all(|f| f.factor_source_id == self.factor_source_id
                && !self
                    .derived
                    .borrow()
                    .iter()
                    .any(|x| x.public_key == f.public_key)));

        self.derived.borrow_mut().extend(response)
    }
}
