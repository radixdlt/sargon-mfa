use crate::prelude::*;

pub struct SplitFactorInstancesFromCache {
    derive_more_requests: IndexSet<DeriveMore>,
    satisfied_by_cache: IndexSet<HierarchicalDeterministicFactorInstance>,
}
impl SplitFactorInstancesFromCache {
    pub(super) fn satisfied_by_cache(&self) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        self.satisfied_by_cache.clone()
    }

    pub(super) fn derive_more_requests(self) -> Option<IndexSet<DeriveMore>> {
        if self.derive_more_requests.is_empty() {
            None
        } else {
            Some(self.derive_more_requests)
        }
    }
}

pub(super) fn split_cache_response(
    take_from_cache_outcome: FactorInstancesFromCache,
) -> SplitFactorInstancesFromCache {
    let mut derive_more_requests = IndexSet::<DeriveMore>::new();
    let mut satisfied_by_cache = IndexSet::<HierarchicalDeterministicFactorInstance>::new();

    for outcome in take_from_cache_outcome.outcomes().into_iter() {
        match outcome.action() {
            Action::FullySatisfiedWithSpare(factor_instances) => {
                satisfied_by_cache.extend(factor_instances);
            }
            Action::FullySatisfiedWithoutSpare(factor_instances, with_start_index) => {
                satisfied_by_cache.extend(factor_instances);

                derive_more_requests.insert(DeriveMore::WithKnownStartIndex {
                    with_start_index,
                    number_of_instances_needed_to_fully_satisfy_request: None,
                });
            }
            Action::PartiallySatisfied {
                partial_from_cache,
                derive_more,
                number_of_instances_needed_to_fully_satisfy_request,
            } => {
                satisfied_by_cache.extend(partial_from_cache);
                derive_more_requests.insert(DeriveMore::WithKnownStartIndex {
                    with_start_index: derive_more,
                    number_of_instances_needed_to_fully_satisfy_request: Some(
                        number_of_instances_needed_to_fully_satisfy_request,
                    ),
                });
            }
            Action::CacheIsEmpty {
                number_of_instances_needed_to_fully_satisfy_request,
            } => {
                derive_more_requests.insert(DeriveMore::WithoutKnownLastIndex {
                    request: outcome.request,
                    number_of_instances_needed_to_fully_satisfy_request,
                });
            }
        }
    }
    SplitFactorInstancesFromCache {
        derive_more_requests,
        satisfied_by_cache,
    }
}
