use crate::prelude::*;

/// An immutable "snapshot" of `PetitionFactorsState`
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PetitionFactorsStateSnapshot {
    /// Factors that have signed.
    signed: IndexSet<HDSignature>,

    /// Factors that user skipped.
    skipped: IndexSet<HierarchicalDeterministicFactorInstance>,
}

impl PetitionFactorsStateSnapshot {
    pub(super) fn new(
        signed: IndexSet<HDSignature>,
        skipped: IndexSet<HierarchicalDeterministicFactorInstance>,
    ) -> Self {
        Self { signed, skipped }
    }
    pub(super) fn prompted_count(&self) -> i8 {
        self.signed_count() + self.skipped_count()
    }

    pub(super) fn signed_count(&self) -> i8 {
        self.signed.len() as i8
    }

    fn skipped_count(&self) -> i8 {
        self.skipped.len() as i8
    }
}
