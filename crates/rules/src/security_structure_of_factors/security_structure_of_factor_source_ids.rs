use crate::prelude::*;

pub type SecurityStructureOfFactorSourceIds = AbstractSecurityStructure<FactorSourceID>;

impl HasSampleValues for SecurityStructureOfFactorSourceIds {
    fn sample() -> Self {
        let metadata = sargon::SecurityStructureMetadata::sample();
        Self::with_metadata(metadata, MatrixWithFactorSourceIds::sample())
    }

    fn sample_other() -> Self {
        let metadata = sargon::SecurityStructureMetadata::sample_other();
        Self::with_metadata(metadata, MatrixWithFactorSourceIds::sample_other())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(clippy::upper_case_acronyms)]
    type SUT = SecurityStructureOfFactorSourceIds;

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
              "metadata": {
                "id": "ffffffff-ffff-ffff-ffff-ffffffffffff",
                "displayName": "Spending Account",
                "createdOn": "2023-09-11T16:05:56.000Z",
                "lastUpdatedOn": "2023-09-11T16:05:56.000Z"
              },
              "matrix_of_factors": {
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
              "metadata": {
                "id": "dededede-dede-dede-dede-dededededede",
                "displayName": "Savings Account",
                "createdOn": "2023-12-24T17:13:56.123Z",
                "lastUpdatedOn": "2023-12-24T17:13:56.123Z"
              },
              "matrix_of_factors": {
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
            }
            "#,
        );
    }
}
