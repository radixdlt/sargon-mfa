use crate::prelude::*;

pub enum OffsetFromCache {
    /// Finding max amongst already loaded (and removed) from cache, saved
    /// locally
    FindMaxInRemoved {
        pf_found_in_cache: IndexMap<FactorSourceIDFromHash, FactorInstances>,
    },
    /// Known max by having peeked into the cache earlier.
    KnownMax {
        instance: HierarchicalDeterministicFactorInstance,
    },
}

impl OffsetFromCache {
    pub fn next(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        index_agnostic_path: IndexAgnosticPath,
    ) -> Result<Option<HDPathComponent>> {
        let Some(m) = self._max(factor_source_id, index_agnostic_path) else {
            return Ok(None);
        };
        m.add_one().map(Some)
    }

    fn _max(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        index_agnostic_path: IndexAgnosticPath,
    ) -> Option<HDPathComponent> {
        match self {
            Self::FindMaxInRemoved { pf_found_in_cache } => pf_found_in_cache
                .get(&factor_source_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|f| f.agnostic_path() == index_agnostic_path)
                .map(|f| f.derivation_path().index)
                .max(),
            Self::KnownMax { instance } => {
                assert_eq!(instance.factor_source_id(), factor_source_id);
                assert_eq!(instance.agnostic_path(), index_agnostic_path);
                Some(instance.derivation_path().index)
            }
        }
    }
}
