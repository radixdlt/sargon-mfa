#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FinishEarlyWhenAllTransactionsAreValid(pub bool);

impl Default for FinishEarlyWhenAllTransactionsAreValid {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FinishEarlyWhenSomeTransactionIsInvalid(pub bool);

impl Default for FinishEarlyWhenSomeTransactionIsInvalid {
    fn default() -> Self {
        Self(true)
    }
}

/// Strategy to use for finishing early, i.e. stop collecting more signatures
#[derive(Clone, Default, Copy, Debug, PartialEq, Eq)]
pub struct SigningFinishEarlyStrategy {
    pub finish_early_when_all_transactions_are_valid: FinishEarlyWhenAllTransactionsAreValid,
    pub finish_early_when_some_transaction_is_invalid: FinishEarlyWhenSomeTransactionIsInvalid,
}
impl SigningFinishEarlyStrategy {
    pub fn new(
        finish_early_when_all_transactions_are_valid: FinishEarlyWhenAllTransactionsAreValid,
        finish_early_when_some_transaction_is_invalid: FinishEarlyWhenSomeTransactionIsInvalid,
    ) -> Self {
        Self {
            finish_early_when_all_transactions_are_valid,
            finish_early_when_some_transaction_is_invalid,
        }
    }
}
