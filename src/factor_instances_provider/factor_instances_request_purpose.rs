use sha2::digest::crypto_common::Key;

use crate::prelude::*;

/// A NonEmpty collection of Accounts all on the SAME Network
/// but mixed if they are securified or unsecurified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Accounts {
    pub network_id: NetworkID,
    accounts: IndexSet<Account>,
}
impl IntoIterator for Accounts {
    type Item = Account;
    type IntoIter = <IndexSet<Account> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.accounts.clone().into_iter()
    }
}
impl Accounts {
    pub fn len(&self) -> usize {
        self.accounts.len()
    }
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }
    pub fn network_id(&self) -> NetworkID {
        self.network_id
    }
}

/// A NonEmpty collection of Accounts all on the SAME Network and all verified
/// to be unsecurified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsecurifiedAccounts {
    pub network_id: NetworkID,
    accounts: IndexSet<UnsecurifiedAccount>,
}
impl UnsecurifiedAccounts {
    pub fn len(&self) -> usize {
        self.accounts.len()
    }
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }
    pub fn network_id(&self) -> NetworkID {
        self.network_id
    }
}
impl From<UnsecurifiedAccounts> for Accounts {
    fn from(_value: UnsecurifiedAccounts) -> Self {
        todo!()
    }
}
impl From<SecurifiedAccounts> for Accounts {
    fn from(_value: SecurifiedAccounts) -> Self {
        todo!()
    }
}

/// A NonEmpty collection of Accounts all on the SAME Network and all verified
/// to be unsecurified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecurifiedAccounts {
    network_id: NetworkID,
    accounts: IndexSet<UnsecurifiedAccount>,
}

impl SecurifiedAccounts {
    pub fn network_id(&self) -> NetworkID {
        self.network_id
    }
    pub fn len(&self) -> usize {
        self.accounts.len()
    }
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FactorInstancesRequestPurpose {
    /// Onboarding Account Recovery Scan
    /// Assumes `Mainnet`
    OARS { factor_sources: FactorSources },

    /// Manual Account Recovery Scan
    /// Done using a single FactorSource
    MARS {
        factor_source: HDFactorSource,
        network_id: NetworkID,
    },

    /// PreDerive FactorInstances for new FactorSource
    PreDeriveInstancesForNewFactorSource { factor_source: HDFactorSource },

    /// New Virtual Unsecurified Account
    NewVirtualUnsecurifiedAccount {
        network_id: NetworkID,
        factor_source: HDFactorSource,
    },

    /// Securify unsecurified Accounts
    UpdateOrSetSecurityShieldForAccounts {
        accounts: Accounts,
        matrix_of_factor_sources: MatrixOfFactorSources,
    },
}
impl FactorInstancesRequestPurpose {
    fn requests_for_entity_key_kind(
        entity_kind: CAP26EntityKind,
        key_kind: CAP26KeyKind,
        network: NetworkID,
        key_spaces: impl IntoIterator<Item = KeySpace>,
    ) -> AnyFactorDerivationRequests {
        key_spaces
            .into_iter()
            .map(|key_space| {
                AnyFactorDerivationRequest::new(network, entity_kind, key_space, key_kind)
            })
            .collect::<AnyFactorDerivationRequests>()
    }

    fn quantity(&self) -> DerivationRequestQuantitySelector {
        match self {
            Self::OARS { .. } => DerivationRequestQuantitySelector::fill_cache_if_needed(),
            Self::MARS { .. } => DerivationRequestQuantitySelector::fill_cache_if_needed(),
            Self::NewVirtualUnsecurifiedAccount { .. } => DerivationRequestQuantitySelector::Mono,
            Self::PreDeriveInstancesForNewFactorSource { .. } => {
                DerivationRequestQuantitySelector::fill_cache_if_needed()
            }
            Self::UpdateOrSetSecurityShieldForAccounts { accounts, .. } => {
                DerivationRequestQuantitySelector::Poly {
                    count: accounts.len(),
                }
            }
        }
    }

    pub fn _requests_with_quantity(
        &self,
        quantity: DerivationRequestQuantitySelector,
    ) -> QuantifiedUnindexDerivationRequests {
        let factor_sources = self.factor_sources();

        // Form requests untied to any FactorSources
        let unfactored = self.unfactored_requests();

        // Form requests tied to FactorSources, but without indices, unquantified
        let unquantified = unfactored.for_each_factor_source(factor_sources);

        

        unquantified
            .into_iter()
            .map(|x| QuantifiedUnindexDerivationRequest::quantifying(x, quantity))
            .collect::<QuantifiedUnindexDerivationRequests>()
    }

    pub fn requests(&self) -> QuantifiedUnindexDerivationRequests {
        self._requests_with_quantity(self.quantity())
    }

    fn requests_for_account(
        network: NetworkID,
        key_kind: CAP26KeyKind,
        key_spaces: impl IntoIterator<Item = KeySpace>,
    ) -> AnyFactorDerivationRequests {
        Self::requests_for_entity_key_kind(CAP26EntityKind::Account, key_kind, network, key_spaces)
    }

    fn requests_for_tx_for_account(
        network: NetworkID,
        key_spaces: impl IntoIterator<Item = KeySpace>,
    ) -> AnyFactorDerivationRequests {
        Self::requests_for_account(network, CAP26KeyKind::TransactionSigning, key_spaces)
    }

    fn requests_for_account_recover_scan(network: NetworkID) -> AnyFactorDerivationRequests {
        Self::requests_for_tx_for_account(network, KeySpace::both())
    }

    fn fill_cache_mainnet() -> AnyFactorDerivationRequests {
        let network = NetworkID::Mainnet;

        let accounts_tx = Self::requests_for_entity_key_kind(
            CAP26EntityKind::Account,
            CAP26KeyKind::TransactionSigning,
            network,
            KeySpace::both(),
        );

        let personas_tx = Self::requests_for_entity_key_kind(
            CAP26EntityKind::Identity,
            CAP26KeyKind::TransactionSigning,
            network,
            KeySpace::both(),
        );

        let rola_key_spaces = KeySpace::both(); // Is this correct? Do we in fact ROLA keys in unsecurified KeySpace?
        let accounts_rola = Self::requests_for_entity_key_kind(
            CAP26EntityKind::Account,
            CAP26KeyKind::AuthenticationSigning,
            network,
            rola_key_spaces,
        );

        let personas_rola = Self::requests_for_entity_key_kind(
            CAP26EntityKind::Identity,
            CAP26KeyKind::TransactionSigning,
            network,
            rola_key_spaces,
        );

        let mut requests = AnyFactorDerivationRequests::default();
        requests.merge(accounts_tx);
        requests.merge(personas_tx);
        requests.merge(accounts_rola);
        requests.merge(personas_rola);

        requests
    }

    /// N.B. if cache is empty we will not only derive to satisfy these requests,
    /// we will derive ALL possible factor instances to fill the cache.
    fn unfactored_requests(&self) -> AnyFactorDerivationRequests {
        match self {
            Self::OARS { .. } => Self::requests_for_account_recover_scan(NetworkID::Mainnet),
            Self::MARS { network_id, .. } => Self::requests_for_account_recover_scan(*network_id),
            Self::NewVirtualUnsecurifiedAccount { network_id, .. } => {
                Self::requests_for_tx_for_account(*network_id, [KeySpace::Unsecurified])
            }
            Self::PreDeriveInstancesForNewFactorSource { .. } => Self::fill_cache_mainnet(),

            Self::UpdateOrSetSecurityShieldForAccounts { accounts, .. } => {
                Self::requests_for_tx_for_account(accounts.network_id(), [KeySpace::Securified])
            }
        }
    }

    pub fn factor_sources(&self) -> FactorSources {
        match self {
            Self::OARS { factor_sources } => factor_sources.clone(),
            Self::MARS { factor_source, .. } => FactorSources::just(factor_source.clone()),
            Self::PreDeriveInstancesForNewFactorSource { factor_source } => {
                FactorSources::just(factor_source.clone())
            }
            Self::NewVirtualUnsecurifiedAccount { factor_source, .. } => {
                FactorSources::just(factor_source.clone())
            }
            Self::UpdateOrSetSecurityShieldForAccounts {
                matrix_of_factor_sources,
                ..
            } => matrix_of_factor_sources
                .all_factors()
                .into_iter()
                .collect::<FactorSources>(),
        }
    }
}
