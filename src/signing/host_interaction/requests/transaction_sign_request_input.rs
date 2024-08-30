use crate::prelude::*;

/// A batch of keys (derivation paths) all being factor instances of a HDFactorSource
/// with id `factor_source_id` to sign a single transaction with, which hash
/// is `intent_hash`.
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct TransactionSignRequestInput {
    /// Hash to sign
    intent_hash: IntentHash,

    /// ID of factor to use to sign
    pub factor_source_id: FactorSourceIDFromHash,

    /// The derivation paths to use to derive the private keys to sign with. The
    /// `factor_source_id` of each item must match `factor_source_id`.
    owned_factor_instances: Vec<OwnedFactorInstance>,
}

impl TransactionSignRequestInput {
    /// # Panics
    /// Panics if any of the owned factor instances does not match the `factor_source_id`.
    ///
    /// Panics if `owned_factor_instances` is empty.
    pub fn new(
        intent_hash: IntentHash,
        factor_source_id: FactorSourceIDFromHash,
        owned_factor_instances: IndexSet<OwnedFactorInstance>,
    ) -> Self {
        assert!(
            !owned_factor_instances.is_empty(),
            "Invalid input, `owned_factor_instances` must not be empty."
        );
        assert!(owned_factor_instances
            .iter()
            .all(|f| f.by_factor_source(factor_source_id)), "Discrepancy! Mismatch between FactorSourceID of owned factor instances and specified FactorSourceID, this is a programmer error.");
        Self {
            intent_hash,
            factor_source_id,
            owned_factor_instances: owned_factor_instances.into_iter().collect_vec(),
        }
    }

    pub fn signature_inputs(&self) -> IndexSet<HDSignatureInput> {
        self.owned_factor_instances
            .clone()
            .into_iter()
            .map(|fi| HDSignatureInput::new(self.intent_hash.clone(), fi))
            .collect()
    }
}

impl HasSampleValues for TransactionSignRequestInput {
    fn sample() -> Self {
        Self::new(
            IntentHash::sample(),
            FactorSourceIDFromHash::sample(),
            IndexSet::from_iter([OwnedFactorInstance::sample()]),
        )
    }

    fn sample_other() -> Self {
        Self::new(
            IntentHash::sample_other(),
            FactorSourceIDFromHash::sample_other(),
            IndexSet::from_iter([OwnedFactorInstance::sample_other()]),
        )
    }
}

#[cfg(test)]
mod tests_batch_req {
    use super::*;

    type Sut = TransactionSignRequestInput;

    #[test]
    fn equality() {
        assert_eq!(Sut::sample(), Sut::sample());
        assert_eq!(Sut::sample_other(), Sut::sample_other());
    }

    #[test]
    fn inequality() {
        assert_ne!(Sut::sample(), Sut::sample_other());
    }

    #[test]
    #[should_panic(expected = "Invalid input, `owned_factor_instances` must not be empty.")]
    fn panics_if_owned_factors_is_empty() {
        Sut::new(
            IntentHash::sample(),
            FactorSourceIDFromHash::sample(),
            IndexSet::new(),
        );
    }

    #[test]
    #[should_panic(
        expected = "Discrepancy! Mismatch between FactorSourceID of owned factor instances and specified FactorSourceID, this is a programmer error."
    )]
    fn panics_mismatch_factor_source_id() {
        Sut::new(
            IntentHash::sample(),
            FactorSourceIDFromHash::sample(),
            IndexSet::from_iter([OwnedFactorInstance::sample_other()]),
        );
    }
}