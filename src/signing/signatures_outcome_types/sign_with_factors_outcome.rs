use crate::prelude::*;

#[derive(Clone, PartialEq, Eq, derive_more::Debug)]
pub enum SignWithFactorsOutcome {
    /// The user successfully signed with the factor source(s), the associated
    /// value contains the produces signatures and any relevant metadata.
    #[debug("Signed: {:#?}", produced_signatures)]
    Signed {
        produced_signatures: BatchSigningResponse,
    },

    /// The factor source got neglected, either due to user explicitly skipping
    /// or due to failire
    #[debug("Neglected")]
    Neglected(NeglectedFactors),
}

impl SignWithFactorsOutcome {
    pub fn signed(produced_signatures: BatchSigningResponse) -> Self {
        Self::Signed {
            produced_signatures,
        }
    }

    pub fn failure_with_factors(ids: IndexSet<FactorSourceIDFromHash>) -> Self {
        Self::Neglected(NeglectedFactors::new(NeglectFactorReason::Failure, ids))
    }

    pub fn user_skipped_factors(ids: IndexSet<FactorSourceIDFromHash>) -> Self {
        Self::Neglected(NeglectedFactors::new(
            NeglectFactorReason::UserExplicitlySkipped,
            ids,
        ))
    }

    pub fn user_skipped_factor(id: FactorSourceIDFromHash) -> Self {
        Self::user_skipped_factors(IndexSet::from_iter([id]))
    }
}
