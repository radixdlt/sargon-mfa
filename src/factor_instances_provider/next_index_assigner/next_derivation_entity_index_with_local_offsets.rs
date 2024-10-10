use crate::prelude::*;

#[derive(Default, Debug)]
pub struct NextDerivationEntityIndexWithLocalOffsets {
    local_offsets_per_factor_source: RwLock<
        HashMap<FactorSourceIDFromHash, NextDerivationEntityIndexWithLocalOffsetsForFactorSource>,
    >,
}

impl NextDerivationEntityIndexWithLocalOffsets {
    pub fn reserve(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        agnostic_path: IndexAgnosticPath,
    ) -> HDPathValue {
        let mut binding = self.local_offsets_per_factor_source.write().unwrap();
        if let Some(for_factor) = binding.get_mut(&factor_source_id) {
            for_factor.reserve(agnostic_path)
        } else {
            let new =
                NextDerivationEntityIndexWithLocalOffsetsForFactorSource::empty(factor_source_id);
            let next = new.reserve(agnostic_path);
            binding.insert(factor_source_id, new);
            next
        }
    }
}
