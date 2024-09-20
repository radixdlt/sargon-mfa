use crate::prelude::*;

use super::probably_free_factor_instances;

#[derive(Debug)]
pub struct PreDerivedKeysCache;

impl PreDerivedKeysCache {
    pub fn new(probably_free_factor_instances: ProbablyFreeFactorInstances) -> Self {
        warn!(
            "TODO: Implement PreDerivedKeysCache::new, IGNORED {:?}",
            probably_free_factor_instances
        );
        Self
    }
}
