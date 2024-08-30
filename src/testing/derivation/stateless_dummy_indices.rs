use crate::prelude::*;

#[derive(Default, Clone, Debug)]
pub struct StatelessDummyIndices;

impl StatelessDummyIndices {
    pub fn next_derivation_index_for(&self, key_space: KeySpace) -> HDPathComponent {
        match key_space {
            KeySpace::Securified => HDPathComponent::non_hardened(BIP32_SECURIFIED_HALF),
            KeySpace::Unsecurified => HDPathComponent::non_hardened(0),
        }
    }

    pub fn next_derivation_path(
        &self,
        network_id: NetworkID,
        key_kind: CAP26KeyKind,
        entity_kind: CAP26EntityKind,
        key_space: KeySpace,
    ) -> DerivationPath {
        let index = self.next_derivation_index_for(key_space);
        DerivationPath::new(network_id, entity_kind, key_kind, index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeySpace {
    Unsecurified,
    Securified,
}
