use crate::prelude::*;

/// The HDFactorInstance, address and possibly third party deposit state of some
/// unsecurified entity.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Getters)]
pub struct UnsecurifiedEntity {
    /// The address which is verified to match the `veci`
    address: AddressOfAccountOrPersona,

    /// Virtual Entity Creating (Factor)Instance
    veci: HierarchicalDeterministicFactorInstance,

    /// If we found this UnsecurifiedEntity while scanning OnChain using
    /// Gateway, we might have been able to read out the third party deposit
    /// settings.
    third_party_deposit: Option<ThirdPartyDepositPreference>,
}

impl UnsecurifiedEntity {
    /// # Panics
    /// Panics if address does not match `veci`
    pub fn new(
        address: AddressOfAccountOrPersona,
        veci: HierarchicalDeterministicFactorInstance,
        third_party_deposit: impl Into<Option<ThirdPartyDepositPreference>>,
    ) -> Self {
        assert!(
            address.derived_from_factor_instance(&veci),
            "Discrepancy, mismatching public keys, this is a programmer error!"
        );
        Self {
            address,
            veci,
            third_party_deposit: third_party_deposit.into(),
        }
    }

    fn with_veci_on_network(
        veci: HierarchicalDeterministicFactorInstance,
        entity_kind: CAP26EntityKind,
        network_id: NetworkID,
    ) -> Self {
        let public_key_hash = veci.public_key_hash();
        let address = match entity_kind {
            CAP26EntityKind::Account => {
                AddressOfAccountOrPersona::from(AccountAddress::new(network_id, public_key_hash))
            }
            CAP26EntityKind::Identity => {
                AddressOfAccountOrPersona::from(IdentityAddress::new(network_id, public_key_hash))
            }
        };
        Self {
            address,
            veci,
            third_party_deposit: None,
        }
    }
}

impl HasSampleValues for UnsecurifiedEntity {
    fn sample() -> Self {
        Self::with_veci_on_network(
            HierarchicalDeterministicFactorInstance::sample(),
            CAP26EntityKind::Account,
            NetworkID::Mainnet,
        )
    }
    fn sample_other() -> Self {
        Self::with_veci_on_network(
            HierarchicalDeterministicFactorInstance::sample_other(),
            CAP26EntityKind::Identity,
            NetworkID::Stokenet,
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
