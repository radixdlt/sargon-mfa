#![cfg(test)]

use std::{
    ops::{Deref, DerefMut},
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use crate::prelude::*;

pub type InMemoryKeysCache =
    HashMap<PreDerivedKeysCacheKey, IndexSet<HierarchicalDeterministicFactorInstance>>;

/// A simple `IsPreDerivedKeysCache` which uses in-memory cache instead of on
/// file which the live implementation will use.
#[derive(Default)]
pub struct InMemoryPreDerivedKeysCache {
    cache: RwLock<InMemoryKeysCache>,
}

impl InMemoryPreDerivedKeysCache {
    fn read<T>(&self, call: impl FnOnce(RwLockReadGuard<'_, InMemoryKeysCache>) -> T) -> Result<T> {
        let cached = self
            .cache
            .try_read()
            .map_err(|_| CommonError::KeysCacheWriteGuard)?;

        Ok(call(cached))
    }

    fn write<T>(
        &self,
        mut call: impl FnOnce(RwLockWriteGuard<'_, InMemoryKeysCache>) -> Result<T>,
    ) -> Result<T> {
        let cached = self
            .cache
            .try_write()
            .map_err(|_| CommonError::KeysCacheWriteGuard)?;

        call(cached)
    }
}

#[async_trait::async_trait]
impl IsPreDerivedKeysCache for InMemoryPreDerivedKeysCache {
    async fn insert(
        &self,
        derived_factors_map: IndexMap<
            PreDerivedKeysCacheKey,
            IndexSet<HierarchicalDeterministicFactorInstance>,
        >,
    ) -> Result<()> {
        self.write(|mut cache| pre_derived_keys_cache_insert(derived_factors_map, &mut cache))
    }

    async fn consume_next_factor_instances(
        &self,
        requests: IndexSet<DerivationRequest>,
    ) -> Result<IndexMap<DerivationRequest, HierarchicalDeterministicFactorInstance>> {
        self.write(|mut cache| pre_derived_keys_cache_consume(requests, &mut cache))
    }

    async fn peek(&self, requests: IndexSet<DerivationRequest>) -> NextDerivationPeekOutcome {
        match self.read(|cache| pre_derived_keys_cache_peek(requests, &cache)) {
            Ok(o) => o,
            Err(e) => NextDerivationPeekOutcome::Failure(e),
        }
    }
}
