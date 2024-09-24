#![allow(unused)]

use std::error::Request;

use crate::prelude::*;

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
#[derive(Debug, Default, Clone)]
pub struct PreDerivedKeysCache {
    /// The probably free factor instances, many Factor Instances per
    /// `QuantifiedUnindexDerivationRequest` - which is agnostic to the derivation index.
    probably_free_factor_instances: IndexMap<UnquantifiedUnindexDerivationRequest, FactorInstances>,
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
    pub fn new(probably_free_factor_instances: ProbablyFreeFactorInstances) -> Self {
        Self {
            probably_free_factor_instances: probably_free_factor_instances
                .into_iter()
                .into_group_map_by(|x| UnquantifiedUnindexDerivationRequest::from(*x))
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().collect::<FactorInstances>()))
                .collect::<IndexMap<UnquantifiedUnindexDerivationRequest, FactorInstances>>(),
        }
    }
}

impl PreDerivedKeysCache {
    fn delete(&self, key: impl Into<UnquantifiedUnindexDerivationRequest>) {
        let key = key.into();
        self.probably_free_factor_instances.swap_remove(&key);
    }

    /// Appends the FactorInstances at the end of any existing FactorInstances, if any,
    /// otherwise creates a new entry
    fn append(
        &self,
        key: impl Into<UnquantifiedUnindexDerivationRequest>,
        to_append: impl Into<FactorInstances>,
    ) {
        let key = key.into();
        let to_append = to_append.into();
        if let Some(ref existing) = self.probably_free_factor_instances.get(&key) {
            existing.append(to_append);
        } else {
            self.probably_free_factor_instances.insert(key, to_append);
        }
    }
    fn peek(
        &self,
        key: impl Into<UnquantifiedUnindexDerivationRequest>,
    ) -> Option<FactorInstances> {
        let key = key.into();
        self.probably_free_factor_instances.get(&key).cloned()
    }
}

impl PreDerivedKeysCache {
    fn _take_many_instances_for_single_request(
        &self,
        request: &QuantifiedUnindexDerivationRequest,
    ) -> LoadFromCacheOutcome {
        let cached = self.peek(request);
        self.delete_all_instances_for_single_request(request);
        match cached {
            Some(cached) => {
                if cached.len() == 0 {
                    return LoadFromCacheOutcome::CacheIsEmpty;
                }
                let requested_quantity = request.requested_quantity();
                if cached.len() == requested_quantity {
                    return LoadFromCacheOutcome::FullySatisfiedWithoutSpare(cached.clone());
                } else if cached.len() > requested_quantity {
                    let (to_return, to_keep) = cached
                        .factor_instances()
                        .into_iter()
                        .collect_vec()
                        .split_at(requested_quantity);

                    assert_eq!(to_return.len(), requested_quantity);
                    assert!(!to_keep.is_empty());

                    self.probably_free_factor_instances.insert(
                        request.clone(),
                        to_keep.iter().cloned().collect::<FactorInstances>(),
                    );
                    return LoadFromCacheOutcome::FullySatisfiedWithSpare(
                        to_return.iter().cloned().collect::<FactorInstances>(),
                    );
                } else {
                    return LoadFromCacheOutcome::PartiallySatisfied(cached.clone());
                }
            }
            None => LoadFromCacheOutcome::CacheIsEmpty,
        }
    }

    pub fn take_many_instances_for_single_request(
        &self,
        request: &QuantifiedUnindexDerivationRequest,
    ) -> Result<LoadFromCacheOutcomeForSingleRequest> {
        let outcome = self._take_many_instances_for_single_request(request);
        Ok(LoadFromCacheOutcomeForSingleRequest {
            request: request.clone(),
            outcome,
        })
    }

    pub fn take_many_instances_for_many_requests(
        &self,
        requests: &UnindexDerivationRequests,
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
}
