use crate::prelude::*;

pub(crate) type RoleWithFactorInstances<const R: u8> =
    AbstractBuiltRoleWithFactor<R, FactorInstance>;

impl<const R: u8> RoleWithFactorSources<R> {
    fn from<const F: u8>(other: &RoleWithFactorSources<F>) -> Self {
        Self::with_factors(
            other.threshold(),
            other.get_threshold_factors().clone(),
            other.get_override_factors().clone(),
        )
    }
}

impl MatrixOfFactorSources {
    pub(crate) fn get_role<const R: u8>(&self) -> RoleWithFactorSources<R> {
        match R {
            ROLE_PRIMARY => RoleWithFactorSources::from(&self.primary_role),
            ROLE_RECOVERY => RoleWithFactorSources::from(&self.recovery_role),
            ROLE_CONFIRMATION => RoleWithFactorSources::from(&self.confirmation_role),
            _ => panic!("unknown"),
        }
    }
}

impl<const R: u8> RoleWithFactorInstances<R> {
    // TODO: MFA - Upgrade this method to follow the rules of when a factor instance might
    // be used by MULTIPLE roles. This is a temporary solution to get the tests to pass.
    // A proper solution should use follow the rules laid out in:
    // https://radixdlt.atlassian.net/wiki/spaces/AT/pages/3758063620/MFA+Rules+for+Factors+and+Security+Shields
    pub(crate) fn fulfilling_role_of_factor_sources_with_factor_instances(
        consuming_instances: &IndexMap<FactorSourceIDFromHash, FactorInstances>,
        matrix_of_factor_sources: &MatrixOfFactorSources,
    ) -> Result<Self, CommonError> {
        let role_kind = RoleKind::from_u8(R).unwrap();

        let role_of_sources = matrix_of_factor_sources.get_role::<R>();
        assert_eq!(role_of_sources.role(), role_kind);
        let threshold: u8 = role_of_sources.get_threshold();

        // Threshold factors
        let threshold_factors =
            Self::try_filling_factor_list_of_role_of_factor_sources_with_factor_instances(
                consuming_instances,
                role_of_sources.get_threshold_factors(),
            )?;

        // Override factors
        let override_factors =
            Self::try_filling_factor_list_of_role_of_factor_sources_with_factor_instances(
                consuming_instances,
                role_of_sources.get_override_factors(),
            )?;

        let role_with_instances =
            Self::with_factors(threshold, threshold_factors, override_factors);

        assert_eq!(role_with_instances.role(), role_kind);
        Ok(role_with_instances)
    }

    fn try_filling_factor_list_of_role_of_factor_sources_with_factor_instances(
        instances: &IndexMap<FactorSourceIDFromHash, FactorInstances>,
        from: &[FactorSource],
    ) -> Result<Vec<FactorInstance>, CommonError> {
        from.iter()
            .map(|f| {
                if let Some(existing) = instances.get(&f.id_from_hash()) {
                    let hd_instance = existing
                        .first()
                        .ok_or(CommonError::MissingFactorMappingInstancesIntoRole)?;
                    let instance = FactorInstance::from(hd_instance);
                    Ok(instance)
                } else {
                    Err(CommonError::MissingFactorMappingInstancesIntoRole)
                }
            })
            .collect::<Result<Vec<FactorInstance>, CommonError>>()
    }
}

pub(crate) type PrimaryRoleWithFactorInstances = RoleWithFactorInstances<{ ROLE_PRIMARY }>;
pub(crate) type RecoveryRoleWithFactorInstances = RoleWithFactorInstances<{ ROLE_RECOVERY }>;
pub(crate) type ConfirmationRoleWithFactorInstances =
    RoleWithFactorInstances<{ ROLE_CONFIRMATION }>;

impl HasSampleValues for PrimaryRoleWithFactorInstances {
    fn sample() -> Self {
        MatrixOfFactorInstances::sample().primary_role
    }

    fn sample_other() -> Self {
        MatrixOfFactorInstances::sample_other().primary_role
    }
}

impl HasSampleValues for ConfirmationRoleWithFactorInstances {
    fn sample() -> Self {
        MatrixOfFactorInstances::sample().confirmation_role
    }

    fn sample_other() -> Self {
        MatrixOfFactorInstances::sample_other().confirmation_role
    }
}

impl HasSampleValues for RecoveryRoleWithFactorInstances {
    fn sample() -> Self {
        MatrixOfFactorInstances::sample().recovery_role
    }

    fn sample_other() -> Self {
        MatrixOfFactorInstances::sample_other().recovery_role
    }
}

#[cfg(test)]
mod primary_tests {
    use super::*;

    #[allow(clippy::upper_case_acronyms)]
    type SUT = PrimaryRoleWithFactorInstances;

    #[test]
    fn equality() {
        assert_eq!(SUT::sample(), SUT::sample());
        assert_eq!(SUT::sample_other(), SUT::sample_other());
    }

    #[test]
    fn inequality() {
        assert_ne!(SUT::sample(), SUT::sample_other());
    }

    #[test]
    #[should_panic]
    fn primary_role_non_securified_threshold_instances_is_err() {
        let _ = SUT::with_factors(
                1,
                [
                    HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_10_unsecurified_at_index(0).into()
                ],
                []
            );
    }

    #[test]
    fn assert_json_sample() {
        let sut = SUT::sample();
        assert_eq_after_json_roundtrip(
            &sut,
            r#"
                        {
              "threshold": 2,
              "thresholdFactors": [
                {
                  "factorSourceID": {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "device",
                      "body": "f1a93d324dd0f2bff89963ab81ed6e0c2ee7e18c0827dc1d3576b2d9f26bbd0a"
                    }
                  },
                  "badge": {
                    "discriminator": "virtualSource",
                    "virtualSource": {
                      "discriminator": "hierarchicalDeterministicPublicKey",
                      "hierarchicalDeterministicPublicKey": {
                        "publicKey": {
                          "curve": "curve25519",
                          "compressedData": "427969814e15d74c3ff4d9971465cb709d210c8a7627af9466bdaa67bd0929b7"
                        },
                        "derivationPath": {
                          "scheme": "cap26",
                          "path": "m/44H/1022H/1H/525H/1460H/0S"
                        }
                      }
                    }
                  }
                },
                {
                  "factorSourceID": {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "ledgerHQHardwareWallet",
                      "body": "ab59987eedd181fe98e512c1ba0f5ff059f11b5c7c56f15614dcc9fe03fec58b"
                    }
                  },
                  "badge": {
                    "discriminator": "virtualSource",
                    "virtualSource": {
                      "discriminator": "hierarchicalDeterministicPublicKey",
                      "hierarchicalDeterministicPublicKey": {
                        "publicKey": {
                          "curve": "curve25519",
                          "compressedData": "92cd6838cd4e7b0523ed93d498e093f71139ffd5d632578189b39a26005be56b"
                        },
                        "derivationPath": {
                          "scheme": "cap26",
                          "path": "m/44H/1022H/1H/525H/1460H/0S"
                        }
                      }
                    }
                  }
                }
              ],
              "overrideFactors": []
            }
            "#,
        );
    }
}

#[cfg(test)]
mod confirmation_tests {
    use super::*;

    #[allow(clippy::upper_case_acronyms)]
    type SUT = ConfirmationRoleWithFactorInstances;

    #[test]
    fn equality() {
        assert_eq!(SUT::sample(), SUT::sample());
        assert_eq!(SUT::sample_other(), SUT::sample_other());
    }

    #[test]
    fn inequality() {
        assert_ne!(SUT::sample(), SUT::sample_other());
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;

    #[allow(clippy::upper_case_acronyms)]
    type SUT = RecoveryRoleWithFactorInstances;

    #[test]
    fn equality() {
        assert_eq!(SUT::sample(), SUT::sample());
        assert_eq!(SUT::sample_other(), SUT::sample_other());
    }

    #[test]
    fn inequality() {
        assert_ne!(SUT::sample(), SUT::sample_other());
    }
}
