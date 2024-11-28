use sargon::HasSampleValues;

use crate::prelude::*;

pub type RoleWithFactorSourceIds = AbstractBuiltRoleWithFactor<FactorSourceID>;

impl RoleWithFactorSourceIds {
    /// Config MFA 1.1
    pub fn sample_primary() -> Self {
        let mut builder = RoleBuilder::primary();
        builder
            .add_factor_source_to_list(FactorSourceID::sample_device(), FactorListKind::Threshold)
            .unwrap();

        builder
            .add_factor_source_to_list(FactorSourceID::sample_ledger(), FactorListKind::Threshold)
            .unwrap();
        builder.set_threshold(2).unwrap();
        builder.build().unwrap()
    }

    /// Config MFA 1.1
    pub fn sample_recovery() -> Self {
        let mut builder = RoleBuilder::recovery();
        builder
            .add_factor_source_to_list(FactorSourceID::sample_device(), FactorListKind::Override)
            .unwrap();

        builder
            .add_factor_source_to_list(FactorSourceID::sample_ledger(), FactorListKind::Override)
            .unwrap();
        builder.build().unwrap()
    }
}

impl HasSampleValues for RoleWithFactorSourceIds {
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
    type SUT = RoleWithFactorSourceIds;

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
    fn assert_json_sample_primary() {
        let sut = SUT::sample_primary();
        assert_eq_after_json_roundtrip(
            &sut,
            r#"
            {
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
            }
            "#,
        );
    }

    #[test]
    fn assert_json_sample_recovery() {
        let sut = SUT::sample_recovery();
        assert_eq_after_json_roundtrip(
            &sut,
            r#"
            {
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
            }
            "#,
        );
    }
}
