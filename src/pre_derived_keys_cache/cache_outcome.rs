use crate::prelude::*;

/// This is SINGLE `QuantifiedUnindexDerivationRequest` level, but remember
/// each `QuantifiedUnindexDerivationRequest` can try to load MANY
/// FactorInstances. e.g. securifying many accounts with a single
/// security shield =>
/// `(MatrixOfFactorSources, Vec<Account>)` -> Vec(Account, MatrixOfFactorInstances)`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LoadFromCacheOutcome {
    /// Successfully loaded cached FactorInstances fulfilling
    /// a single `QuantifiedUnindexDerivationRequest`.
    ///
    /// No need to derive more, as the cache has more free instances after
    /// consuming the loaded ones.
    FullySatisfiedWithSpare(FactorInstances),

    /// Successfully loaded cached FactorInstances fulfilling
    /// a single `QuantifiedUnindexDerivationRequest`.
    ///
    /// SHOULD derive more, since the cache has no more free instances after
    /// consuming the loaded ones.
    FullySatisfiedWithoutSpare(FactorInstances),

    /// The single `QuantifiedUnindexDerivationRequest` could only be partially
    /// satisfied
    ///
    /// We MUST derive more FactorInstances, and
    /// we SHOULD derive FactorInstance with an abundance so that we can
    /// fill the cache.
    PartiallySatisfied {
        partial_from_cache: FactorInstances,
        number_of_instances_needed_to_fully_satisfy_request: usize,
    },

    /// The cache contained no FactorInstances for the single request.
    CacheIsEmpty {
        number_of_instances_needed_to_fully_satisfy_request: usize,
    },
}

impl LoadFromCacheOutcome {
    fn non_empty_factor_instances(&self) -> Option<FactorInstances> {
        match self {
            LoadFromCacheOutcome::FullySatisfiedWithSpare(from_cache) => Some(from_cache.clone()),
            LoadFromCacheOutcome::FullySatisfiedWithoutSpare(from_cache) => {
                Some(from_cache.clone())
            }
            LoadFromCacheOutcome::PartiallySatisfied {
                partial_from_cache, ..
            } => Some(partial_from_cache.clone()),
            LoadFromCacheOutcome::CacheIsEmpty { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LoadFromCacheOutcomeForSingleRequest {
    hidden: HiddenConstructor,
    pub request: QuantifiedUnindexDerivationRequest,
    outcome: LoadFromCacheOutcome,
}

impl LoadFromCacheOutcomeForSingleRequest {
    /// # Panics
    /// If FactorInstances is NOT empty inside the outcome, this ctor
    /// will panic if ANY of them does not match the requested:
    /// * FactorSourceID
    /// * KeyKind
    /// * KeySpace
    /// * EntityKind
    /// * Network
    pub fn new(request: QuantifiedUnindexDerivationRequest, outcome: LoadFromCacheOutcome) -> Self {
        if let Some(factor_instances) = outcome.non_empty_factor_instances() {
            assert!(factor_instances
                .factor_instances()
                .iter()
                .all(|factor_instance| {
                    factor_instance
                        .satisfies(UnquantifiedUnindexDerivationRequest::from(request.clone()))
                }));
        }
        Self {
            hidden: HiddenConstructor,
            request,
            outcome,
        }
    }

    fn last_index_of(instances: &FactorInstances) -> HDPathValue {
        assert!(!instances.is_empty());
        let mut instances = instances.factor_instances().into_iter().collect_vec();
        instances.sort_by_key(|instance| instance.derivation_entity_base_index());
        instances.last().unwrap().derivation_entity_base_index()
    }

    pub fn action(&self) -> Action {
        match self.outcome {
            LoadFromCacheOutcome::FullySatisfiedWithSpare(ref from_cache) => {
                Action::FullySatisfiedWithSpare(from_cache.clone())
            }
            LoadFromCacheOutcome::FullySatisfiedWithoutSpare(ref from_cache) => {
                let last_index = Self::last_index_of(from_cache);
                Action::FullySatisfiedWithoutSpare(
                    from_cache.clone(),
                    DerivationRequestWithRange::from((self.request.clone(), last_index)),
                )
            }
            LoadFromCacheOutcome::PartiallySatisfied {
                ref partial_from_cache,
                number_of_instances_needed_to_fully_satisfy_request,
            } => {
                let last_index = Self::last_index_of(partial_from_cache);
                Action::PartiallySatisfied {
                    partial_from_cache: partial_from_cache.clone(),
                    derive_more: DerivationRequestWithRange::from((
                        self.request.clone(),
                        last_index,
                    )),
                    number_of_instances_needed_to_fully_satisfy_request,
                }
            }
            LoadFromCacheOutcome::CacheIsEmpty {
                number_of_instances_needed_to_fully_satisfy_request,
            } => Action::CacheIsEmpty {
                number_of_instances_needed_to_fully_satisfy_request,
            },
        }
    }
}

pub enum Action {
    FullySatisfiedWithSpare(FactorInstances),
    FullySatisfiedWithoutSpare(FactorInstances, DerivationRequestWithRange),
    PartiallySatisfied {
        partial_from_cache: FactorInstances,
        derive_more: DerivationRequestWithRange,
        number_of_instances_needed_to_fully_satisfy_request: usize,
    },
    CacheIsEmpty {
        number_of_instances_needed_to_fully_satisfy_request: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactorInstancesFromCache {
    hidden: HiddenConstructor,
    outcomes: Vec<LoadFromCacheOutcomeForSingleRequest>,
}

impl FactorInstancesFromCache {
    pub fn new(iter: impl IntoIterator<Item = LoadFromCacheOutcomeForSingleRequest>) -> Self {
        Self {
            hidden: HiddenConstructor,
            outcomes: iter
                .into_iter()
                .collect::<IndexSet<LoadFromCacheOutcomeForSingleRequest>>()
                .into_iter()
                .collect(),
        }
    }

    pub fn outcomes(&self) -> IndexSet<LoadFromCacheOutcomeForSingleRequest> {
        self.outcomes.clone().into_iter().collect()
    }
}
