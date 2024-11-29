use crate::prelude::*;

pub type RoleWithFactorSourceIds<const R: u8> = AbstractBuiltRoleWithFactor<R, FactorSourceID>;

pub type PrimaryRoleWithFactorSourceIds = RoleWithFactorSourceIds<{ ROLE_PRIMARY }>;
pub type RecoveryRoleWithFactorSourceIds = RoleWithFactorSourceIds<{ ROLE_RECOVERY }>;
pub type ConfirmationRoleWithFactorSourceIds = RoleWithFactorSourceIds<{ ROLE_CONFIRMATION }>;

impl PrimaryRoleWithFactorSourceIds {
    /// Config MFA 1.1
    pub fn sample_primary() -> Self {
        let mut builder = RoleBuilder::new();
        builder
            .add_factor_source_to_list(FactorSourceID::sample_device(), FactorListKind::Threshold)
            .unwrap();

        builder
            .add_factor_source_to_list(FactorSourceID::sample_ledger(), FactorListKind::Threshold)
            .unwrap();
        builder.set_threshold(2).unwrap();
        builder.build().unwrap()
    }
}

impl HasSampleValues for PrimaryRoleWithFactorSourceIds {
    fn sample() -> Self {
        Self::sample_primary()
    }

    fn sample_other() -> Self {
        let mut builder = RoleBuilder::new();
        builder
            .add_factor_source_to_list(FactorSourceID::sample_device(), FactorListKind::Threshold)
            .unwrap();

        builder
            .add_factor_source_to_list(FactorSourceID::sample_ledger(), FactorListKind::Threshold)
            .unwrap();
        builder.set_threshold(1).unwrap();
        builder.build().unwrap()
    }
}

impl HasSampleValues for ConfirmationRoleWithFactorSourceIds {
    /// Config MFA 1.1
    fn sample() -> Self {
        let mut builder = RoleBuilder::new();
        builder
            .add_factor_source_to_list(FactorSourceID::sample_password(), FactorListKind::Override)
            .unwrap();
        builder.build().unwrap()
    }

    /// Config MFA 2.1
    fn sample_other() -> Self {
        let mut builder = RoleBuilder::new();
        builder
            .add_factor_source_to_list(FactorSourceID::sample_device(), FactorListKind::Override)
            .unwrap();
        builder.build().unwrap()
    }
}
impl HasSampleValues for RecoveryRoleWithFactorSourceIds {
    /// Config MFA 1.1
    fn sample() -> Self {
        let mut builder = RoleBuilder::new();
        builder
            .add_factor_source_to_list(FactorSourceID::sample_device(), FactorListKind::Override)
            .unwrap();

        builder
            .add_factor_source_to_list(FactorSourceID::sample_ledger(), FactorListKind::Override)
            .unwrap();
        builder.build().unwrap()
    }

    /// Config MFA 3.3
    fn sample_other() -> Self {
        let mut builder = RoleBuilder::new();
        builder
            .add_factor_source_to_list(
                FactorSourceID::sample_ledger_other(),
                FactorListKind::Override,
            )
            .unwrap();

        builder.build().unwrap()
    }
}

#[cfg(test)]
mod primary_tests {

    use super::*;

    #[allow(clippy::upper_case_acronyms)]
    type SUT = PrimaryRoleWithFactorSourceIds;

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
    fn get_all_factors() {
        let sut = SUT::sample_primary();
        let factors = sut.all_factors();
        assert_eq!(
            factors.len(),
            sut.get_override_factors().len() + sut.get_threshold_factors().len()
        );
    }

    #[test]
    fn get_threshold() {
        let sut = SUT::sample_primary();
        assert_eq!(sut.get_threshold(), 2);
    }

    #[test]
    fn assert_json_sample_primary() {
        let sut = SUT::sample_primary();
        assert_eq_after_json_roundtrip(
            &sut,
            r#"
            {
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
            }
            "#,
        );
    }
}

#[cfg(test)]
mod recovery_tests {

    use super::*;

    #[allow(clippy::upper_case_acronyms)]
    type SUT = RecoveryRoleWithFactorSourceIds;

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
    fn get_all_factors() {
        let sut = SUT::sample();
        let factors = sut.all_factors();
        assert_eq!(
            factors.len(),
            sut.get_override_factors().len() + sut.get_threshold_factors().len()
        );
    }

    #[test]
    fn assert_json() {
        let sut = SUT::sample();
        assert_eq_after_json_roundtrip(
            &sut,
            r#"
            {
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
            }
            "#,
        );
    }
}

#[cfg(test)]
mod confirmation_tests {

    use super::*;

    #[allow(clippy::upper_case_acronyms)]
    type SUT = ConfirmationRoleWithFactorSourceIds;

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
    fn get_all_factors() {
        let sut = SUT::sample();
        let factors = sut.all_factors();
        assert_eq!(
            factors.len(),
            sut.get_override_factors().len() + sut.get_threshold_factors().len()
        );
    }

    #[test]
    fn assert_json() {
        let sut = SUT::sample();
        assert_eq_after_json_roundtrip(
            &sut,
            r#"
           {
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
            }
            "#,
        );
    }
}
