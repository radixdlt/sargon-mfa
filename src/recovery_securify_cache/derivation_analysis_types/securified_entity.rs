use crate::prelude::*;

/// The `SecurifiedEntityControl`, address and possibly third party deposit state of some
/// Securified entity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SecurifiedEntity {
    /// The address which is verified to match the `veci`
    address: AddressOfAccountOrPersona,

    securified_entity_control: SecurifiedEntityControl,

    /// If we found this UnsecurifiedEntity while scanning OnChain using
    /// Gateway, we might have been able to read out the third party deposit
    /// settings.
    third_party_deposit: Option<ThirdPartyDepositPreference>,
}

impl SecurifiedEntity {
    pub fn new(
        address: AddressOfAccountOrPersona,
        securified_entity_control: SecurifiedEntityControl,
        third_party_deposit: impl Into<Option<ThirdPartyDepositPreference>>,
    ) -> Self {
        Self {
            address,
            securified_entity_control,
            third_party_deposit: third_party_deposit.into(),
        }
    }

    pub fn address(&self) -> AddressOfAccountOrPersona {
        self.address.clone()
    }

    pub fn securified_entity_control(&self) -> SecurifiedEntityControl {
        self.securified_entity_control.clone()
    }

    pub fn third_party_deposit(&self) -> Option<ThirdPartyDepositPreference> {
        self.third_party_deposit
    }
}

impl HasSampleValues for SecurifiedEntity {
    fn sample() -> Self {
        Self::new(
            AddressOfAccountOrPersona::sample(),
            SecurifiedEntityControl::sample(),
            ThirdPartyDepositPreference::sample(),
        )
    }
    fn sample_other() -> Self {
        Self::new(
            AddressOfAccountOrPersona::sample_other(),
            SecurifiedEntityControl::sample_other(),
            ThirdPartyDepositPreference::sample_other(),
        )
    }
}

impl From<SecurifiedEntity> for AccountOrPersona {
    fn from(value: SecurifiedEntity) -> Self {
        let address = value.address();
        let name = "Recovered";
        let security_state = EntitySecurityState::Securified(value.securified_entity_control());

        if let Ok(account_address) = address.clone().into_account() {
            Account::new(name, account_address, security_state).into()
        } else if let Ok(identity_address) = address.clone().into_identity() {
            Persona::new(name, identity_address, security_state).into()
        } else {
            unreachable!("Either account or persona.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = SecurifiedEntity;

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
