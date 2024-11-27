use crate::prelude::*;

/// A kind of factor list, either threshold, or override kind.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum FactorListKind {
    Threshold,
    Override,
}

/// TODO move to Sargon!!!!
pub trait HasFactorSourceKindObjectSafe {
    fn get_factor_source_kind(&self) -> FactorSourceKind;
}
impl HasFactorSourceKindObjectSafe for FactorSourceID {
    fn get_factor_source_kind(&self) -> FactorSourceKind {
        match self {
            FactorSourceID::Hash { value } => value.kind,
            FactorSourceID::Address { value } => value.kind,
        }
    }
}

#[allow(dead_code)]
// TODO REMOVE once migrated to sargon
pub trait SampleValues: Sized {
    fn sample_device() -> Self;
    fn sample_device_other() -> Self;
    fn sample_ledger() -> Self;
    fn sample_ledger_other() -> Self;
    fn sample_arculus() -> Self;
    fn sample_arculus_other() -> Self;
    fn sample_password() -> Self;
    fn sample_password_other() -> Self;
    fn sample_passphrase() -> Self;
    fn sample_passphrase_other() -> Self;
    fn sample_security_questions() -> Self;

    fn sample_security_questions_other() -> Self;
    fn sample_trusted_contact() -> Self;
    fn sample_trusted_contact_other() -> Self;
}

impl SampleValues for FactorSourceID {
    fn sample_device() -> Self {
        FactorSourceIDFromHash::sample_device().into()
    }
    fn sample_ledger() -> Self {
        FactorSourceIDFromHash::sample_ledger().into()
    }
    fn sample_ledger_other() -> Self {
        FactorSourceIDFromHash::sample_ledger_other().into()
    }
    fn sample_arculus() -> Self {
        FactorSourceIDFromHash::sample_arculus().into()
    }
    fn sample_arculus_other() -> Self {
        FactorSourceIDFromHash::sample_arculus_other().into()
    }

    /// Matt calls `passphrase` "password"
    fn sample_password() -> Self {
        FactorSourceIDFromHash::sample_passphrase().into()
    }
    /// Matt calls `passphrase` "password"
    fn sample_password_other() -> Self {
        FactorSourceIDFromHash::sample_passphrase_other().into()
    }

    /// Matt calls `off_device_mnemonic` "passphrase"
    fn sample_passphrase() -> Self {
        FactorSourceIDFromHash::sample_off_device().into()
    }
    /// Matt calls `off_device_mnemonic` "passphrase"
    fn sample_passphrase_other() -> Self {
        FactorSourceIDFromHash::sample_off_device_other().into()
    }
    fn sample_security_questions() -> Self {
        FactorSourceIDFromHash::sample_security_questions().into()
    }
    fn sample_device_other() -> Self {
        FactorSourceIDFromHash::sample_device_other().into()
    }
    fn sample_security_questions_other() -> Self {
        FactorSourceIDFromHash::sample_security_questions_other().into()
    }
    fn sample_trusted_contact() -> Self {
        sargon::FactorSource::sample_trusted_contact_frank().id()
    }
    fn sample_trusted_contact_other() -> Self {
        sargon::FactorSource::sample_trusted_contact_grace().id()
    }
}
