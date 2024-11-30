use std::vec;

use crate::prelude::*;

pub type MatrixOfFactorSourceIds = AbstractMatrixBuilt<FactorSourceID>;

#[cfg(test)]
impl MatrixOfFactorSourceIds {
    pub(crate) fn with_roles_and_days(
        primary: PrimaryRoleWithFactorSourceIds,
        recovery: RecoveryRoleWithFactorSourceIds,
        confirmation: ConfirmationRoleWithFactorSourceIds,
        number_of_days_until_auto_confirm: u16,
    ) -> Self {
        assert_eq!(primary.role(), sargon::RoleKind::Primary);
        assert_eq!(recovery.role(), sargon::RoleKind::Recovery);
        assert_eq!(confirmation.role(), sargon::RoleKind::Confirmation);
        Self {
            built: PhantomData,
            primary_role: primary,
            recovery_role: recovery,
            confirmation_role: confirmation,
            number_of_days_until_auto_confirm,
        }
    }

    pub(crate) fn with_roles(
        primary: PrimaryRoleWithFactorSourceIds,
        recovery: RecoveryRoleWithFactorSourceIds,
        confirmation: ConfirmationRoleWithFactorSourceIds,
    ) -> Self {
        Self::with_roles_and_days(
            primary,
            recovery,
            confirmation,
            Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        )
    }
}

impl MatrixOfFactorSourceIds {
    pub fn sample_config_11() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_device())
            .unwrap();
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_ledger())
            .unwrap();
        builder.set_threshold(2).unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_device())
            .unwrap();
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_password())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_12() -> Self {
        let mut builder = MatrixBuilder::new();
        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_ledger())
            .unwrap();
        _ = builder.add_factor_source_to_primary_threshold(FactorSourceID::sample_password());

        _ = builder.set_threshold(2);

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_device())
            .unwrap();
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_password())
            .unwrap();

        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_13() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_device())
            .unwrap();
        let res = builder.add_factor_source_to_primary_threshold(FactorSourceID::sample_password());

        assert_eq!(
                    res,
                    Err(MatrixBuilderValidation::RoleInIsolation { role: RoleKind::Primary, violation: RoleBuilderValidation::NotYetValid(NotYetValidReason::PrimaryRoleWithPasswordInThresholdListMustThresholdGreaterThanOne)}
                ));
        builder.set_threshold(2).unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_device())
            .unwrap();
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_password())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_14() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_device())
            .unwrap();
        builder.set_threshold(1).unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_password())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_15() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_ledger())
            .unwrap();
        builder.set_threshold(1).unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_device())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_password())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_21() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_device())
            .unwrap();
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_ledger())
            .unwrap();
        builder.set_threshold(2).unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger())
            .unwrap();
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger_other())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_device())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_22() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_ledger())
            .unwrap();
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_ledger_other())
            .unwrap();
        builder.set_threshold(2).unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger())
            .unwrap();
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger_other())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_device())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_23() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_ledger())
            .unwrap();

        builder.set_threshold(1).unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger_other())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_device())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_24() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_device())
            .unwrap();
        builder.set_threshold(1).unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_ledger_other())
            .unwrap();

        // Build
        builder.build().unwrap()
    }

    pub fn sample_config_30() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_device())
            .unwrap();
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_ledger())
            .unwrap();
        builder.set_threshold(2).unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger())
            .unwrap();
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger_other())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_device())
            .unwrap();
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_password())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_40() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_device())
            .unwrap();
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_ledger())
            .unwrap();
        builder.set_threshold(2).unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_device())
            .unwrap();
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_password())
            .unwrap();
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_password_other())
            .unwrap();
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_passphrase())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_51() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_device())
            .unwrap();
        let _ = builder.set_threshold(2);
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_password())
            .unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_trusted_contact())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_password())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_52() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_device())
            .unwrap();
        let _ = builder.set_threshold(2);
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_password())
            .unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_trusted_contact())
            .unwrap();
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_trusted_contact_other())
            .unwrap();
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_device())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_password())
            .unwrap();
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_password_other())
            .unwrap();
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_passphrase())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_60() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_device())
            .unwrap();
        let _ = builder.set_threshold(1);

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_trusted_contact())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_security_questions())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_70() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_device())
            .unwrap();
        let _ = builder.set_threshold(2);
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_ledger())
            .unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_trusted_contact())
            .unwrap();
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_device())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_80() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_device())
            .unwrap();
        let _ = builder.set_threshold(2);
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_ledger())
            .unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_ledger())
            .unwrap();
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_device())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_security_questions())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
    }

    pub fn sample_config_90() -> Self {
        let mut builder = MatrixBuilder::new();

        // Primary
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_device())
            .unwrap();
        let _ = builder.set_threshold(2);
        builder
            .add_factor_source_to_primary_threshold(FactorSourceID::sample_ledger())
            .unwrap();

        // Recovery
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_trusted_contact())
            .unwrap();
        builder
            .add_factor_source_to_recovery_override(FactorSourceID::sample_device())
            .unwrap();

        // Confirmation
        builder
            .add_factor_source_to_confirmation_override(FactorSourceID::sample_security_questions())
            .unwrap();

        // Build
        assert!(builder.validate().is_ok());
        builder.build().unwrap()
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

impl HasSampleValues for MatrixOfFactorSourceIds {
    fn sample() -> Self {
        Self::sample_config_11()
    }

    fn sample_other() -> Self {
        Self::sample_config_24()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(clippy::upper_case_acronyms)]
    type SUT = MatrixOfFactorSourceIds;

    #[test]
    fn template() {}

    #[test]
    fn equality() {
        assert_eq!(SUT::sample(), SUT::sample());
        assert_eq!(SUT::sample_other(), SUT::sample_other());
    }

    #[test]
    fn inequality() {
        assert_ne!(SUT::sample(), SUT::sample_other());
        assert_ne!(SUT::sample(), SUT::sample_config_12());
        assert_ne!(SUT::sample().primary(), SUT::sample_other().primary());
        assert_ne!(SUT::sample().recovery(), SUT::sample_other().recovery());
        assert_ne!(
            SUT::sample().confirmation(),
            SUT::sample_other().confirmation()
        );
    }

    #[test]
    fn hash() {
        assert_eq!(
            HashSet::<SUT>::from_iter([
                SUT::sample_config_11(),
                SUT::sample_config_12(),
                SUT::sample_config_13(),
                SUT::sample_config_14(),
                SUT::sample_config_15(),
                SUT::sample_config_21(),
                SUT::sample_config_22(),
                SUT::sample_config_23(),
                SUT::sample_config_24(),
                SUT::sample_config_30(),
                SUT::sample_config_40(),
                SUT::sample_config_51(),
                SUT::sample_config_52(),
                SUT::sample_config_60(),
                SUT::sample_config_70(),
                SUT::sample_config_80(),
                SUT::sample_config_90(),
                // Duplicates should be removed
                SUT::sample_config_11(),
                SUT::sample_config_12(),
                SUT::sample_config_13(),
                SUT::sample_config_14(),
                SUT::sample_config_15(),
                SUT::sample_config_21(),
                SUT::sample_config_22(),
                SUT::sample_config_23(),
                SUT::sample_config_24(),
                SUT::sample_config_30(),
                SUT::sample_config_40(),
                SUT::sample_config_51(),
                SUT::sample_config_52(),
                SUT::sample_config_60(),
                SUT::sample_config_70(),
                SUT::sample_config_80(),
                SUT::sample_config_90(),
            ])
            .len(),
            17
        );
    }

    #[test]
    fn assert_json_sample() {
        let sut = SUT::sample();

        assert_eq_after_json_roundtrip(
            &sut,
            r#"
            {
              "primaryRole": {
                "threshold": 2,
                "thresholdFactors": [
                  {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "device",
                      "body": "f1a93d324dd0f2bff89963ab81ed6e0c2ee7e18c0827dc1d3576b2d9f26bbd0a"
                    }
                  },
                  {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "ledgerHQHardwareWallet",
                      "body": "ab59987eedd181fe98e512c1ba0f5ff059f11b5c7c56f15614dcc9fe03fec58b"
                    }
                  }
                ],
                "overrideFactors": []
              },
              "recoveryRole": {
                "threshold": 0,
                "thresholdFactors": [],
                "overrideFactors": [
                  {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "device",
                      "body": "f1a93d324dd0f2bff89963ab81ed6e0c2ee7e18c0827dc1d3576b2d9f26bbd0a"
                    }
                  },
                  {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "ledgerHQHardwareWallet",
                      "body": "ab59987eedd181fe98e512c1ba0f5ff059f11b5c7c56f15614dcc9fe03fec58b"
                    }
                  }
                ]
              },
              "confirmationRole": {
                "threshold": 0,
                "thresholdFactors": [],
                "overrideFactors": [
                  {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "password",
                      "body": "181ab662e19fac3ad9f08d5c673b286d4a5ed9cd3762356dc9831dc42427c1b9"
                    }
                  }
                ]
              },
              "numberOfDaysUntilAutoConfirm": 14
            }
            "#,
        );
    }

    #[test]
    fn assert_json_sample_other() {
        let sut = SUT::sample_other();
        assert_eq_after_json_roundtrip(
            &sut,
            r#"
            {
              "primaryRole": {
                "threshold": 1,
                "thresholdFactors": [
                    {
                    "discriminator": "fromHash",
                    "fromHash": {
                        "kind": "device",
                        "body": "f1a93d324dd0f2bff89963ab81ed6e0c2ee7e18c0827dc1d3576b2d9f26bbd0a"
                        }
                    }
                ],
                "overrideFactors": []
              },
              "recoveryRole": {
                "threshold": 0,
                "thresholdFactors": [],
                "overrideFactors": [
                  {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "ledgerHQHardwareWallet",
                      "body": "ab59987eedd181fe98e512c1ba0f5ff059f11b5c7c56f15614dcc9fe03fec58b"
                    }
                  }
                ]
              },
              "confirmationRole": {
                "threshold": 0,
                "thresholdFactors": [],
                "overrideFactors": [
                  {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "ledgerHQHardwareWallet",
                      "body": "52ef052a0642a94279b296d6b3b17dedc035a7ae37b76c1d60f11f2725100077"
                    }
                  }
                ]
              },
              "numberOfDaysUntilAutoConfirm": 14
            }
            "#,
        );
    }
}
