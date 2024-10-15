use crate::prelude::*;

/// The `SecurifiedEntityControl`, address and possibly third party deposit state of some
/// Securified entity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SecurifiedPersona {
    name: String,
    /// The address which is verified to match the `veci`
    identity_address: IdentityAddress,
    securified_entity_control: SecurifiedEntityControl,
}
impl IsNetworkAware for SecurifiedPersona {
    fn network_id(&self) -> NetworkID {
        self.address().network_id()
    }
}

impl IsSecurifiedEntity for SecurifiedPersona {
    type BaseEntity = Persona;
    fn securified_entity_control(&self) -> SecurifiedEntityControl {
        self.securified_entity_control()
    }

    fn new(
        name: impl AsRef<str>,
        address: IdentityAddress,
        securified_entity_control: SecurifiedEntityControl,
        _third_party_deposit: impl Into<Option<ThirdPartyDepositPreference>>,
    ) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            identity_address: address,
            securified_entity_control,
        }
    }
}

impl SecurifiedPersona {
    pub fn persona(&self) -> Persona {
        Persona::new(
            self.name.clone(),
            self.address(),
            EntitySecurityState::Securified(self.securified_entity_control()),
            None,
        )
    }
    pub fn address(&self) -> IdentityAddress {
        self.identity_address.clone()
    }
    pub fn securified_entity_control(&self) -> SecurifiedEntityControl {
        self.securified_entity_control.clone()
    }
    pub fn third_party_deposit(&self) -> Option<ThirdPartyDepositPreference> {
        None
    }
}
impl HasSampleValues for SecurifiedPersona {
    fn sample() -> Self {
        Self::new(
            "SecurifiedPersona",
            IdentityAddress::sample(),
            SecurifiedEntityControl::sample(),
            None,
        )
    }
    fn sample_other() -> Self {
        Self::new(
            "SecurifiedPersona Other",
            IdentityAddress::sample_other(),
            SecurifiedEntityControl::sample_other(),
            None,
        )
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    type Sut = SecurifiedPersona;

    #[test]
    fn equality() {
        assert_eq!(Sut::sample(), Sut::sample());
        assert_eq!(Sut::sample_other(), Sut::sample_other());
    }

    #[test]
    fn inequality() {
        assert_ne!(Sut::sample(), Sut::sample_other());
    }
}
