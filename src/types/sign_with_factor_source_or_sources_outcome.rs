use crate::prelude::*;

#[derive(Clone, PartialEq, Eq, derive_more::Debug)]
pub enum SignWithFactorSourceOrSourcesOutcome {
    /// The user successfully signed with the factor source(s), the associated
    /// value contains the produces signatures and any relevant metadata.
    #[debug("Signed: {:#?}", produced_signatures)]
    Signed {
        produced_signatures: BatchSigningResponse,
    },

    /// The user skipped signing with the factor sources with ids
    #[debug("Skipped")]
    Skipped {
        ids_of_skipped_factors_sources: Vec<FactorSourceIDFromHash>,
    },
}

impl SignWithFactorSourceOrSourcesOutcome {
    pub fn signed(produced_signatures: BatchSigningResponse) -> Self {
        Self::Signed {
            produced_signatures,
        }
    }

    pub fn skipped(ids_of_skipped_factors_sources: IndexSet<FactorSourceIDFromHash>) -> Self {
        Self::Skipped {
            ids_of_skipped_factors_sources: ids_of_skipped_factors_sources
                .into_iter()
                .collect_vec(),
        }
    }
    pub fn skipped_factor_source(factor_source_id: FactorSourceIDFromHash) -> Self {
        Self::skipped(IndexSet::from_iter([factor_source_id]))
    }
}
