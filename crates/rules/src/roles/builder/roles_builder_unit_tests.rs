#![cfg(test)]

use crate::prelude::*;

use NotYetValidReason::*;
type Validation = RoleBuilderValidation;

#[allow(clippy::upper_case_acronyms)]
type SUT = RoleBuilder;
type MutRes = RoleBuilderMutateResult;
type BuildRes = RoleBuilderBuildResult;

mod test_helper_functions {

    use super::*;

    #[test]
    fn factor_sources_not_of_kind_to_list_of_kind_in_override() {
        let mut sut = SUT::primary();
        sut.add_factor_source_to_list(FactorSourceID::sample_device(), FactorListKind::Override)
            .unwrap();
        sut.add_factor_source_to_list(FactorSourceID::sample_ledger(), FactorListKind::Override)
            .unwrap();
        sut.add_factor_source_to_list(FactorSourceID::sample_arculus(), FactorListKind::Override)
            .unwrap();

        let xs = sut.factor_sources_not_of_kind_to_list_of_kind(
            FactorSourceKind::Device,
            FactorListKind::Override,
        );
        assert_eq!(
            xs,
            vec![
                FactorSourceID::sample_ledger(),
                FactorSourceID::sample_arculus()
            ]
        );

        let xs = sut.factor_sources_not_of_kind_to_list_of_kind(
            FactorSourceKind::LedgerHQHardwareWallet,
            FactorListKind::Override,
        );
        assert_eq!(
            xs,
            vec![
                FactorSourceID::sample_device(),
                FactorSourceID::sample_arculus()
            ]
        );

        let xs = sut.factor_sources_not_of_kind_to_list_of_kind(
            FactorSourceKind::ArculusCard,
            FactorListKind::Override,
        );
        assert_eq!(
            xs,
            vec![
                FactorSourceID::sample_device(),
                FactorSourceID::sample_ledger()
            ]
        );
    }

    #[test]
    fn factor_sources_not_of_kind_to_list_of_kind_in_threshold() {
        let mut sut = SUT::primary();
        sut.add_factor_source_to_list(FactorSourceID::sample_device(), FactorListKind::Threshold)
            .unwrap();
        sut.add_factor_source_to_list(FactorSourceID::sample_ledger(), FactorListKind::Threshold)
            .unwrap();
        sut.add_factor_source_to_list(FactorSourceID::sample_arculus(), FactorListKind::Threshold)
            .unwrap();

        let xs = sut.factor_sources_not_of_kind_to_list_of_kind(
            FactorSourceKind::Device,
            FactorListKind::Threshold,
        );
        assert_eq!(
            xs,
            vec![
                FactorSourceID::sample_ledger(),
                FactorSourceID::sample_arculus()
            ]
        );

        let xs = sut.factor_sources_not_of_kind_to_list_of_kind(
            FactorSourceKind::LedgerHQHardwareWallet,
            FactorListKind::Threshold,
        );
        assert_eq!(
            xs,
            vec![
                FactorSourceID::sample_device(),
                FactorSourceID::sample_arculus()
            ]
        );

        let xs = sut.factor_sources_not_of_kind_to_list_of_kind(
            FactorSourceKind::ArculusCard,
            FactorListKind::Threshold,
        );
        assert_eq!(
            xs,
            vec![
                FactorSourceID::sample_device(),
                FactorSourceID::sample_ledger()
            ]
        );
    }
}

fn test_duplicates_not_allowed(sut: SUT, list: FactorListKind, factor_source_id: FactorSourceID) {
    // Arrange
    let mut sut = sut;

    sut.add_factor_source_to_list(factor_source_id, list)
        .unwrap();

    // Act
    let res = sut.add_factor_source_to_list(
        factor_source_id, // oh no, duplicate!
        list,
    );

    // Assert
    assert!(matches!(
        res,
        MutRes::Err(Validation::ForeverInvalid(
            ForeverInvalidReason::FactorSourceAlreadyPresent
        ))
    ));
}

#[test]
fn new_builders() {
    assert_eq!(SUT::primary().role(), RoleKind::Primary);
    assert_eq!(SUT::recovery().role(), RoleKind::Recovery);
    assert_eq!(SUT::confirmation().role(), RoleKind::Confirmation);
}

#[test]
fn empty_is_err() {
    [
        RoleKind::Primary,
        RoleKind::Recovery,
        RoleKind::Confirmation,
    ]
    .iter()
    .for_each(|role| {
        let sut = SUT::new(*role);
        let res = sut.build();
        assert_eq!(
            res,
            BuildRes::not_yet_valid(NotYetValidReason::RoleMustHaveAtLeastOneFactor)
        );
    });
}

#[test]
fn validate_override_for_ever_invalid() {
    let sut = SUT::with_factors(
        RoleKind::Primary,
        0,
        vec![],
        vec![
            FactorSourceID::sample_ledger(),
            FactorSourceID::sample_ledger(),
        ],
    );
    let res = sut.validate();
    assert_eq!(
        res,
        MutRes::forever_invalid(ForeverInvalidReason::FactorSourceAlreadyPresent)
    );
}

#[test]
fn validate_threshold_for_ever_invalid() {
    let sut = SUT::with_factors(
        RoleKind::Primary,
        1,
        vec![
            FactorSourceID::sample_ledger(),
            FactorSourceID::sample_ledger(),
        ],
        vec![],
    );
    let res = sut.validate();
    assert_eq!(
        res,
        MutRes::forever_invalid(ForeverInvalidReason::FactorSourceAlreadyPresent)
    );
}

#[test]
fn confirmation_validate_basic_violation() {
    let sut = SUT::with_factors(
        RoleKind::Confirmation,
        1,
        vec![],
        vec![FactorSourceID::sample_ledger()],
    );
    let res = sut.validate();
    assert_eq!(
        res,
        MutRes::basic_violation(BasicViolation::ConfirmationCannotSetThreshold)
    );
}

#[test]
fn recovery_validate_basic_violation() {
    let sut = SUT::with_factors(
        RoleKind::Recovery,
        1,
        vec![],
        vec![FactorSourceID::sample_ledger()],
    );
    let res = sut.validate();
    assert_eq!(
        res,
        MutRes::basic_violation(BasicViolation::RecoveryCannotSetThreshold)
    );
}

#[test]
fn primary_validate_not_yet_valid_for_threshold_greater_than_threshold_factors() {
    let sut = SUT::with_factors(
        RoleKind::Primary,
        1,
        vec![],
        vec![FactorSourceID::sample_ledger()],
    );
    let res = sut.validate();
    assert_eq!(
        res,
        MutRes::not_yet_valid(ThresholdHigherThanThresholdFactorsLen)
    );
}

#[cfg(test)]
mod recovery_in_isolation {

    use super::*;

    fn role() -> RoleKind {
        RoleKind::Recovery
    }

    fn make() -> SUT {
        SUT::new(role())
    }

    fn list() -> FactorListKind {
        FactorListKind::Override
    }

    fn sample() -> FactorSourceID {
        FactorSourceID::sample_device()
    }

    #[test]
    fn duplicates_not_allowed() {
        test_duplicates_not_allowed(make(), list(), sample())
    }

    #[test]
    fn validation_for_addition_of_factor_source_of_kind_to_list_is_err_for_threshold() {
        let sut = make();
        let res = sut.validation_for_addition_of_factor_source_of_kind_to_list(
            FactorSourceKind::Device,
            FactorListKind::Threshold,
        );
        assert_eq!(
            res,
            MutRes::forever_invalid(ForeverInvalidReason::threshold_list_not_supported_for_role(
                role()
            ))
        );
    }

    #[test]
    fn validation_for_addition_of_factor_source_of_kind_to_list() {
        use FactorSourceKind::*;
        let sut = make();
        let not_ok = |kind: FactorSourceKind| {
            let res = sut.validation_for_addition_of_factor_source_of_kind_to_list(kind, list());
            assert!(res.is_err());
        };
        let ok = |kind: FactorSourceKind| {
            let res = sut.validation_for_addition_of_factor_source_of_kind_to_list(kind, list());
            assert!(res.is_ok());
        };
        ok(Device);
        ok(LedgerHQHardwareWallet);
        ok(ArculusCard);
        ok(TrustedContact);
        ok(OffDeviceMnemonic);

        not_ok(Passphrase);
        not_ok(SecurityQuestions);
    }

    #[test]
    fn set_threshold_is_unsupported() {
        let mut sut = make();
        assert_eq!(
            sut.set_threshold(1),
            MutRes::basic_violation(BasicViolation::RecoveryCannotSetThreshold)
        );
    }

    #[test]
    fn cannot_add_factors_to_threshold() {
        let mut sut = make();
        let res = sut.add_factor_source_to_list(sample(), FactorListKind::Threshold);
        assert_eq!(
            res,
            Err(Validation::ForeverInvalid(
                ForeverInvalidReason::RecoveryRoleThresholdFactorsNotSupported
            ))
        );
    }

    mod device_in_isolation {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_device()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_device_other()
        }

        #[test]
        fn allowed_as_first_and_only() {
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::recovery_with_factors([sample()])
            );
        }

        #[test]
        fn two_of_same_kind_allowed() {
            // TODO: Ask Matt
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::recovery_with_factors([sample(), sample_other()],)
            );
        }

        #[test]
        fn validation_for_addition_of_factor_source_for_each() {
            let sut = make();
            let xs = sut.validation_for_addition_of_factor_source_for_each(
                list(),
                &IndexSet::from_iter([sample(), sample_other()]),
            );
            assert_eq!(
                xs.into_iter().collect::<Vec<_>>(),
                vec![
                    FactorSourceInRoleBuilderValidationStatus::ok(RoleKind::Recovery, sample()),
                    FactorSourceInRoleBuilderValidationStatus::ok(
                        RoleKind::Recovery,
                        sample_other(),
                    )
                ]
            );
        }
    }

    mod ledger_in_isolation {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_ledger()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_ledger_other()
        }

        #[test]
        fn allowed_as_first_and_only() {
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::recovery_with_factors([sample()],)
            );
        }

        #[test]
        fn two_of_same_kind_allowed() {
            // TODO: Ask Matt
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::recovery_with_factors([sample(), sample_other()])
            );
        }
    }

    mod arculus_in_isolation {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_arculus()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_arculus_other()
        }

        #[test]
        fn allowed_as_first_and_only() {
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::recovery_with_factors([sample(),])
            );
        }

        #[test]
        fn two_of_same_kind_allowed() {
            // TODO: Ask Matt
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::recovery_with_factors([sample(), sample_other()])
            );
        }
    }

    mod passphrase_in_isolation {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_passphrase()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_passphrase_other()
        }

        #[test]
        fn allowed_as_first_and_only() {
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::recovery_with_factors([sample()])
            );
        }

        #[test]
        fn two_of_same_kind_allowed() {
            // TODO: Ask Matt
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::recovery_with_factors([sample(), sample_other()])
            );
        }
    }

    mod trusted_contact_in_isolation {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_trusted_contact()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_trusted_contact_other()
        }

        #[test]
        fn allowed_as_first_and_only() {
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::recovery_with_factors([sample(),])
            );
        }

        #[test]
        fn two_of_same_kind_allowed() {
            // TODO: Ask Matt
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::recovery_with_factors([sample(), sample_other()])
            );
        }
    }

    mod password_in_isolation {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_password()
        }

        #[test]
        fn unsupported() {
            // Arrange
            let mut sut = make();

            // Act
            let res = sut.add_factor_source_to_list(sample(), list());

            // Assert
            assert_eq!(
                res,
                MutRes::forever_invalid(ForeverInvalidReason::RecoveryRolePasswordNotSupported)
            );
        }

        #[test]
        fn valid_then_invalid_because_unsupported() {
            // Arrange
            let mut sut = make();

            sut.add_factor_source_to_list(FactorSourceID::sample_ledger(), list())
                .unwrap();
            sut.add_factor_source_to_list(FactorSourceID::sample_arculus(), list())
                .unwrap();

            // Act
            let res = sut.add_factor_source_to_list(sample(), list());

            // Assert
            assert_eq!(
                res,
                MutRes::forever_invalid(ForeverInvalidReason::RecoveryRolePasswordNotSupported)
            );
        }
    }

    mod security_questions_in_isolation {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_security_questions()
        }
        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_security_questions_other()
        }

        #[test]
        fn unsupported() {
            // Arrange
            let mut sut = make();

            // Act
            let res = sut.add_factor_source_to_list(sample(), list());

            // Assert
            assert_eq!(
                res,
                MutRes::forever_invalid(
                    ForeverInvalidReason::RecoveryRoleSecurityQuestionsNotSupported
                )
            );
        }

        #[test]
        fn valid_then_invalid_because_unsupported() {
            // Arrange
            let mut sut = make();

            sut.add_factor_source_to_list(FactorSourceID::sample_ledger(), list())
                .unwrap();
            sut.add_factor_source_to_list(FactorSourceID::sample_arculus(), list())
                .unwrap();

            // Act
            let res = sut.add_factor_source_to_list(sample_other(), list());

            // Assert
            let reason = ForeverInvalidReason::RecoveryRoleSecurityQuestionsNotSupported;
            let err = MutRes::forever_invalid(reason);
            assert_eq!(res, err);

            // .. erroneous action above did not change the state of the builder (SUT),
            // so we can build and `sample` is not present in the built result.
            assert_eq!(
                sut.build(),
                Ok(RoleWithFactorSourceIds::recovery_with_factors([
                    FactorSourceID::sample_ledger(),
                    FactorSourceID::sample_arculus()
                ]))
            );
        }
    }
}

#[cfg(test)]
mod confirmation_in_isolation {

    use super::*;

    fn role() -> RoleKind {
        RoleKind::Confirmation
    }

    fn make() -> SUT {
        SUT::new(role())
    }

    fn list() -> FactorListKind {
        FactorListKind::Override
    }

    fn sample() -> FactorSourceID {
        FactorSourceID::sample_device()
    }

    #[test]
    fn validation_for_addition_of_factor_source_of_kind_to_list_is_err_for_threshold() {
        let sut = make();
        let res = sut.validation_for_addition_of_factor_source_of_kind_to_list(
            FactorSourceKind::Device,
            FactorListKind::Threshold,
        );
        assert_eq!(
            res,
            MutRes::forever_invalid(ForeverInvalidReason::threshold_list_not_supported_for_role(
                role()
            ))
        );
    }

    #[test]
    fn validation_for_addition_of_factor_source_of_kind_to_list() {
        use FactorSourceKind::*;
        let sut = make();
        let not_ok = |kind: FactorSourceKind| {
            let res = sut.validation_for_addition_of_factor_source_of_kind_to_list(kind, list());
            assert!(res.is_err());
        };
        let ok = |kind: FactorSourceKind| {
            let res = sut.validation_for_addition_of_factor_source_of_kind_to_list(kind, list());
            assert!(res.is_ok());
        };
        ok(Device);
        ok(LedgerHQHardwareWallet);
        ok(ArculusCard);
        ok(SecurityQuestions);
        ok(Passphrase);
        ok(OffDeviceMnemonic);
        not_ok(TrustedContact);
    }

    #[test]
    fn duplicates_not_allowed() {
        test_duplicates_not_allowed(make(), list(), sample())
    }

    #[test]
    fn cannot_add_factors_to_threshold() {
        let mut sut = make();
        let res = sut.add_factor_source_to_list(sample(), FactorListKind::Threshold);
        assert_eq!(
            res,
            Err(Validation::ForeverInvalid(
                ForeverInvalidReason::ConfirmationRoleThresholdFactorsNotSupported
            ))
        );
    }

    mod device_in_isolation {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_device()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_device_other()
        }

        #[test]
        fn set_threshold_is_unsupported() {
            let mut sut = make();
            assert_eq!(
                sut.set_threshold(1),
                MutRes::basic_violation(BasicViolation::ConfirmationCannotSetThreshold)
            );
        }

        #[test]
        fn allowed_as_first_and_only() {
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::confirmation_with_factors([sample()])
            );
        }

        #[test]
        fn two_of_same_kind_allowed() {
            // TODO: Ask Matt
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();

            // Assert
            let built = sut.build().unwrap();
            assert!(built.get_threshold_factors().is_empty());
            assert_eq!(
                built,
                RoleWithFactorSourceIds::confirmation_with_factors([sample(), sample_other()])
            );
        }
    }

    mod ledger_in_isolation {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_ledger()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_ledger_other()
        }

        #[test]
        fn allowed_as_first_and_only() {
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::confirmation_with_factors([sample(),])
            );
        }

        #[test]
        fn two_of_same_kind_allowed() {
            // TODO: Ask Matt
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::confirmation_with_factors([sample(), sample_other()])
            );
        }
    }

    mod arculus_in_isolation {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_arculus()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_arculus_other()
        }

        #[test]
        fn allowed_as_first_and_only() {
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::confirmation_with_factors([sample(),])
            );
        }

        #[test]
        fn two_of_same_kind_allowed() {
            // TODO: Ask Matt
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::confirmation_with_factors([sample(), sample_other()])
            );
        }
    }

    mod passphrase_in_isolation {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_passphrase()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_passphrase_other()
        }

        #[test]
        fn allowed_as_first_and_only() {
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::confirmation_with_factors([sample(),])
            );
        }

        #[test]
        fn two_of_same_kind_allowed() {
            // TODO: Ask Matt
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::confirmation_with_factors([sample(), sample_other()])
            );
        }
    }

    mod trusted_contact_in_isolation {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_trusted_contact()
        }

        #[test]
        fn unsupported() {
            // Arrange
            let mut sut = make();

            // Act
            let res = sut.add_factor_source_to_list(sample(), list());

            // Assert
            assert_eq!(
                res,
                MutRes::forever_invalid(
                    ForeverInvalidReason::ConfirmationRoleTrustedContactNotSupported
                )
            );
        }

        #[test]
        fn valid_then_invalid_because_unsupported() {
            // Arrange
            let mut sut = make();

            sut.add_factor_source_to_list(FactorSourceID::sample_ledger(), list())
                .unwrap();
            sut.add_factor_source_to_list(FactorSourceID::sample_arculus(), list())
                .unwrap();

            // Act
            let res = sut.add_factor_source_to_list(sample(), list());

            // Assert
            assert_eq!(
                res,
                MutRes::forever_invalid(
                    ForeverInvalidReason::ConfirmationRoleTrustedContactNotSupported
                )
            );
        }
    }

    mod password_in_isolation {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_password()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_password_other()
        }

        #[test]
        fn allowed_as_first_and_only() {
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::confirmation_with_factors([sample(),])
            );
        }

        #[test]
        fn two_of_same_kind_allowed() {
            // TODO: Ask Matt
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample(), list()).unwrap();
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();

            // Assert
            assert_eq!(
                sut.build().unwrap(),
                RoleWithFactorSourceIds::confirmation_with_factors([sample(), sample_other()])
            );
        }
    }
}

#[cfg(test)]
mod primary_in_isolation {

    use super::*;

    fn role() -> RoleKind {
        RoleKind::Primary
    }

    fn make() -> SUT {
        SUT::new(role())
    }

    #[cfg(test)]
    mod threshold_suite {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_device()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_ledger()
        }

        fn sample_third() -> FactorSourceID {
            FactorSourceID::sample_arculus()
        }

        fn list() -> FactorListKind {
            FactorListKind::Threshold
        }

        #[test]
        fn remove_lowers_threshold_from_1_to_0() {
            let mut sut = make();
            let fs = sample();
            sut.add_factor_source_to_list(fs, list()).unwrap();
            sut.set_threshold(1).unwrap();
            assert_eq!(sut.get_threshold(), 1);
            assert_eq!(
                sut.remove_factor_source(&fs),
                Err(Validation::NotYetValid(RoleMustHaveAtLeastOneFactor))
            );
            assert_eq!(sut.get_threshold(), 0);
        }

        #[test]
        fn remove_lowers_threshold_from_3_to_1() {
            let mut sut = make();
            let fs0 = sample();
            let fs1 = sample_other();
            sut.add_factor_source_to_list(fs0, list()).unwrap();
            sut.add_factor_source_to_list(fs1, list()).unwrap();
            sut.add_factor_source_to_list(FactorSourceID::sample_arculus_other(), list())
                .unwrap();
            sut.set_threshold(3).unwrap();
            assert_eq!(sut.get_threshold(), 3);
            sut.remove_factor_source(&fs0).unwrap();
            sut.remove_factor_source(&fs1).unwrap();
            assert_eq!(sut.get_threshold(), 1);
        }

        #[test]
        fn remove_from_override_does_not_change_threshold() {
            let mut sut = make();
            sut.add_factor_source_to_list(sample(), list()).unwrap();
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();
            let fs = FactorSourceID::sample_arculus_other();
            sut.add_factor_source_to_list(fs, FactorListKind::Override)
                .unwrap();
            sut.set_threshold(2).unwrap();
            assert_eq!(sut.get_threshold(), 2);
            sut.remove_factor_source(&fs).unwrap();
            assert_eq!(sut.get_threshold(), 2);

            let built = sut.build().unwrap();
            assert_eq!(built.get_threshold(), 2);

            assert_eq!(built.role(), RoleKind::Primary);

            assert_eq!(
                built.get_threshold_factors(),
                &vec![sample(), sample_other()]
            );

            assert_eq!(built.get_override_factors(), &Vec::new());
        }

        #[test]
        fn one_factor_then_set_threshold_to_one_is_ok() {
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();
            sut.set_threshold(1).unwrap();

            // Assert
            let expected = RoleWithFactorSourceIds::primary_with_factors(1, [sample_other()], []);
            assert_eq!(sut.build().unwrap(), expected);
        }

        #[test]
        fn zero_factor_then_set_threshold_to_one_is_not_yet_valid_then_add_one_factor_is_ok() {
            // Arrange
            let mut sut = make();

            // Act
            assert_eq!(
                sut.set_threshold(1),
                Err(Validation::NotYetValid(
                    ThresholdHigherThanThresholdFactorsLen
                ))
            );
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();

            // Assert
            let expected = RoleWithFactorSourceIds::primary_with_factors(1, [sample_other()], []);
            assert_eq!(sut.build().unwrap(), expected);
        }

        #[test]
        fn zero_factor_then_set_threshold_to_two_is_not_yet_valid_then_add_two_factor_is_ok() {
            // Arrange
            let mut sut = make();

            // Act
            assert_eq!(
                sut.set_threshold(2),
                Err(Validation::NotYetValid(
                    ThresholdHigherThanThresholdFactorsLen
                ))
            );
            sut.add_factor_source_to_list(sample(), list()).unwrap();

            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();

            // Assert
            let expected =
                RoleWithFactorSourceIds::primary_with_factors(2, [sample(), sample_other()], []);
            assert_eq!(sut.build().unwrap(), expected);
        }

        #[test]
        fn add_two_factors_then_set_threshold_to_two_is_ok() {
            // Arrange
            let mut sut = make();

            sut.add_factor_source_to_list(sample(), list()).unwrap();
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();

            // Act
            assert_eq!(sut.set_threshold(2), Ok(()));

            // Assert
            let expected =
                RoleWithFactorSourceIds::primary_with_factors(2, [sample(), sample_other()], []);
            assert_eq!(sut.build().unwrap(), expected);
        }

        #[test]
        fn add_two_factors_then_set_threshold_to_three_is_not_yet_valid_then_add_third_factor_is_ok(
        ) {
            // Arrange
            let mut sut = make();

            sut.add_factor_source_to_list(sample(), list()).unwrap();
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();

            // Act
            assert_eq!(
                sut.set_threshold(3),
                Err(Validation::NotYetValid(
                    ThresholdHigherThanThresholdFactorsLen
                ))
            );

            sut.add_factor_source_to_list(sample_third(), list())
                .unwrap();

            // Assert
            let expected = RoleWithFactorSourceIds::primary_with_factors(
                3,
                [sample(), sample_other(), sample_third()],
                [],
            );
            assert_eq!(sut.build().unwrap(), expected);
        }

        #[test]
        fn one_factors_set_threshold_of_one_is_ok() {
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample_other(), list())
                .unwrap();
            sut.set_threshold(1).unwrap();

            // Assert
            let expected = RoleWithFactorSourceIds::primary_with_factors(1, [sample_other()], []);
            assert_eq!(sut.build().unwrap(), expected);
        }

        #[test]
        fn one_override_factors_set_threshold_to_one_is_not_yet_valid() {
            // Arrange
            let mut sut = make();

            // Act
            sut.add_factor_source_to_list(sample_other(), FactorListKind::Override)
                .unwrap();
            assert_eq!(
                sut.set_threshold(1),
                Err(Validation::NotYetValid(
                    ThresholdHigherThanThresholdFactorsLen
                ))
            );

            // Assert

            assert_eq!(
                sut.build(),
                Err(Validation::NotYetValid(
                    ThresholdHigherThanThresholdFactorsLen
                ))
            );
        }

        #[test]
        fn validation_for_addition_of_factor_source_for_each_before_after_adding_a_factor() {
            let mut sut = make();
            let fs0 = FactorSourceID::sample_ledger();
            let fs1 = FactorSourceID::sample_password();
            let fs2 = FactorSourceID::sample_arculus();
            let xs = sut.validation_for_addition_of_factor_source_for_each(
                list(),
                &IndexSet::from_iter([fs0, fs1, fs2]),
            );
            assert_eq!(
                    xs.into_iter().collect::<Vec<_>>(),
                    vec![
                        FactorSourceInRoleBuilderValidationStatus::ok(
                            RoleKind::Primary,
                            fs0,
                        ),
                        FactorSourceInRoleBuilderValidationStatus::not_yet_valid(
                            RoleKind::Primary,
                            fs1,
                                NotYetValidReason::PrimaryRoleWithPasswordInThresholdListMustHaveAnotherFactor
                        ),
                        FactorSourceInRoleBuilderValidationStatus::ok(
                            RoleKind::Primary,
                            fs2,
                        ),
                    ]
                );
            _ = sut.add_factor_source_to_list(fs0, list());
            _ = sut.set_threshold(2);

            let xs = sut.validation_for_addition_of_factor_source_for_each(
                list(),
                &IndexSet::from_iter([fs0, fs1, fs2]),
            );
            assert_eq!(
                xs.into_iter().collect::<Vec<_>>(),
                vec![
                    FactorSourceInRoleBuilderValidationStatus::forever_invalid(
                        RoleKind::Primary,
                        fs0,
                        ForeverInvalidReason::FactorSourceAlreadyPresent
                    ),
                    FactorSourceInRoleBuilderValidationStatus::ok(RoleKind::Primary, fs1,),
                    FactorSourceInRoleBuilderValidationStatus::ok(RoleKind::Primary, fs2,),
                ]
            );
        }
    }

    #[cfg(test)]
    mod password {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_password()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_password_other()
        }

        #[test]
        fn test_suite_prerequisite() {
            assert_eq!(sample(), sample());
            assert_eq!(sample_other(), sample_other());
            assert_ne!(sample(), sample_other());
        }

        mod threshold_in_isolation {
            use super::*;

            fn list() -> FactorListKind {
                FactorListKind::Threshold
            }

            #[test]
            fn duplicates_not_allowed() {
                let mut sut = make();
                sut.add_factor_source_to_list(
                    FactorSourceID::sample_device(),
                    FactorListKind::Threshold,
                )
                .unwrap();
                _ = sut.set_threshold(2);
                test_duplicates_not_allowed(sut, list(), sample());
            }

            #[test]
            fn alone_is_not_ok() {
                // Arrange
                let mut sut = make();

                // Act
                let res = sut.add_factor_source_to_list(sample(), list());

                // Assert
                assert_eq!(
                        res,
                        MutRes::not_yet_valid(
                            NotYetValidReason::PrimaryRoleWithPasswordInThresholdListMustHaveAnotherFactor
                        )
                    );
            }

            #[test]
            fn validation_for_addition_of_factor_source_of_kind_to_list() {
                use FactorSourceKind::*;

                let not_ok = |kind: FactorSourceKind| {
                    let sut = make();
                    let res =
                        sut.validation_for_addition_of_factor_source_of_kind_to_list(kind, list());
                    assert!(res.is_err());
                };

                let ok_with = |kind: FactorSourceKind, setup: fn(&mut SUT)| {
                    let mut sut = make();
                    setup(&mut sut);
                    let res =
                        sut.validation_for_addition_of_factor_source_of_kind_to_list(kind, list());
                    assert!(res.is_ok());
                };
                let ok = |kind: FactorSourceKind| {
                    ok_with(kind, |_| {});
                };

                ok(LedgerHQHardwareWallet);
                ok(ArculusCard);
                ok(OffDeviceMnemonic);

                ok_with(Device, |sut| {
                    sut.add_factor_source_to_list(FactorSourceID::sample_ledger(), list())
                        .unwrap();
                });
                ok_with(Passphrase, |sut| {
                    sut.add_factor_source_to_list(FactorSourceID::sample_device(), list())
                        .unwrap();
                    _ = sut.set_threshold(2);
                });

                not_ok(SecurityQuestions);
                not_ok(TrustedContact);
            }
        }

        mod override_in_isolation {
            use super::*;

            fn list() -> FactorListKind {
                FactorListKind::Override
            }

            #[test]
            fn unsupported() {
                // Arrange
                let mut sut = make();

                // Act
                let res = sut.add_factor_source_to_list(sample(), list());

                // Assert
                assert_eq!(
                    res,
                    MutRes::forever_invalid(
                        ForeverInvalidReason::PrimaryCannotHavePasswordInOverrideList
                    )
                );
            }

            #[test]
            fn valid_then_invalid_because_unsupported() {
                // Arrange
                let mut sut = make();
                sut.add_factor_source_to_list(
                    FactorSourceID::sample_device(),
                    FactorListKind::Threshold,
                )
                .unwrap();
                sut.add_factor_source_to_list(FactorSourceID::sample_ledger(), list())
                    .unwrap();
                sut.add_factor_source_to_list(FactorSourceID::sample_arculus(), list())
                    .unwrap();

                // Act
                let res = sut.add_factor_source_to_list(sample(), list());

                // Assert
                assert_eq!(
                    res,
                    MutRes::forever_invalid(
                        ForeverInvalidReason::PrimaryCannotHavePasswordInOverrideList
                    )
                );
            }

            #[test]
            fn validation_for_addition_of_factor_source_of_kind_to_list() {
                use FactorSourceKind::*;

                let not_ok = |kind: FactorSourceKind| {
                    let sut = make();
                    let res =
                        sut.validation_for_addition_of_factor_source_of_kind_to_list(kind, list());
                    assert!(res.is_err());
                };

                let ok_with = |kind: FactorSourceKind, setup: fn(&mut SUT)| {
                    let mut sut = make();
                    setup(&mut sut);
                    let res =
                        sut.validation_for_addition_of_factor_source_of_kind_to_list(kind, list());
                    assert!(res.is_ok());
                };
                let ok = |kind: FactorSourceKind| {
                    ok_with(kind, |_| {});
                };

                ok(LedgerHQHardwareWallet);
                ok(ArculusCard);
                ok(OffDeviceMnemonic);

                ok_with(Device, |sut| {
                    sut.add_factor_source_to_list(FactorSourceID::sample_ledger(), list())
                        .unwrap();
                });

                not_ok(Passphrase);

                not_ok(SecurityQuestions);
                not_ok(TrustedContact);
            }
        }
    }

    #[cfg(test)]
    mod ledger {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_ledger()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_ledger_other()
        }

        #[test]
        fn test_suite_prerequisite() {
            assert_eq!(sample(), sample());
            assert_eq!(sample_other(), sample_other());
            assert_ne!(sample(), sample_other());
        }

        mod threshold_in_isolation {
            use super::*;
            fn list() -> FactorListKind {
                FactorListKind::Threshold
            }

            #[test]
            fn duplicates_not_allowed() {
                test_duplicates_not_allowed(make(), list(), sample());
            }

            #[test]
            fn one_is_ok() {
                // Arrange
                let mut sut = make();

                // Act
                sut.add_factor_source_to_list(sample(), list()).unwrap();
                sut.set_threshold(1).unwrap();

                // Assert
                let expected = RoleWithFactorSourceIds::primary_with_factors(1, [sample()], []);
                assert_eq!(sut.build().unwrap(), expected);
            }

            #[test]
            fn one_with_threshold_of_zero_is_err() {
                // Arrange
                let mut sut = make();

                // Act
                sut.add_factor_source_to_list(sample(), list()).unwrap();

                // Assert
                assert_eq!(
                    sut.build(),
                    RoleBuilderBuildResult::Err(RoleBuilderValidation::NotYetValid(
                        NotYetValidReason::PrimaryRoleWithThresholdCannotBeZeroWithFactors
                    ))
                );
            }

            #[test]
            fn two_different_is_ok() {
                // Arrange
                let mut sut = make();

                // Act
                sut.add_factor_source_to_list(sample(), list()).unwrap();
                sut.add_factor_source_to_list(sample_other(), list())
                    .unwrap();
                sut.set_threshold(2).unwrap();

                // Assert
                let expected = RoleWithFactorSourceIds::primary_with_factors(
                    2,
                    [sample(), sample_other()],
                    [],
                );
                assert_eq!(sut.build().unwrap(), expected);
            }
        }

        mod override_in_isolation {
            use super::*;
            fn list() -> FactorListKind {
                FactorListKind::Override
            }

            #[test]
            fn duplicates_not_allowed() {
                test_duplicates_not_allowed(make(), list(), sample());
            }

            #[test]
            fn one_is_ok() {
                // Arrange
                let mut sut = make();

                // Act
                sut.add_factor_source_to_list(sample(), list()).unwrap();

                // Assert
                let expected = RoleWithFactorSourceIds::primary_with_factors(0, [], [sample()]);
                assert_eq!(sut.build().unwrap(), expected);
            }

            #[test]
            fn two_different_is_ok() {
                // Arrange
                let mut sut = make();

                // Act
                sut.add_factor_source_to_list(sample(), list()).unwrap();
                sut.add_factor_source_to_list(sample_other(), list())
                    .unwrap();

                // Assert
                let expected = RoleWithFactorSourceIds::primary_with_factors(
                    0,
                    [],
                    [sample(), sample_other()],
                );
                assert_eq!(sut.build().unwrap(), expected);
            }
        }
    }

    #[cfg(test)]
    mod arculus {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_arculus()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_arculus_other()
        }

        #[test]
        fn test_suite_prerequisite() {
            assert_eq!(sample(), sample());
            assert_eq!(sample_other(), sample_other());
            assert_ne!(sample(), sample_other());
        }

        mod threshold_in_isolation {
            use super::*;
            fn list() -> FactorListKind {
                FactorListKind::Threshold
            }

            #[test]
            fn duplicates_not_allowed() {
                test_duplicates_not_allowed(make(), list(), sample());
            }

            #[test]
            fn one_is_ok() {
                // Arrange
                let mut sut = make();

                // Act
                sut.add_factor_source_to_list(sample(), list()).unwrap();
                sut.set_threshold(1).unwrap();

                // Assert
                let expected = RoleWithFactorSourceIds::primary_with_factors(1, [sample()], []);
                assert_eq!(sut.build().unwrap(), expected);
            }

            #[test]
            fn two_different_is_ok() {
                // Arrange
                let mut sut = make();

                // Act
                sut.add_factor_source_to_list(sample(), list()).unwrap();
                sut.add_factor_source_to_list(sample_other(), list())
                    .unwrap();
                sut.set_threshold(1).unwrap();

                // Assert
                let expected = RoleWithFactorSourceIds::primary_with_factors(
                    1,
                    [sample(), sample_other()],
                    [],
                );
                assert_eq!(sut.build().unwrap(), expected);
            }
        }

        mod override_in_isolation {
            use super::*;
            fn list() -> FactorListKind {
                FactorListKind::Override
            }

            #[test]
            fn duplicates_not_allowed() {
                test_duplicates_not_allowed(make(), list(), sample());
            }

            #[test]
            fn one_is_ok() {
                // Arrange
                let mut sut = make();

                // Act
                sut.add_factor_source_to_list(sample(), list()).unwrap();

                // Assert
                let expected = RoleWithFactorSourceIds::primary_with_factors(0, [], [sample()]);
                assert_eq!(sut.build().unwrap(), expected);
            }

            #[test]
            fn two_different_is_ok() {
                // Arrange
                let mut sut = make();

                // Act
                sut.add_factor_source_to_list(sample(), list()).unwrap();
                sut.add_factor_source_to_list(sample_other(), list())
                    .unwrap();

                // Assert
                let expected = RoleWithFactorSourceIds::primary_with_factors(
                    0,
                    [],
                    [sample(), sample_other()],
                );
                assert_eq!(sut.build().unwrap(), expected);
            }
        }
    }

    #[cfg(test)]
    mod device_factor_source {
        use super::*;

        fn sample() -> FactorSourceID {
            FactorSourceID::sample_device()
        }

        fn sample_other() -> FactorSourceID {
            FactorSourceID::sample_device_other()
        }

        #[test]
        fn test_suite_prerequisite() {
            assert_eq!(sample(), sample());
            assert_eq!(sample_other(), sample_other());
            assert_ne!(sample(), sample_other());
        }

        #[cfg(test)]
        mod threshold_in_isolation {
            use super::*;

            fn list() -> FactorListKind {
                FactorListKind::Threshold
            }

            #[test]
            fn duplicates_not_allowed() {
                test_duplicates_not_allowed(make(), list(), sample())
            }

            #[test]
            fn one_is_ok() {
                // Arrange
                let mut sut = make();

                // Act
                sut.add_factor_source_to_list(sample(), list()).unwrap();
                sut.set_threshold(1).unwrap();

                // Assert
                let expected = RoleWithFactorSourceIds::primary_with_factors(1, [sample()], []);
                assert_eq!(sut.build().unwrap(), expected);
            }

            #[test]
            fn two_different_is_err() {
                // Arrange
                let mut sut = make();

                sut.add_factor_source_to_list(sample(), list()).unwrap();

                // Act
                let res = sut.add_factor_source_to_list(sample_other(), list());

                // Assert
                assert!(matches!(
                    res,
                    MutRes::Err(Validation::ForeverInvalid(
                        ForeverInvalidReason::PrimaryCannotHaveMultipleDevices
                    ))
                ));
            }
        }

        mod override_in_isolation {

            use super::*;

            fn list() -> FactorListKind {
                FactorListKind::Override
            }

            #[test]
            fn duplicates_not_allowed() {
                test_duplicates_not_allowed(make(), list(), sample())
            }

            #[test]
            fn one_is_ok() {
                // Arrange
                let mut sut = make();

                // Act
                sut.add_factor_source_to_list(sample(), list()).unwrap();

                // Assert
                let expected = RoleWithFactorSourceIds::primary_with_factors(0, [], [sample()]);
                assert_eq!(sut.build().unwrap(), expected);
            }
        }
    }
}
