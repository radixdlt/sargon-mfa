#![allow(clippy::new_without_default)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::sync::{Arc, RwLock};

use sargon::IndexSet;

use crate::prelude::*;

#[derive(Debug, uniffi::Object)]
pub struct SecurityShieldBuilder {
    wrapped: RwLock<Option<MatrixBuilder>>,
}

#[derive(Debug, PartialEq, Eq, Hash, uniffi::Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct SecurityStructureOfFactorSourceIds {
    wrapped_matrix: MatrixWithFactorSourceIds,
    name: String,
}

impl SecurityShieldBuilder {
    fn with<R, E: Into<CommonError>>(
        &self,
        mut with_non_consumed_builder: impl FnMut(&mut MatrixBuilder) -> Result<R, E>,
    ) -> Result<R, CommonError> {
        let guard = self.wrapped.write();

        let mut binding = guard.map_err(|_| CommonError::MatrixBuilderRwLockPoisoned)?;

        let Some(builder) = binding.as_mut() else {
            return Err(CommonError::AlreadyBuilt);
        };
        with_non_consumed_builder(builder).map_err(|e| Into::<CommonError>::into(e))
    }

    fn validation_for_addition_of_factor_source_by_calling(
        &self,
        factor_sources: Vec<Arc<FactorSourceID>>,
        call: impl Fn(
            &MatrixBuilder,
            &IndexSet<sargon::FactorSourceID>,
        ) -> IndexSet<rules::FactorSourceInRoleBuilderValidationStatus>,
    ) -> Result<Vec<Arc<FactorSourceValidationStatus>>, CommonError> {
        let input = &factor_sources
            .clone()
            .into_iter()
            .map(|x| x.inner)
            .collect::<IndexSet<_>>();
        self.with(|builder| {
            let xs = call(builder, input);

            let xs = xs
                .into_iter()
                .map(Into::<FactorSourceValidationStatus>::into)
                .map(Arc::new)
                .collect();

            Ok::<_, CommonError>(xs)
        })
    }
}

#[uniffi::export]
impl SecurityShieldBuilder {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            wrapped: RwLock::new(Some(MatrixBuilder::new())),
        })
    }

    /// Adds the factor source to the primary role threshold list.
    pub fn add_factor_source_to_primary_threshold(
        &self,
        factor_source_id: Arc<FactorSourceID>,
    ) -> Result<(), CommonError> {
        self.with(|builder| builder.add_factor_source_to_primary_threshold(factor_source_id.inner))
    }

    pub fn add_factor_source_to_primary_override(
        &self,
        factor_source_id: Arc<FactorSourceID>,
    ) -> Result<(), CommonError> {
        self.with(|builder| builder.add_factor_source_to_primary_override(factor_source_id.inner))
    }

    pub fn remove_factor(&self, factor_source_id: Arc<FactorSourceID>) -> Result<(), CommonError> {
        self.with(|builder| builder.remove_factor(&factor_source_id.inner))
    }

    pub fn set_threshold(&self, threshold: u8) -> Result<(), CommonError> {
        self.with(|builder| builder.set_threshold(threshold))
    }

    pub fn set_number_of_days_until_auto_confirm(
        &self,
        number_of_days: u16,
    ) -> Result<(), CommonError> {
        self.with(|builder| builder.set_number_of_days_until_auto_confirm(number_of_days))
    }

    pub fn add_factor_source_to_recovery_override(
        &self,
        factor_source_id: Arc<FactorSourceID>,
    ) -> Result<(), CommonError> {
        self.with(|builder| builder.add_factor_source_to_recovery_override(factor_source_id.inner))
    }

    pub fn add_factor_source_to_confirmation_override(
        &self,
        factor_source_id: Arc<FactorSourceID>,
    ) -> Result<(), CommonError> {
        self.with(|builder| {
            builder.add_factor_source_to_confirmation_override(factor_source_id.inner)
        })
    }

    pub fn validation_for_addition_of_factor_source_of_kind_to_confirmation_override(
        &self,
        factor_source_kind: FactorSourceKind,
    ) -> Result<(), CommonError> {
        self.with(|builder| {
            builder.validation_for_addition_of_factor_source_of_kind_to_confirmation_override(
                factor_source_kind.into(),
            )
        })
    }

    pub fn validation_for_addition_of_factor_source_of_kind_to_recovery_override(
        &self,
        factor_source_kind: FactorSourceKind,
    ) -> Result<(), CommonError> {
        self.with(|builder| {
            builder.validation_for_addition_of_factor_source_of_kind_to_recovery_override(
                factor_source_kind.into(),
            )
        })
    }

    pub fn validation_for_addition_of_factor_source_of_kind_to_primary_override(
        &self,
        factor_source_kind: FactorSourceKind,
    ) -> Result<(), CommonError> {
        self.with(|builder| {
            builder.validation_for_addition_of_factor_source_of_kind_to_primary_override(
                factor_source_kind.into(),
            )
        })
    }

    pub fn validation_for_addition_of_factor_source_of_kind_to_primary_threshold(
        &self,
        factor_source_kind: FactorSourceKind,
    ) -> Result<(), CommonError> {
        self.with(|builder| {
            builder.validation_for_addition_of_factor_source_of_kind_to_primary_threshold(
                factor_source_kind.into(),
            )
        })
    }

    pub fn validation_for_addition_of_factor_source_to_primary_threshold_for_each(
        &self,
        factor_sources: Vec<Arc<FactorSourceID>>,
    ) -> Result<Vec<Arc<FactorSourceValidationStatus>>, CommonError> {
        self.validation_for_addition_of_factor_source_by_calling(
            factor_sources,
            |builder, input| {
                builder
                    .validation_for_addition_of_factor_source_to_primary_threshold_for_each(input)
            },
        )
    }

    pub fn validation_for_addition_of_factor_source_to_primary_override_for_each(
        &self,
        factor_sources: Vec<Arc<FactorSourceID>>,
    ) -> Result<Vec<Arc<FactorSourceValidationStatus>>, CommonError> {
        self.validation_for_addition_of_factor_source_by_calling(
            factor_sources,
            |builder, input| {
                builder.validation_for_addition_of_factor_source_to_primary_override_for_each(input)
            },
        )
    }

    pub fn validation_for_addition_of_factor_source_to_recovery_override_for_each(
        &self,
        factor_sources: Vec<Arc<FactorSourceID>>,
    ) -> Result<Vec<Arc<FactorSourceValidationStatus>>, CommonError> {
        self.validation_for_addition_of_factor_source_by_calling(
            factor_sources,
            |builder, input| {
                builder
                    .validation_for_addition_of_factor_source_to_recovery_override_for_each(input)
            },
        )
    }

    pub fn validation_for_addition_of_factor_source_to_confirmation_override_for_each(
        &self,
        factor_sources: Vec<Arc<FactorSourceID>>,
    ) -> Result<Vec<Arc<FactorSourceValidationStatus>>, CommonError> {
        self.validation_for_addition_of_factor_source_by_calling(
            factor_sources,
            |builder, input| {
                builder.validation_for_addition_of_factor_source_to_confirmation_override_for_each(
                    input,
                )
            },
        )
    }

    pub fn build(
        self: Arc<Self>,
        name: String,
    ) -> Result<SecurityStructureOfFactorSourceIds, CommonError> {
        let mut binding = self
            .wrapped
            .write()
            .map_err(|_| CommonError::MatrixBuilderRwLockPoisoned)?;
        let builder = binding.take().ok_or(CommonError::AlreadyBuilt)?;
        let wrapped_matrix = builder
            .build()
            .map_err(|e| CommonError::BuildError(format!("{:?}", e)))?;

        let shield = SecurityStructureOfFactorSourceIds {
            wrapped_matrix,
            name,
        };

        Ok(shield)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[allow(clippy::upper_case_acronyms)]
    type SUT = SecurityShieldBuilder;

    #[test]
    fn test() {
        let sut = SUT::new();
        sut.add_factor_source_to_primary_override(FactorSourceID::sample_arculus())
            .unwrap();
        sut.add_factor_source_to_primary_override(FactorSourceID::sample_arculus_other())
            .unwrap();
        sut.add_factor_source_to_recovery_override(FactorSourceID::sample_ledger())
            .unwrap();
        sut.add_factor_source_to_recovery_override(FactorSourceID::sample_ledger_other())
            .unwrap();
        sut.add_factor_source_to_confirmation_override(FactorSourceID::sample_device())
            .unwrap();

        sut.remove_factor(FactorSourceID::sample_arculus_other())
            .unwrap();
        sut.remove_factor(FactorSourceID::sample_ledger_other())
            .unwrap();

        let shield = sut.build("test".to_owned()).unwrap();
        assert_eq!(
            shield.wrapped_matrix.primary().get_override_factors(),
            &vec![FactorSourceID::sample_arculus().inner]
        );
        assert_eq!(
            shield.wrapped_matrix.recovery().get_override_factors(),
            &vec![FactorSourceID::sample_ledger().inner]
        );
        assert_eq!(
            shield.wrapped_matrix.confirmation().get_override_factors(),
            &vec![FactorSourceID::sample_device().inner]
        );
    }
}
