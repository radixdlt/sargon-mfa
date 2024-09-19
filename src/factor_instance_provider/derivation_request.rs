#![allow(clippy::type_complexity)]

use crate::prelude::*;

use rand::Rng;
use sha2::{Digest, Sha256, Sha512};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DerivationRequest {
    pub key_space: KeySpace,
    pub entity_kind: CAP26EntityKind,
    pub key_kind: CAP26KeyKind,
    pub factor_source_id: FactorSourceIDFromHash,
    pub network_id: NetworkID,
}

impl DerivationRequest {
    pub fn new(
        key_space: KeySpace,
        entity_kind: CAP26EntityKind,
        key_kind: CAP26KeyKind,
        factor_source_id: FactorSourceIDFromHash,
        network_id: NetworkID,
    ) -> Self {
        Self {
            key_space,
            entity_kind,
            key_kind,
            factor_source_id,
            network_id,
        }
    }
    pub fn securify(
        entity_kind: CAP26EntityKind,
        key_kind: CAP26KeyKind,
        factor_source_id: FactorSourceIDFromHash,
        network_id: NetworkID,
    ) -> Self {
        Self::new(
            KeySpace::Securified,
            entity_kind,
            key_kind,
            factor_source_id,
            network_id,
        )
    }

    pub fn virtual_entity_creating_factor_instance(
        entity_kind: CAP26EntityKind,
        factor_source_id: FactorSourceIDFromHash,
        network_id: NetworkID,
    ) -> Self {
        Self::new(
            KeySpace::Securified,
            entity_kind,
            CAP26KeyKind::TransactionSigning,
            factor_source_id,
            network_id,
        )
    }

    pub fn factor_source_kind(&self) -> FactorSourceKind {
        self.factor_source_id.kind
    }
    pub fn derivation_size(&self) -> Option<HDPathValue> {
        self.factor_source_kind()
            .derivation_size(self.key_space, self.key_kind, self.entity_kind)
            .map(|size| size as HDPathValue)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::Display, derive_more::Debug)]
pub enum KeySpace {
    #[display("Unsecurified")]
    #[debug("Unsecurified")]
    Unsecurified,
    #[display("Securified")]
    #[debug("Securified")]
    Securified,
}

#[cfg(test)]
impl Profile {
    pub fn accounts<'a>(accounts: impl IntoIterator<Item = &'a Account>) -> Self {
        Self::new([], accounts, [])
    }
}
