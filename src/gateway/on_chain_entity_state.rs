#![allow(unused)]

use crate::prelude::*;

/// A snapshot of the information that Gateway can hand us
/// about an entities Address - or queried by a PublicKeyHash
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OnChainEntityState {
    pub address: AddressOfAccountOrPersona,
    /// `None` if not securified, `Some` if securified
    pub access_controller: Option<AccessController>,
    pub owner_keys: Vec<PublicKeyHash>,
    /// TODO: we should read this...
    pub third_party_deposits: (),
}

impl OnChainEntityState {
    pub fn new(
        address: impl Into<AddressOfAccountOrPersona>,
        access_controller: impl Into<Option<AccessController>>,
        owner_keys: impl IntoIterator<Item = PublicKeyHash>,
    ) -> Self {
        Self {
            address: address.into(),
            access_controller: access_controller.into(),
            owner_keys: owner_keys.into_iter().collect_vec(),
            third_party_deposits: (),
        }
    }

    pub fn unsecurified(owner: AddressOfAccountOrPersona, owner_key: PublicKeyHash) -> Self {
        Self::new(owner, None, [owner_key])
    }

    pub fn securified(
        owner: AddressOfAccountOrPersona,
        access_controller: AccessController,
        owner_keys: IndexSet<PublicKeyHash>,
    ) -> Self {
        Self::new(owner, access_controller, owner_keys)
    }
}

impl OnChainEntityState {
    pub fn address(&self) -> AddressOfAccountOrPersona {
        self.address.clone()
    }

    pub fn owner_keys(&self) -> HashSet<PublicKeyHash> {
        self.owner_keys.clone().into_iter().collect()
    }

    pub fn is_securified(&self) -> bool {
        self.access_controller.is_some()
    }
}

impl HasSampleValues for OnChainEntityState {
    fn sample() -> Self {
        Self::securified(
            AddressOfAccountOrPersona::sample(),
            AccessController::sample(),
            IndexSet::from_iter([PublicKeyHash::sample(), PublicKeyHash::sample_other()]),
        )
    }

    fn sample_other() -> Self {
        Self::unsecurified(
            AddressOfAccountOrPersona::sample(),
            PublicKeyHash::sample_other(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = OnChainEntityState;

    #[test]
    fn equality() {
        assert_eq!(Sut::sample(), Sut::sample());
        assert_eq!(Sut::sample_other(), Sut::sample_other());
    }

    #[test]
    fn inequality() {
        assert_ne!(Sut::sample(), Sut::sample_other());
    }

    #[test]
    fn securified_is_securified() {
        assert!(Sut::sample().is_securified());
    }

    #[test]
    fn address() {
        assert_eq!(Sut::sample().address(), AddressOfAccountOrPersona::sample());
    }

    #[test]
    fn unsecurified_is_not_securified() {
        assert!(!Sut::sample_other().is_securified());
    }

    #[test]
    fn owner_keys() {
        assert_eq!(
            Sut::sample().owner_keys(),
            HashSet::from_iter([PublicKeyHash::sample(), PublicKeyHash::sample_other()])
        );
    }
}
