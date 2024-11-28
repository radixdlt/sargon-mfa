use sargon::HasSampleValues;

use crate::prelude::*;

pub type MatrixWithFactorSourceIds = AbstractMatrixBuilderOrBuilt<FactorSourceID, (), ()>;

#[cfg(test)]
impl MatrixWithFactorSourceIds {
    pub(crate) fn with_roles_and_days(
        primary: RoleWithFactorSourceIds,
        recovery: RoleWithFactorSourceIds,
        confirmation: RoleWithFactorSourceIds,
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
        primary: RoleWithFactorSourceIds,
        recovery: RoleWithFactorSourceIds,
        confirmation: RoleWithFactorSourceIds,
    ) -> Self {
        Self::with_roles_and_days(
            primary,
            recovery,
            confirmation,
            Self::DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM,
        )
    }
}

impl MatrixWithFactorSourceIds {
    pub fn primary(&self) -> &RoleWithFactorSourceIds {
        &self.primary_role
    }

    pub fn recovery(&self) -> &RoleWithFactorSourceIds {
        &self.recovery_role
    }

    pub fn confirmation(&self) -> &RoleWithFactorSourceIds {
        &self.confirmation_role
    }
}

impl MatrixWithFactorSourceIds {
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

        builder.build().unwrap()
    }

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
        builder.build().unwrap()
    }
}

impl HasSampleValues for MatrixWithFactorSourceIds {
    fn sample() -> Self {
        Self::sample_config_11()
    }

    fn sample_other() -> Self {
        Self::sample_config_12()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(clippy::upper_case_acronyms)]
    type SUT = MatrixWithFactorSourceIds;

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
    fn assert_json_sample() {
        let sut = SUT::sample();
        assert_eq_after_json_roundtrip(
            &sut,
            r#"
            {
              "primary_role": {
                "role": "primary",
                "threshold": 2,
                "threshold_factors": [
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
                "override_factors": []
              },
              "recovery_role": {
                "role": "recovery",
                "threshold": 0,
                "threshold_factors": [],
                "override_factors": [
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
              "confirmation_role": {
                "role": "confirmation",
                "threshold": 0,
                "threshold_factors": [],
                "override_factors": [
                  {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "passphrase",
                      "body": "181ab662e19fac3ad9f08d5c673b286d4a5ed9cd3762356dc9831dc42427c1b9"
                    }
                  }
                ]
              },
              "number_of_days_until_auto_confirm": 14
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
              "primary_role": {
                "role": "primary",
                "threshold": 2,
                "threshold_factors": [
                  {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "ledgerHQHardwareWallet",
                      "body": "ab59987eedd181fe98e512c1ba0f5ff059f11b5c7c56f15614dcc9fe03fec58b"
                    }
                  },
                  {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "passphrase",
                      "body": "181ab662e19fac3ad9f08d5c673b286d4a5ed9cd3762356dc9831dc42427c1b9"
                    }
                  }
                ],
                "override_factors": []
              },
              "recovery_role": {
                "role": "recovery",
                "threshold": 0,
                "threshold_factors": [],
                "override_factors": [
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
              "confirmation_role": {
                "role": "confirmation",
                "threshold": 0,
                "threshold_factors": [],
                "override_factors": [
                  {
                    "discriminator": "fromHash",
                    "fromHash": {
                      "kind": "passphrase",
                      "body": "181ab662e19fac3ad9f08d5c673b286d4a5ed9cd3762356dc9831dc42427c1b9"
                    }
                  }
                ]
              },
              "number_of_days_until_auto_confirm": 14
            }
            "#,
        );
    }
}
