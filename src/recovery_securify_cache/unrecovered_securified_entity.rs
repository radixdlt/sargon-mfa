use crate::prelude::*;

/// The hash of the public key of a missing factor instance and the factor list
/// kind it belongs to - according to ScryptoAccessRule checked against gateway.
///
/// # Future
/// We might want to add information about which role, primary, confirmation
/// or recover.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MissingFactorInstance {
    /// The hash of the public key of a missing factor instance.
    pub public_key_hash: PublicKeyHash,

    /// Threshold or override factor
    pub factor_list: FactorListKind,
}

impl HasSampleValues for MissingFactorInstance {
    fn sample() -> Self {
        Self {
            public_key_hash: PublicKeyHash::sample(),
            factor_list: FactorListKind::sample(),
        }
    }
    fn sample_other() -> Self {
        Self {
            public_key_hash: PublicKeyHash::sample_other(),
            factor_list: FactorListKind::sample_other(),
        }
    }
}

/// An unrecovered securified entity means that we did not regain
/// control of it, that is, some FactorInstances are missing to have
/// control of the entity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnrecoveredSecurifiedEntity {
    /// The address which is verified to match the `veci`
    pub address: AddressOfAccountOrPersona,

    /// FactorInstances we have control over.
    matched_factor_instances: Vec<HierarchicalDeterministicFactorInstance>,

    /// List of FactorInstances we were not able to locally re-derive, but set
    /// on this entity according to ScryptoAccessRule checked against Gateway.
    missing_factor_instances: Vec<MissingFactorInstance>,

    /// If we found this UnsecurifiedEntity while scanning OnChain using
    /// Gateway, we might have been able to read out the third party deposit
    /// settings.
    pub third_party_deposit: Option<ThirdPartyDepositPreference>,
}

impl UnrecoveredSecurifiedEntity {
    pub fn new(
        address: AddressOfAccountOrPersona,
        matched_factor_instances: IndexSet<HierarchicalDeterministicFactorInstance>,
        missing_factor_instances: IndexSet<MissingFactorInstance>,
        third_party_deposit: impl Into<Option<ThirdPartyDepositPreference>>,
    ) -> Self {
        Self {
            address,
            matched_factor_instances: matched_factor_instances.into_iter().collect(),
            missing_factor_instances: missing_factor_instances.into_iter().collect(),
            third_party_deposit: third_party_deposit.into(),
        }
    }

    /// FactorInstances we have control over.
    pub fn matched_factor_instances(&self) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        self.matched_factor_instances.clone().into_iter().collect()
    }

    /// List of FactorInstances we were not able to locally re-derive, but set
    /// on this entity according to ScryptoAccessRule checked against Gateway.
    pub fn missing_factor_instances(&self) -> IndexSet<MissingFactorInstance> {
        self.missing_factor_instances.clone().into_iter().collect()
    }
}

impl HasSampleValues for UnrecoveredSecurifiedEntity {
    fn sample() -> Self {
        Self::new(
            AddressOfAccountOrPersona::sample(),
            IndexSet::from_iter([
                HierarchicalDeterministicFactorInstance::sample(),
                HierarchicalDeterministicFactorInstance::sample_other(),
            ]),
            IndexSet::from_iter([
                MissingFactorInstance::sample(),
                MissingFactorInstance::sample_other(),
            ]),
            ThirdPartyDepositPreference::sample(),
        )
    }
    fn sample_other() -> Self {
        Self::new(
            AddressOfAccountOrPersona::sample_other(),
            IndexSet::from_iter([HierarchicalDeterministicFactorInstance::sample_other()]),
            IndexSet::from_iter([MissingFactorInstance::sample_other()]),
            ThirdPartyDepositPreference::sample_other(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = UnrecoveredSecurifiedEntity;

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
