use crate::prelude::*;

/// Used as a map key in `InMemoryPreDerivedKeysCache`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PreDeriveKeysCacheKey {
    pub factor_source_id: FactorSourceIDFromHash,
    pub path_without_index: DerivationPathWithoutIndex,
}

impl From<HierarchicalDeterministicFactorInstance> for PreDeriveKeysCacheKey {
    fn from(value: HierarchicalDeterministicFactorInstance) -> Self {
        Self::new(
            value.factor_source_id(),
            DerivationPathWithoutIndex::from(value),
        )
    }
}
impl PreDeriveKeysCacheKey {
    pub fn new(
        factor_source_id: FactorSourceIDFromHash,
        path_without_index: DerivationPathWithoutIndex,
    ) -> Self {
        Self {
            factor_source_id,
            path_without_index,
        }
    }
}
