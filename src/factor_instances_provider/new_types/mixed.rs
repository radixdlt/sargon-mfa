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
    fn factor_source_id(&self) -> FactorSourceIDFromHash {
        self.instance().factor_source_id()
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

/// A FactorInstance with a derivation path that is used for
/// Account, Securified, TransactionSigning
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IdentityMfaFactorInstance {
    #[allow(dead_code)]
    hidden_constructor: HiddenConstructor,
    instance: HierarchicalDeterministicFactorInstance,
}
impl IdentityMfaFactorInstance {
    pub fn new(instance: HierarchicalDeterministicFactorInstance) -> Result<Self> {
        let derivation_path = instance.derivation_path();
        if derivation_path.entity_kind != CAP26EntityKind::Identity {
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
impl IsHDFactorInstance for IdentityMfaFactorInstance {
    fn instance(&self) -> HierarchicalDeterministicFactorInstance {
        self.instance.clone()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, enum_iterator::Sequence)]
pub enum DerivationTemplate {
    /// Account, Unsecurified, TransactionSigning,
    /// Veci: Virtual Entity Creating (Factor)Instance
    AccountVeci,

    /// Account, Securified, TransactionSigning
    AccountMfa,

    /// Identity, Unsecurified, TransactionSigning
    /// Veci: Virtual Entity Creating (Factor)Instance
    IdentityVeci,

    /// Identity, Securified, TransactionSigning
    IdentityMfa,
    // AccountRola, // TODO, this
}
impl DerivationTemplate {
    pub fn entity_kind(&self) -> CAP26EntityKind {
        match self {
            Self::AccountVeci | Self::AccountMfa => CAP26EntityKind::Account,
            Self::IdentityVeci | Self::IdentityMfa => CAP26EntityKind::Identity,
        }
    }
    pub fn key_space(&self) -> KeySpace {
        match self {
            Self::AccountVeci | Self::IdentityVeci => KeySpace::Unsecurified,
            Self::AccountMfa | Self::IdentityMfa => KeySpace::Securified,
        }
    }
    pub fn key_kind(&self) -> CAP26KeyKind {
        match self {
            Self::AccountVeci | Self::AccountMfa | Self::IdentityVeci | Self::IdentityMfa => {
                CAP26KeyKind::TransactionSigning
            }
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

    pub account_veci: IndexSet<AccountVeci>,
    pub account_mfa: IndexSet<AccountMfaFactorInstance>,

    pub identity_veci: IndexSet<IdentityVeci>,
    pub identity_mfa: IndexSet<IdentityMfaFactorInstance>,
}
impl CollectionsOfFactorInstances {
    pub fn empty(network: NetworkID, factor_source_id: FactorSourceIDFromHash) -> Self {
        Self::new(
            network,
            factor_source_id,
            IndexSet::new(),
            IndexSet::new(),
            IndexSet::new(),
            IndexSet::new(),
        )
        .unwrap()
    }

    pub fn is_full(&self) -> bool {
        println!(
            "😈 CACHE_SIZE: {}, #account_veci: {}, #account_mfa: {}, #identity_veci: {}, #identity_mfa: {}",
            CACHE_SIZE,
            self.account_veci.len(),
            self.account_mfa.len(),
            self.identity_veci.len(),
            self.identity_mfa.len(),
        );
        self.account_veci.len() == CACHE_SIZE
            && self.account_mfa.len() == CACHE_SIZE
            && self.identity_veci.len() == CACHE_SIZE
            && self.identity_mfa.len() == CACHE_SIZE
    }

    pub fn new(
        network: NetworkID,
        factor_source_id: FactorSourceIDFromHash,
        account_veci: IndexSet<AccountVeci>,
        account_mfa: IndexSet<AccountMfaFactorInstance>,

        identity_veci: IndexSet<IdentityVeci>,
        identity_mfa: IndexSet<IdentityMfaFactorInstance>,
    ) -> Result<Self> {
        if !(account_veci.iter().all(|f| f.network_id() == network)
            && account_mfa.iter().all(|f| f.network_id() == network)
            && identity_veci.iter().all(|f| f.network_id() == network)
            && identity_mfa.iter().all(|f| f.network_id() == network))
        {
            return Err(CommonError::NetworkDiscrepancy);
        }

        if !(account_veci
            .iter()
            .all(|f| f.factor_source_id() == factor_source_id)
            && account_mfa
                .iter()
                .all(|f| f.factor_source_id() == factor_source_id)
            && identity_veci
                .iter()
                .all(|f| f.factor_source_id() == factor_source_id)
            && identity_mfa
                .iter()
                .all(|f| f.factor_source_id() == factor_source_id))
        {
            return Err(CommonError::FactorSourceDiscrepancy);
        }

        Ok(Self {
            hidden_constructor: HiddenConstructor,
            network,
            factor_source_id,
            account_veci,
            account_mfa,
            identity_veci,
            identity_mfa,
        })
    }

    pub fn quantity_for_template(&self, derivation_template: DerivationTemplate) -> usize {
        match derivation_template {
            DerivationTemplate::AccountVeci => self.account_veci.len(),
            DerivationTemplate::AccountMfa => self.account_mfa.len(),
            DerivationTemplate::IdentityVeci => self.identity_veci.len(),
            DerivationTemplate::IdentityMfa => self.identity_mfa.len(),
        }
    }

    pub fn append(&mut self, other: Self) {
        assert_eq!(other.network, self.network);
        assert_eq!(other.factor_source_id, self.factor_source_id);

        // TODO CLEAN UP this repetition mess! Use trait, or local closure...
        assert!(self
            .account_veci
            .intersection(&other.account_veci)
            .next()
            .is_none());
        assert!(self
            .account_mfa
            .intersection(&other.account_mfa)
            .next()
            .is_none());
        assert!(self
            .identity_veci
            .intersection(&other.identity_veci)
            .next()
            .is_none());
        assert!(self
            .identity_mfa
            .intersection(&other.identity_mfa)
            .next()
            .is_none());

        if let Some(last_unsec_acc) = self.account_veci.last() {
            if let Some(first_unsec_acc) = other.account_veci.first() {
                assert!(
                    first_unsec_acc.derivation_entity_index()
                        > last_unsec_acc.derivation_entity_index(),
                    "First index of new unsecurified account is not larger than last of existing!"
                );
            }
        }

        if let Some(last_sec_acc) = self.account_mfa.last() {
            if let Some(first_sec_acc) = other.account_mfa.first() {
                assert!(
                    first_sec_acc.derivation_entity_index()
                        > last_sec_acc.derivation_entity_index(),
                    "First index of new securified account is not larger than last of existing!"
                );
            }
        }

        if let Some(last_unsec_ident) = self.identity_veci.last() {
            if let Some(first_unsec_ident) = other.identity_veci.first() {
                assert!(
                    first_unsec_ident.derivation_entity_index()
                        > last_unsec_ident.derivation_entity_index(),
                    "First index of new unsecurified identity is not larger than last of existing!"
                );
            }
        }

        if let Some(last_sec_ident) = self.identity_mfa.last() {
            if let Some(first_sec_ident) = other.identity_mfa.first() {
                assert!(
                    first_sec_ident.derivation_entity_index()
                        > last_sec_ident.derivation_entity_index(),
                    "First index of new securified identity is not larger than last of existing!"
                );
            }
        }

        self.account_veci.extend(other.account_veci);
        self.account_mfa.extend(other.account_mfa);
        self.identity_veci.extend(other.identity_veci);
        self.identity_mfa.extend(other.identity_mfa);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InstancesToUseDirectly(IndexSet<HierarchicalDeterministicFactorInstance>);
impl InstancesToUseDirectly {
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
pub struct InstancesToCache(pub IndexMap<FactorSourceIDFromHash, CollectionsOfFactorInstances>);

impl From<(NetworkID, KeyDerivationOutcome)> for InstancesToCache {
    fn from(value: (NetworkID, KeyDerivationOutcome)) -> Self {
        let (network_id, derivation_outcome) = value;
        Self::from((network_id, derivation_outcome.factors_by_source))
    }
}
impl
    From<(
        NetworkID,
        IndexMap<FactorSourceIDFromHash, IndexSet<HierarchicalDeterministicFactorInstance>>,
    )> for InstancesToCache
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
            let account_veci = factors
                .clone()
                .into_iter()
                .filter_map(|f| AccountVeci::new(f).ok())
                .collect::<IndexSet<_>>();

            let account_mfa = factors
                .clone()
                .into_iter()
                .filter_map(|f| AccountMfaFactorInstance::new(f).ok())
                .collect::<IndexSet<_>>();

            let identity_veci = factors
                .clone()
                .into_iter()
                .filter_map(|f| IdentityVeci::new(f).ok())
                .collect::<IndexSet<_>>();

            let identity_mfa = factors
                .clone()
                .into_iter()
                .filter_map(|f| IdentityMfaFactorInstance::new(f).ok())
                .collect::<IndexSet<_>>();

            // assert_eq!(
            //     factors.len(),
            //     account_veci.len() +
            //         + account_mfa.len() +
            //         + identity_veci.len() +
            //         + identity_mfa.len(),
            //     "Discrepancy skipped some factors, ROLA?"
            // );

            let collections_for_factor = CollectionsOfFactorInstances::new(
                network_id,
                factor_source_id,
                account_veci,
                account_mfa,
                identity_veci,
                identity_mfa,
            )
            .unwrap();

            assert!(set_of_collections
                .insert(factor_source_id, collections_for_factor)
                .is_none());
        }
        Self(set_of_collections)
    }
}
