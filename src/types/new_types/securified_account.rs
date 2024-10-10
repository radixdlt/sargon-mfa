use crate::prelude::*;

use super::securified_entity;

/// The `SecurifiedEntityControl`, address and possibly third party deposit state of some
/// Securified entity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SecurifiedAccount {
    name: String,
    /// The address which is verified to match the `veci`
    account_address: AccountAddress,
    securified_entity_control: SecurifiedEntityControl,
    /// If we found this UnsecurifiedEntity while scanning OnChain using
    /// Gateway, we might have been able to read out the third party deposit
    /// settings.
    third_party_deposit: Option<ThirdPartyDepositPreference>,
}
impl From<SecurifiedAccount> for Account {
    fn from(value: SecurifiedAccount) -> Account {
        value.account()
    }
}
impl IsSecurifiedEntity for SecurifiedAccount {
    type BaseEntity = Account;
    fn securified_entity_control(&self) -> SecurifiedEntityControl {
        self.securified_entity_control()
    }

    fn new(
        name: impl AsRef<str>,
        address: AccountAddress,
        securified_entity_control: SecurifiedEntityControl,
        third_party_deposit: impl Into<Option<ThirdPartyDepositPreference>>,
    ) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            account_address: address,
            securified_entity_control,
            third_party_deposit: third_party_deposit.into(),
        }
    }
}

impl IsNetworkAware for SecurifiedAccount {
    fn network_id(&self) -> NetworkID {
        self.address().network_id()
    }
}

impl TryFrom<Account> for SecurifiedAccount {
    type Error = CommonError;
    fn try_from(value: Account) -> Result<Self> {
        let securified_entity_control = value
            .security_state()
            .as_securified()
            .cloned()
            .ok_or(CommonError::AccountNotSecurified)?;
        Ok(SecurifiedAccount::new(
            value.name(),
            value.entity_address(),
            securified_entity_control,
            value.third_party_deposit(),
        ))
    }
}

impl TryFrom<AccountOrPersona> for SecurifiedAccount {
    type Error = CommonError;
    fn try_from(value: AccountOrPersona) -> Result<Self> {
        Account::try_from(value).and_then(SecurifiedAccount::try_from)
    }
}
impl From<SecurifiedPersona> for Persona {
    fn from(value: SecurifiedPersona) -> Persona {
        value.persona()
    }
}
impl TryFrom<Persona> for SecurifiedPersona {
    type Error = CommonError;
    fn try_from(value: Persona) -> Result<Self> {
        let securified_entity_control = value
            .security_state()
            .as_securified()
            .cloned()
            .ok_or(CommonError::PersonaNotSecurified)?;
        Ok(SecurifiedPersona::new(
            value.name(),
            value.entity_address(),
            securified_entity_control,
            value.third_party_deposit(),
        ))
    }
}

impl TryFrom<AccountOrPersona> for SecurifiedPersona {
    type Error = CommonError;
    fn try_from(value: AccountOrPersona) -> Result<Self> {
        Persona::try_from(value).and_then(SecurifiedPersona::try_from)
    }
}

impl SecurifiedAccount {
    pub fn account(&self) -> Account {
        Account::new(
            self.name.clone(),
            self.address(),
            EntitySecurityState::Securified(self.securified_entity_control()),
            self.third_party_deposit,
        )
    }
    pub fn address(&self) -> AccountAddress {
        self.account_address.clone()
    }
    pub fn securified_entity_control(&self) -> SecurifiedEntityControl {
        self.securified_entity_control.clone()
    }
    pub fn third_party_deposit(&self) -> Option<ThirdPartyDepositPreference> {
        self.third_party_deposit
    }
}

impl HasSampleValues for SecurifiedAccount {
    fn sample() -> Self {
        Self::new(
            "SecurifiedAccount",
            AccountAddress::sample(),
            SecurifiedEntityControl::sample(),
            None,
        )
    }
    fn sample_other() -> Self {
        Self::new(
            "SecurifiedAccount Other",
            AccountAddress::sample_other(),
            SecurifiedEntityControl::sample_other(),
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type Sut = SecurifiedAccount;
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
    fn third_party_dep() {
        let test = |dep: ThirdPartyDepositPreference| {
            let sut = Sut::new(
                "name",
                AccountAddress::sample_0(),
                SecurifiedEntityControl::sample(),
                dep,
            );
            assert_eq!(sut.third_party_deposit(), Some(dep));
        };
        test(ThirdPartyDepositPreference::DenyAll);
        test(ThirdPartyDepositPreference::AllowAll);
        test(ThirdPartyDepositPreference::AllowKnown);
    }
}
