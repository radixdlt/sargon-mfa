use crate::prelude::*;

use FactorListKind::*;

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

#[cfg(test)]
impl RecoveryRoleWithFactorSourceIds {
    pub(crate) fn recovery_with_factors(
        override_factors: impl IntoIterator<Item = FactorSourceID>,
    ) -> Self {
        Self::with_factors(0, vec![], override_factors)
    }
}

#[cfg(test)]
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
use RoleBuilderValidation::*;

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
        Err(ForeverInvalid(reason))
    }
}

pub(crate) trait FromNotYetValid {
    fn not_yet_valid(reason: NotYetValidReason) -> Self;
}
impl<T> FromNotYetValid for std::result::Result<T, RoleBuilderValidation> {
    fn not_yet_valid(reason: NotYetValidReason) -> Self {
        Err(NotYetValid(reason))
    }
}

pub(crate) trait FromBasicViolation {
    fn basic_violation(reason: BasicViolation) -> Self;
}
impl<T> FromBasicViolation for std::result::Result<T, RoleBuilderValidation> {
    fn basic_violation(reason: BasicViolation) -> Self {
        Err(BasicViolation(reason))
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

pub enum Assert<const CHECK: bool> {}
pub trait IsTrue {}
impl IsTrue for Assert<true> {}

impl<const R: u8> RoleBuilder<R>
where
    Assert<{ R == ROLE_PRIMARY }>: IsTrue,
{
    /// If Ok => self is mutated
    /// If Err(NotYetValid) => self is mutated
    /// If Err(ForeverInvalid) => self is not mutated
    pub(crate) fn add_factor_source_to_threshold(
        &mut self,
        factor_source_id: FactorSourceID,
    ) -> RoleBuilderMutateResult {
        self._add_factor_source_to_list(factor_source_id, Threshold)
    }

    /// If we would add a factor of kind `factor_source_kind` to the list of kind `factor_list_kind`
    /// what would be the validation status?
    pub(crate) fn validation_for_addition_of_factor_source_of_kind_to_threshold(
        &self,
        factor_source_kind: FactorSourceKind,
    ) -> RoleBuilderMutateResult {
        self._validation_add(factor_source_kind, Threshold)
    }

    #[cfg(test)]
    pub(crate) fn validation_for_addition_of_factor_source_of_kind_to_list(
        &self,
        factor_source_kind: FactorSourceKind,
        list: FactorListKind,
    ) -> RoleBuilderMutateResult {
        self._validation_add(factor_source_kind, list)
    }
}

impl<const R: u8> RoleBuilder<R>
where
    Assert<{ R > ROLE_PRIMARY }>: IsTrue,
{
    /// If Ok => self is mutated
    /// If Err(NotYetValid) => self is mutated
    /// If Err(ForeverInvalid) => self is not mutated
    pub(crate) fn add_factor_source(
        &mut self,
        factor_source_id: FactorSourceID,
    ) -> RoleBuilderMutateResult {
        self.add_factor_source_to_override(factor_source_id)
    }
}

impl<const R: u8> RoleBuilder<R> {
    /// If Ok => self is mutated
    /// If Err(NotYetValid) => self is mutated
    /// If Err(ForeverInvalid) => self is not mutated
    pub(crate) fn add_factor_source_to_override(
        &mut self,
        factor_source_id: FactorSourceID,
    ) -> RoleBuilderMutateResult {
        self._add_factor_source_to_list(factor_source_id, Override)
    }

    /// If Ok => self is mutated
    /// If Err(NotYetValid) => self is mutated
    /// If Err(ForeverInvalid) => self is not mutated
    fn _add_factor_source_to_list(
        &mut self,
        factor_source_id: FactorSourceID,
        factor_list_kind: FactorListKind,
    ) -> RoleBuilderMutateResult {
        let validation = self
            .validation_for_addition_of_factor_source_to_list(&factor_source_id, factor_list_kind);
        match validation.as_ref() {
            Ok(()) | Err(NotYetValid(_)) => {
                self.unchecked_add_factor_source_to_list(factor_source_id, factor_list_kind);
            }
            Err(ForeverInvalid(_)) | Err(BasicViolation(_)) => {}
        }
        validation
    }

    /// If we would add a factor of kind `factor_source_kind` to the list of kind `factor_list_kind`
    /// what would be the validation status?
    pub(crate) fn validation_for_addition_of_factor_source_of_kind_to_override(
        &self,
        factor_source_kind: FactorSourceKind,
    ) -> RoleBuilderMutateResult {
        self._validation_add(factor_source_kind, Override)
    }

    /// If we would add a factor of kind `factor_source_kind` to the list of kind `factor_list_kind`
    /// what would be the validation status?
    fn _validation_add(
        &self,
        factor_source_kind: FactorSourceKind,
        factor_list_kind: FactorListKind,
    ) -> RoleBuilderMutateResult {
        match self.role() {
            RoleKind::Primary => {
                return self.validation_for_addition_of_factor_source_of_kind_to_list_for_primary(
                    factor_source_kind,
                    factor_list_kind,
                )
            }
            RoleKind::Recovery | RoleKind::Confirmation => match factor_list_kind {
                Threshold => {
                    return Result::forever_invalid(
                        ForeverInvalidReason::threshold_list_not_supported_for_role(self.role()),
                    )
                }
                Override => {}
            },
        }
        self.validation_for_addition_of_factor_source_of_kind_to_override_for_non_primary_role(
            factor_source_kind,
        )
    }
}

impl<const R: u8> RoleBuilder<R> {
    pub(crate) fn build(self) -> Result<RoleWithFactorSourceIds<R>, RoleBuilderValidation> {
        self.validate().map(|_| {
            RoleWithFactorSourceIds::with_factors(
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

    /// Validates `self` by "replaying" the addition of each factor source in `self` to a
    /// "simulation" (clone). If the simulation is valid, then `self` is valid.
    pub(crate) fn validate(&self) -> RoleBuilderMutateResult {
        let mut simulation = Self::new();

        // Validate override factors
        for f in self.get_override_factors() {
            let validation = simulation.add_factor_source_to_override(*f);
            match validation.as_ref() {
                Ok(()) | Err(NotYetValid(_)) => continue,
                Err(ForeverInvalid(_)) | Err(BasicViolation(_)) => return validation,
            }
        }

        // Validate threshold factors
        for f in self.get_threshold_factors() {
            let validation = simulation._add_factor_source_to_list(*f, Threshold);
            match validation.as_ref() {
                Ok(()) | Err(NotYetValid(_)) => continue,
                Err(ForeverInvalid(_)) | Err(BasicViolation(_)) => return validation,
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
        self._validation_add(factor_source_kind, factor_list_kind)
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

    #[cfg(not(tarpaulin_include))] // false negative
    fn validation_for_addition_of_factor_source_of_kind_to_list_for_primary(
        &self,
        factor_source_kind: FactorSourceKind,
        factor_list_kind: FactorListKind,
    ) -> RoleBuilderMutateResult {
        match factor_source_kind {
            FactorSourceKind::Password => {
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

    #[cfg(not(tarpaulin_include))] // false negative
    fn validation_for_addition_of_factor_source_of_kind_to_override_for_confirmation(
        &self,
        factor_source_kind: FactorSourceKind,
    ) -> RoleBuilderMutateResult {
        assert_eq!(self.role(), RoleKind::Confirmation);
        match factor_source_kind {
            FactorSourceKind::Device
            | FactorSourceKind::LedgerHQHardwareWallet
            | FactorSourceKind::ArculusCard
            | FactorSourceKind::Password
            | FactorSourceKind::OffDeviceMnemonic
            | FactorSourceKind::SecurityQuestions => Ok(()),
            FactorSourceKind::TrustedContact => {
                RoleBuilderMutateResult::forever_invalid(ConfirmationRoleTrustedContactNotSupported)
            }
        }
    }

    #[cfg(not(tarpaulin_include))] // false negative
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
            FactorSourceKind::Password => {
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
        let factor_source_kind = FactorSourceKind::Password;
        match factor_list_kind {
            Threshold => {
                let is_alone = self
                    .factor_sources_not_of_kind_to_list_of_kind(factor_source_kind, Threshold)
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
            Override => {
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
            Override => filter(self.get_override_factors()),
            Threshold => filter(self.get_threshold_factors()),
        }
    }
}

#[cfg(test)]
pub(crate) fn test_duplicates_not_allowed<const R: u8>(
    sut: RoleBuilder<R>,
    list: FactorListKind,
    factor_source_id: FactorSourceID,
) {
    // Arrange
    let mut sut = sut;

    sut._add_factor_source_to_list(factor_source_id, list)
        .unwrap();

    // Act
    let res = sut._add_factor_source_to_list(
        factor_source_id, // oh no, duplicate!
        list,
    );

    // Assert
    assert!(matches!(
        res,
        RoleBuilderMutateResult::Err(ForeverInvalid(
            ForeverInvalidReason::FactorSourceAlreadyPresent
        ))
    ));
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn primary_duplicates_not_allowed() {
        test_duplicates_not_allowed(
            PrimaryRoleBuilder::new(),
            Override,
            FactorSourceID::sample_arculus(),
        );
        test_duplicates_not_allowed(
            PrimaryRoleBuilder::new(),
            Threshold,
            FactorSourceID::sample_arculus(),
        );
    }

    #[test]
    fn recovery_duplicates_not_allowed() {
        test_duplicates_not_allowed(
            RecoveryRoleBuilder::new(),
            Override,
            FactorSourceID::sample_arculus(),
        );
    }

    #[test]
    fn confirmation_duplicates_not_allowed() {
        test_duplicates_not_allowed(
            ConfirmationRoleBuilder::new(),
            Override,
            FactorSourceID::sample_arculus(),
        );
    }

    #[test]
    fn recovery_cannot_add_factors_to_threshold() {
        let mut sut = RecoveryRoleBuilder::new();
        let res = sut._add_factor_source_to_list(FactorSourceID::sample_ledger(), Threshold);
        assert_eq!(
            res,
            Err(ForeverInvalid(
                ForeverInvalidReason::RecoveryRoleThresholdFactorsNotSupported
            ))
        );
    }

    #[test]
    fn confirmation_cannot_add_factors_to_threshold() {
        let mut sut = ConfirmationRoleBuilder::new();
        let res = sut._add_factor_source_to_list(FactorSourceID::sample_ledger(), Threshold);
        assert_eq!(
            res,
            Err(ForeverInvalid(
                ForeverInvalidReason::ConfirmationRoleThresholdFactorsNotSupported
            ))
        );
    }

    #[test]
    fn recovery_validation_add_is_err_for_threshold() {
        let sut = RecoveryRoleBuilder::new();
        let res = sut._validation_add(FactorSourceKind::Device, Threshold);
        assert_eq!(
            res,
            RoleBuilderMutateResult::forever_invalid(
                ForeverInvalidReason::threshold_list_not_supported_for_role(RoleKind::Recovery)
            )
        );
    }

    #[test]
    fn confirmation_validation_add_is_err_for_threshold() {
        let sut = ConfirmationRoleBuilder::new();
        let res = sut._validation_add(FactorSourceKind::Device, Threshold);
        assert_eq!(
            res,
            RoleBuilderMutateResult::forever_invalid(
                ForeverInvalidReason::threshold_list_not_supported_for_role(RoleKind::Confirmation)
            )
        );
    }
}
