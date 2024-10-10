use std::ops::{AddAssign, Index};

use crate::prelude::*;

#[derive(Debug)]
pub struct NextDerivationEntityIndexWithLocalOffsetsForFactorSource {
    #[allow(dead_code)]
    factor_source_id: FactorSourceIDFromHash,
    local_offsets: RwLock<HashMap<IndexAgnosticPath, HDPathValue>>,
}

impl NextDerivationEntityIndexWithLocalOffsetsForFactorSource {
    pub fn empty(factor_source_id: FactorSourceIDFromHash) -> Self {
        Self {
            factor_source_id,
            local_offsets: RwLock::new(HashMap::new()),
        }
    }
    pub fn reserve(&self, agnostic_path: IndexAgnosticPath) -> HDPathValue {
        let mut binding = self.local_offsets.write().unwrap();
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
