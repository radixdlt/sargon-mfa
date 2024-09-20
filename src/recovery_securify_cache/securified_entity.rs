use crate::prelude::*;

/// The `SecurifiedEntityControl`, address and possibly third party deposit state of some
/// Securified entity.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Getters)]
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
