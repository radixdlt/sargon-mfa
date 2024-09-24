use crate::prelude::*;

/// The outcome of a cache query.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CacheOutcome {
    /// Successfully loaded cached FactorInstances fulfilling
    /// all the requested ones.
    ///
    /// No need to derive more, as the cache has more free instances after
    /// consuming the loaded ones.
    FullySatisfiedWithSpare(UnindexDerivationRequests, FactorInstances),

    /// Successfully loaded cached FactorInstances fulfilling
    /// all the requested ones.
    ///
    /// SHOULD derive more, since the cache has no more free instances after
    /// consuming the loaded ones.
    FullySatisfiedWithoutSpare(UnindexDerivationRequests, FactorInstances),

    /// Some of the requested FactorInstances could be satisfied, but some
    /// requests was not satisfied. We MUST derive more FactorInstances, and
    /// we SHOULD derive FactorInstance with an abundance so that we can
    /// fill the cache.
    PartiallySatisfied(UnindexDerivationRequests, FactorInstances),

    /// None of the requested FactorInstances could be satisfied. We MUST
    /// derive more and should derive FactorInstances with an abundance so
    /// we can fill the cache.
    CacheIsEmpty(UnindexDerivationRequests),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UnsatisfiedDerivationRequest {
    CacheWasEmpty(UnindexDerivationRequest),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnsatisfiedDerivationRequests {
    unsatisfied: Vec<UnsatisfiedDerivationRequest>,
}
impl UnsatisfiedDerivationRequests {
    pub fn cache_is_empty(requests: UnindexDerivationRequests) -> Self {
        Self {
            unsatisfied: requests
                .requests()
                .into_iter()
                .map(UnsatisfiedDerivationRequest::CacheWasEmpty)
                .collect(),
        }
    }
}

impl FactorInstances {
    pub fn unindex_derivation_requests(&self) -> UnindexDerivationRequests {
        self.clone()
            .into_iter()
            .map(|f| f.erase_to_derivation_request())
            .collect()
    }
}
impl UnindexDerivationRequests {
    pub fn unsatisfied(
        &self,
        partially_satisfying_response: &FactorInstances,
    ) -> Option<UnsatisfiedDerivationRequests> {
        let satisfied = partially_satisfying_response.unindex_derivation_requests();
        let diff = self
            .requests()
            .difference(&satisfied.requests())
            .cloned()
            .collect::<UnindexDerivationRequests>();
        if diff.is_empty() {
            None
        } else {
            Some(UnsatisfiedUnindexedDerivationRequests::new(diff))
        }
    }
}

impl CacheOutcome {
    /// If we should derive FactorInstances at indices stretching even further
    /// than those initially requested.
    ///
    /// If cache was empty we will derive **exactly** the requested amount.
    /// If cache could fully satisfy the request and would have spare instances,
    /// we would not derive more at all.
    ///
    /// But if we could only partially satisfy the request, we should derive more
    /// and we should derive not only enough to satisfy the remaining requests,
    /// but further than that to fill the cache.
    pub fn should_derive_at_indices_past_initially_requested(&self) -> bool {
        match self {
            Self::FullySatisfiedWithSpare(_, _) => false, // never derive more
            Self::FullySatisfiedWithoutSpare(_, _) => true, // derive more to fill cache
            Self::PartiallySatisfied(_, _) => true,       // derive more to fill cache
            Self::CacheIsEmpty(_) => false,               // derive **exactly** the requested
        }
    }

    /// If we should derive more FactorInstances, either because we could not
    /// satisfy all requests or because we should fill the cache.
    pub fn should_derive_more(&self) -> bool {
        self.should_derive_at_indices_past_initially_requested() || self.unsatisfied().is_some()
    }

    /// The remaining un
    pub fn unsatisfied(&self) -> Option<UnsatisfiedDerivationRequests> {
        match self {
            Self::FullySatisfiedWithSpare(_, _) => None,
            Self::FullySatisfiedWithoutSpare(_, _) => None,
            Self::PartiallySatisfied(unsatisfied_request, partially_satisfying_response) => {
                unsatisfied_request.unsatisfied(partially_satisfying_response)
            }
            Self::CacheIsEmpty(unsatisfied) => Some(UnsatisfiedDerivationRequests::cache_is_empty(
                unsatisfied.clone(),
            )),
        }
    }

    pub fn get(&self) -> FactorInstances {
        match self {
            Self::FullySatisfiedWithSpare(_, instances) => instances.clone(),
            Self::FullySatisfiedWithoutSpare(_, instances) => instances.clone(),
            Self::PartiallySatisfied(_, instances) => instances.clone(),
            Self::CacheIsEmpty(_) => FactorInstances::default(),
        }
    }
}
