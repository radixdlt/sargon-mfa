use crate::prelude::*;

pub type MatrixOfFactorInstances = AbstractMatrixBuilderOrBuilt<FactorInstance, (), ()>;

impl HasFactorInstances for MatrixOfFactorInstances {
    fn unique_factor_instances(&self) -> IndexSet<FactorInstance> {
        let mut set = IndexSet::new();
        set.extend(self.primary_role.all_factors().into_iter().cloned());
        set.extend(self.recovery_role.all_factors().into_iter().cloned());
        set.extend(self.confirmation_role.all_factors().into_iter().cloned());
        set
    }
}

impl HasSampleValues for MatrixOfFactorInstances {
    fn sample() -> Self {
        Self {
            built: PhantomData,
            primary_role: RoleWithFactorInstances::sample_primary(),
            recovery_role: RoleWithFactorInstances::sample_recovery(),
            confirmation_role: RoleWithFactorInstances::sample_confirmation(),
            number_of_days_until_auto_confirm: 30,
        }
    }

    fn sample_other() -> Self {
        Self {
            built: PhantomData,
            primary_role: RoleWithFactorInstances::sample_primary_other(),
            recovery_role: RoleWithFactorInstances::sample_recovery_other(),
            confirmation_role: RoleWithFactorInstances::sample_confirmation_other(),
            number_of_days_until_auto_confirm: 15,
        }
    }
}

impl MatrixOfFactorInstances {
    /// Maps `MatrixOfFactorSources -> MatrixOfFactorInstances` by
    /// "assigning" FactorInstances to each MatrixOfFactorInstances from
    /// `consuming_instances`.
    ///
    /// NOTE:
    /// **One FactorInstance might be used multiple times in the MatrixOfFactorInstances,
    /// e.g. ones in the PrimaryRole(WithFactorInstances) and again in RecoveryRole(WithFactorInstances) or
    /// in RecoveryRole(WithFactorInstances)**.
    ///
    /// However, the same FactorInstance is NEVER used in two different MatrixOfFactorInstances.
    ///
    ///
    pub fn fulfilling_matrix_of_factor_sources_with_instances(
        consuming_instances: &mut IndexMap<FactorSourceIDFromHash, FactorInstances>,
        matrix_of_factor_sources: MatrixOfFactorSources,
    ) -> Result<Self, CommonError> {
        let instances = &consuming_instances.clone();

        let primary_role =
            PrimaryRoleWithFactorInstances::fulfilling_role_of_factor_sources_with_factor_instances(
                instances,
                &matrix_of_factor_sources,
            )?;
        let recovery_role =
            RecoveryRoleWithFactorInstances::fulfilling_role_of_factor_sources_with_factor_instances(
                instances,
                &matrix_of_factor_sources,
            )?;
        let confirmation_role =
            ConfirmationRoleWithFactorInstances::fulfilling_role_of_factor_sources_with_factor_instances(
                instances,
                &matrix_of_factor_sources,
            )?;

        let matrix = Self {
            built: PhantomData,
            primary_role,
            recovery_role,
            confirmation_role,
            number_of_days_until_auto_confirm: matrix_of_factor_sources
                .number_of_days_until_auto_confirm,
        };

        // Now that we have assigned instances, **possibly the SAME INSTANCE to multiple roles**,
        // lets delete them from the `consuming_instances` map.
        for instance in matrix.all_factors() {
            let fsid = &FactorSourceIDFromHash::try_from(instance.factor_source_id).unwrap();
            let existing = consuming_instances.get_mut(fsid).unwrap();

            let to_remove =
                HierarchicalDeterministicFactorInstance::try_from(instance.clone()).unwrap();

            // We remove at the beginning of the list first.
            existing.shift_remove(&to_remove);

            if existing.is_empty() {
                // not needed per se, but feels prudent to "prune".
                consuming_instances.shift_remove_entry(fsid);
            }
        }

        Ok(matrix)
    }
}
#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use sargon::{indexmap::IndexSet, FactorInstancesCacheClient, FactorInstancesProvider, FileSystemClient, HostId, HostInfo, InMemoryFileSystemDriver, Mnemonic, MnemonicWithPassphrase, Profile};

    use super::*;

    #[allow(clippy::upper_case_acronyms)]
    type SUT = MatrixOfFactorInstances;

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
    fn err_if_no_instance_found_for_factor_source() {
        assert!(matches!(
            SUT::fulfilling_matrix_of_factor_sources_with_instances(
                &mut IndexMap::new(),
                MatrixOfFactorSources::sample()
            ),
            Err(CommonError::MissingFactorMappingInstancesIntoRole)
        ));
    }

    #[test]
    fn err_if_empty_instance_found_for_factor_source() {
        assert!(matches!(
            SUT::fulfilling_matrix_of_factor_sources_with_instances(
                &mut IndexMap::kv(
                    FactorSource::sample_device_babylon().id_from_hash(),
                    FactorInstances::from_iter([])
                ),
                MatrixOfFactorSources::sample()
            ),
            Err(CommonError::MissingFactorMappingInstancesIntoRole)
        ));
    }

    #[test]
    fn assert_json_sample() {
        let sut = SUT::sample();
        assert_eq_after_json_roundtrip(
            &sut,
            r#"
            {
              "primary_role": {
                "threshold": 1,
                "threshold_factors": [
                  {
                    "factorSourceID": {
                      "discriminator": "fromHash",
                      "fromHash": {
                        "kind": "device",
                        "body": "f1a93d324dd0f2bff89963ab81ed6e0c2ee7e18c0827dc1d3576b2d9f26bbd0a"
                      }
                    },
                    "badge": {
                      "discriminator": "virtualSource",
                      "virtualSource": {
                        "discriminator": "hierarchicalDeterministicPublicKey",
                        "hierarchicalDeterministicPublicKey": {
                          "publicKey": {
                            "curve": "curve25519",
                            "compressedData": "427969814e15d74c3ff4d9971465cb709d210c8a7627af9466bdaa67bd0929b7"
                          },
                          "derivationPath": {
                            "scheme": "cap26",
                            "path": "m/44H/1022H/1H/525H/1460H/0S"
                          }
                        }
                      }
                    }
                  }
                ],
                "override_factors": [
                  {
                    "factorSourceID": {
                      "discriminator": "fromHash",
                      "fromHash": {
                        "kind": "device",
                        "body": "5255999c65076ce9ced5a1881f1a621bba1ce3f1f68a61df462d96822a5190cd"
                      }
                    },
                    "badge": {
                      "discriminator": "virtualSource",
                      "virtualSource": {
                        "discriminator": "hierarchicalDeterministicPublicKey",
                        "hierarchicalDeterministicPublicKey": {
                          "publicKey": {
                            "curve": "curve25519",
                            "compressedData": "e0293d4979bc303ea4fe361a62baf9c060c7d90267972b05c61eead9ef3eed3e"
                          },
                          "derivationPath": {
                            "scheme": "cap26",
                            "path": "m/44H/1022H/1H/525H/1460H/0S"
                          }
                        }
                      }
                    }
                  }
                ]
              },
              "recovery_role": {
                "threshold": 0,
                "threshold_factors": [],
                "override_factors": [
                  {
                    "factorSourceID": {
                      "discriminator": "fromHash",
                      "fromHash": {
                        "kind": "device",
                        "body": "5255999c65076ce9ced5a1881f1a621bba1ce3f1f68a61df462d96822a5190cd"
                      }
                    },
                    "badge": {
                      "discriminator": "virtualSource",
                      "virtualSource": {
                        "discriminator": "hierarchicalDeterministicPublicKey",
                        "hierarchicalDeterministicPublicKey": {
                          "publicKey": {
                            "curve": "curve25519",
                            "compressedData": "23fa85f95c79684d2768f46ec4379b5e031757b727f76cfd01a50bd4cf8afcff"
                          },
                          "derivationPath": {
                            "scheme": "cap26",
                            "path": "m/44H/1022H/1H/525H/1460H/237S"
                          }
                        }
                      }
                    }
                  }
                ]
              },
              "confirmation_role": {
                "threshold": 0,
                "threshold_factors": [],
                "override_factors": [
                  {
                    "factorSourceID": {
                      "discriminator": "fromHash",
                      "fromHash": {
                        "kind": "device",
                        "body": "f1a93d324dd0f2bff89963ab81ed6e0c2ee7e18c0827dc1d3576b2d9f26bbd0a"
                      }
                    },
                    "badge": {
                      "discriminator": "virtualSource",
                      "virtualSource": {
                        "discriminator": "hierarchicalDeterministicPublicKey",
                        "hierarchicalDeterministicPublicKey": {
                          "publicKey": {
                            "curve": "curve25519",
                            "compressedData": "10ccd9b906660d49b3fe89651baa1284fc7b19ad2c3d423a7828ec350c0e5fe0"
                          },
                          "derivationPath": {
                            "scheme": "cap26",
                            "path": "m/44H/1022H/1H/525H/1460H/1S"
                          }
                        }
                      }
                    }
                  },
                  {
                    "factorSourceID": {
                      "discriminator": "fromHash",
                      "fromHash": {
                        "kind": "device",
                        "body": "5255999c65076ce9ced5a1881f1a621bba1ce3f1f68a61df462d96822a5190cd"
                      }
                    },
                    "badge": {
                      "discriminator": "virtualSource",
                      "virtualSource": {
                        "discriminator": "hierarchicalDeterministicPublicKey",
                        "hierarchicalDeterministicPublicKey": {
                          "publicKey": {
                            "curve": "curve25519",
                            "compressedData": "6a92b3338dc74a50e8b3fff896a7e0f43c42742544af52de20353675d8bc7907"
                          },
                          "derivationPath": {
                            "scheme": "cap26",
                            "path": "m/44H/1022H/1H/525H/1460H/2S"
                          }
                        }
                      }
                    }
                  }
                ]
              },
              "number_of_days_until_auto_confirm": 30
            }
            "#,
        );
    }

    #[test]
    fn fulfilling_role_of_factor_sources_with_factor_instances() {
        let matrix = MatrixOfFactorSources::sample();
        let mut consuming_instances = MnemonicWithPassphrase::derive_instances_for_all_factor_sources();
        let sut = SUT::fulfilling_matrix_of_factor_sources_with_instances(&mut consuming_instances, matrix).unwrap();
    }

}
