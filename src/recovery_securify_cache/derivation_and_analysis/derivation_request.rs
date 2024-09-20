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

impl AnyFactorDerivationRequest {
    pub fn new(
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        key_space: KeySpace,
        key_kind: CAP26KeyKind,
    ) -> Self {
        Self {
            network_id,
            entity_kind,
            key_kind,
            key_space,
        }
    }

    pub fn derivation_request_with_factor_source_id(
        &self,
        factor_source_id: FactorSourceIDFromHash,
    ) -> DerivationRequest {
        DerivationRequest::new(
            factor_source_id,
            self.network_id,
            self.entity_kind,
            self.key_space,
            self.key_kind,
        )
    }

    pub fn many_for_each_on(
        network_id: NetworkID,
        entity_kinds: impl IntoIterator<Item = CAP26EntityKind>,
        key_kinds: impl IntoIterator<Item = CAP26KeyKind>,
        key_spaces: impl IntoIterator<Item = KeySpace>,
    ) -> IndexSet<Self> {
        let entity_kinds = entity_kinds.into_iter().collect::<IndexSet<_>>();
        let key_kinds = key_kinds.into_iter().collect::<IndexSet<_>>();
        let key_spaces = key_spaces.into_iter().collect::<IndexSet<_>>();

        let mut requests = IndexSet::<Self>::new();

        for entity_kind in entity_kinds.into_iter() {
            for key_kind in key_kinds.clone().into_iter() {
                for key_space in key_spaces.clone().into_iter() {
                    let request = Self::new(network_id, entity_kind, key_space, key_kind);
                    requests.insert(request);
                }
            }
        }
        requests
    }
}
