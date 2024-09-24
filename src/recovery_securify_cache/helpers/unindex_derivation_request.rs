use crate::prelude::*;

/// A "partial" DerivationPath of sorts, without
/// any specified Derivation Entity Index, but with
/// a known KeySpace, and with an intended quantity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnquantifiedUnindexDerivationRequest {
    pub factor_source_id: FactorSourceIDFromHash,
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_kind: CAP26KeyKind,
    pub key_space: KeySpace,
}
impl From<&QuantifiedUnindexDerivationRequest> for &UnquantifiedUnindexDerivationRequest {
    fn from(value: &QuantifiedUnindexDerivationRequest) -> Self {
        todo!()
    }
}
impl From<QuantifiedUnindexDerivationRequest> for UnquantifiedUnindexDerivationRequest {
    fn from(value: QuantifiedUnindexDerivationRequest) -> Self {
        UnquantifiedUnindexDerivationRequest::new(
            value.factor_source_id,
            value.network_id,
            value.entity_kind,
            value.key_kind,
            value.key_space,
        )
    }
}
impl UnquantifiedUnindexDerivationRequest {
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

/// Note that this might be used as the intended purpose selector
/// but if we are filling the cache, we will create many instances
/// anyway.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DerivationRequestQuantitySelector {
    /// Used for creating a new single account, persona, a new ROLA key etc,
    /// but not for securing many accounts with a single security shield.
    Mono,
    /// Used when we are securing many accounts with a single security shield,
    /// the `count` will be the number of entities.
    ///
    /// Or when we are doing (MARS) Manual Account Recovery scan (OARS does not have cache).
    Poly { count: usize },
}

/// A "partial" DerivationPath of sorts, without
/// any specified Derivation Entity Index, but with
/// a known KeySpace, and with an intended quantity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QuantifiedUnindexDerivationRequest {
    pub factor_source_id: FactorSourceIDFromHash,
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_kind: CAP26KeyKind,
    pub key_space: KeySpace,
    /// single or batch, reflecting the intention of the operation
    /// being performed (account recovery scan, securifying many accounts, create new account etc)
    pub intended_purpose_quantity_selector: DerivationRequestQuantitySelector,
}

impl QuantifiedUnindexDerivationRequest {
    pub fn requested_quantity(&self) -> usize {
        match &self.intended_purpose_quantity_selector {
            DerivationRequestQuantitySelector::Mono => 1,
            DerivationRequestQuantitySelector::Poly { count } => *count,
        }
    }
    fn new(
        factor_source_id: FactorSourceIDFromHash,
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        key_kind: CAP26KeyKind,
        key_space: KeySpace,
        intended_purpose_quantity_selector: DerivationRequestQuantitySelector,
    ) -> Self {
        Self {
            factor_source_id,
            network_id,
            entity_kind,
            key_kind,
            key_space,
            intended_purpose_quantity_selector,
        }
    }

    /// Used when we are securing many accounts with a single security shield,
    /// the `count` will be the number of entities.
    ///
    /// Or when we are doing (MARS) Manual Account Recovery scan (OARS does not have cache).
    ///
    /// When this is used to fill the cache, use BATCH size for `count` (typically `30`).
    pub fn poly_instances(
        factor_source_id: FactorSourceIDFromHash,
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        key_kind: CAP26KeyKind,
        key_space: KeySpace,
        count: usize,
    ) -> Self {
        Self::new(
            factor_source_id,
            network_id,
            entity_kind,
            key_kind,
            key_space,
            DerivationRequestQuantitySelector::Poly { count },
        )
    }

    pub fn mono(
        factor_source_id: FactorSourceIDFromHash,
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        key_kind: CAP26KeyKind,
        key_space: KeySpace,
    ) -> Self {
        Self::new(
            factor_source_id,
            network_id,
            entity_kind,
            key_kind,
            key_space,
            DerivationRequestQuantitySelector::Mono,
        )
    }
}