use crate::prelude::*;

/// An analyzer of a `Profile` for some `network_id` (i.e. analyzer of `ProfileNetwork`),
/// reading out the max derivation entity index for Unsecurified/Securified Accounts/Personas
/// for some factor source id.
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
    /// `Profile` is optional so that one can use the same initializer from `FactorInstancesProvider`,
    /// which accepts an optional Profile. Will just default to empty lists if `None` is passed,
    /// effectively making this whole assigner NOOP.
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

    /// Returns the Max Derivation Entity Index of Unsecurified Accounts controlled
    /// by `factor_source_id`, or `None` if no unsecurified account controlled by that
    /// factor source id found.
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

    /// Returns the Max Derivation Entity Index of Unsecurified Personas controlled
    /// by `factor_source_id`, or `None` if no unsecurified persona controlled by that
    /// factor source id found.
    fn max_identity_veci(
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

    /// Returns the Max Derivation Entity Index of Securified Accounts controlled
    /// by `factor_source_id`, or `None` if no securified account controlled by that
    /// factor source id found, by controlled by we mean having a MatrixOfFactorInstances
    /// which has that factor in **any role** in its MatrixOfFactorInstances.
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

    /// Returns the Max Derivation Entity Index of Securified Persona controlled
    /// by `factor_source_id`, or `None` if no securified persona controlled by that
    /// factor source id found, by controlled by we mean having a MatrixOfFactorInstances
    /// which has that factor in **any role** in its MatrixOfFactorInstances.
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

    /// Finds the "next" derivation entity index `HDPathComponent`, for
    /// the `IndexAgnosticPath` for `factor_source_id`, which is `Max + 1`, or
    /// returns `None` if `Max` is `None`. See `max_account_veci`, `max_identity_mfa`
    /// for more details.
    pub fn next(
        &self,
        agnostic_path: IndexAgnosticPath,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Result<Option<HDPathComponent>> {
        if agnostic_path.network_id != self.network_id {
            return Err(CommonError::NetworkDiscrepancy);
        }
        let derivation_preset = DerivationPreset::try_from(agnostic_path)?;

        let last = match derivation_preset {
            DerivationPreset::AccountVeci => self.max_account_veci(factor_source_id),
            DerivationPreset::AccountMfa => self.max_account_mfa(factor_source_id),
            DerivationPreset::IdentityVeci => self.max_identity_veci(factor_source_id),
            DerivationPreset::IdentityMfa => self.max_identity_mfa(factor_source_id),
        };

        let Some(last) = last else { return Ok(None) };
        last.add_one().map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = NextDerivationEntityIndexProfileAnalyzingAssigner;

    #[test]
    fn test_network_discrepancy() {
        let sut = Sut::new(NetworkID::Mainnet, None);
        assert_eq!(
            sut.next(
                DerivationPreset::AccountVeci.index_agnostic_path_on_network(NetworkID::Stokenet),
                FactorSourceIDFromHash::fs0()
            ),
            Err(CommonError::NetworkDiscrepancy)
        );
    }

    #[test]
    fn test_next_account_veci_with_single_at_0_is_1() {
        let preset = DerivationPreset::AccountVeci;
        let network_id = NetworkID::Mainnet;
        let sut = Sut::new(
            network_id,
            Some(Profile::new(HDFactorSource::all(), [&Account::a0()], [])),
        );
        let next = sut
            .next(
                preset.index_agnostic_path_on_network(network_id),
                FactorSourceIDFromHash::fs0(),
            )
            .unwrap();

        assert_eq!(
            next,
            Some(HDPathComponent::unsecurified_hardening_base_index(1))
        )
    }

    #[test]
    fn test_next_account_veci_with_unused_factor_is_none() {
        let preset = DerivationPreset::AccountVeci;
        let network_id = NetworkID::Mainnet;
        let sut = Sut::new(
            network_id,
            Some(Profile::new(HDFactorSource::all(), [&Account::a0()], [])),
        );
        let next = sut
            .next(
                preset.index_agnostic_path_on_network(network_id),
                FactorSourceIDFromHash::fs1(), // <-- UNUSED
            )
            .unwrap();

        assert_eq!(next, None)
    }

    #[test]
    fn test_next_account_mfa_with_single_unsecurified_is_none() {
        let preset = DerivationPreset::AccountMfa;
        let network_id = NetworkID::Mainnet;
        let sut = Sut::new(
            network_id,
            Some(Profile::new(HDFactorSource::all(), [&Account::a0()], [])),
        );
        let next = sut
            .next(
                preset.index_agnostic_path_on_network(network_id),
                FactorSourceIDFromHash::fs0(),
            )
            .unwrap();

        assert_eq!(next, None)
    }

    #[test]
    fn test_next_account_veci_with_single_at_8_is_9() {
        let preset = DerivationPreset::AccountVeci;
        let network_id = NetworkID::Mainnet;
        let sut = Sut::new(
            network_id,
            Some(Profile::new(
                HDFactorSource::all(),
                [
                    &Account::a8(),
                    &Account::a2(), /* securified, should not interfere */
                ],
                [],
            )),
        );
        let next = sut
            .next(
                preset.index_agnostic_path_on_network(network_id),
                FactorSourceIDFromHash::fs10(),
            )
            .unwrap();

        assert_eq!(
            next,
            Some(HDPathComponent::unsecurified_hardening_base_index(9))
        )
    }

    #[test]
    fn test_next_account_mfa_with_single_at_7_is_8() {
        let preset = DerivationPreset::AccountMfa;
        let network_id = NetworkID::Mainnet;
        let sut = Sut::new(
            network_id,
            Some(Profile::new(
                HDFactorSource::all(),
                [
                    &Account::a8(), /* unsecurified, should not interfere */
                    &Account::a7(),
                ],
                [],
            )),
        );
        type F = FactorSourceIDFromHash;
        for fid in [F::fs2(), F::fs6(), F::fs7(), F::fs8(), F::fs9()] {
            let next = sut
                .next(preset.index_agnostic_path_on_network(network_id), fid)
                .unwrap();

            assert_eq!(next, Some(HDPathComponent::securifying_base_index(8)))
        }
    }

    #[test]
    fn test_next_identity_mfa_with_single_at_7_is_8() {
        let preset = DerivationPreset::IdentityMfa;
        let network_id = NetworkID::Mainnet;
        let sut = Sut::new(
            network_id,
            Some(Profile::new(HDFactorSource::all(), [], [&Persona::p7()])),
        );
        type F = FactorSourceIDFromHash;
        for fid in [F::fs2(), F::fs6(), F::fs7(), F::fs8(), F::fs9()] {
            let next = sut
                .next(preset.index_agnostic_path_on_network(network_id), fid)
                .unwrap();

            assert_eq!(next, Some(HDPathComponent::securifying_base_index(8)))
        }
    }

    #[test]
    fn test_next_identity_veci_with_single_at_1_is_2() {
        let preset = DerivationPreset::IdentityVeci;
        let network_id = NetworkID::Mainnet;
        let sut = Sut::new(
            network_id,
            Some(Profile::new(
                HDFactorSource::all(),
                [],
                [
                    &Persona::p7(), /* securified should not interfere */
                    &Persona::p1(),
                ],
            )),
        );
        let next = sut
            .next(
                preset.index_agnostic_path_on_network(network_id),
                FactorSourceIDFromHash::fs1(),
            )
            .unwrap();

        assert_eq!(
            next,
            Some(HDPathComponent::unsecurified_hardening_base_index(2))
        )
    }

    #[test]
    fn test_next_account_veci_with_non_contiguous_at_0_1_7_is_8() {
        let fsid = FactorSourceIDFromHash::fs0();

        let fi0 = HierarchicalDeterministicFactorInstance::mainnet_tx(
            CAP26EntityKind::Account,
            HDPathComponent::unsecurified_hardening_base_index(0),
            fsid,
        );
        let fi1 = HierarchicalDeterministicFactorInstance::mainnet_tx(
            CAP26EntityKind::Account,
            HDPathComponent::unsecurified_hardening_base_index(1),
            fsid,
        );

        let fi7 = HierarchicalDeterministicFactorInstance::mainnet_tx(
            CAP26EntityKind::Account,
            HDPathComponent::unsecurified_hardening_base_index(7),
            fsid,
        );

        let network_id = NetworkID::Mainnet;
        let accounts = [fi0, fi1, fi7].map(|fi| {
            Account::new(
                "acco",
                AccountAddress::new(network_id, fi.public_key_hash()),
                EntitySecurityState::Unsecured(fi),
                ThirdPartyDepositPreference::default(),
            )
        });
        let sut = Sut::new(
            network_id,
            Some(Profile::new(HDFactorSource::all(), &accounts, [])),
        );
        let next = sut
            .next(
                DerivationPreset::AccountVeci.index_agnostic_path_on_network(network_id),
                fsid,
            )
            .unwrap();

        assert_eq!(
            next,
            Some(HDPathComponent::unsecurified_hardening_base_index(8))
        )
    }
}
