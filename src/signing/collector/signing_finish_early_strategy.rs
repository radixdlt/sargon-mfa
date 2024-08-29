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

    pub fn r#continue() -> Self {
        Self::new(
            WhenAllTransactionsAreValid(SignaturesCollectingContinuation::Continue),
            WhenSomeTransactionIsInvalid(SignaturesCollectingContinuation::Continue),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type Sut = SigningFinishEarlyStrategy;

    #[test]
    fn test_continue() {
        let sut = Sut::r#continue();
        assert_eq!(
            sut.when_all_transactions_are_valid.0,
            SignaturesCollectingContinuation::Continue
        );
        assert_eq!(
            sut.when_some_transaction_is_invalid.0,
            SignaturesCollectingContinuation::Continue
        );
    }
}
