use crate::prelude::*;

pub struct NextDerivationEntityIndexCacheAnalyzingAssigner {
    cache: FactorInstancesCache,
}
impl NextDerivationEntityIndexCacheAnalyzingAssigner {
    pub fn cache(&self) -> FactorInstancesCache {
        self.cache.clone()
    }

    pub fn new(cache: FactorInstancesCache) -> Self {
        Self { cache }
    }

    pub fn next(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        index_agnostic_path: IndexAgnosticPath,
    ) -> Result<Option<HDPathComponent>> {
        let max = self
            .cache
            .max_index_for(factor_source_id, index_agnostic_path);
        let Some(max) = max else { return Ok(None) };
        max.add_one().map(Some)
    }
}
