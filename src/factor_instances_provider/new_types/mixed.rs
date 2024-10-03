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
    pub securified_accounts: IndexSet<AccountMfaFactorInstance>,

    pub unsecurified_identities: IndexSet<IdentityVeci>,
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

    pub fn append(&mut self, other: Self) {
        assert_eq!(other.network, self.network);
        assert_eq!(other.factor_source_id, self.factor_source_id);

        // TODO CLEAN UP this repetition mess! Use trait, or local closure...
        assert!(self
            .unsecurified_identities
            .intersection(&other.unsecurified_identities)
            .next()
            .is_none());
        assert!(self
            .unsecurified_accounts
            .intersection(&other.unsecurified_accounts)
            .next()
            .is_none());
        assert!(self
            .securified_accounts
            .intersection(&other.securified_accounts)
            .next()
            .is_none());

        if let Some(last_sec_acc) = self.securified_accounts.last() {
            if let Some(first_sec_acc) = other.securified_accounts.first() {
                assert!(
                    first_sec_acc.derivation_entity_index()
                        > last_sec_acc.derivation_entity_index(),
                    "First index of new securified account is not larger than last of existing!"
                );
            }
        }

        if let Some(last_unsec_acc) = self.unsecurified_accounts.last() {
            if let Some(first_unsec_acc) = other.unsecurified_accounts.first() {
                assert!(
                    first_unsec_acc.derivation_entity_index()
                        > last_unsec_acc.derivation_entity_index(),
                    "First index of new unsecurified account is not larger than last of existing!"
                );
            }
        }

        if let Some(last_unsec_ident) = self.unsecurified_identities.last() {
            if let Some(first_unsec_ident) = other.unsecurified_identities.first() {
                assert!(
                    first_unsec_ident.derivation_entity_index()
                        > last_unsec_ident.derivation_entity_index(),
                    "First index of new unsecurified identity is not larger than last of existing!"
                );
            }
        }

        self.securified_accounts.extend(other.securified_accounts);
        self.unsecurified_accounts
            .extend(other.unsecurified_accounts);
        self.unsecurified_identities
            .extend(other.unsecurified_identities);
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
        assert_eq!(self.0.len(), 1);
        AccountVeci::new(self.0.into_iter().next().unwrap())
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
pub struct ToCache(pub IndexMap<FactorSourceIDFromHash, CollectionsOfFactorInstances>);

impl From<(NetworkID, KeyDerivationOutcome)> for ToCache {
    fn from(value: (NetworkID, KeyDerivationOutcome)) -> Self {
        let (network_id, derivation_outcome) = value;
        Self::from((network_id, derivation_outcome.factors_by_source))
    }
}
impl
    From<(
        NetworkID,
        IndexMap<FactorSourceIDFromHash, IndexSet<HierarchicalDeterministicFactorInstance>>,
    )> for ToCache
{
    fn from(
        value: (
            NetworkID,
            IndexMap<FactorSourceIDFromHash, IndexSet<HierarchicalDeterministicFactorInstance>>,
        ),
    ) -> Self {
        let (network_id, factors_by_source) = value;
        let mut set_of_collections =
            IndexMap::<FactorSourceIDFromHash, CollectionsOfFactorInstances>::new();
        for (factor_source_id, factors) in factors_by_source {
            let unsecurified_accounts = factors
                .clone()
                .into_iter()
                .filter_map(|f| AccountVeci::new(f).ok())
                .collect::<IndexSet<_>>();

            let securified_accounts = factors
                .clone()
                .into_iter()
                .filter_map(|f| AccountMfaFactorInstance::new(f).ok())
                .collect::<IndexSet<_>>();

            let unsecurified_identities = factors
                .clone()
                .into_iter()
                .filter_map(|f| IdentityVeci::new(f).ok())
                .collect::<IndexSet<_>>();

            assert_eq!(
                factors.len(),
                unsecurified_accounts.len()
                    + securified_accounts.len()
                    + unsecurified_identities.len(),
                "Discrepancy skipped some factors, MFA Identity? ROLA?"
            );

            let collections_for_factor = CollectionsOfFactorInstances::new(
                network_id,
                factor_source_id,
                unsecurified_accounts,
                unsecurified_identities,
                securified_accounts,
            )
            .unwrap();

            assert!(set_of_collections
                .insert(factor_source_id, collections_for_factor)
                .is_none());
        }
        Self(set_of_collections)
    }
}
