use crate::prelude::*;

#[derive(Debug)]
pub struct ProvidedInstances {
    #[allow(dead_code)]
    hidden_constructor: HiddenConstructor,

    /// The caller of FactorInstancesProvider::provide MUST override their
    /// original cache with this updated one if they want to persist the changes.
    pub cache_to_persist: FactorInstancesForSpecificNetworkCache,

    /// The factor instances that were provided to be used directly, this is sometimes
    /// empty, e.g. in the case of PreDeriveKeys for new FactorSource.
    ///
    /// And often this contains just some of the newly created instances, because
    /// some might have gone into the `cache_to_persist` instead.
    pub instances_to_be_used: InstancesToUseDirectly,

    /// Statistics about the execution
    pub statistics: FactorInstancesProviderStatistics,
}
impl ProvidedInstances {
    pub fn new(
        cache: FactorInstancesForSpecificNetworkCache,
        to_use_directly: InstancesToUseDirectly,
        statistics: FactorInstancesProviderStatistics,
    ) -> Self {
        Self {
            hidden_constructor: HiddenConstructor,
            cache_to_persist: cache,
            instances_to_be_used: to_use_directly,
            statistics,
        }
    }
    pub fn for_account_veci(
        cache: FactorInstancesForSpecificNetworkCache,
        instance: HierarchicalDeterministicFactorInstance,
        statistics: FactorInstancesProviderStatistics,
    ) -> Self {
        Self::new(cache, InstancesToUseDirectly::just(instance), statistics)
    }
}
