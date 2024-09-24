use crate::prelude::*;

/// This is SINGLE `QuantifiedUnindexDerivationRequest` level, but remember
/// each `QuantifiedUnindexDerivationRequest` can try to load MANY
/// FactorInstances. e.g. securifying many accounts with a single
/// security shield =>
/// `(MatrixOfFactorSources, Vec<Account>)` -> Vec(Account, MatrixOfFactorInstances)`
#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadFromCacheOutcomeForSingleRequest {
    pub request: QuantifiedUnindexDerivationRequest,
    pub outcome: LoadFromCacheOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactorInstancesFromCache {
    pub per_request:
        IndexMap<QuantifiedUnindexDerivationRequest, LoadFromCacheOutcomeForSingleRequest>,
}
