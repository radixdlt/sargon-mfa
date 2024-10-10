use crate::prelude::*;

pub const CACHE_FILLING_QUANTITY: usize = 30;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum QuantityToCacheToUseDirectly {
    OnlyCacheFilling {
        /// `CACHE_FILLING_QUANTITY` - `FOUND_IN_CACHE`
        fill_cache: usize,
        /// We peeked into the cache and found FactorInstance with the max index, which we must used
        /// when we calculate the next index path, we are gonna do `max(max_from_profile, max_from_cache)`
        /// where `max_from_cache` is this `max_index` field.
        instance_with_max_index: Option<HierarchicalDeterministicFactorInstance>,
    },

    /// We will derive `remaining + extra_to_fill_cache` more instances
    ///
    /// If:
    /// CACHE_FILLING_QUANTITY: 30
    /// FOUND_IN_CACHE: 12
    /// REQUESTED: 14
    ///
    /// Then `remaining` below will be `2` and `extra_to_fill_cache` will be
    /// `CACHE_FILLING_QUANTITY` (`30`) since all `FOUND_IN_CACHE` instances
    /// will be used and method `total_quantity_to_derive` below will return
    /// `2 + 30 = 32`
    ToCacheToUseDirectly {
        /// Remaining quantity to satisfy the request, `originally_requested - from_cache_instances.len()`
        /// Used later to split the newly derived instances into two groups, to cache and to use directly,
        /// can be zero.
        remaining: usize,

        /// Typically `CACHE_FILLING_QUANTITY` (always?)
        extra_to_fill_cache: usize,
    },
}

impl QuantityToCacheToUseDirectly {
    pub fn max_index(&self) -> Option<HierarchicalDeterministicFactorInstance> {
        match self {
            Self::OnlyCacheFilling {
                fill_cache: _,
                instance_with_max_index,
            } => instance_with_max_index.clone(),
            Self::ToCacheToUseDirectly { .. } => None,
        }
    }
    pub fn total_quantity_to_derive(&self) -> usize {
        match self {
            Self::OnlyCacheFilling { fill_cache, .. } => *fill_cache,
            Self::ToCacheToUseDirectly {
                remaining,
                extra_to_fill_cache,
            } => *remaining + *extra_to_fill_cache,
        }
    }
}
