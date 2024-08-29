use crate::prelude::*;

pub trait FactorSourceReferencing: std::hash::Hash + PartialEq + Eq + Clone {
    fn factor_source_id(&self) -> FactorSourceIDFromHash;
}

impl FactorSourceReferencing for HierarchicalDeterministicFactorInstance {
    fn factor_source_id(&self) -> FactorSourceIDFromHash {
        self.factor_source_id
    }
}

impl FactorSourceReferencing for HDSignature {
    fn factor_source_id(&self) -> FactorSourceIDFromHash {
        self.owned_factor_instance()
            .factor_instance()
            .factor_source_id
    }
}