#![cfg(test)]
#![allow(unused)]

use crate::prelude::*;

#[derive(Default, Clone, Debug)]
pub(crate) struct StatelessDummyIndices;

impl StatelessDummyIndices {
    pub(crate) fn next_derivation_index_for(&self, key_space: KeySpace) -> HDPathComponent {
        match key_space {
            KeySpace::Securified => HDPathComponent::securifying_base_index(0),
            KeySpace::Unsecurified => HDPathComponent::unsecurified_hardening_base_index(0),
        }
    }

    pub(crate) fn next_derivation_path(
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
