use crate::prelude::*;

pub type MatrixOfFactorInstances = AbstractMatrixBuilderOrBuilt<FactorInstance, (), ()>;

impl HasFactorInstances for MatrixOfFactorInstances {
    fn unique_factor_instances(&self) -> IndexSet<FactorInstance> {
        let mut set = IndexSet::new();
        set.extend(self.primary_role.all_factors().into_iter().cloned());
        set.extend(self.recovery_role.all_factors().into_iter().cloned());
        set.extend(self.confirmation_role.all_factors().into_iter().cloned());
        set
    }
}

impl HasSampleValues for MatrixOfFactorInstances {
    fn sample() -> Self {
        Self {
            built: PhantomData,
            primary_role: RoleWithFactorInstances::sample_primary(),
            recovery_role: RoleWithFactorInstances::sample_recovery(),
            confirmation_role: RoleWithFactorInstances::sample_confirmation(),
            number_of_days_until_auto_confirm: 30,
        }
    }

    fn sample_other() -> Self {
        Self {
            built: PhantomData,
            primary_role: RoleWithFactorInstances::sample_primary_other(),
            recovery_role: RoleWithFactorInstances::sample_recovery_other(),
            confirmation_role: RoleWithFactorInstances::sample_confirmation_other(),
            number_of_days_until_auto_confirm: 15,
        }
    }
}

impl MatrixOfFactorInstances {
    /// Maps `MatrixOfFactorSources -> MatrixOfFactorInstances` by
    /// "assigning" FactorInstances to each MatrixOfFactorInstances from
    /// `consuming_instances`.
    ///
    /// NOTE:
    /// **One FactorInstance might be used multiple times in the MatrixOfFactorInstances,
    /// e.g. ones in the PrimaryRole(WithFactorInstances) and again in RecoveryRole(WithFactorInstances) or
    /// in RecoveryRole(WithFactorInstances)**.
    ///
    /// However, the same FactorInstance is NEVER used in two different MatrixOfFactorInstances.
    ///
    ///
    pub fn fulfilling_matrix_of_factor_sources_with_instances(
        consuming_instances: &mut IndexMap<FactorSourceIDFromHash, FactorInstances>,
        matrix_of_factor_sources: MatrixOfFactorSources,
    ) -> Result<Self, CommonError> {
        let instances = &consuming_instances.clone();

        let primary_role =
            RoleWithFactorInstances::fulfilling_role_of_factor_sources_with_factor_instances(
                RoleKind::Primary,
                instances,
                &matrix_of_factor_sources,
            )?;
        let recovery_role =
            RoleWithFactorInstances::fulfilling_role_of_factor_sources_with_factor_instances(
                RoleKind::Recovery,
                instances,
                &matrix_of_factor_sources,
            )?;
        let confirmation_role =
            RoleWithFactorInstances::fulfilling_role_of_factor_sources_with_factor_instances(
                RoleKind::Confirmation,
                instances,
                &matrix_of_factor_sources,
            )?;

        let matrix = Self {
            built: PhantomData,
            primary_role,
            recovery_role,
            confirmation_role,
            number_of_days_until_auto_confirm: matrix_of_factor_sources
                .number_of_days_until_auto_confirm,
        };

        // Now that we have assigned instances, **possibly the SAME INSTANCE to multiple roles**,
        // lets delete them from the `consuming_instances` map.
        for instance in matrix.all_factors() {
            let fsid = &FactorSourceIDFromHash::try_from(instance.factor_source_id).unwrap();
            let existing = consuming_instances.get_mut(fsid).unwrap();

            let to_remove =
                HierarchicalDeterministicFactorInstance::try_from(instance.clone()).unwrap();

            // We remove at the beginning of the list first.
            existing.shift_remove(&to_remove);

            if existing.is_empty() {
                // not needed per se, but feels prudent to "prune".
                consuming_instances.shift_remove_entry(fsid);
            }
        }

        Ok(matrix)
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[allow(clippy::upper_case_acronyms)]
    type SUT = MatrixOfFactorInstances;

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
    fn err_if_no_instance_found_for_factor_source() {
        assert!(matches!(
            SUT::fulfilling_matrix_of_factor_sources_with_instances(
                &mut IndexMap::new(),
                MatrixOfFactorSources::sample()
            ),
            Err(CommonError::MissingFactorMappingInstancesIntoRole)
        ));
    }

    #[test]
    fn err_if_empty_instance_found_for_factor_source() {
        assert!(matches!(
            SUT::fulfilling_matrix_of_factor_sources_with_instances(
                &mut IndexMap::kv(
                    FactorSource::sample_device_babylon().id_from_hash(),
                    FactorInstances::from_iter([])
                ),
                MatrixOfFactorSources::sample()
            ),
            Err(CommonError::MissingFactorMappingInstancesIntoRole)
        ));
    }
}
