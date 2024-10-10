use std::ops::{AddAssign, Index};

use crate::prelude::*;

/// Ephemeral / "Local" offsets, is a collection of counters with offset added
/// on top of next index analysis based on cache or profile. This is used so that
/// the FactorInstanceProvider can consecutively call `next` N times to get a range of
/// of `N` unique indices, added to the otherwise next based on cache/profile analysis.
#[derive(Debug, Default)]
pub struct NextDerivationEntityIndexWithEphemeralOffsetsForFactorSource {
    ephemeral_offsets: RwLock<HashMap<IndexAgnosticPath, HDPathValue>>,
}

impl NextDerivationEntityIndexWithEphemeralOffsetsForFactorSource {
    pub fn reserve(&self, agnostic_path: IndexAgnosticPath) -> HDPathValue {
        let mut binding = self.ephemeral_offsets.write().unwrap();
        if let Some(existing) = binding.get_mut(&agnostic_path) {
            let free = *existing;
            existing.add_assign(1);
            free
        } else {
            binding.insert(agnostic_path, 1);
            0
        }
    }
}
