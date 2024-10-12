use crate::prelude::*;

use super::quantities;

/// A DerivationPath which is not indexed. On a specific network.
#[derive(Clone, Copy, Hash, PartialEq, Eq, derive_more::Debug, derive_more::Display)]
#[display("{}/{}/{}/?{}", network_id, entity_kind, key_kind, key_space.indicator())]
#[debug("{:?}/{:?}/{:?}/?{}", network_id, entity_kind, key_kind, key_space.indicator())]
pub struct IndexAgnosticPath {
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_kind: CAP26KeyKind,
    pub key_space: KeySpace,
}

impl IndexAgnosticPath {
    pub fn new(
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        key_kind: CAP26KeyKind,
        key_space: KeySpace,
    ) -> Self {
        Self {
            network_id,
            entity_kind,
            key_kind,
            key_space,
        }
    }
}

impl From<(NetworkID, DerivationPreset)> for IndexAgnosticPath {
    fn from((network_id, agnostic_path): (NetworkID, DerivationPreset)) -> Self {
        Self::new(
            network_id,
            agnostic_path.entity_kind(),
            agnostic_path.key_kind(),
            agnostic_path.key_space(),
        )
    }
}
impl TryFrom<IndexAgnosticPath> for DerivationPreset {
    type Error = CommonError;
    fn try_from(value: IndexAgnosticPath) -> Result<DerivationPreset> {
        match (value.entity_kind, value.key_kind, value.key_space) {
            (
                CAP26EntityKind::Account,
                CAP26KeyKind::TransactionSigning,
                KeySpace::Unsecurified,
            ) => Ok(DerivationPreset::AccountVeci),
            (
                CAP26EntityKind::Identity,
                CAP26KeyKind::TransactionSigning,
                KeySpace::Unsecurified,
            ) => Ok(DerivationPreset::IdentityVeci),
            (CAP26EntityKind::Account, CAP26KeyKind::TransactionSigning, KeySpace::Securified) => {
                Ok(DerivationPreset::AccountMfa)
            }
            (CAP26EntityKind::Identity, CAP26KeyKind::TransactionSigning, KeySpace::Securified) => {
                Ok(DerivationPreset::IdentityMfa)
            }
            _ => Err(CommonError::NonStandardDerivationPath),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, derive_more::Debug)]
#[debug("🎯: {:?} #{}", self.derivation_preset, self.quantity)]
pub struct QuantifiedDerivationPreset {
    pub derivation_preset: DerivationPreset,
    pub quantity: usize,
}
impl QuantifiedDerivationPreset {
    pub fn new(derivation_preset: DerivationPreset, quantity: usize) -> Self {
        Self {
            derivation_preset,
            quantity,
        }
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
