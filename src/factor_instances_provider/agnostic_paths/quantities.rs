use crate::prelude::*;

pub const CACHE_FILLING_QUANTITY: usize = 30;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum QuantityToCacheToUseDirectly {
    OnlyCacheFilling {
        /// Typically (always?) `CACHE_FILLING_QUANTITY`
        fill_cache: usize,
    },

    /// We will derive `remaining + extra_to_fill_cache` more instances
    ToCacheToUseDirectly {
        /// Remaining quantity to satisfy the request, `originally_requested - from_cache_instances.len()`
        /// Used later to split the newly derived instances into two groups, to cache and to use directly,
        /// can be zero.
        remaining: usize,

        /// Typically (always?) `CACHE_FILLING_QUANTITY`
        extra_to_fill_cache: usize,
    },
}

impl QuantityToCacheToUseDirectly {
    pub fn total_quantity_to_derive(&self) -> usize {
        match self {
            Self::OnlyCacheFilling { fill_cache } => *fill_cache,
            Self::ToCacheToUseDirectly {
                remaining,
                extra_to_fill_cache,
            } => {
                let total = *remaining + *extra_to_fill_cache;
                println!(
                    "🐌 total: {} ({} + {})",
                    total, *remaining, *extra_to_fill_cache
                );
                total
            }
        }
    }
}
