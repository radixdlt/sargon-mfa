use crate::prelude::*;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct IntermediaryDerivationAndAnalysis {
    pub entities_from_analysis: EntitiesFromAnalysis,
}

impl IntermediaryDerivationAndAnalysis {
    pub fn all_account_addresses(&self) -> IndexSet<AccountAddress> {
        todo!()
    }
}
