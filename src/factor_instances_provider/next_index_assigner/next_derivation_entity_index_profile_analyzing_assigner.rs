use crate::prelude::*;

pub struct NextDerivationEntityIndexProfileAnalyzingAssigner {
    network_id: NetworkID,

    /// might be empty
    unsecurified_accounts_on_network: IndexSet<UnsecurifiedEntity>,

    /// might be empty
    securified_accounts_on_network: IndexSet<SecurifiedAccount>,

    /// might be empty
    unsecurified_identities_on_network: IndexSet<UnsecurifiedEntity>,

    /// might be empty
    securified_identities_on_network: IndexSet<SecurifiedPersona>,
}

impl NextDerivationEntityIndexProfileAnalyzingAssigner {
    pub fn new(network_id: NetworkID, profile: Option<Profile>) -> Self {
        let unsecurified_accounts_on_network = profile
            .as_ref()
            .map(|p| p.unsecurified_accounts_on_network(network_id))
            .unwrap_or_default();

        let securified_accounts_on_network = profile
            .as_ref()
            .map(|p| p.securified_accounts_on_network(network_id))
            .unwrap_or_default();

        let unsecurified_identities_on_network = profile
            .as_ref()
            .map(|p| p.unsecurified_identities_on_network(network_id))
            .unwrap_or_default();

        let securified_identities_on_network = profile
            .as_ref()
            .map(|p| p.securified_identities_on_network(network_id))
            .unwrap_or_default();

        Self {
            network_id,
            unsecurified_accounts_on_network,
            securified_accounts_on_network,
            unsecurified_identities_on_network,
            securified_identities_on_network,
        }
    }

    fn max_account_veci(
        &self,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Option<HDPathComponent> {
        self.unsecurified_accounts_on_network
            .clone()
            .into_iter()
            .map(|x: UnsecurifiedEntity| x.veci().factor_instance())
            .filter(|f| f.factor_source_id == factor_source_id)
            .map(|f| f.derivation_path())
            .map(|p| {
                AssertMatches {
                    network_id: self.network_id,
                    key_kind: CAP26KeyKind::TransactionSigning,
                    entity_kind: CAP26EntityKind::Account,
                    key_space: KeySpace::Unsecurified,
                }
                .matches(&p)
            })
            .map(|fi| fi.index)
            .max()
    }

    pub fn max_identity_veci(
        &self,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Option<HDPathComponent> {
        self.unsecurified_identities_on_network
            .clone()
            .into_iter()
            .map(|x: UnsecurifiedEntity| x.veci().factor_instance())
            .filter(|f| f.factor_source_id == factor_source_id)
            .map(|f| f.derivation_path())
            .map(|p| {
                AssertMatches {
                    network_id: self.network_id,
                    key_kind: CAP26KeyKind::TransactionSigning,
                    entity_kind: CAP26EntityKind::Identity,
                    key_space: KeySpace::Unsecurified,
                }
                .matches(&p)
            })
            .map(|fi| fi.index)
            .max()
    }

    fn max_account_mfa(&self, factor_source_id: FactorSourceIDFromHash) -> Option<HDPathComponent> {
        self.securified_accounts_on_network
            .clone()
            .into_iter()
            .flat_map(|e: SecurifiedAccount| {
                e.highest_derivation_path_index(
                    factor_source_id,
                    AssertMatches {
                        network_id: self.network_id,
                        key_kind: CAP26KeyKind::TransactionSigning,
                        entity_kind: CAP26EntityKind::Account,
                        key_space: KeySpace::Securified,
                    },
                )
            })
            .max()
    }

    fn max_identity_mfa(
        &self,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Option<HDPathComponent> {
        self.securified_identities_on_network
            .clone()
            .into_iter()
            .flat_map(|e: SecurifiedPersona| {
                e.highest_derivation_path_index(
                    factor_source_id,
                    AssertMatches {
                        network_id: self.network_id,
                        key_kind: CAP26KeyKind::TransactionSigning,
                        entity_kind: CAP26EntityKind::Identity,
                        key_space: KeySpace::Securified,
                    },
                )
            })
            .max()
    }

    pub fn next(
        &self,
        agnostic_path: DerivationPreset,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Result<Option<HDPathComponent>> {
        let last = if agnostic_path == DerivationPreset::AccountVeci {
            self.max_account_veci(factor_source_id)
        } else if agnostic_path == DerivationPreset::AccountMfa {
            self.max_account_mfa(factor_source_id)
        } else if agnostic_path == DerivationPreset::IdentityMfa {
            self.max_identity_mfa(factor_source_id)
        } else if agnostic_path == DerivationPreset::IdentityVeci {
            self.max_identity_veci(factor_source_id)
        } else {
            unreachable!("Unrecognized agnostic_path: {:?}", agnostic_path);
        };
        let Some(last) = last else { return Ok(None) };
        last.add_one().map(Some)
    }
}
