use crate::prelude::*;

/// The outcome of collecting signatures for a specific
/// transasction - either valid or invalid - and a
/// set of collected signatues (might be empty) and
/// a set of neglected factors (might be empty).
#[derive(Clone, PartialEq, Eq)]
pub struct PetitionTransactionOutcome {
    intent_hash: IntentHash,
    pub transaction_valid: bool,
    pub signatures: IndexSet<HDSignature>,
    pub neglected_factors: IndexSet<NeglectedFactor>,
}

impl PetitionTransactionOutcome {
    /// # Panics
    /// Panics if the intent hash in any signatures does not
    /// match `intent_hash`
    pub fn new(
        transaction_valid: bool,
        intent_hash: IntentHash,
        signatures: IndexSet<HDSignature>,
        neglected_factors: IndexSet<NeglectedFactor>,
    ) -> Self {
        assert!(
            signatures.iter().all(|s| *s.intent_hash() == intent_hash),
            "Discprenacy! Mismatching intent hash found in a signature."
        );
        Self {
            intent_hash,
            transaction_valid,
            signatures,
            neglected_factors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = PetitionTransactionOutcome;

    #[test]
    #[should_panic(expected = "Discprenacy! Mismatching intent hash found in a signature.")]
    fn panic() {
        Sut::new(
            true,
            IntentHash::sample(),
            IndexSet::from_iter([HDSignature::sample_other()]),
            IndexSet::new(),
        );
    }
}