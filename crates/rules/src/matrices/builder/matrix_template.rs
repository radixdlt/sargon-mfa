use crate::prelude::*;

pub type MatrixTemplate = AbstractMatrixBuilt<FactorSourceTemplate>;

impl<const R: u8> AbstractBuiltRoleWithFactor<R, FactorSourceTemplate> {
    pub(crate) fn fulfill(
        self,
        factor_source_id_assigner: &mut FactorSourceIdAssigner,
    ) -> Result<RoleWithFactorSourceIds<R>, CommonError> {
        let mut fulfill =
            |xs: &Vec<FactorSourceTemplate>| -> Result<Vec<FactorSourceID>, CommonError> {
                xs.iter()
                    .map(|f| factor_source_id_assigner.next(f))
                    .collect::<Result<Vec<_>, CommonError>>()
            };
        Ok(RoleWithFactorSourceIds::with_factors(
            self.get_threshold(),
            fulfill(self.get_threshold_factors())?,
            fulfill(self.get_override_factors())?,
        ))
    }
}
pub(crate) struct FactorSourceIdAssigner {
    factor_source_ids: Vec<FactorSourceID>,
    map: IndexMap<FactorSourceTemplate, FactorSourceID>,
}

impl FactorSourceIdAssigner {
    fn new(factor_source_ids: impl IntoIterator<Item = FactorSourceID>) -> Self {
        Self {
            factor_source_ids: factor_source_ids.into_iter().collect_vec(),
            map: IndexMap::new(),
        }
    }

    fn next(&mut self, template: &FactorSourceTemplate) -> Result<FactorSourceID, CommonError> {
        if let Some(existing) = self.map.get(template) {
            Ok(*existing)
        } else if let Some(index_of_next) = self
            .factor_source_ids
            .iter()
            .position(|f| f.get_factor_source_kind() == template.kind)
        {
            let next = self.factor_source_ids.remove(index_of_next);
            self.map.insert(template.clone(), next);
            Ok(next)
        } else {
            Err(CommonError::Unknown)
        }
    }
}

impl MatrixTemplate {
    pub fn fulfill(
        self,
        factor_source_ids: impl IntoIterator<Item = FactorSourceID>,
    ) -> Result<MatrixOfFactorSourceIds, CommonError> {
        let mut assigner = FactorSourceIdAssigner::new(factor_source_ids);
        let primary_role = self.primary_role.fulfill(&mut assigner)?;
        let recovery_role = self.recovery_role.fulfill(&mut assigner)?;
        let confirmation_role = self.confirmation_role.fulfill(&mut assigner)?;

        Ok(MatrixOfFactorSourceIds {
            built: PhantomData,
            primary_role,
            recovery_role,
            confirmation_role,
            number_of_days_until_auto_confirm:
                MatrixOfFactorSourceIds::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        })
    }
}

impl MatrixTemplate {
    pub fn config_11() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                2,
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::ledger(0),
                ],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::ledger(0),
                ],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::password(0)],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_12() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                2,
                vec![
                    FactorSourceTemplate::ledger(0),
                    FactorSourceTemplate::password(0),
                ],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::ledger(0),
                ],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::password(0)],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_13() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                2,
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::password(0),
                ],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::ledger(0),
                ],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::password(0)],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_14() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                1,
                vec![FactorSourceTemplate::device(0)],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::ledger(0)],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::password(0)],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_15() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                1,
                vec![FactorSourceTemplate::ledger(0)],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::device(0)],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::password(0)],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_21() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                2,
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::ledger(0),
                ],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![
                    FactorSourceTemplate::ledger(0),
                    FactorSourceTemplate::ledger(1),
                ],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::device(0)],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_22() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                2,
                vec![
                    FactorSourceTemplate::ledger(0),
                    FactorSourceTemplate::ledger(1),
                ],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![
                    FactorSourceTemplate::ledger(0),
                    FactorSourceTemplate::ledger(1),
                ],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::device(0)],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_23() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                1,
                vec![FactorSourceTemplate::ledger(0)],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::ledger(1)],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::device(0)],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_24() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                1,
                vec![FactorSourceTemplate::device(0)],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::ledger(0)],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::ledger(1)],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_30() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                2,
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::ledger(0),
                ],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![
                    FactorSourceTemplate::ledger(0),
                    FactorSourceTemplate::ledger(1),
                ],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::password(0),
                ],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_40() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                2,
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::ledger(0),
                ],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::ledger(0),
                ],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![
                    FactorSourceTemplate::password(0),
                    FactorSourceTemplate::password(1),
                    FactorSourceTemplate::off_device_mnemonic(0),
                ],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_51() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                2,
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::password(0),
                ],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::trusted_contact(0)],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::password(0)],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_52() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                2,
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::password(0),
                ],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![
                    FactorSourceTemplate::trusted_contact(0),
                    FactorSourceTemplate::trusted_contact(1),
                    FactorSourceTemplate::device(0),
                ],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![
                    FactorSourceTemplate::password(0),
                    FactorSourceTemplate::password(1),
                    FactorSourceTemplate::off_device_mnemonic(0),
                ],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_60() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                1,
                vec![FactorSourceTemplate::device(0)],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::trusted_contact(0)],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::security_questions(0)],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_70() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                2,
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::ledger(0),
                ],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![
                    FactorSourceTemplate::trusted_contact(0),
                    FactorSourceTemplate::ledger(0),
                ],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::device(0)],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_80() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                2,
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::ledger(0),
                ],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![
                    FactorSourceTemplate::ledger(0),
                    FactorSourceTemplate::device(0),
                ],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::security_questions(0)],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }

    pub fn config_90() -> Self {
        Self {
            built: PhantomData,
            primary_role: PrimaryRoleTemplate::with_factors(
                2,
                vec![
                    FactorSourceTemplate::device(0),
                    FactorSourceTemplate::ledger(0),
                ],
                vec![],
            ),
            recovery_role: RecoveryRoleTemplate::with_factors(
                0,
                vec![],
                vec![
                    FactorSourceTemplate::trusted_contact(0),
                    FactorSourceTemplate::device(0),
                ],
            ),
            confirmation_role: ConfirmationRoleTemplate::with_factors(
                0,
                vec![],
                vec![FactorSourceTemplate::security_questions(0)],
            ),
            number_of_days_until_auto_confirm: Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        }
    }
}

#[cfg(test)]
mod test_templates {
    use super::*;

    fn test_template(template: MatrixTemplate, expected: MatrixOfFactorSourceIds) {
        let m = template
            .fulfill(*ALL_FACTOR_SOURCE_ID_SAMPLES_INC_NON_HD)
            .unwrap();
        pretty_assertions::assert_eq!(m, expected);
    }

    #[test]
    fn template_config_11() {
        test_template(
            MatrixTemplate::config_11(),
            MatrixOfFactorSourceIds::sample_config_11(),
        )
    }

    #[test]
    fn template_config_12() {
        test_template(
            MatrixTemplate::config_12(),
            MatrixOfFactorSourceIds::sample_config_12(),
        )
    }

    #[test]
    fn template_config_13() {
        test_template(
            MatrixTemplate::config_13(),
            MatrixOfFactorSourceIds::sample_config_13(),
        )
    }

    #[test]
    fn template_config_14() {
        test_template(
            MatrixTemplate::config_14(),
            MatrixOfFactorSourceIds::sample_config_14(),
        )
    }

    #[test]
    fn template_config_15() {
        test_template(
            MatrixTemplate::config_15(),
            MatrixOfFactorSourceIds::sample_config_15(),
        )
    }

    #[test]
    fn template_config_21() {
        test_template(
            MatrixTemplate::config_21(),
            MatrixOfFactorSourceIds::sample_config_21(),
        )
    }

    #[test]
    fn template_config_22() {
        test_template(
            MatrixTemplate::config_22(),
            MatrixOfFactorSourceIds::sample_config_22(),
        )
    }

    #[test]
    fn template_config_23() {
        test_template(
            MatrixTemplate::config_23(),
            MatrixOfFactorSourceIds::sample_config_23(),
        )
    }

    #[test]
    fn template_config_24() {
        test_template(
            MatrixTemplate::config_24(),
            MatrixOfFactorSourceIds::sample_config_24(),
        )
    }

    #[test]
    fn template_config_30() {
        test_template(
            MatrixTemplate::config_30(),
            MatrixOfFactorSourceIds::sample_config_30(),
        )
    }

    #[test]
    fn template_config_40() {
        test_template(
            MatrixTemplate::config_40(),
            MatrixOfFactorSourceIds::sample_config_40(),
        )
    }

    #[test]
    fn template_config_51() {
        test_template(
            MatrixTemplate::config_51(),
            MatrixOfFactorSourceIds::sample_config_51(),
        )
    }

    #[test]
    fn template_config_52() {
        test_template(
            MatrixTemplate::config_52(),
            MatrixOfFactorSourceIds::sample_config_52(),
        )
    }

    #[test]
    fn template_config_60() {
        test_template(
            MatrixTemplate::config_60(),
            MatrixOfFactorSourceIds::sample_config_60(),
        )
    }

    #[test]
    fn template_config_70() {
        test_template(
            MatrixTemplate::config_70(),
            MatrixOfFactorSourceIds::sample_config_70(),
        )
    }

    #[test]
    fn template_config_80() {
        test_template(
            MatrixTemplate::config_80(),
            MatrixOfFactorSourceIds::sample_config_80(),
        )
    }

    #[test]
    fn template_config_90() {
        test_template(
            MatrixTemplate::config_90(),
            MatrixOfFactorSourceIds::sample_config_90(),
        )
    }
}
