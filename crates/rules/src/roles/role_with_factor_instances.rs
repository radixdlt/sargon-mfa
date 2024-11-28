use crate::prelude::*;

pub(crate) type RoleWithFactorInstances = AbstractBuiltRoleWithFactor<FactorInstance>;

impl RoleWithFactorInstances {
    // TODO: MFA - Upgrade this method to follow the rules of when a factor instance might
    // be used by MULTIPLE roles. This is a temporary solution to get the tests to pass.
    // A proper solution should use follow the rules laid out in:
    // https://radixdlt.atlassian.net/wiki/spaces/AT/pages/3758063620/MFA+Rules+for+Factors+and+Security+Shields
    pub(crate) fn fulfilling_role_of_factor_sources_with_factor_instances(
        role_kind: RoleKind,
        consuming_instances: &IndexMap<FactorSourceIDFromHash, FactorInstances>,
        matrix_of_factor_sources: &MatrixOfFactorSources,
    ) -> Result<Self, CommonError> {
        let role_of_sources = {
            match role_kind {
                RoleKind::Primary => &matrix_of_factor_sources.primary_role,
                RoleKind::Recovery => &matrix_of_factor_sources.recovery_role,
                RoleKind::Confirmation => &matrix_of_factor_sources.confirmation_role,
            }
        };
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
            Self::with_factors(role_kind, threshold, threshold_factors, override_factors);

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

impl RoleWithFactorInstances {
    // TODO: MFA Rules change this, this might not be compatible with the rules!
    pub fn sample_primary() -> Self {
        Self::with_factors(RoleKind::Primary, 1, [
        HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_0_securified_at_index(0).into()
       ], [
        HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_10_securified_at_index(0).into()
       ])
    }

    // TODO: MFA Rules change this, this might not be compatible with the rules!
    pub fn sample_primary_other() -> Self {
        Self::with_factors(
            RoleKind::Primary,
            1,
            [HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_0_securified_at_index(10).into(),],
            [HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_10_securified_at_index(60).into()],
        )
    }

    // TODO: MFA Rules change this, this might not be compatible with the rules!
    pub fn sample_recovery() -> Self {
        Self::with_factors(
            RoleKind::Recovery,
            0,[], [HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_10_securified_at_index(237).into()]
        )
    }

    // TODO: MFA Rules change this, this might not be compatible with the rules!
    pub fn sample_recovery_other() -> Self {
        Self::with_factors(
            RoleKind::Recovery,
            0,[], [HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_10_securified_at_index(42).into()]
        )
    }

    // TODO: MFA Rules change this, this might not be compatible with the rules!
    pub fn sample_confirmation() -> Self {
        Self::with_factors(
            RoleKind::Confirmation,
            0,[], [HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_0_securified_at_index(1).into(), HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_10_securified_at_index(2).into()]
        )
    }

    // TODO: MFA Rules change this, this might not be compatible with the rules!
    pub fn sample_confirmation_other() -> Self {
        Self::with_factors(
            RoleKind::Confirmation,
            0,[], [HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_0_securified_at_index(10).into(), HierarchicalDeterministicFactorInstance::sample_mainnet_account_device_factor_fs_10_securified_at_index(20).into()]
        )
    }
}

impl HasSampleValues for RoleWithFactorInstances {
    fn sample() -> Self {
        Self::sample_primary()
    }

    fn sample_other() -> Self {
        Self::sample_recovery()
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
}
