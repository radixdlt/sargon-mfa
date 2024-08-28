use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WhenAllTransactionsAreValid(pub SignaturesCollectingContinuation);

impl Default for WhenAllTransactionsAreValid {
    fn default() -> Self {
        Self(SignaturesCollectingContinuation::FinishEarly)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WhenSomeTransactionIsInvalid(pub SignaturesCollectingContinuation);

impl Default for WhenSomeTransactionIsInvalid {
    fn default() -> Self {
        Self(SignaturesCollectingContinuation::FinishEarly)
    }
}

/// Strategy to use for finishing early, i.e. stop collecting more signatures
#[derive(Clone, Default, Copy, Debug, PartialEq, Eq)]
pub struct SigningFinishEarlyStrategy {
    pub when_all_transactions_are_valid: WhenAllTransactionsAreValid,
    pub when_some_transaction_is_invalid: WhenSomeTransactionIsInvalid,
}
impl SigningFinishEarlyStrategy {
    pub fn new(
        when_all_transactions_are_valid: WhenAllTransactionsAreValid,
        when_some_transaction_is_invalid: WhenSomeTransactionIsInvalid,
    ) -> Self {
        Self {
            when_all_transactions_are_valid,
            when_some_transaction_is_invalid,
        }
    }
}
