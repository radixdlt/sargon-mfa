use crate::prelude::*;

/// An async GUI callback allowing us to ask user
/// if she is content with the derivations (recovered
/// accounts) so far.
#[async_trait::async_trait]
pub trait IsDerivationDoneQuery {
    async fn is_done(&self, intermediary: &IntermediaryDerivationAndAnalysis) -> Result<bool>;
}

/// Simplest possible implementation of `IsDerivationDoneQuery`
/// which immediately returns `true`
pub struct YesDone;

#[async_trait::async_trait]
impl IsDerivationDoneQuery for YesDone {
    async fn is_done(&self, _intermediary: &IntermediaryDerivationAndAnalysis) -> Result<bool> {
        Ok(true)
    }
}
