use crate::prelude::*;

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct HiddenConstructor;

pub trait IsHDFactorInstance {
    fn instance(&self) -> HierarchicalDeterministicFactorInstance;
    fn derivation_path(&self) -> DerivationPath {
        self.instance().derivation_path().clone()
    }
    fn derivation_entity_index(&self) -> HDPathComponent {
        self.derivation_path().index.clone()
    }
    fn network_id(&self) -> NetworkID {
        self.derivation_path().network_id.clone()
    }
}

/// A FactorInstance with a derivation path that is used for
/// Account, Unsecurified, TransactionSigning
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccountVeci {
    #[allow(dead_code)]
    hidden_constructor: HiddenConstructor,
    instance: HierarchicalDeterministicFactorInstance,
}
impl IsHDFactorInstance for AccountVeci {
    fn instance(&self) -> HierarchicalDeterministicFactorInstance {
        self.instance.clone()
    }
}
impl AccountVeci {
    pub fn new(instance: HierarchicalDeterministicFactorInstance) -> Result<Self> {
        let derivation_path = instance.derivation_path();

        if derivation_path.entity_kind != CAP26EntityKind::Account {
            return Err(CommonError::EntityKindDiscrepancy);
        }

        if derivation_path.key_space() != KeySpace::Unsecurified {
            return Err(CommonError::KeySpaceDiscrepancy);
        }

        if derivation_path.key_kind != CAP26KeyKind::TransactionSigning {
            return Err(CommonError::KeyKindDiscrepancy);
        }

        Ok(Self {
            hidden_constructor: HiddenConstructor,
            instance,
        })
    }
}

/// A FactorInstance with a derivation path that is used for
/// Identity, Unsecurified, TransactionSigning
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IdentityVeci {
    #[allow(dead_code)]
    hidden_constructor: HiddenConstructor,
    instance: HierarchicalDeterministicFactorInstance,
}
impl IdentityVeci {
    pub fn new(instance: HierarchicalDeterministicFactorInstance) -> Result<Self> {
        let derivation_path = instance.derivation_path();
        if derivation_path.entity_kind != CAP26EntityKind::Identity {
            return Err(CommonError::EntityKindDiscrepancy);
        }

        if derivation_path.key_space() != KeySpace::Unsecurified {
            return Err(CommonError::KeySpaceDiscrepancy);
        }

        if derivation_path.key_kind != CAP26KeyKind::TransactionSigning {
            return Err(CommonError::KeyKindDiscrepancy);
        }

        Ok(Self {
            hidden_constructor: HiddenConstructor,
            instance,
        })
    }
}
impl IsHDFactorInstance for IdentityVeci {
    fn instance(&self) -> HierarchicalDeterministicFactorInstance {
        self.instance.clone()
    }
}

/// A FactorInstance with a derivation path that is used for
/// Account, Securified, TransactionSigning
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccountMfaFactorInstance {
    #[allow(dead_code)]
    hidden_constructor: HiddenConstructor,
    instance: HierarchicalDeterministicFactorInstance,
}
impl AccountMfaFactorInstance {
    pub fn new(instance: HierarchicalDeterministicFactorInstance) -> Result<Self> {
        let derivation_path = instance.derivation_path();
        if derivation_path.entity_kind != CAP26EntityKind::Account {
            return Err(CommonError::EntityKindDiscrepancy);
        }

        if derivation_path.key_space() != KeySpace::Securified {
            return Err(CommonError::KeySpaceDiscrepancy);
        }

        if derivation_path.key_kind != CAP26KeyKind::TransactionSigning {
            return Err(CommonError::KeyKindDiscrepancy);
        }

        Ok(Self {
            hidden_constructor: HiddenConstructor,
            instance,
        })
    }
}
impl IsHDFactorInstance for AccountMfaFactorInstance {
    fn instance(&self) -> HierarchicalDeterministicFactorInstance {
        self.instance.clone()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DerivationTemplate {
    /// Account, Unsecurified, TransactionSigning,
    /// Veci: Virtual Entity Creating (Factor)Instance
    AccountVeci,

    /// Identity, Unsecurified, TransactionSigning
    /// Veci: Virtual Entity Creating (Factor)Instance
    IdentityVeci,

    /// Account, Securified, AuthenticationSigning
    AccountRola,

    /// Account, Securified, TransactionSigning
    AccountMfa,

    /// Identity, Securified, TransactionSigning
    IdentityMfa,
}

/// A collection of sets of FactorInstances,
/// all on the same network
/// all from the same factor source
/// for different DerivationTemplates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionsOfFactorInstances {
    #[allow(dead_code)]
    hidden_constructor: HiddenConstructor,
    pub network: NetworkID,
    pub factor_source_id: FactorSourceIDFromHash,
    pub unsecurified_accounts: IndexSet<AccountVeci>,
    pub unsecurified_identities: IndexSet<IdentityVeci>,
    pub securified_accounts: IndexSet<AccountMfaFactorInstance>,
}
impl CollectionsOfFactorInstances {
    pub fn empty(network: NetworkID, factor_source_id: FactorSourceIDFromHash) -> Self {
        Self::new(
            network,
            factor_source_id,
            IndexSet::new(),
            IndexSet::new(),
            IndexSet::new(),
        )
        .unwrap()
    }
    pub fn is_full(&self) -> bool {
        self.unsecurified_accounts.len() == CACHE_SIZE as usize
            && self.unsecurified_identities.len() == CACHE_SIZE as usize
    }
    pub fn new(
        network: NetworkID,
        factor_source_id: FactorSourceIDFromHash,
        unsecurified_accounts: IndexSet<AccountVeci>,
        unsecurified_identities: IndexSet<IdentityVeci>,
        securified_accounts: IndexSet<AccountMfaFactorInstance>,
    ) -> Result<Self> {
        if !(unsecurified_accounts
            .iter()
            .all(|f| f.network_id() == network)
            && unsecurified_identities
                .iter()
                .all(|f| f.network_id() == network)
            && securified_accounts
                .iter()
                .all(|f| f.network_id() == network))
        {
            return Err(CommonError::NetworkDiscrepancy);
        }

        if !(unsecurified_accounts
            .iter()
            .all(|f| f.network_id() == network)
            && unsecurified_identities
                .iter()
                .all(|f| f.network_id() == network))
        {
            return Err(CommonError::FactorSourceDiscrepancy);
        }

        Ok(Self {
            hidden_constructor: HiddenConstructor,
            network,
            factor_source_id,
            unsecurified_accounts,
            unsecurified_identities,
            securified_accounts,
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ToUseDirectly(IndexSet<HierarchicalDeterministicFactorInstance>);
impl ToUseDirectly {
    pub fn new(factor_instances: IndexSet<HierarchicalDeterministicFactorInstance>) -> Self {
        Self(factor_instances)
    }
    pub fn just(factor_instance: HierarchicalDeterministicFactorInstance) -> Self {
        Self::new(IndexSet::from_iter([factor_instance]))
    }
    pub fn account_veci(self) -> Result<AccountVeci> {
        todo!()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DerivationPathPerFactorSource {
    per_factor_source: IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToCache(pub CollectionsOfFactorInstances);
