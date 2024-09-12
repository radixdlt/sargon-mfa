#![allow(clippy::type_complexity)]

use crate::prelude::*;

use rand::Rng;
use sha2::{Digest, Sha256, Sha512};

#[derive(Clone, Copy, PartialEq)]
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
}

#[derive(Clone, Copy, PartialEq)]
pub struct NextFactorInstanceRequest<'p> {
    pub derivation_request: DerivationRequest,
    pub profile: &'p Profile,
}

impl<'p> NextFactorInstanceRequest<'p> {
    pub fn new(
        key_space: KeySpace,
        entity_kind: CAP26EntityKind,
        key_kind: CAP26KeyKind,
        factor_source_id: FactorSourceIDFromHash,
        network_id: NetworkID,
        profile: &'p Profile,
    ) -> Self {
        Self {
            derivation_request: DerivationRequest::new(
                key_space,
                entity_kind,
                key_kind,
                factor_source_id,
                network_id,
            ),
            profile,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeySpace {
    Unsecurified,
    Securified,
}

impl HierarchicalDeterministicFactorInstance {
    fn entity_index(&self) -> HDPathComponent {
        self.derivation_path().index
    }
}

impl Profile {
    fn security_states_for_entities_of_kind(
        &self,
        key_space: KeySpace,
        kind: CAP26EntityKind,
        network_id: NetworkID,
    ) -> IndexSet<EntitySecurityState> {
        let entities: Vec<AccountOrPersona> = match kind {
            CAP26EntityKind::Account => self
                .accounts
                .values()
                .cloned()
                .map(AccountOrPersona::from)
                .collect(),
            CAP26EntityKind::Identity => self
                .personas
                .values()
                .cloned()
                .map(AccountOrPersona::from)
                .collect(),
        };
        entities
            .into_iter()
            .filter(|e| e.network_id() == network_id)
            .map(|e| e.security_state())
            .filter_map(|s| match (&s, key_space) {
                (EntitySecurityState::Unsecured(_), KeySpace::Unsecurified) => Some(s),
                (EntitySecurityState::Securified(_), KeySpace::Securified) => Some(s),
                _ => None,
            })
            .collect::<IndexSet<_>>()
    }
}

impl EntitySecurityState {
    fn factors_from_source(
        &self,
        id: FactorSourceIDFromHash,
    ) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        self.all_factor_instances()
            .into_iter()
            .filter(|f| f.factor_source_id() == id)
            .collect()
    }
}

#[cfg(test)]
impl Profile {
    pub fn accounts<'a>(accounts: impl IntoIterator<Item = &'a Account>) -> Self {
        Self::new([], accounts, [])
    }
}
