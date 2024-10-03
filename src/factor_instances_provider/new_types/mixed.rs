use crate::prelude::*;

pub trait IsHDFactorInstance {
    fn instance(&self) -> HierarchicalDeterministicFactorInstance;
    fn derivation_path(&self) -> DerivationPath {
        self.instance().derivation_path().clone()
    }
    fn derivation_entity_index(&self) -> HDPathComponent {
        self.derivation_path().index
    }
    fn network_id(&self) -> NetworkID {
        self.derivation_path().network_id
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
impl DerivationTemplate {
    pub fn entity_kind(&self) -> CAP26EntityKind {
        match self {
            Self::AccountVeci => CAP26EntityKind::Account,
            Self::AccountRola => CAP26EntityKind::Account,
            Self::AccountMfa => CAP26EntityKind::Account,
            Self::IdentityVeci => CAP26EntityKind::Identity,
            Self::IdentityMfa => CAP26EntityKind::Identity,
        }
    }
    pub fn key_space(&self) -> KeySpace {
        match self {
            Self::AccountVeci => KeySpace::Unsecurified,
            Self::AccountRola => KeySpace::Securified, // TODO: I think we don't create ROLA keys for UnsecurifiedAccounts, if we do, split this into two variants.
            Self::AccountMfa => KeySpace::Securified,
            Self::IdentityVeci => KeySpace::Unsecurified,
            Self::IdentityMfa => KeySpace::Securified,
        }
    }
    pub fn key_kind(&self) -> CAP26KeyKind {
        match self {
            Self::AccountVeci => CAP26KeyKind::TransactionSigning,
            Self::AccountMfa => CAP26KeyKind::TransactionSigning,
            Self::IdentityVeci => CAP26KeyKind::TransactionSigning,
            Self::IdentityMfa => CAP26KeyKind::TransactionSigning,

            Self::AccountRola => CAP26KeyKind::AuthenticationSigning,
        }
    }
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
        self.unsecurified_accounts.len() == CACHE_SIZE
            && self.unsecurified_identities.len() == CACHE_SIZE
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
    pub paths_per_template_per_factor:
        IndexMap<FactorSourceIDFromHash, IndexMap<DerivationTemplate, IndexSet<DerivationPath>>>,
}

impl DerivationPathPerFactorSource {
    /// Flattens the collection, merging all DerivationPaths for the same FactorSource together,
    /// effectively removing the DerivationTemplate level.
    pub fn flatten(&self) -> IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>> {
        self.paths_per_template_per_factor
            .iter()
            .map(|(factor_source_id, paths_per_template)| {
                let paths = paths_per_template
                    .iter()
                    .flat_map(|(_, paths)| paths.iter().cloned())
                    .collect();
                (*factor_source_id, paths)
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToCache(pub CollectionsOfFactorInstances);
