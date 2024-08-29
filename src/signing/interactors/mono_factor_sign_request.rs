use crate::prelude::*;

/// A batch signing request used with a MonoFactorSignInteractor, containing
/// a collection of transactions to sign with multiple keys (derivation paths),
/// and a collection of transactions which would be invalid if the user skips
/// signing with this factor source, or if we fail to sign.
#[derive(derive_more::Debug, Clone)]
#[debug("input: {:#?}", input)]
pub struct MonoFactorSignRequest {
    pub input: MonoFactorSignRequestInput,
    /// A collection of transactions which would be invalid if the user skips
    /// signing with this factor source, or if we fail to sign
    pub invalid_transactions_if_neglected: IndexSet<InvalidTransactionIfNeglected>,
}

impl MonoFactorSignRequest {
    pub fn new(
        input: MonoFactorSignRequestInput,
        invalid_transactions_if_neglected: IndexSet<InvalidTransactionIfNeglected>,
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
