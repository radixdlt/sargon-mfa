#![cfg(test)]

use crate::prelude::*;

#[derive(Clone, PartialEq, Eq, Hash)]
struct Tuple {
    request: DerivationRequest,
    path: DerivationPathWithoutIndex,
}
impl Tuple {
    #[allow(unused)]
    fn cache_key(&self) -> PreDeriveKeysCacheKey {
        PreDeriveKeysCacheKey::new(self.request.factor_source_id, self.path.clone())
    }
}

/// A simple `IsPreDerivedKeysCache` which uses in-memory cache instead of on
/// file which the live implementation will use.
#[derive(Default)]
pub struct InMemoryPreDerivedKeysCache {
    cache:
        RwLock<HashMap<PreDeriveKeysCacheKey, IndexSet<HierarchicalDeterministicFactorInstance>>>,
}

impl InMemoryPreDerivedKeysCache {
    fn tuples(requests: IndexSet<DerivationRequest>) -> IndexSet<Tuple> {
        requests
            .clone()
            .into_iter()
            .map(|request| Tuple {
                request,
                path: DerivationPathWithoutIndex::from(request),
            })
            .collect::<IndexSet<Tuple>>()
    }

    /// Internal helper for implementing `peek`.
    /// `peek` will will this and map:
    /// 1. `Err(e)` -> `NextDerivationPeekOutcome::Failure(e)`
    /// 1. `Ok(None)` -> `NextDerivationPeekOutcome::Fulfillable`
    /// 1. `Ok(requests)` -> `NextDerivationPeekOutcome::Unfulfillable(UnfulfillableRequests::new(requests))`
    async fn try_peek(
        &self,
        requests: IndexSet<DerivationRequest>,
    ) -> Result<Option<UnfulfillableRequests>> {
        let cached = self
            .cache
            .try_read()
            .map_err(|_| CommonError::KeysCacheReadGuard)?;

        let request_and_path_tuples = InMemoryPreDerivedKeysCache::tuples(requests.clone());

        let mut unfulfillable = IndexSet::<UnfulfillableRequest>::new();
        for tuple in request_and_path_tuples.iter() {
            let request = tuple.request;
            let Some(for_key) = cached.get(&tuple.cache_key()) else {
                unfulfillable.insert(UnfulfillableRequest::empty(request));
                continue;
            };

            let factors_left = for_key.len();
            if factors_left == 0 {
                unfulfillable.insert(UnfulfillableRequest::empty(request));
            } else if factors_left == 1 {
                let last_factor = for_key.last().expect("Just checked length.");
                unfulfillable.insert(UnfulfillableRequest::last(request, last_factor));
            } else {
                // all good
                continue;
            }
        }

        if unfulfillable.is_empty() {
            Ok(None)
        } else {
            Ok(Some(UnfulfillableRequests::new(unfulfillable)))
        }
    }
}

#[async_trait::async_trait]
impl IsPreDerivedKeysCache for InMemoryPreDerivedKeysCache {
    async fn insert(
        &self,
        derived_factors_map: IndexMap<
            PreDeriveKeysCacheKey,
            IndexSet<HierarchicalDeterministicFactorInstance>,
        >,
    ) -> Result<()> {
        for (key, values) in derived_factors_map.iter() {
            for value in values {
                assert_eq!(
                    value.factor_source_id(),
                    key.factor_source_id,
                    "Discrepancy! FactorSourceID mismatch, this is a developer error."
                );
            }
        }

        let mut write_guard = self
            .cache
            .try_write()
            .map_err(|_| CommonError::KeysCacheWriteGuard)?;

        for (key, derived_factors) in derived_factors_map {
            if let Some(existing_factors) = write_guard.get_mut(&key) {
                existing_factors.extend(derived_factors);
            } else {
                write_guard.insert(key, derived_factors);
            }
        }
        drop(write_guard);

        Ok(())
    }

    async fn consume_next_factor_instances(
        &self,
        requests: IndexSet<DerivationRequest>,
    ) -> Result<IndexMap<DerivationRequest, HierarchicalDeterministicFactorInstance>> {
        let mut cached = self
            .cache
            .try_write()
            .map_err(|_| CommonError::KeysCacheWriteGuard)?;

        let mut instances_read_from_cache =
            IndexMap::<DerivationRequest, HierarchicalDeterministicFactorInstance>::new();

        let request_and_path_tuples = InMemoryPreDerivedKeysCache::tuples(requests.clone());

        for tuple in request_and_path_tuples {
            let for_key = cached
                .get_mut(&tuple.cache_key())
                .ok_or(CommonError::KeysCacheUnknownKey)?;
            let read_from_cache = for_key
                .first()
                .ok_or(CommonError::KeysCacheEmptyForKey)?
                .clone();
            assert!(
                read_from_cache.matches(&tuple.request),
                "incorrect implementation"
            );
            for_key.shift_remove(&read_from_cache);
            instances_read_from_cache.insert(tuple.request, read_from_cache);
        }

        Ok(instances_read_from_cache)
    }

    async fn peek(&self, requests: IndexSet<DerivationRequest>) -> NextDerivationPeekOutcome {
        let outcome = self.try_peek(requests).await;
        match outcome {
            Ok(None) => NextDerivationPeekOutcome::Fulfillable,
            Ok(Some(unfulfillable)) => NextDerivationPeekOutcome::Unfulfillable(unfulfillable),
            Err(e) => NextDerivationPeekOutcome::Failure(e),
        }
    }
}
