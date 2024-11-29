use std::{borrow::Borrow, sync::Arc};

#[cfg(test)]
use rules::SampleValues;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FactorSourceKind {
    Device,
    LedgerHQHardwareWallet,
    Passphrase,
    OffDeviceMnemonic,
    TrustedContact,
    SecurityQuestions,
    ArculusCard,
}
impl From<FactorSourceKind> for sargon::FactorSourceKind {
    fn from(value: FactorSourceKind) -> Self {
        match value {
            FactorSourceKind::Device => sargon::FactorSourceKind::Device,
            FactorSourceKind::LedgerHQHardwareWallet => {
                sargon::FactorSourceKind::LedgerHQHardwareWallet
            }
            FactorSourceKind::Passphrase => sargon::FactorSourceKind::Passphrase,
            FactorSourceKind::OffDeviceMnemonic => sargon::FactorSourceKind::OffDeviceMnemonic,
            FactorSourceKind::TrustedContact => sargon::FactorSourceKind::TrustedContact,
            FactorSourceKind::SecurityQuestions => sargon::FactorSourceKind::SecurityQuestions,
            FactorSourceKind::ArculusCard => sargon::FactorSourceKind::ArculusCard,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Object)]
pub struct FactorSourceID {
    pub inner: sargon::FactorSourceID,
}
impl FactorSourceID {
    pub fn new(inner: impl Borrow<sargon::FactorSourceID>) -> Arc<Self> {
        Arc::new(Self {
            inner: *inner.borrow(),
        })
    }
}

#[cfg(test)]
impl FactorSourceID {
    pub fn sample_device() -> Arc<Self> {
        Self::new(sargon::FactorSourceID::sample_device())
    }

    pub fn sample_device_other() -> Arc<Self> {
        Self::new(sargon::FactorSourceID::sample_device_other())
    }

    pub fn sample_ledger() -> Arc<Self> {
        Self::new(sargon::FactorSourceID::sample_ledger())
    }

    pub fn sample_ledger_other() -> Arc<Self> {
        Self::new(sargon::FactorSourceID::sample_ledger_other())
    }

    pub fn sample_arculus() -> Arc<Self> {
        Self::new(sargon::FactorSourceID::sample_arculus())
    }

    pub fn sample_arculus_other() -> Arc<Self> {
        Self::new(sargon::FactorSourceID::sample_arculus_other())
    }
}
