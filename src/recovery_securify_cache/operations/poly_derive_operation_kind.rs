use sha2::digest::crypto_common::Key;

use crate::prelude::*;

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

    /// Securify unsecurified Account
    SecurifyUnsecurifiedAccount {
        unsecurified_account: UnsecurifiedEntity,
        matrix_of_factor_sources: MatrixOfFactorSources,
    },

    /// Securify unsecurified Account
    UpdateSecurifiedAccount {
        securified_account: SecurifiedEntity,
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
    pub fn requests(&self) -> AnyFactorDerivationRequests {
        match self {
            Self::OARS { .. } => Self::requests_for_account_recover_scan(NetworkID::Mainnet),
            Self::MARS { network_id, .. } => Self::requests_for_account_recover_scan(*network_id),
            Self::NewVirtualUnsecurifiedAccount { network_id, .. } => {
                Self::requests_for_tx_for_account(*network_id, [KeySpace::Unsecurified])
            }
            Self::PreDeriveInstancesForNewFactorSource { .. } => Self::fill_cache_mainnet(),
            Self::SecurifyUnsecurifiedAccount {
                unsecurified_account,
                ..
            } => Self::requests_for_tx_for_account(
                unsecurified_account.network_id(),
                [KeySpace::Securified],
            ),
            Self::UpdateSecurifiedAccount {
                securified_account, ..
            } => Self::requests_for_tx_for_account(
                securified_account.network_id(),
                [KeySpace::Securified],
            ),
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
            Self::SecurifyUnsecurifiedAccount {
                matrix_of_factor_sources,
                ..
            } => matrix_of_factor_sources
                .all_factors()
                .into_iter()
                .collect::<FactorSources>(),
            Self::UpdateSecurifiedAccount {
                matrix_of_factor_sources,
                ..
            } => matrix_of_factor_sources
                .all_factors()
                .into_iter()
                .collect::<FactorSources>(),
        }
    }
}
