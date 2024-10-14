use std::ops::{AddAssign, Index};

use crate::prelude::*;

/// Ephemeral / "Local" offsets, is a collection of counters with offset added
/// on top of next index analysis based on cache or profile. This is used so that
/// the FactorInstanceProvider can consecutively call `next` N times to get a range of
/// of `N` unique indices, added to the otherwise next based on cache/profile analysis.
#[derive(Debug)]
pub struct NextDerivationEntityIndexWithEphemeralOffsetsForFactorSource {
    factor_source_id: FactorSourceIDFromHash,
    ephemeral_offsets: RwLock<HashMap<IndexAgnosticPath, HDPathValue>>,
}

impl NextDerivationEntityIndexWithEphemeralOffsetsForFactorSource {
    pub fn empty(factor_source_id: FactorSourceIDFromHash) -> Self {
        Self {
            factor_source_id,
            ephemeral_offsets: RwLock::new(HashMap::new()),
        }
    }

    /// Returns the next free index for the FactorSourceID and IndexAgnosticPath,
    /// and increases the local ephemeral offset.
    pub fn reserve(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        agnostic_path: IndexAgnosticPath,
    ) -> HDPathValue {
        assert_eq!(self.factor_source_id, factor_source_id);
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
