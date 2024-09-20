use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DerivationRequest {
    pub factor_source_id: FactorSourceIDFromHash,
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_space: KeySpace,
    pub key_kind: CAP26KeyKind,
}
