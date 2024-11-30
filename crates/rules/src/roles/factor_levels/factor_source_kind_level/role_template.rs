use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactorSourceTemplate {
    pub kind: FactorSourceKind,
    pub id: u8,
}

pub(crate) type PrimaryRoleTemplate =
    AbstractBuiltRoleWithFactor<{ ROLE_PRIMARY }, FactorSourceTemplate>;

impl PrimaryRoleTemplate {
    pub(crate) fn new(threshold_factors: impl IntoIterator<Item = FactorSourceTemplate>) -> Self {
        let threshold_factors = threshold_factors.into_iter().collect_vec();
        Self::with_factors(threshold_factors.len() as u8, threshold_factors, [])
    }
}

pub(crate) type RecoveryRoleTemplate =
    AbstractBuiltRoleWithFactor<{ ROLE_RECOVERY }, FactorSourceTemplate>;

impl RecoveryRoleTemplate {
    pub(crate) fn new(override_factors: impl IntoIterator<Item = FactorSourceTemplate>) -> Self {
        Self::with_factors(0, [], override_factors)
    }
}

pub(crate) type ConfirmationRoleTemplate =
    AbstractBuiltRoleWithFactor<{ ROLE_CONFIRMATION }, FactorSourceTemplate>;

impl ConfirmationRoleTemplate {
    pub(crate) fn new(override_factors: impl IntoIterator<Item = FactorSourceTemplate>) -> Self {
        Self::with_factors(0, [], override_factors)
    }
}

impl FactorSourceTemplate {
    pub fn new(kind: FactorSourceKind, id: u8) -> Self {
        Self { kind, id }
    }

    pub fn device() -> Self {
        Self::new(FactorSourceKind::Device, 0)
    }

    fn ledger_id(id: u8) -> Self {
        Self::new(FactorSourceKind::LedgerHQHardwareWallet, id)
    }
    pub fn ledger() -> Self {
        Self::ledger_id(0)
    }

    pub fn ledger_other() -> Self {
        Self::ledger_id(1)
    }

    fn password_id(id: u8) -> Self {
        Self::new(FactorSourceKind::Password, id)
    }
    pub fn password() -> Self {
        Self::password_id(0)
    }
    pub fn password_other() -> Self {
        Self::password_id(1)
    }

    /// Radix Wallet (UI) calls this "Passphrase"
    pub fn off_device_mnemonic() -> Self {
        Self::new(FactorSourceKind::OffDeviceMnemonic, 0)
    }

    fn trusted_contact_id(id: u8) -> Self {
        Self::new(FactorSourceKind::TrustedContact, id)
    }

    pub fn trusted_contact() -> Self {
        Self::trusted_contact_id(0)
    }

    pub fn trusted_contact_other() -> Self {
        Self::trusted_contact_id(1)
    }

    pub fn security_questions() -> Self {
        Self::new(FactorSourceKind::SecurityQuestions, 0)
    }
}

impl IsMaybeKeySpaceAware for FactorSourceTemplate {
    fn maybe_key_space(&self) -> Option<KeySpace> {
        None
    }
}
