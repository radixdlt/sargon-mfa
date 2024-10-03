use std::ops::SubAssign;

use crate::prelude::*;

pub const CACHE_SIZE: usize = 30;

/// Map per DerivationTemplate request to Derivation Entity Index
/// to use for this factor
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillCacheIndicesForFactor {
    /// The factor these indices are to be used for
    pub factor_source_id: FactorSourceIDFromHash,

    /// The indices have been calculated from an optional Profile and local
    /// offsets.
    pub indices: IndexMap<DerivationTemplate, HDPathComponent>,
}
impl FillCacheIndicesForFactor {
    pub fn just(
        factor_source_id: FactorSourceIDFromHash,
        derivation_template: DerivationTemplate,
        index: HDPathComponent,
    ) -> Self {
        Self {
            factor_source_id,
            indices: IndexMap::kv(derivation_template, index),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillCacheIndicesPerFactor {
    #[allow(dead_code)]
    hidden_constructor: HiddenConstructor,
    pub per_factor_source: IndexMap<FactorSourceIDFromHash, FillCacheIndicesForFactor>,
}
impl FillCacheIndicesPerFactor {
    pub fn just(
        factor_source_id: FactorSourceIDFromHash,
        derivation_template: DerivationTemplate,
        index: HDPathComponent,
    ) -> Self {
        Self {
            hidden_constructor: HiddenConstructor,
            per_factor_source: IndexMap::kv(
                factor_source_id,
                FillCacheIndicesForFactor::just(factor_source_id, derivation_template, index),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillCacheQuantitiesForFactor {
    pub factor_source_id: FactorSourceIDFromHash,

    /// Number of "account veci" instances to derive, using
    /// `factor_source_id` as the factor source
    pub account_vecis: usize,

    /// Number of "account mfa" instances to derive
    /// `factor_source_id` as the factor source
    pub account_mfa: usize,
}
impl FillCacheQuantitiesForFactor {
    pub fn fill(factor_source_id: FactorSourceIDFromHash) -> Self {
        Self::new(factor_source_id, CACHE_SIZE, CACHE_SIZE)
    }

    pub fn new(
        factor_source_id: FactorSourceIDFromHash,
        account_vecis: usize,
        account_mfa: usize,
    ) -> Self {
        Self {
            factor_source_id,
            account_mfa,
            account_vecis,
        }
    }

    pub fn quantity_for_template(&self, derivation_template: DerivationTemplate) -> usize {
        match derivation_template {
            DerivationTemplate::AccountVeci => self.account_vecis,
            DerivationTemplate::AccountMfa => self.account_mfa,
            DerivationTemplate::IdentityVeci => todo!(),
            DerivationTemplate::AccountRola => todo!(),
            DerivationTemplate::IdentityMfa => todo!(),
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
            .sub_assign(existing.unsecurified_accounts.len());

        fill_cache
            .account_mfa
            .sub_assign(existing.securified_accounts.len());

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
            per_factor_source: IndexMap::from_iter([(item.factor_source_id, item)]),
        }
    }
}
