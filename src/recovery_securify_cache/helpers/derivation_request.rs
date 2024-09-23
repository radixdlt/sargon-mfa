use crate::prelude::*;

/// A "partial" DerivationPath of sorts, without
/// any specifiedd Derivation Entity Index, but with
/// a known KeySpace.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DerivationRequest {
    pub factor_source_id: FactorSourceIDFromHash,
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_kind: CAP26KeyKind,
    pub key_space: KeySpace,
}

impl DerivationRequest {
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
