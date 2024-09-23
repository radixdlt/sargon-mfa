use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DerivationRequest {
    pub factor_source_id: FactorSourceIDFromHash,
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_space: KeySpace,
    pub key_kind: CAP26KeyKind,
}

impl DerivationRequest {
    pub fn new(
        factor_source_id: FactorSourceIDFromHash,
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        key_space: KeySpace,
        key_kind: CAP26KeyKind,
    ) -> Self {
        Self {
            factor_source_id,
            network_id,
            entity_kind,
            key_space,
            key_kind,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AnyFactorDerivationRequest {
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_space: KeySpace,
    pub key_kind: CAP26KeyKind,
}
