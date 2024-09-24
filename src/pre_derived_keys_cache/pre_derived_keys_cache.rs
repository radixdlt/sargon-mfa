#![allow(unused)]

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
    /// `UnindexDerivationRequest` - which is agnostic to the derivation index.
    probably_free_factor_instances: IndexMap<UnindexDerivationRequest, FactorInstances>,
}

impl HierarchicalDeterministicFactorInstance {
    pub fn erase_to_derivation_request(&self) -> UnindexDerivationRequest {
        UnindexDerivationRequest::new(
            self.factor_source_id,
            self.derivation_path().network_id,
            self.derivation_path().entity_kind,
            self.derivation_path().key_kind,
            self.key_space(),
        )
    }
}

impl PreDerivedKeysCache {
    pub fn new(probably_free_factor_instances: ProbablyFreeFactorInstances) -> Self {
        Self {
            probably_free_factor_instances: probably_free_factor_instances
                .into_iter()
                .into_group_map_by(|x| x.erase_to_derivation_request())
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().collect::<FactorInstances>()))
                .collect::<IndexMap<UnindexDerivationRequest, FactorInstances>>(),
        }
    }
}

impl PreDerivedKeysCache {
    async fn _take_many_instances_for_single_request(
        &self,
        request: &UnindexDerivationRequest,
    ) -> Result<LoadFromCacheOutcome> {
        let cached = self.probably_free_factor_instances.get(key);
        match cached {
            Some(cached) => {
                let mut cached = cached.clone();
                let mut instances = FactorInstances::default();
                while instances.len() < request.number_of_instances {
                    if let Some(instance) = cached.factor_instances.pop() {
                        instances.factor_instances.push(instance);
                    } else {
                        break;
                    }
                }
                if instances.len() == request.number_of_instances {
                    Ok(LoadFromCacheOutcome::FullySatisfiedWithSpare(instances))
                } else if instances.len() > 0 {
                    Ok(LoadFromCacheOutcome::FullySatisfiedWithoutSpare(instances))
                } else {
                    Ok(LoadFromCacheOutcome::CacheIsEmpty)
                }
            }
        }
    }
    pub async fn take_many_instances_for_single_request(
        &self,
        request: &UnindexDerivationRequest,
    ) -> Result<LoadFromCacheOutcomeForSingleRequest> {
        todo!()
    }

    pub async fn take_many_instances_for_many_requests(
        &self,
        requests: &UnindexDerivationRequests,
    ) -> Result<FactorInstancesFromCache> {
        let mut outcome_map =
            IndexMap::<UnindexDerivationRequest, LoadFromCacheOutcomeForSingleRequest>::new();
        for request in requests.requests() {
            let outcome = self
                .take_many_instances_for_single_request(&request)
                .await?;
            outcome_map.insert(outcome.request.clone(), outcome);
        }
        Ok(FactorInstancesFromCache {
            per_request: outcome_map,
        })
    }
}
