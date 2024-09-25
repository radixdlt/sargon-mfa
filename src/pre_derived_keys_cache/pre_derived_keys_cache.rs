#![allow(unused)]

use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::prelude::*;

type InstancesForRequestMap = IndexMap<UnquantifiedUnindexDerivationRequest, FactorInstances>;

/// A cache of FactorInstances which according to Profile is
/// not known to be taken, i.e. they are "probably free".
///
/// We never query the cache with a `DerivationPath` - which
/// contains a derivation index, rather we ask the cache "give me the next N
/// Factor Instances for this FactorSourceID, on this network, for this KeyKind,
/// for this EntityKind, in this KeySpace" - the outcome of which might be:
/// * No Factor Instances for that request
/// * Some Factor Instances for that request, but fewer than requested
/// * Exactly the requested number of Factor Instances for that request - in which
/// the caller SHOULD re-fill the cache before the caller finishes its operation.
/// * More Factor Instances than requested, use them and no need to re-fill the cache.
#[derive(Debug, Default)]
pub struct PreDerivedKeysCache {
    /// The probably free factor instances, many Factor Instances per
    /// `QuantifiedUnindexDerivationRequest` - which is agnostic to the derivation index.
    probably_free_factor_instances: RwLock<InstancesForRequestMap>,
}

impl From<HierarchicalDeterministicFactorInstance> for UnquantifiedUnindexDerivationRequest {
    fn from(value: HierarchicalDeterministicFactorInstance) -> Self {
        UnquantifiedUnindexDerivationRequest::new(
            value.factor_source_id,
            value.derivation_path().network_id,
            value.derivation_path().entity_kind,
            value.derivation_path().key_kind,
            value.key_space(),
        )
    }
}

impl PreDerivedKeysCache {
    fn read<T>(
        &self,
        call: impl FnOnce(RwLockReadGuard<'_, InstancesForRequestMap>) -> T,
    ) -> Result<T> {
        let cached = self
            .probably_free_factor_instances
            .try_read()
            .map_err(|_| CommonError::KeysCacheWriteGuard)?;
        Ok(call(cached))
    }
    fn write<T>(
        &self,
        mut call: impl FnOnce(RwLockWriteGuard<'_, InstancesForRequestMap>) -> T,
    ) -> Result<T> {
        let cached = self
            .probably_free_factor_instances
            .try_write()
            .map_err(|_| CommonError::KeysCacheWriteGuard)?;
        Ok(call(cached))
    }

    pub fn new(probably_free_factor_instances: ProbablyFreeFactorInstances) -> Self {
        Self {
            probably_free_factor_instances: RwLock::new(
                probably_free_factor_instances
                    .into_iter()
                    .into_group_map_by(|x| UnquantifiedUnindexDerivationRequest::from(x.clone()))
                    .into_iter()
                    .map(|(k, v)| (k, v.into_iter().collect::<FactorInstances>()))
                    .collect::<IndexMap<UnquantifiedUnindexDerivationRequest, FactorInstances>>(),
            ),
        }
    }
}

impl PreDerivedKeysCache {
    /// Appends the FactorInstances at the end of any existing FactorInstances, if any,
    /// otherwise creates a new entry
    fn append(
        &self,
        key: impl Into<UnquantifiedUnindexDerivationRequest>,
        to_append: impl Into<FactorInstances>,
    ) -> Result<()> {
        let key = key.into();
        let to_append = to_append.into();

        let maybe_existing = self.consume(key.clone())?;
        let mut values = maybe_existing.unwrap_or_default();

        assert!(values
            .factor_instances()
            .is_disjoint(&to_append.factor_instances()));

        values.append(to_append);

        self.write(|mut c| c.insert(key, values))?;

        Ok(())
    }
    fn peek(
        &self,
        key: impl Into<UnquantifiedUnindexDerivationRequest>,
    ) -> Result<Option<FactorInstances>> {
        let key = key.into();
        self.read(|c| c.get(&key).cloned())
    }
    fn consume(
        &self,
        key: impl Into<UnquantifiedUnindexDerivationRequest>,
    ) -> Result<Option<FactorInstances>> {
        let key = key.into();
        self.write(|mut c| c.swap_remove(&key))
    }
}
impl From<&[HierarchicalDeterministicFactorInstance]> for FactorInstances {
    fn from(value: &[HierarchicalDeterministicFactorInstance]) -> Self {
        Self::from_iter(value.iter().cloned())
    }
}
use std::cmp::Ordering;
impl PreDerivedKeysCache {
    fn _take_many_instances_for_single_request(
        &self,
        request: &QuantifiedUnindexDerivationRequest,
    ) -> Result<LoadFromCacheOutcome> {
        let cached = self.consume(request.clone())?;
        match cached {
            Some(cached) => {
                if cached.is_empty() {
                    return Ok(LoadFromCacheOutcome::CacheIsEmpty);
                }
                let requested_quantity = request.requested_quantity();
                match cached.len().cmp(&requested_quantity) {
                    Ordering::Equal => Ok(LoadFromCacheOutcome::FullySatisfiedWithoutSpare(
                        cached.clone(),
                    )),
                    Ordering::Greater => {
                        let to_split = cached.clone().into_iter().collect_vec();

                        let (to_return, to_keep) = to_split.split_at(requested_quantity);

                        assert_eq!(to_return.len(), requested_quantity);
                        assert!(!to_keep.is_empty());

                        self.append(request.clone(), to_keep);
                        Ok(LoadFromCacheOutcome::FullySatisfiedWithSpare(
                            FactorInstances::from(to_return),
                        ))
                    }
                    Ordering::Less => Ok(LoadFromCacheOutcome::PartiallySatisfied(cached.clone())),
                }
            }
            None => Ok(LoadFromCacheOutcome::CacheIsEmpty),
        }
    }

    fn take_many_instances_for_single_request(
        &self,
        request: &QuantifiedUnindexDerivationRequest,
    ) -> Result<LoadFromCacheOutcomeForSingleRequest> {
        let outcome = self._take_many_instances_for_single_request(request)?;
        Ok(LoadFromCacheOutcomeForSingleRequest {
            request: request.clone(),
            outcome,
        })
    }

    fn take_many_instances_for_many_requests(
        &self,
        requests: &QuantifiedUnindexDerivationRequests,
    ) -> Result<FactorInstancesFromCache> {
        let mut outcome_map = IndexMap::<
            QuantifiedUnindexDerivationRequest,
            LoadFromCacheOutcomeForSingleRequest,
        >::new();
        for request in requests.requests() {
            let outcome = self.take_many_instances_for_single_request(&request)?;
            outcome_map.insert(outcome.request.clone(), outcome);
        }
        Ok(FactorInstancesFromCache {
            per_request: outcome_map,
        })
    }

    pub fn take(
        &self,
        requests: &QuantifiedUnindexDerivationRequests,
    ) -> Result<FactorInstancesFromCache> {
        self.take_many_instances_for_many_requests(requests)
    }
}
