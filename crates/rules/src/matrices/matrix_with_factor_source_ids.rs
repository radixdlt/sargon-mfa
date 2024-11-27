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
