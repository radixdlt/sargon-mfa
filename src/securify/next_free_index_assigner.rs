#![allow(clippy::type_complexity)]

use crate::prelude::*;

use rand::Rng;
use sha2::{Digest, Sha256, Sha512};

#[derive(Clone, Copy, PartialEq)]
pub struct NextFreeIndexAssignerRequest<'p> {
    pub key_space: KeySpace,
    pub entity_kind: CAP26EntityKind,
    pub factor_source_id: FactorSourceIDFromHash,
    pub profile: &'p Profile,
    pub network_id: NetworkID,
}

pub struct NextFreeIndexAssigner {
    next: Box<dyn Fn(NextFreeIndexAssignerRequest) -> HDPathComponent>,
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

impl NextFreeIndexAssigner {
    fn new(next: impl Fn(NextFreeIndexAssignerRequest) -> HDPathComponent + 'static) -> Self {
        Self {
            next: Box::new(next),
        }
    }
    pub fn live() -> Self {
        Self::new(|request| {
            let NextFreeIndexAssignerRequest {
                key_space,
                entity_kind,
                factor_source_id,
                profile,
                network_id,
                ..
            } = request;

            profile
                .security_states_for_entities_of_kind(key_space, entity_kind, network_id)
                .into_iter()
                .filter_map(|s| {
                    let instances = s.factors_from_source(factor_source_id);
                    if instances.is_empty() {
                        None
                    } else {
                        instances
                            .into_iter()
                            .map(|x| x.entity_index())
                            .filter(|c| c.is_in_key_space(key_space))
                            .max()
                    }
                })
                .max()
                .map(|max| max.add_one())
                .unwrap_or(HDPathComponent::new_in_key_space(0, key_space))
        })
    }

    #[cfg(test)]
    pub fn test(hardcoded: HDPathValue) -> Self {
        Self::new(move |_| HDPathComponent::securified(hardcoded))
    }

    fn next_path_component(&self, request: NextFreeIndexAssignerRequest<'_>) -> HDPathComponent {
        (self.next)(request)
    }
}
impl Default for NextFreeIndexAssigner {
    fn default() -> Self {
        Self::live()
    }
}

impl DerivationIndexWhenSecurifiedAssigner for NextFreeIndexAssigner {
    fn derivation_index_for_factor_source(
        &self,
        request: NextFreeIndexAssignerRequest,
    ) -> HDPathComponent {
        self.next_path_component(request)
    }
}

#[cfg(test)]
impl Profile {
    pub fn accounts<'a>(accounts: impl IntoIterator<Item = &'a Account>) -> Self {
        Self::new([], accounts, [])
    }
}

#[cfg(test)]
mod test_next_free_index_assigner {

    use super::*;

    type Sut = NextFreeIndexAssigner;

    #[test]
    fn live_first() {
        let sut = Sut::live();
        let a = &Account::sample_unsecurified();

        let profile = &Profile::accounts([a]);

        let index = sut.derivation_index_for_factor_source(NextFreeIndexAssignerRequest {
            key_space: KeySpace::Securified,
            entity_kind: CAP26EntityKind::Account,
            factor_source_id: FactorSourceIDFromHash::fs0(),
            profile,
            network_id: NetworkID::Mainnet,
        });
        assert_eq!(index.securified_index(), Some(0));
    }

    #[test]
    fn live_second() {
        let sut = Sut::live();
        let a = &Account::sample_unsecurified();
        let b = &Account::securified_mainnet("Bob", AccountAddress::sample_1(), || {
            let i = HDPathComponent::securified(0);
            MatrixOfFactorInstances::m6(HierarchicalDeterministicFactorInstance::f(
                Account::entity_kind(),
                i,
            ))
        });

        let profile = &Profile::accounts([a, b]);
        let index = sut.derivation_index_for_factor_source(NextFreeIndexAssignerRequest {
            key_space: KeySpace::Securified,
            entity_kind: CAP26EntityKind::Account,
            factor_source_id: FactorSourceIDFromHash::fs0(),
            profile,
            network_id: NetworkID::Mainnet,
        });
        assert_eq!(index.securified_index(), Some(1));
    }
}
