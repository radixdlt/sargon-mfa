use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DerivationRequestQuantitySelector {
    /// Used for creating a new single account, persona, a new ROLA key etc,
    /// but not for securing many accounts with a single security shield.
    Single,
    /// Used when we are securing many accounts with a single security shield,
    /// the `count` will be the number of entities.
    Batch { count: usize },
}

/// A "partial" DerivationPath of sorts, without
/// any specified Derivation Entity Index, but with
/// a known KeySpace, and with a n
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnindexDerivationRequest {
    pub factor_source_id: FactorSourceIDFromHash,
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_kind: CAP26KeyKind,
    pub key_space: KeySpace,
    /// single or batch
    pub quantity_selector: DerivationRequestQuantitySelector,
}

impl UnindexDerivationRequest {
    pub fn new(
        factor_source_id: FactorSourceIDFromHash,
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        key_kind: CAP26KeyKind,
        key_space: KeySpace,
    ) -> Self {
        Self {
            factor_source_id,
            network_id,
            entity_kind,
            key_kind,
            key_space,
        }
    }
}
