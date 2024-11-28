use crate::prelude::*;

pub type PrimaryRoleBuilder = RoleBuilder<{ ROLE_PRIMARY }>;
pub type RecoveryRoleBuilder = RoleBuilder<{ ROLE_RECOVERY }>;
pub type ConfirmationRoleBuilder = RoleBuilder<{ ROLE_CONFIRMATION }>;

#[cfg(test)]
impl PrimaryRoleWithFactorSourceIds {
    pub(crate) fn primary_with_factors(
        threshold: u8,
        threshold_factors: impl IntoIterator<Item = FactorSourceID>,
        override_factors: impl IntoIterator<Item = FactorSourceID>,
    ) -> Self {
        Self::with_factors(threshold, threshold_factors, override_factors)
    }
}

impl RecoveryRoleWithFactorSourceIds {
    pub(crate) fn recovery_with_factors(
        override_factors: impl IntoIterator<Item = FactorSourceID>,
    ) -> Self {
        Self::with_factors(0, vec![], override_factors)
    }
}

impl ConfirmationRoleWithFactorSourceIds {
    pub(crate) fn confirmation_with_factors(
        override_factors: impl IntoIterator<Item = FactorSourceID>,
    ) -> Self {
        Self::with_factors(0, vec![], override_factors)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
pub enum RoleBuilderValidation {
    #[error("Basic violation: {0}")]
    BasicViolation(#[from] BasicViolation),

    #[error("Forever invalid: {0}")]
    ForeverInvalid(#[from] ForeverInvalidReason),

    #[error("Not yet valid: {0}")]
    NotYetValid(#[from] NotYetValidReason),
}
type Validation = RoleBuilderValidation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
pub enum BasicViolation {
    /// e.g. tried to remove a factor source which was not found.
    #[error("FactorSourceID not found")]
    FactorSourceNotFound,

    #[error("Recovery cannot set threshold")]
    RecoveryCannotSetThreshold,

    #[error("Confirmation cannot set threshold")]
    ConfirmationCannotSetThreshold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
pub enum NotYetValidReason {
    #[error("Role must have at least one factor")]
    RoleMustHaveAtLeastOneFactor,

    #[error("Primary role with password in threshold list must have another factor")]
    PrimaryRoleWithPasswordInThresholdListMustHaveAnotherFactor,

    #[error("Primary role with threshold factors cannot have a threshold of zero")]
    PrimaryRoleWithThresholdCannotBeZeroWithFactors,

    #[error("Primary role with password in threshold list must have threshold greater than one")]
    PrimaryRoleWithPasswordInThresholdListMustThresholdGreaterThanOne,

    #[error("Threshold higher than threshold factors len")]
    ThresholdHigherThanThresholdFactorsLen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
pub enum ForeverInvalidReason {
    #[error("Factor source already present")]
    FactorSourceAlreadyPresent,

    #[error("Primary role cannot have multiple devices")]
    PrimaryCannotHaveMultipleDevices,

    #[error("Primary role cannot have password in override list")]
    PrimaryCannotHavePasswordInOverrideList,

    #[error("Primary role cannot contain Security Questions")]
    PrimaryCannotContainSecurityQuestions,

    #[error("Primary role cannot contain Trusted Contact")]
    PrimaryCannotContainTrustedContact,

    #[error("Recovery role threshold list not supported")]
    RecoveryRoleThresholdFactorsNotSupported,

    #[error("Recovery role Security Questions not supported")]
    RecoveryRoleSecurityQuestionsNotSupported,

    #[error("Recovery role password not supported")]
    RecoveryRolePasswordNotSupported,

    #[error("Confirmation role threshold list not supported")]
    ConfirmationRoleThresholdFactorsNotSupported,

    #[error("Confirmation role cannot contain Trusted Contact")]
    ConfirmationRoleTrustedContactNotSupported,
}

pub(crate) trait FromForeverInvalid {
    fn forever_invalid(reason: ForeverInvalidReason) -> Self;
}
impl<T> FromForeverInvalid for std::result::Result<T, RoleBuilderValidation> {
    fn forever_invalid(reason: ForeverInvalidReason) -> Self {
        Err(Validation::ForeverInvalid(reason))
    }
}

pub(crate) trait FromNotYetValid {
    fn not_yet_valid(reason: NotYetValidReason) -> Self;
}
impl<T> FromNotYetValid for std::result::Result<T, RoleBuilderValidation> {
    fn not_yet_valid(reason: NotYetValidReason) -> Self {
        Err(Validation::NotYetValid(reason))
    }
}

pub(crate) trait FromBasicViolation {
    fn basic_violation(reason: BasicViolation) -> Self;
}
impl<T> FromBasicViolation for std::result::Result<T, RoleBuilderValidation> {
    fn basic_violation(reason: BasicViolation) -> Self {
        Err(Validation::BasicViolation(reason))
    }
}

impl ForeverInvalidReason {
    pub(crate) fn threshold_list_not_supported_for_role(role: RoleKind) -> Self {
        match role {
            RoleKind::Recovery => Self::RecoveryRoleThresholdFactorsNotSupported,
            RoleKind::Confirmation => Self::ConfirmationRoleThresholdFactorsNotSupported,
            RoleKind::Primary => {
                unreachable!("Primary role DOES support threshold list. This is programmer error.")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactorSourceInRoleBuilderValidationStatus {
    pub role: RoleKind,
    pub factor_source_id: FactorSourceID,
    pub validation: RoleBuilderMutateResult,
}

impl FactorSourceInRoleBuilderValidationStatus {
    pub(crate) fn new(
        role: RoleKind,
        factor_source_id: FactorSourceID,
        validation: RoleBuilderMutateResult,
    ) -> Self {
        Self {
            role,
            factor_source_id,
            validation,
        }
    }
}

#[cfg(test)]
impl FactorSourceInRoleBuilderValidationStatus {
    pub(crate) fn ok(role: RoleKind, factor_source_id: FactorSourceID) -> Self {
        Self::new(role, factor_source_id, Ok(()))
    }

    pub(crate) fn forever_invalid(
        role: RoleKind,
        factor_source_id: FactorSourceID,
        reason: ForeverInvalidReason,
    ) -> Self {
        Self::new(
            role,
            factor_source_id,
            RoleBuilderMutateResult::forever_invalid(reason),
        )
    }

    pub(crate) fn not_yet_valid(
        role: RoleKind,
        factor_source_id: FactorSourceID,
        reason: NotYetValidReason,
    ) -> Self {
        Self::new(
            role,
            factor_source_id,
            RoleBuilderMutateResult::not_yet_valid(reason),
        )
    }
}

use BasicViolation::*;
use ForeverInvalidReason::*;
use NotYetValidReason::*;
use RoleKind::*;

pub type RoleBuilderMutateResult = Result<(), RoleBuilderValidation>;

impl<const R: u8> RoleBuilder<R> {
    pub type RoleBuilderBuildResult = Result<RoleWithFactorSourceIds<R>, RoleBuilderValidation>;

    pub(crate) fn build(self) -> Self::RoleBuilderBuildResult {
        self.validate().map(|_| {
            RoleWithFactorSourceIds::with_factors(
                // self.role(),
                self.get_threshold(),
                self.get_threshold_factors().clone(),
                self.get_override_factors().clone(),
            )
        })
    }

    #[allow(dead_code)]
    pub(crate) fn set_threshold(&mut self, threshold: u8) -> RoleBuilderMutateResult {
        match self.role() {
            Primary => {
                self.unchecked_set_threshold(threshold);
                self.validate()
            }
            Recovery => RoleBuilderMutateResult::basic_violation(RecoveryCannotSetThreshold),
            Confirmation => {
                RoleBuilderMutateResult::basic_violation(ConfirmationCannotSetThreshold)
            }
        }
    }

    fn override_contains_factor_source(&self, factor_source_id: &FactorSourceID) -> bool {
        self.get_override_factors().contains(factor_source_id)
    }

    fn threshold_contains_factor_source(&self, factor_source_id: &FactorSourceID) -> bool {
        self.get_threshold_factors().contains(factor_source_id)
    }

    fn override_contains_factor_source_of_kind(
        &self,
        factor_source_kind: FactorSourceKind,
    ) -> bool {
        self.get_override_factors()
            .iter()
            .any(|f| f.get_factor_source_kind() == factor_source_kind)
    }

    fn threshold_contains_factor_source_of_kind(
        &self,
        factor_source_kind: FactorSourceKind,
    ) -> bool {
        self.get_threshold_factors()
            .iter()
            .any(|f| f.get_factor_source_kind() == factor_source_kind)
    }

    /// If Ok => self is mutated
    /// If Err(NotYetValid) => self is mutated
    /// If Err(ForeverInvalid) => self is not mutated
    pub(crate) fn add_factor_source_to_list(
        &mut self,
        factor_source_id: FactorSourceID,
        factor_list_kind: FactorListKind,
    ) -> RoleBuilderMutateResult {
        let validation = self
            .validation_for_addition_of_factor_source_to_list(&factor_source_id, factor_list_kind);
        match validation.as_ref() {
            Ok(()) | Err(Validation::NotYetValid(_)) => {
                self.unchecked_add_factor_source_to_list(factor_source_id, factor_list_kind);
            }
            Err(Validation::ForeverInvalid(_)) | Err(Validation::BasicViolation(_)) => {}
        }
        validation
    }

    /// Validates `self` by "replaying" the addition of each factor source in `self` to a
    /// "simulation" (clone). If the simulation is valid, then `self` is valid.
    pub(crate) fn validate(&self) -> RoleBuilderMutateResult {
        let mut simulation = Self::new();

        // Validate override factors
        for override_factor in self.get_override_factors() {
            let validation =
                simulation.add_factor_source_to_list(*override_factor, FactorListKind::Override);
            match validation.as_ref() {
                Ok(()) | Err(Validation::NotYetValid(_)) => continue,
                Err(Validation::ForeverInvalid(_)) | Err(Validation::BasicViolation(_)) => {
                    return validation
                }
            }
        }

        // Validate threshold factors
        for threshold_factor in self.get_threshold_factors() {
            let validation =
                simulation.add_factor_source_to_list(*threshold_factor, FactorListKind::Threshold);
            match validation.as_ref() {
                Ok(()) | Err(Validation::NotYetValid(_)) => continue,
                Err(Validation::ForeverInvalid(_)) | Err(Validation::BasicViolation(_)) => {
                    return validation
                }
            }
        }

        // Validate threshold count
        if self.role() == RoleKind::Primary {
            if self.get_threshold_factors().len() < self.get_threshold() as usize {
                return RoleBuilderMutateResult::not_yet_valid(
                    NotYetValidReason::ThresholdHigherThanThresholdFactorsLen,
                );
            }
            if self.get_threshold() == 0 && !self.get_threshold_factors().is_empty() {
                return RoleBuilderMutateResult::not_yet_valid(
                    NotYetValidReason::PrimaryRoleWithThresholdCannotBeZeroWithFactors,
                );
            }
        } else if self.get_threshold() != 0 {
            match self.role() {
                Primary => unreachable!("Primary role should have been handled earlier"),
                Recovery => {
                    return RoleBuilderMutateResult::basic_violation(RecoveryCannotSetThreshold)
                }
                Confirmation => {
                    return RoleBuilderMutateResult::basic_violation(ConfirmationCannotSetThreshold)
                }
            }
        }

        if self.all_factors().is_empty() {
            return RoleBuilderMutateResult::not_yet_valid(RoleMustHaveAtLeastOneFactor);
        }

        Ok(())
    }

    /// If we would add a factor of kind `factor_source_kind` to the list of kind `factor_list_kind`
    /// what would be the validation status?
    pub(crate) fn validation_for_addition_of_factor_source_of_kind_to_list(
        &self,
        factor_source_kind: FactorSourceKind,
        factor_list_kind: FactorListKind,
    ) -> RoleBuilderMutateResult {
        match self.role() {
            RoleKind::Primary => self.validation_for_addition_of_factor_source_of_kind_to_list_for_primary(factor_source_kind, factor_list_kind),
            RoleKind::Recovery | RoleKind::Confirmation => match factor_list_kind {
                FactorListKind::Threshold => {
                    RoleBuilderMutateResult::forever_invalid(ForeverInvalidReason::threshold_list_not_supported_for_role(self.role()))
                }
                FactorListKind::Override => self
                    .validation_for_addition_of_factor_source_of_kind_to_override_for_non_primary_role(
                        factor_source_kind,
                    ),
            },
        }
    }

    fn validation_for_addition_of_factor_source_of_kind_to_override_for_non_primary_role(
        &self,
        factor_source_kind: FactorSourceKind,
    ) -> RoleBuilderMutateResult {
        match self.role() {
            RoleKind::Primary => {
                unreachable!("Should have branched to 'primary' earlier, this is programmer error.")
            }
            RoleKind::Confirmation => self
                .validation_for_addition_of_factor_source_of_kind_to_override_for_confirmation(
                    factor_source_kind,
                ),
            RoleKind::Recovery => self
                .validation_for_addition_of_factor_source_of_kind_to_override_for_recovery(
                    factor_source_kind,
                ),
        }
    }

    #[allow(dead_code)]
    /// For each factor source in the given set, return a validation status
    /// for adding it to factor list of the given kind (`factor_list_kind`)
    pub(crate) fn validation_for_addition_of_factor_source_for_each(
        &self,
        factor_list_kind: FactorListKind,
        factor_sources: &IndexSet<FactorSourceID>,
    ) -> IndexSet<FactorSourceInRoleBuilderValidationStatus> {
        factor_sources
            .iter()
            .map(|factor_source_id| {
                let validation_status = self.validation_for_addition_of_factor_source_to_list(
                    factor_source_id,
                    factor_list_kind,
                );
                FactorSourceInRoleBuilderValidationStatus::new(
                    self.role(),
                    *factor_source_id,
                    validation_status,
                )
            })
            .collect()
    }

    fn validation_for_addition_of_factor_source_to_list(
        &self,
        factor_source_id: &FactorSourceID,
        factor_list_kind: FactorListKind,
    ) -> RoleBuilderMutateResult {
        if self.contains_factor_source(factor_source_id) {
            return RoleBuilderMutateResult::forever_invalid(FactorSourceAlreadyPresent);
        }
        let factor_source_kind = factor_source_id.get_factor_source_kind();
        self.validation_for_addition_of_factor_source_of_kind_to_list(
            factor_source_kind,
            factor_list_kind,
        )
    }

    fn contains_factor_source(&self, factor_source_id: &FactorSourceID) -> bool {
        self.override_contains_factor_source(factor_source_id)
            || self.threshold_contains_factor_source(factor_source_id)
    }

    fn contains_factor_source_of_kind(&self, factor_source_kind: FactorSourceKind) -> bool {
        self.override_contains_factor_source_of_kind(factor_source_kind)
            || self.threshold_contains_factor_source_of_kind(factor_source_kind)
    }

    /// Lowers the threshold if the deleted factor source is in the threshold list
    /// and if after removal of `factor_source_id` `self.threshold > self.threshold_factors.len()`
    ///
    /// Returns `Ok` if `factor_source_id` was found and deleted. However, does not call `self.validate()`,
    /// So state might still be invalid, i.e. we return the result of the action of removal, not the
    /// state validation status.
    pub(crate) fn remove_factor_source(
        &mut self,
        factor_source_id: &FactorSourceID,
    ) -> RoleBuilderMutateResult {
        if !self.contains_factor_source(factor_source_id) {
            return RoleBuilderMutateResult::basic_violation(FactorSourceNotFound);
        }
        let remove = |xs: &mut Vec<FactorSourceID>| {
            let index = xs
                    .iter()
                    .position(|f| f == factor_source_id)
                    .expect("Called remove of non existing FactorSourceID, this is a programmer error, should have checked if it exists before calling remove.");
            xs.remove(index);
        };

        if self.override_contains_factor_source(factor_source_id) {
            remove(self.mut_override_factors())
        }
        if self.threshold_contains_factor_source(factor_source_id) {
            remove(self.mut_threshold_factors());
            let threshold_factors_len = self.get_threshold_factors().len() as u8;
            if self.get_threshold() > threshold_factors_len {
                self.set_threshold(threshold_factors_len)?;
            }
        }

        Ok(())
    }

    fn validation_for_addition_of_factor_source_of_kind_to_list_for_primary(
        &self,
        factor_source_kind: FactorSourceKind,
        factor_list_kind: FactorListKind,
    ) -> RoleBuilderMutateResult {
        match factor_source_kind {
            FactorSourceKind::Passphrase => {
                return self.validation_for_addition_of_password_to_primary(factor_list_kind)
            }
            FactorSourceKind::SecurityQuestions => {
                return RoleBuilderMutateResult::forever_invalid(
                    PrimaryCannotContainSecurityQuestions,
                );
            }
            FactorSourceKind::TrustedContact => {
                return RoleBuilderMutateResult::forever_invalid(
                    PrimaryCannotContainTrustedContact,
                );
            }
            FactorSourceKind::Device => {
                if self.contains_factor_source_of_kind(FactorSourceKind::Device) {
                    return RoleBuilderMutateResult::forever_invalid(
                        PrimaryCannotHaveMultipleDevices,
                    );
                }
            }
            FactorSourceKind::LedgerHQHardwareWallet
            | FactorSourceKind::ArculusCard
            | FactorSourceKind::OffDeviceMnemonic => {}
        }
        Ok(())
    }

    fn validation_for_addition_of_factor_source_of_kind_to_override_for_confirmation(
        &self,
        factor_source_kind: FactorSourceKind,
    ) -> RoleBuilderMutateResult {
        assert_eq!(self.role(), RoleKind::Confirmation);
        match factor_source_kind {
            FactorSourceKind::Device
            | FactorSourceKind::LedgerHQHardwareWallet
            | FactorSourceKind::ArculusCard
            | FactorSourceKind::Passphrase
            | FactorSourceKind::OffDeviceMnemonic
            | FactorSourceKind::SecurityQuestions => Ok(()),
            FactorSourceKind::TrustedContact => {
                RoleBuilderMutateResult::forever_invalid(ConfirmationRoleTrustedContactNotSupported)
            }
        }
    }

    fn validation_for_addition_of_factor_source_of_kind_to_override_for_recovery(
        &self,
        factor_source_kind: FactorSourceKind,
    ) -> RoleBuilderMutateResult {
        assert_eq!(self.role(), RoleKind::Recovery);
        match factor_source_kind {
            FactorSourceKind::Device
            | FactorSourceKind::LedgerHQHardwareWallet
            | FactorSourceKind::ArculusCard
            | FactorSourceKind::OffDeviceMnemonic
            | FactorSourceKind::TrustedContact => Ok(()),
            FactorSourceKind::SecurityQuestions => {
                RoleBuilderMutateResult::forever_invalid(RecoveryRoleSecurityQuestionsNotSupported)
            }
            FactorSourceKind::Passphrase => {
                RoleBuilderMutateResult::forever_invalid(RecoveryRolePasswordNotSupported)
            }
        }
    }
}

// =======================
// ======== RULES ========
// =======================
impl<const R: u8> RoleBuilder<R> {
    fn validation_for_addition_of_password_to_primary(
        &self,
        factor_list_kind: FactorListKind,
    ) -> RoleBuilderMutateResult {
        assert_eq!(self.role(), RoleKind::Primary);
        let factor_source_kind = FactorSourceKind::Passphrase;
        match factor_list_kind {
            FactorListKind::Threshold => {
                let is_alone = self
                    .factor_sources_not_of_kind_to_list_of_kind(
                        factor_source_kind,
                        FactorListKind::Threshold,
                    )
                    .is_empty();
                if is_alone {
                    return RoleBuilderMutateResult::not_yet_valid(
                        PrimaryRoleWithPasswordInThresholdListMustHaveAnotherFactor,
                    );
                }
                if self.get_threshold() < 2 {
                    return RoleBuilderMutateResult::not_yet_valid(
                        PrimaryRoleWithPasswordInThresholdListMustThresholdGreaterThanOne,
                    );
                }
            }
            FactorListKind::Override => {
                return RoleBuilderMutateResult::forever_invalid(
                    PrimaryCannotHavePasswordInOverrideList,
                );
            }
        }

        Ok(())
    }

    pub(crate) fn factor_sources_not_of_kind_to_list_of_kind(
        &self,
        factor_source_kind: FactorSourceKind,
        factor_list_kind: FactorListKind,
    ) -> Vec<FactorSourceID> {
        let filter = |xs: &Vec<FactorSourceID>| -> Vec<FactorSourceID> {
            xs.iter()
                .filter(|f| f.get_factor_source_kind() != factor_source_kind)
                .cloned()
                .collect()
        };
        match factor_list_kind {
            FactorListKind::Override => filter(self.get_override_factors()),
            FactorListKind::Threshold => filter(self.get_threshold_factors()),
        }
    }
}
