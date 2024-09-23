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
#[derive(Debug, Default)]
pub struct PreDerivedKeysCache {
    /// The probably free factor instances, many Factor Instances per
    /// `DerivationRequest` - which is agnostic to the derivation index.
    probably_free_factor_instances: IndexMap<DerivationRequest, FactorInstances>,
}

impl HierarchicalDeterministicFactorInstance {
    fn erase_to_derivation_request(&self) -> DerivationRequest {
        DerivationRequest::new(
            self.factor_source_id,
            self.derivation_path().network_id,
            self.derivation_path().entity_kind,
            self.key_space(),
            self.derivation_path().key_kind,
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
                .collect::<IndexMap<DerivationRequest, FactorInstances>>(),
        }
    }
}

impl PreDerivedKeysCache {
    /// We never query the cache with a `DerivationPath` - which
    /// contains a derivation index, rather we ask the cache "give me the next N
    /// Factor Instances for this FactorSourceID, on this network, for this KeyKind,
    /// for this EntityKind, in this KeySpace" - the outcome of which might be:
    /// * No Factor Instances for that request
    /// * Some Factor Instances for that request, but fewer than requested
    /// * Exactly the requested number of Factor Instances for that request - in which
    /// the caller SHOULD re-fill the cache before the caller finishes its operation.
    /// * More Factor Instances than requested, use them and no need to re-fill the cache.
    ///
    /// In fact this load function does not work with a single `DerivationRequest`
    /// but rather with many, since we might care about reading FactorInstances in
    /// two different KeySpaces for example, or for multiple FactorSourceIDs.
    pub async fn load(&self, requests: &DerivationRequests) -> Result<CacheOutcome> {
        panic!("impl me")
    }
}
