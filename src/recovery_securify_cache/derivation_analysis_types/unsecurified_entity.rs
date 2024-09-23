use std::vec;

use crate::prelude::*;

/// The HDFactorInstance, address and possibly third party deposit state of some
/// unsecurified entity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnsecurifiedEntity {
    veci: VirtualEntityCreatingInstance,

    /// If we found this UnsecurifiedEntity while scanning OnChain using
    /// Gateway, we might have been able to read out the third party deposit
    /// settings.
    third_party_deposit: Option<ThirdPartyDepositPreference>,
}

impl UnsecurifiedEntity {
    pub fn with_veci(
        veci: VirtualEntityCreatingInstance,
        third_party_deposit: impl Into<Option<ThirdPartyDepositPreference>>,
    ) -> Self {
        Self {
            veci,
            third_party_deposit: third_party_deposit.into(),
        }
    }

    /// # Panics
    /// Panics if address does not match `factor_instance`
    pub fn new(
        address: AddressOfAccountOrPersona,
        factor_instance: HierarchicalDeterministicFactorInstance,
        third_party_deposit: impl Into<Option<ThirdPartyDepositPreference>>,
    ) -> Self {
        let veci = VirtualEntityCreatingInstance::new(factor_instance, address);
        Self::with_veci(veci, third_party_deposit)
    }

    pub fn address(&self) -> AddressOfAccountOrPersona {
        self.veci.clone().address()
    }

    pub fn factor_instance(&self) -> HierarchicalDeterministicFactorInstance {
        self.veci.factor_instance()
    }

    pub fn veci(&self) -> VirtualEntityCreatingInstance {
        self.veci.clone()
    }

    pub fn third_party_deposit(&self) -> Option<ThirdPartyDepositPreference> {
        self.third_party_deposit
    }
}

impl From<UnsecurifiedEntity> for Account {
    fn from(value: UnsecurifiedEntity) -> Self {
        let address = value.address();
        let name = "Recovered";
        let security_state = EntitySecurityState::Unsecured(value.factor_instance());

        if let Ok(account_address) = address.clone().into_account() {
            Account::new(name, account_address, security_state)
        } else {
            panic!("Not an AccountAddress")
        }
    }
}
impl From<UnsecurifiedEntity> for AccountOrPersona {
    fn from(value: UnsecurifiedEntity) -> Self {
        let address = value.address();
        let name = "Recovered";
        let security_state = EntitySecurityState::Unsecured(value.factor_instance());

        if let Ok(account_address) = address.clone().into_account() {
            Account::new(name, account_address, security_state).into()
        } else if let Ok(identity_address) = address.clone().into_identity() {
            Persona::new(name, identity_address, security_state).into()
        } else {
            unreachable!("Either account or persona.")
        }
    }
}

impl HasSampleValues for UnsecurifiedEntity {
    fn sample() -> Self {
        Self::with_veci(
            VirtualEntityCreatingInstance::sample(),
            ThirdPartyDepositPreference::sample(),
        )
    }
    fn sample_other() -> Self {
        Self::with_veci(
            VirtualEntityCreatingInstance::sample_other(),
            ThirdPartyDepositPreference::sample_other(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = UnsecurifiedEntity;

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
