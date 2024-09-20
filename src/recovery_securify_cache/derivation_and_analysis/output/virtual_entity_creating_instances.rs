use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VirtualEntityCreatingInstance {
    /// The instance which as known to have created `address`
    factor_instance: HierarchicalDeterministicFactorInstance,

    /// The address of the entity.
    address: AddressOfAccountOrPersona,
}
impl VirtualEntityCreatingInstance {
    /// # Panics
    /// Panics if factor_instance does not result in address.
    pub fn new(
        factor_instance: HierarchicalDeterministicFactorInstance,
        address: AddressOfAccountOrPersona,
    ) -> Self {
        assert_eq!(
            address.public_key_hash(),
            factor_instance.public_key_hash(),
            "Discrepancy! PublicKeys does not match, this is a programmer error!"
        );
        Self {
            address,
            factor_instance,
        }
    }

    pub fn address(&self) -> AddressOfAccountOrPersona {
        self.address.clone()
    }

    pub fn factor_instance(&self) -> HierarchicalDeterministicFactorInstance {
        self.factor_instance.clone()
    }

    fn with_factor_instance_on_network(
        factor_instance: HierarchicalDeterministicFactorInstance,
        entity_kind: CAP26EntityKind,
        network_id: NetworkID,
    ) -> Self {
        let public_key_hash = factor_instance.public_key_hash();
        let address = match entity_kind {
            CAP26EntityKind::Account => {
                AddressOfAccountOrPersona::from(AccountAddress::new(network_id, public_key_hash))
            }
            CAP26EntityKind::Identity => {
                AddressOfAccountOrPersona::from(IdentityAddress::new(network_id, public_key_hash))
            }
        };
        Self::new(factor_instance, address)
    }
}

impl HasSampleValues for VirtualEntityCreatingInstance {
    fn sample() -> Self {
        Self::with_factor_instance_on_network(
            HierarchicalDeterministicFactorInstance::sample(),
            CAP26EntityKind::Account,
            NetworkID::Mainnet,
        )
    }
    fn sample_other() -> Self {
        Self::with_factor_instance_on_network(
            HierarchicalDeterministicFactorInstance::sample_other(),
            CAP26EntityKind::Identity,
            NetworkID::Stokenet,
        )
    }
}
/// FactorInstances which we have mananage to match against a securified entity
/// in Profile, as the FactorInstance which was used to create said entities address.
///
/// Those entities should NOT be put in the field:
/// `recovered_unsecurified_entities: RecoveredUnsecurifiedEntities` inside the
/// `DerivationAndAnalysis`, since those instances are known to be VECIs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VirtualEntityCreatingInstances {
    vecis: Vec<VirtualEntityCreatingInstance>,
}

impl VirtualEntityCreatingInstances {
    pub fn new(vecis: IndexSet<VirtualEntityCreatingInstance>) -> Self {
        Self {
            vecis: vecis.into_iter().collect(),
        }
    }

    pub fn vecis(&self) -> IndexSet<VirtualEntityCreatingInstance> {
        self.vecis.iter().cloned().collect()
    }
}

impl IsFactorInstanceCollectionBase for VirtualEntityCreatingInstances {
    fn factor_instances(&self) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        self.vecis()
            .into_iter()
            .map(|x| x.factor_instance())
            .collect()
    }
}

impl HasSampleValues for VirtualEntityCreatingInstances {
    fn sample() -> Self {
        Self::new(IndexSet::from_iter([
            VirtualEntityCreatingInstance::sample(),
            VirtualEntityCreatingInstance::sample_other(),
        ]))
    }

    fn sample_other() -> Self {
        Self::new(IndexSet::just(VirtualEntityCreatingInstance::sample_other()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = VirtualEntityCreatingInstances;

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
