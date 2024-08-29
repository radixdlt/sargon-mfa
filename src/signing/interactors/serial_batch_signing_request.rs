use crate::prelude::*;

/// A batch signing request used with a SignWithFactorSerialInteractor, containing
/// a collection of transactions to sign with multiple keys (derivation paths),
/// and a collection of transactions which would be invalid if the user skips
/// signing with this factor source, or if we fail to sign.
#[derive(derive_more::Debug, Clone)]
#[debug("input: {:#?}", input)]
pub struct SerialBatchSigningRequest {
    pub input: BatchTXBatchKeySigningRequest,
    /// A collection of transactions which would be invalid if the user skips
    /// signing with this factor source, or if we fail to sign
    pub invalid_transactions_if_neglected: Vec<InvalidTransactionIfNeglected>,
}

impl SerialBatchSigningRequest {
    pub fn new(
        input: BatchTXBatchKeySigningRequest,
        invalid_transactions_if_neglected: Vec<InvalidTransactionIfNeglected>,
    ) -> Self {
        Self {
            input,
            invalid_transactions_if_neglected,
        }
    }

    pub fn factor_source_kind(&self) -> FactorSourceKind {
        self.input.factor_source_kind()
    }
}
