#![allow(clippy::type_complexity)]

use crate::prelude::*;

use rand::Rng;
use sha2::{Digest, Sha256, Sha512};

/// This index assigner ASSUMES CEI strategy 1 of
/// https://radixdlt.atlassian.net/wiki/spaces/AT/pages/3640655873/Yet+Another+Page+about+Derivation+Indices
///
/// Meaning Canonical Entity Indexing - CEI, using "next" index for accounts, using counters per network.
///
/// CEI means the SAME derivation path is used for ALL FactorInstances of a security entity.
pub struct CanonicalEntityIndexingNextFreeIndexAssigner {
    next: Box<dyn Fn(&Profile, NetworkID) -> HDPathComponent>,
}

impl HierarchicalDeterministicFactorInstance {
    fn entity_index(&self) -> HDPathComponent {
        self.derivation_path().index
    }
    fn network_id(&self) -> NetworkID {
        self.derivation_path().network_id
    }
}

fn canonical_entity_index_if_securified<E: IsEntity>(
    entity: &E,
    asserting_network: NetworkID,
) -> Option<HDPathComponent> {
    if !entity.is_securified() {
        return None;
    }
    let factors = entity.all_factor_instances();
    assert!(!factors.is_empty());
    assert!(factors.iter().all(|f| f.network_id() == asserting_network));
    let canonical = factors.iter().last().unwrap().entity_index();
    assert!(factors.iter().all(|f| f.entity_index() == canonical));
    Some(canonical)
}

impl CanonicalEntityIndexingNextFreeIndexAssigner {
    fn new(next: impl Fn(&Profile, NetworkID) -> HDPathComponent + 'static) -> Self {
        Self {
            next: Box::new(next),
        }
    }
    pub fn live() -> Self {
        Self::new(|profile, network| {
            profile
                .accounts
                .values()
                .filter(|a| a.network_id() == network)
                .filter(|a| a.is_securified())
                .map(|a| canonical_entity_index_if_securified(a, network).unwrap())
                .max()
                .map(|max| max.add_one())
                .unwrap_or(HDPathComponent::securified(0))
        })
    }

    #[cfg(test)]
    pub fn test(hardcoded: HDPathValue) -> Self {
        Self::new(move |_, _| HDPathComponent::securified(hardcoded))
    }

    fn next_path_component(&self, profile: &Profile, network_id: NetworkID) -> HDPathComponent {
        let component = (self.next)(profile, network_id);
        assert!(component.is_securified());
        component
    }
}
impl Default for CanonicalEntityIndexingNextFreeIndexAssigner {
    fn default() -> Self {
        Self::live()
    }
}

impl DerivationIndexWhenSecurifiedAssigner for CanonicalEntityIndexingNextFreeIndexAssigner {
    fn assign_derivation_index(&self, profile: &Profile, network_id: NetworkID) -> HDPathComponent {
        self.next_path_component(profile, network_id)
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

    type Sut = CanonicalEntityIndexingNextFreeIndexAssigner;

    #[test]
    fn live_first() {
        let sut = Sut::live();
        let a = &Account::sample_unsecurified();

        let profile = &Profile::accounts([a]);
        let index = sut.assign_derivation_index(profile, NetworkID::Mainnet);
        assert_eq!(index.securified_index(), Some(0));
    }

    #[test]
    fn live_second() {
        let sut = Sut::live();
        let a = &Account::sample_unsecurified();
        let b = &Account::securified_mainnet(0, "Bob", |idx| {
            MatrixOfFactorInstances::m6(HierarchicalDeterministicFactorInstance::f(
                Account::entity_kind(),
                idx,
            ))
        });

        let profile = &Profile::accounts([a, b]);
        let index = sut.assign_derivation_index(profile, NetworkID::Mainnet);
        assert_eq!(index.securified_index(), Some(1));
    }
}
