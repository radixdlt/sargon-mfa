use crate::prelude::*;

/// Like a `DerivationPath` but without the last path component.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DerivationPathWithoutIndex {
    network_id: NetworkID,
    entity_kind: CAP26EntityKind,
    key_kind: CAP26KeyKind,
    key_space: KeySpace,
}
impl DerivationPathWithoutIndex {
    fn new(
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        key_kind: CAP26KeyKind,
        key_space: KeySpace,
    ) -> Self {
        Self {
            network_id,
            entity_kind,
            key_kind,
            key_space,
        }
    }
}

impl From<DerivationRequest> for DerivationPathWithoutIndex {
    fn from(value: DerivationRequest) -> Self {
        Self::new(
            value.network_id,
            value.entity_kind,
            value.key_kind,
            value.key_space,
        )
    }
}

impl From<HierarchicalDeterministicFactorInstance> for DerivationPathWithoutIndex {
    fn from(value: HierarchicalDeterministicFactorInstance) -> Self {
        Self::new(
            value.derivation_path().network_id,
            value.derivation_path().entity_kind,
            value.derivation_path().key_kind,
            value.derivation_path().index.key_space(),
        )
    }
}
impl From<DerivationPath> for DerivationPathWithoutIndex {
    fn from(value: DerivationPath) -> Self {
        Self::new(
            value.network_id,
            value.entity_kind,
            value.key_kind,
            value.index.key_space(),
        )
    }
}

impl From<(DerivationPathWithoutIndex, HDPathComponent)> for DerivationPath {
    fn from(value: (DerivationPathWithoutIndex, HDPathComponent)) -> Self {
        let (without_index, index) = value;
        assert!(index.is_in_key_space(without_index.key_space));
        Self::new(
            without_index.network_id,
            without_index.entity_kind,
            without_index.key_kind,
            index,
        )
    }
}
