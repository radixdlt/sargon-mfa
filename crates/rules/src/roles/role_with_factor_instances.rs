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

impl PrimaryRoleWithFactorInstances {
    // TODO: MFA Rules change this, this might not be compatible with the rules!
    pub fn sample_primary() -> Self {
        Self::with_factors(
            // RoleKind::Primary,
             1, [
        HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_0_securified_at_index(0).into()
       ], [
        HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_10_securified_at_index(0).into()
       ])
    }

    // TODO: MFA Rules change this, this might not be compatible with the rules!
    pub fn sample_primary_other() -> Self {
        Self::with_factors(
            // RoleKind::Primary,
            1,
            [HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_0_securified_at_index(10).into(),],
            [HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_10_securified_at_index(60).into()],
        )
    }
}

impl RecoveryRoleWithFactorInstances {
    // TODO: MFA Rules change this, this might not be compatible with the rules!
    pub fn sample_recovery() -> Self {
        Self::with_factors(
            // RoleKind::Recovery,
            0,[], [HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_10_securified_at_index(237).into()]
        )
    }

    // TODO: MFA Rules change this, this might not be compatible with the rules!
    pub fn sample_recovery_other() -> Self {
        Self::with_factors(
            // RoleKind::Recovery,
            0,[], [HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_10_securified_at_index(42).into()]
        )
    }
}

impl ConfirmationRoleWithFactorInstances {
    // TODO: MFA Rules change this, this might not be compatible with the rules!
    pub fn sample_confirmation() -> Self {
        Self::with_factors(
            // RoleKind::Confirmation,
            0,[], [HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_0_securified_at_index(1).into(), HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_10_securified_at_index(2).into()]
        )
    }

    // TODO: MFA Rules change this, this might not be compatible with the rules!
    pub fn sample_confirmation_other() -> Self {
        Self::with_factors(
            // RoleKind::Confirmation,
            0,[], [HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_0_securified_at_index(10).into(), HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_10_securified_at_index(20).into()]
        )
    }
}
/*

impl HasSampleValues for PrimaryRoleWithFactorInstances {
    fn sample() -> Self {
        Self::sample_primary()
    }

    fn sample_other() -> Self {
        Self::sample_primary_other()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::upper_case_acronyms)]
    type SUT = RoleWithFactorInstances;

    #[test]
    fn equality() {
        assert_eq!(SUT::sample_primary(), SUT::sample_primary());
        assert_eq!(SUT::sample_primary_other(), SUT::sample_primary_other());
        assert_eq!(SUT::sample_recovery(), SUT::sample_recovery());
        assert_eq!(SUT::sample_recovery_other(), SUT::sample_recovery_other());
        assert_eq!(SUT::sample_confirmation(), SUT::sample_confirmation());
        assert_eq!(
            SUT::sample_confirmation_other(),
            SUT::sample_confirmation_other()
        );
    }

    #[test]
    fn inequality() {
        assert_ne!(SUT::sample(), SUT::sample_other());
    }

    #[test]
    fn hash() {
        let hash = HashSet::<SUT>::from_iter([
            SUT::sample_primary(),
            SUT::sample_primary_other(),
            SUT::sample_recovery(),
            SUT::sample_recovery_other(),
            SUT::sample_confirmation(),
            SUT::sample_confirmation_other(),
            // Duplicates should be removed
            SUT::sample_primary(),
            SUT::sample_primary_other(),
            SUT::sample_recovery(),
            SUT::sample_recovery_other(),
            SUT::sample_confirmation(),
            SUT::sample_confirmation_other(),
        ]);
        assert_eq!(hash.len(), 6);
    }

    #[test]
    #[should_panic]
    fn primary_role_non_securified_threshold_instances_is_err() {
        let _ = SUT::with_factors(
                RoleKind::Primary,
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
              "role": "primary",
              "threshold": 1,
              "threshold_factors": [
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
                }
              ],
              "override_factors": [
                {
                  "factorSourceID": {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "device",
                      "body": "5255999c65076ce9ced5a1881f1a621bba1ce3f1f68a61df462d96822a5190cd"
                    }
                  },
                  "badge": {
                    "discriminator": "virtualSource",
                    "virtualSource": {
                      "discriminator": "hierarchicalDeterministicPublicKey",
                      "hierarchicalDeterministicPublicKey": {
                        "publicKey": {
                          "curve": "curve25519",
                          "compressedData": "e0293d4979bc303ea4fe361a62baf9c060c7d90267972b05c61eead9ef3eed3e"
                        },
                        "derivationPath": {
                          "scheme": "cap26",
                          "path": "m/44H/1022H/1H/525H/1460H/0S"
                        }
                      }
                    }
                  }
                }
              ]
            }
            "#,
        );
    }
}
*/
