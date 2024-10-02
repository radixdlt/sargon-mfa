use std::ops::SubAssign;

use crate::prelude::*;

pub const CACHE_SIZE: u32 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillCacheQuantitiesForFactor {
    pub factor_source_id: FactorSourceIDFromHash,

    /// Number of "account veci" instances to derive, using
    /// `factor_source_id` as the factor source
    pub account_vecis: u32,

    /// Number of "account mfa" instances to derive
    /// `factor_source_id` as the factor source
    pub account_mfa: u32,
}
impl FillCacheQuantitiesForFactor {
    pub fn fill(factor_source_id: FactorSourceIDFromHash) -> Self {
        Self::new(factor_source_id, CACHE_SIZE, CACHE_SIZE)
    }
    pub fn new(
        factor_source_id: FactorSourceIDFromHash,
        account_vecis: u32,
        account_mfa: u32,
    ) -> Self {
        Self {
            factor_source_id,
            account_mfa,
            account_vecis,
        }
    }

    pub fn subtracting_existing(
        self,
        existing: impl Into<Option<CollectionsOfFactorInstances>>,
    ) -> Self {
        let Some(existing) = existing.into() else {
            return self;
        };
        let mut fill_cache = self;
        fill_cache
            .account_vecis
            .sub_assign(existing.unsecurified_accounts.len() as u32);

        fill_cache
            .account_mfa
            .sub_assign(existing.securified_accounts.len() as u32);

        fill_cache
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillCacheQuantitiesPerFactor {
    #[allow(dead_code)]
    hidden_constructor: HiddenConstructor,
    pub per_factor_source: IndexMap<FactorSourceIDFromHash, FillCacheQuantitiesForFactor>,
}
impl FillCacheQuantitiesPerFactor {
    pub fn just(item: FillCacheQuantitiesForFactor) -> Self {
        Self {
            hidden_constructor: HiddenConstructor,
            per_factor_source: IndexMap::from_iter([(item.factor_source_id.clone(), item)]),
        }
    }
}
