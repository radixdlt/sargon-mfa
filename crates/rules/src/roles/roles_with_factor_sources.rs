use crate::prelude::*;

pub(crate) type RoleWithFactorSources<const R: u8> = AbstractBuiltRoleWithFactor<R, FactorSource>;
pub(crate) type PrimaryRoleWithFactorSources = RoleWithFactorSources<{ ROLE_PRIMARY }>;
pub(crate) type RecoveryRoleWithFactorSources = RoleWithFactorSources<{ ROLE_RECOVERY }>;
pub(crate) type ConfirmationRoleWithFactorSources = RoleWithFactorSources<{ ROLE_CONFIRMATION }>;

impl<const R: u8> RoleWithFactorSources<R> {
    pub fn new(
        role_with_factor_source_ids: RoleWithFactorSourceIds<R>,
        factor_sources: &FactorSources,
    ) -> Result<Self, CommonError> {
        let lookup_f = |id: &FactorSourceID| -> Result<FactorSource, CommonError> {
            factor_sources
                .get_id(id)
                .ok_or(CommonError::FactorSourceDiscrepancy)
                .cloned()
        };

        let lookup = |ids: &Vec<FactorSourceID>| -> Result<Vec<FactorSource>, CommonError> {
            ids.iter()
                .map(lookup_f)
                .collect::<Result<Vec<_>, CommonError>>()
        };

        let threshold_factors = lookup(role_with_factor_source_ids.get_threshold_factors())?;
        let override_factors = lookup(role_with_factor_source_ids.get_override_factors())?;

        Ok(Self::with_factors(
            role_with_factor_source_ids.get_threshold(),
            threshold_factors,
            override_factors,
        ))
    }
}

impl HasSampleValues for PrimaryRoleWithFactorSources {
    fn sample() -> Self {
        let ids = PrimaryRoleWithFactorSourceIds::sample();
        let factor_sources = FactorSources::sample_values_all();
        Self::new(ids, &factor_sources).unwrap()
    }

    fn sample_other() -> Self {
        let ids = PrimaryRoleWithFactorSourceIds::sample_other();
        let factor_sources = FactorSources::sample_values_all();
        Self::new(ids, &factor_sources).unwrap()
    }
}

impl HasSampleValues for RecoveryRoleWithFactorSources {
    fn sample() -> Self {
        let ids = RecoveryRoleWithFactorSourceIds::sample();
        let factor_sources = FactorSources::sample_values_all();
        Self::new(ids, &factor_sources).unwrap()
    }

    fn sample_other() -> Self {
        let ids = RecoveryRoleWithFactorSourceIds::sample_other();
        let factor_sources = FactorSources::sample_values_all();
        Self::new(ids, &factor_sources).unwrap()
    }
}

impl HasSampleValues for ConfirmationRoleWithFactorSources {
    fn sample() -> Self {
        let ids = ConfirmationRoleWithFactorSourceIds::sample();
        let factor_sources = FactorSources::sample_values_all();
        Self::new(ids, &factor_sources).unwrap()
    }

    fn sample_other() -> Self {
        let ids = ConfirmationRoleWithFactorSourceIds::sample_other();
        let factor_sources = FactorSources::sample_values_all();
        Self::new(ids, &factor_sources).unwrap()
    }
}

#[cfg(test)]
mod primary_tests {
    use super::*;

    #[allow(clippy::upper_case_acronyms)]
    type SUT = PrimaryRoleWithFactorSources;

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
    type SUT = RecoveryRoleWithFactorSources;

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
mod confirmation_tests {
    use super::*;

    #[allow(clippy::upper_case_acronyms)]
    type SUT = ConfirmationRoleWithFactorSources;

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
