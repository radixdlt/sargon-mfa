use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactorSourceTemplate {
    pub kind: FactorSourceKind,
    pub id: u8,
}

pub(crate) type PrimaryRoleTemplate =
    AbstractBuiltRoleWithFactor<{ ROLE_PRIMARY }, FactorSourceTemplate>;

pub(crate) type RecoveryRoleTemplate =
    AbstractBuiltRoleWithFactor<{ ROLE_RECOVERY }, FactorSourceTemplate>;

pub(crate) type ConfirmationRoleTemplate =
    AbstractBuiltRoleWithFactor<{ ROLE_CONFIRMATION }, FactorSourceTemplate>;

impl FactorSourceTemplate {
    pub fn new(kind: FactorSourceKind, id: u8) -> Self {
        Self { kind, id }
    }

    pub fn device(id: u8) -> Self {
        Self::new(FactorSourceKind::Device, id)
    }

    pub fn ledger(id: u8) -> Self {
        Self::new(FactorSourceKind::LedgerHQHardwareWallet, id)
    }

    pub fn arculus(id: u8) -> Self {
        Self::new(FactorSourceKind::ArculusCard, id)
    }

    pub fn password(id: u8) -> Self {
        Self::new(FactorSourceKind::Password, id)
    }

    /// Radix Wallet (UI) calls this "Passphrase"
    pub fn off_device_mnemonic(id: u8) -> Self {
        Self::new(FactorSourceKind::OffDeviceMnemonic, id)
    }

    pub fn trusted_contact(id: u8) -> Self {
        Self::new(FactorSourceKind::TrustedContact, id)
    }

    pub fn security_questions(id: u8) -> Self {
        Self::new(FactorSourceKind::SecurityQuestions, id)
    }
}

impl IsMaybeKeySpaceAware for FactorSourceTemplate {
    fn maybe_key_space(&self) -> Option<KeySpace> {
        None
    }
}
