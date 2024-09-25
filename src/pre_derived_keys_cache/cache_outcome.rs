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

    /// The single `QuantifiedUnindexDerivationRequest` couls only be partially
    /// statisfied
    ///
    /// We MUST derive more FactorInstances, and
    /// we SHOULD derive FactorInstance with an abundance so that we can
    /// fill the cache.
    PartiallySatisfied(FactorInstances),

    /// The cache countained no FactorInstances for the single request.
    CacheIsEmpty,
}

impl LoadFromCacheOutcome {
    fn non_empty_factor_instances(&self) -> Option<FactorInstances> {
        match self {
            LoadFromCacheOutcome::FullySatisfiedWithSpare(factor_instances) => {
                Some(factor_instances.clone())
            }
            LoadFromCacheOutcome::FullySatisfiedWithoutSpare(factor_instances) => {
                Some(factor_instances.clone())
            }
            LoadFromCacheOutcome::PartiallySatisfied(factor_instances) => {
                Some(factor_instances.clone())
            }
            LoadFromCacheOutcome::CacheIsEmpty => None,
        }
    }
}
// impl LoadFromCacheOutcome {
//     fn satisfaction(&self) -> Satisfaction {
//         match self {
//             LoadFromCacheOutcome::FullySatisfiedWithSpare(_) => Satisfaction::FullyWithSpare,
//             LoadFromCacheOutcome::FullySatisfiedWithoutSpare(_) => Satisfaction::FullyWithoutSpare,
//             LoadFromCacheOutcome::PartiallySatisfied(_) => Satisfaction::Partial,
//             LoadFromCacheOutcome::CacheIsEmpty => Satisfaction::Empty,
//         }
//     }
// }

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LoadFromCacheOutcomeForSingleRequest {
    hidden: HiddenConstructor,
    pub request: QuantifiedUnindexDerivationRequest,
    outcome: LoadFromCacheOutcome,
    // factor_instances: Option<FactorInstances>,
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
            // satisfaction: outcome.satisfaction(),
            // factor_instances: outcome.non_empty_factor_instances(),
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
            LoadFromCacheOutcome::FullySatisfiedWithSpare(ref factor_instances) => {
                Action::FullySatisfiedWithSpare(factor_instances.clone())
            }
            LoadFromCacheOutcome::FullySatisfiedWithoutSpare(ref factor_instances) => {
                let last_index = Self::last_index_of(factor_instances);
                Action::FullySatisfiedWithoutSpare(
                    factor_instances.clone(),
                    QuantifiedDerivationRequestWithStartIndex::from((
                        self.request.clone(),
                        last_index,
                    )),
                )
            }
            LoadFromCacheOutcome::PartiallySatisfied(ref factor_instances) => {
                let last_index = Self::last_index_of(factor_instances);
                Action::PartiallySatisfied(
                    factor_instances.clone(),
                    QuantifiedDerivationRequestWithStartIndex::from((
                        self.request.clone(),
                        last_index,
                    )),
                )
            }
            LoadFromCacheOutcome::CacheIsEmpty => Action::CacheIsEmpty,
        }
    }
}

pub enum Action {
    FullySatisfiedWithSpare(FactorInstances),
    FullySatisfiedWithoutSpare(FactorInstances, QuantifiedDerivationRequestWithStartIndex),
    PartiallySatisfied(FactorInstances, QuantifiedDerivationRequestWithStartIndex),
    CacheIsEmpty,
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
