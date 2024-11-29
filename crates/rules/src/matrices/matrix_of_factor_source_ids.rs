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
        // TODO: Ask Matt about this, does he mean Threshold(1) or Override?
        builder
            .add_factor_source_to_primary_override(FactorSourceID::sample_ledger())
            .unwrap();

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
        // TODO: Ask Matt about this, does he mean Threshold(1) or Override?
        builder
            .add_factor_source_to_primary_override(FactorSourceID::sample_device())
            .unwrap();

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
            ])
            .len(),
            11
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
                      "kind": "passphrase",
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
                "threshold": 0,
                "thresholdFactors": [],
                "overrideFactors": [
                  {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "device",
                      "body": "f1a93d324dd0f2bff89963ab81ed6e0c2ee7e18c0827dc1d3576b2d9f26bbd0a"
                    }
                  }
                ]
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
