use crate::prelude::*;

/// Derivation Presets are Network agnostic and Index agnostic
/// "templates" for DerivationPaths.
#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq, enum_iterator::Sequence)]
pub enum DerivationPreset {
    /// Used to form DerivationPaths used to derive FactorInstances
    /// for "veci": Virtual Entity Creating (Factor)Instance for accounts.
    /// `(EntityKind::Account, KeySpace::Unsecurified, KeyKind::TransactionSigning)`
    AccountVeci,

    /// Used to form DerivationPaths used to derive FactorInstances
    /// for "mfa" to securify accounts.
    /// `(EntityKind::Account, KeySpace::Securified, KeyKind::TransactionSigning)`
    AccountMfa,

    /// Used to form DerivationPaths used to derive FactorInstances
    /// for "veci": Virtual Entity Creating (Factor)Instance for personas.
    /// `(EntityKind::Identity, KeySpace::Unsecurified, KeyKind::TransactionSigning)`
    IdentityVeci,

    /// Used to form DerivationPaths used to derive FactorInstances
    /// for "mfa" to securify personas.
    /// `(EntityKind::Identity, KeySpace::Securified, KeyKind::TransactionSigning)`
    IdentityMfa,
}
impl DerivationPreset {
    pub fn all() -> IndexSet<Self> {
        enum_iterator::all::<Self>().collect()
    }

    pub fn index_agnostic_path_on_network(&self, network_id: NetworkID) -> IndexAgnosticPath {
        IndexAgnosticPath::from((network_id, *self))
    }

    pub fn veci_entity_kind(entity_kind: CAP26EntityKind) -> Self {
        match entity_kind {
            CAP26EntityKind::Account => Self::AccountVeci,
            CAP26EntityKind::Identity => Self::IdentityVeci,
        }
    }

    pub fn mfa_entity_kind(entity_kind: CAP26EntityKind) -> Self {
        match entity_kind {
            CAP26EntityKind::Account => Self::AccountMfa,
            CAP26EntityKind::Identity => Self::IdentityMfa,
        }
    }
    pub fn entity_kind(&self) -> CAP26EntityKind {
        match self {
            Self::AccountVeci | Self::AccountMfa => CAP26EntityKind::Account,
            Self::IdentityVeci | Self::IdentityMfa => CAP26EntityKind::Identity,
        }
    }
    pub fn key_kind(&self) -> CAP26KeyKind {
        match self {
            Self::AccountVeci | Self::IdentityVeci => CAP26KeyKind::TransactionSigning,
            Self::AccountMfa | Self::IdentityMfa => CAP26KeyKind::TransactionSigning,
        }
    }
    pub fn key_space(&self) -> KeySpace {
        match self {
            Self::AccountVeci | Self::IdentityVeci => KeySpace::Unsecurified,
            Self::AccountMfa | Self::IdentityMfa => KeySpace::Securified,
        }
    }
}
