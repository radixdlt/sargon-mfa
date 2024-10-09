use crate::prelude::*;

/// Identical to `InternalFactorInstancesProviderOutcomeForFactor` but with
/// different field names, making it clear that the instances of `to_cache` field in the
/// "non-final" counterpart has already been cached, thus here named
/// `debug_was_cached`.
/// Furthermore all fields except `to_use_directly` are renamed to `debug_*` to make it clear they are only included for debugging purposes,
/// in fact, they are all put behind `#[cfg(test)]`
#[derive(Clone, Debug)]
pub struct FactorInstancesProviderOutcomeForFactor {
    #[allow(dead_code)]
    hidden: HiddenConstructor,

    /// The FactorSourceID of all the factor instances of this type.
    pub factor_source_id: FactorSourceIDFromHash,

    /// FactorInstances which are not saved into the cache.
    ///
    /// Might be empty
    pub to_use_directly: FactorInstances,

    /// FactorInstances which were saved into the cache
    ///
    /// Might be empty
    ///
    /// Useful for unit tests.
    #[cfg(test)]
    pub debug_was_cached: FactorInstances,

    /// FactorInstances which was found in the cache before the operation was
    /// executed.
    ///
    /// Might be empty
    ///
    /// Useful for unit tests.
    ///
    /// Might overlap with `to_use_directly`
    #[cfg(test)]
    pub debug_found_in_cache: FactorInstances,

    /// FactorInstances which was derived.
    ///
    /// Might be empty
    ///
    /// Useful for unit tests.
    ///
    /// Might overlap with `to_cache` and `to_use_directly`
    #[cfg(test)]
    pub debug_was_derived: FactorInstances,
}

impl From<InternalFactorInstancesProviderOutcomeForFactor>
    for FactorInstancesProviderOutcomeForFactor
{
    fn from(value: InternalFactorInstancesProviderOutcomeForFactor) -> Self {
        #[cfg(test)]
        let _self = Self {
            hidden: HiddenConstructor,
            factor_source_id: value.factor_source_id,
            to_use_directly: value.to_use_directly,
            debug_was_cached: value.to_cache,
            debug_found_in_cache: value.found_in_cache,
            debug_was_derived: value.newly_derived,
        };

        #[cfg(not(test))]
        let _self = Self {
            hidden: HiddenConstructor,
            factor_source_id: value.factor_source_id,
            to_use_directly: value.to_use_directly,
        };

        _self
    }
}
