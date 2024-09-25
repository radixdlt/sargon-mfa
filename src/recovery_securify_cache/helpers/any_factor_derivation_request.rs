use crate::prelude::*;

/// A partial `QuantifiedUnindexDerivationRequest` of sorts, without
/// any specified FactorSource.
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

    pub fn unquantified_derivation_request_with_factor_source(
        &self,
        factor_source_id: FactorSourceIDFromHash,
    ) -> UnquantifiedUnindexDerivationRequest {
        UnquantifiedUnindexDerivationRequest::new(
            factor_source_id,
            self.network_id,
            self.entity_kind,
            self.key_kind,
            self.key_space,
        )
    }
}
