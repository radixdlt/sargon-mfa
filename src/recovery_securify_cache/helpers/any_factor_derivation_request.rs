use crate::prelude::*;

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
}
