use crate::prelude::*;

/// Used as "presets"
#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
pub struct NetworkIndexAgnosticPath {
    pub entity_kind: CAP26EntityKind,
    pub key_kind: CAP26KeyKind,
    pub key_space: KeySpace,
}
impl NetworkIndexAgnosticPath {
    fn new(entity_kind: CAP26EntityKind, key_kind: CAP26KeyKind, key_space: KeySpace) -> Self {
        Self {
            entity_kind,
            key_kind,
            key_space,
        }
    }
    fn transaction_signing(entity_kind: CAP26EntityKind, key_space: KeySpace) -> Self {
        Self::new(entity_kind, CAP26KeyKind::TransactionSigning, key_space)
    }
    pub fn account_veci() -> Self {
        Self::transaction_signing(CAP26EntityKind::Account, KeySpace::Unsecurified)
    }
    pub fn account_mfa() -> Self {
        Self::transaction_signing(CAP26EntityKind::Account, KeySpace::Securified)
    }
    pub fn identity_veci() -> Self {
        Self::transaction_signing(CAP26EntityKind::Identity, KeySpace::Unsecurified)
    }
    pub fn identity_mfa() -> Self {
        Self::transaction_signing(CAP26EntityKind::Identity, KeySpace::Securified)
    }
    pub fn all_presets() -> IndexSet<Self> {
        IndexSet::from_iter([
            Self::account_veci(),
            Self::account_mfa(),
            Self::identity_veci(),
            Self::identity_mfa(),
        ])
    }
    pub fn on_network(&self, network_id: NetworkID) -> IndexAgnosticPath {
        IndexAgnosticPath::from((network_id, *self))
    }
}

/// A DerivationPath that is not on any specified
/// network and which is not indexed.
#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
pub struct IndexAgnosticPath {
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_kind: CAP26KeyKind,
    pub key_space: KeySpace,
}
impl From<(NetworkID, NetworkIndexAgnosticPath)> for IndexAgnosticPath {
    fn from((network_id, agnostic_path): (NetworkID, NetworkIndexAgnosticPath)) -> Self {
        Self {
            network_id,
            entity_kind: agnostic_path.entity_kind,
            key_kind: agnostic_path.key_kind,
            key_space: agnostic_path.key_space,
        }
    }
}
impl IndexAgnosticPath {
    pub fn network_agnostic(&self) -> NetworkIndexAgnosticPath {
        NetworkIndexAgnosticPath::new(self.entity_kind, self.key_kind, self.key_space)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct QuantifiedNetworkIndexAgnosticPath {
    pub agnostic_path: NetworkIndexAgnosticPath,
    pub quantity: usize,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct QuantifiedToCacheToUseNetworkIndexAgnosticPath {
    pub agnostic_path: NetworkIndexAgnosticPath,
    pub quantity: QuantityToCacheToUseDirectly,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct QuantifiedToCacheToUseIndexAgnosticPath {
    pub agnostic_path: IndexAgnosticPath,
    pub quantity: QuantityToCacheToUseDirectly,
}
impl QuantifiedToCacheToUseIndexAgnosticPath {
    pub fn network_agnostic(&self) -> NetworkIndexAgnosticPath {
        self.agnostic_path.network_agnostic()
    }
}

impl From<(IndexAgnosticPath, HDPathComponent)> for DerivationPath {
    fn from((path, index): (IndexAgnosticPath, HDPathComponent)) -> Self {
        assert_eq!(index.key_space(), path.key_space);
        Self::new(path.network_id, path.entity_kind, path.key_kind, index)
    }
}

impl DerivationPath {
    pub fn agnostic(&self) -> IndexAgnosticPath {
        IndexAgnosticPath {
            network_id: self.network_id,
            entity_kind: self.entity_kind,
            key_kind: self.key_kind,
            key_space: self.key_space(),
        }
    }
}
impl HierarchicalDeterministicFactorInstance {
    pub fn agnostic_path(&self) -> IndexAgnosticPath {
        self.derivation_path().agnostic()
    }
}
