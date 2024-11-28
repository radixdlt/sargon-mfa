use crate::prelude::*;

pub type MatrixWithFactorSources = AbstractMatrixBuilderOrBuilt<FactorSource, (), ()>;

impl MatrixWithFactorSources {
    pub fn new(
        matrix_of_factor_source_ids: MatrixWithFactorSourceIds,
        factor_sources: &FactorSources,
    ) -> Result<Self, CommonError> {
        let primary_role =
            RoleWithFactorSources::new(matrix_of_factor_source_ids.primary_role, factor_sources)?;

        let recovery_role =
            RoleWithFactorSources::new(matrix_of_factor_source_ids.recovery_role, factor_sources)?;

        let confirmation_role = RoleWithFactorSources::new(
            matrix_of_factor_source_ids.confirmation_role,
            factor_sources,
        )?;

        if primary_role.role() != RoleKind::Primary
            || recovery_role.role() != RoleKind::Recovery
            || confirmation_role.role() != RoleKind::Confirmation
        {
            unreachable!("Programmer error!")
        }

        Ok(Self {
            built: PhantomData,
            primary_role,
            recovery_role,
            confirmation_role,
            number_of_days_until_auto_confirm: matrix_of_factor_source_ids
                .number_of_days_until_auto_confirm,
        })
    }
}

impl HasSampleValues for MatrixWithFactorSources {
    fn sample() -> Self {
        let ids = MatrixWithFactorSourceIds::sample();
        let factor_sources = FactorSources::sample_values_all();
        Self::new(ids, &factor_sources).unwrap()
    }

    fn sample_other() -> Self {
        let ids = MatrixWithFactorSourceIds::sample_other();
        let factor_sources = FactorSources::sample_values_all();
        Self::new(ids, &factor_sources).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::upper_case_acronyms)]
    type SUT = MatrixWithFactorSources;

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
