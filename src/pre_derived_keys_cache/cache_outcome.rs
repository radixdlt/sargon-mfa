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
            outcome,
        }
    }
    fn last_index_of(instances: &FactorInstances) -> HDPathValue {
        assert!(!instances.is_empty());
        let mut instances = instances.factor_instances().into_iter().collect_vec();
        instances.sort_by_key(|instance| instance.derivation_entity_base_index());
        instances.last().unwrap().derivation_entity_base_index()
    }
    pub fn aggregatable(&self) -> AggregatableLoadFromCacheOutcomeForSingleRequest {
        match self.outcome {
            LoadFromCacheOutcome::FullySatisfiedWithSpare(ref factor_instances) => {
                AggregatableLoadFromCacheOutcomeForSingleRequest::fully_satisfied_with_spare(
                    factor_instances.clone(),
                )
            }
            LoadFromCacheOutcome::FullySatisfiedWithoutSpare(ref factor_instances) => {
                let last_index = Self::last_index_of(&factor_instances);
                let derive_more = QuantifiedDerivationRequestWithStartIndex::from((
                    self.request.clone(),
                    last_index,
                ));
                AggregatableLoadFromCacheOutcomeForSingleRequest::fully_satisfied_without_spare(
                    factor_instances.clone(),
                    derive_more,
                )
            }
            LoadFromCacheOutcome::PartiallySatisfied(ref factor_instances) => {
                let last_index = Self::last_index_of(&factor_instances);
                let derive_more = QuantifiedDerivationRequestWithStartIndex::from((
                    self.request.clone(),
                    last_index,
                ));
                AggregatableLoadFromCacheOutcomeForSingleRequest::partially_satisfied(
                    factor_instances.clone(),
                    derive_more,
                )
            }
            LoadFromCacheOutcome::CacheIsEmpty => {
                AggregatableLoadFromCacheOutcomeForSingleRequest::cache_is_empty(
                    self.request.clone(),
                )
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AggregatableLoadFromCacheOutcomeForSingleRequest {
    hidden: HiddenConstructor,

    /// might be empty
    pub loaded: FactorInstances,

    pub derive_more: Option<DeriveMore>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeriveMore {
    FromLast(QuantifiedDerivationRequestWithStartIndex),
    ReadNextFromProfile(QuantifiedUnindexDerivationRequest),
}
impl AggregatableLoadFromCacheOutcomeForSingleRequest {
    /// We must not make any decision regarding what the next index is, we must
    /// let next index assigner decide that, since we might need to read it out
    /// from profile.
    fn cache_is_empty(req: QuantifiedUnindexDerivationRequest) -> Self {
        Self::new(
            FactorInstances::default(),
            Some(DeriveMore::ReadNextFromProfile(req)),
        )
    }
    fn partially_satisfied(
        loaded: FactorInstances,
        derive_more: QuantifiedDerivationRequestWithStartIndex,
    ) -> Self {
        Self::new(loaded, Some(DeriveMore::FromLast(derive_more)))
    }
    fn fully_satisfied_without_spare(
        loaded: FactorInstances,
        derive_more: QuantifiedDerivationRequestWithStartIndex,
    ) -> Self {
        Self::new(loaded, Some(DeriveMore::FromLast(derive_more)))
    }
    fn fully_satisfied_with_spare(loaded: FactorInstances) -> Self {
        Self::new(loaded, None)
    }
    fn new(loaded: FactorInstances, derive_more: Option<DeriveMore>) -> Self {
        assert!(!(loaded.is_empty() && derive_more.is_none()));
        Self {
            loaded,
            derive_more,
            hidden: HiddenConstructor,
        }
    }
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
