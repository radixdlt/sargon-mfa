use std::{marker::PhantomData, ops::SubAssign};

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
    pub fn merge_filling_cache(
        self,
        next_index_assigner: &NextDerivationEntityIndexAssigner,
    ) -> Self {
        let mut indices: IndexMap<DerivationTemplate, HDPathComponent> = self.indices;
        for template in enum_iterator::all::<DerivationTemplate>().into_iter() {
            if indices.get(&template).is_none() {
                let next = next_index_assigner.next(template, self.factor_source_id);
                indices.insert(template, next);
            }
        }
        Self {
            indices,
            factor_source_id: self.factor_source_id,
        }
    }
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
    pub fn merge_filling_cache(
        self,
        next_index_assigner: &NextDerivationEntityIndexAssigner,
    ) -> IndexMap<FactorSourceIDFromHash, FillCacheIndicesForFactor> {
        self.per_factor_source
            .into_iter()
            .map(|(k, v)| (k, v.merge_filling_cache(next_index_assigner)))
            .collect::<IndexMap<FactorSourceIDFromHash, FillCacheIndicesForFactor>>()
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstancesQuantity {
    // phantom: PhantomData<T>,
    pub value: usize,
}
impl InstancesQuantity {
    pub fn new(value: usize) -> Self {
        Self {
            // phantom: PhantomData,
            value,
        }
    }
}
impl Default for InstancesQuantity {
    fn default() -> Self {
        Self::new(CACHE_SIZE)
    }
}
impl From<usize> for InstancesQuantity {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quantities {
    /// Number of "account veci" instances to derive, using
    /// `factor_source_id` as the factor source
    pub account_veci: InstancesQuantity,

    /// Number of "account mfa" instances to derive
    /// `factor_source_id` as the factor source
    pub account_mfa: InstancesQuantity,

    /// Number of "identity veci" instances to derive, using
    /// `factor_source_id` as the factor source
    pub identity_veci: InstancesQuantity,

    /// Number of "identities mfa" instances to derive
    /// `factor_source_id` as the factor source
    pub identity_mfa: InstancesQuantity,
}
impl Default for Quantities {
    fn default() -> Self {
        Self {
            account_veci: Default::default(),
            account_mfa: Default::default(),
            identity_veci: Default::default(),
            identity_mfa: Default::default(),
        }
    }
}
impl Quantities {
    pub fn only(quantity: usize, template: DerivationTemplate) -> Self {
        match template {
            DerivationTemplate::AccountVeci => Self::new(quantity, 0, 0, 0),
            DerivationTemplate::AccountMfa => Self::new(0, quantity, 0, 0),
            DerivationTemplate::IdentityVeci => Self::new(0, 0, quantity, 0),
            DerivationTemplate::IdentityMfa => Self::new(0, 0, 0, quantity),
        }
    }
    pub fn all(quantity: usize) -> Self {
        Self::new(quantity, quantity, quantity, quantity)
    }
    pub fn new(
        account_veci: usize,
        account_mfa: usize,
        identity_veci: usize,
        identity_mfa: usize,
    ) -> Self {
        Self {
            account_mfa: account_mfa.into(),
            account_veci: account_veci.into(),
            identity_veci: identity_veci.into(),
            identity_mfa: identity_mfa.into(),
        }
    }

    pub fn quantity_for_template(
        &self,
        derivation_template: DerivationTemplate,
    ) -> InstancesQuantity {
        match derivation_template {
            DerivationTemplate::AccountVeci => self.account_veci,
            DerivationTemplate::AccountMfa => self.account_mfa,
            DerivationTemplate::IdentityVeci => self.identity_veci,
            DerivationTemplate::IdentityMfa => self.identity_mfa,
        }
    }

    pub fn set_quantity_for_template(
        &mut self,
        derivation_template: DerivationTemplate,
        quantity: impl Into<InstancesQuantity>,
    ) {
        let quantity = quantity.into();
        match derivation_template {
            DerivationTemplate::AccountVeci => self.account_veci = quantity,
            DerivationTemplate::AccountMfa => self.account_mfa = quantity,
            DerivationTemplate::IdentityVeci => self.identity_veci = quantity,
            DerivationTemplate::IdentityMfa => self.identity_mfa = quantity,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantitiesForFactor {
    pub factor_source_id: FactorSourceIDFromHash,

    pub quantities: Quantities,
}
impl QuantitiesForFactor {
    pub fn new(factor_source_id: FactorSourceIDFromHash, quantities: Quantities) -> Self {
        Self {
            factor_source_id,
            quantities,
        }
    }
    pub fn fill(factor_source_id: FactorSourceIDFromHash) -> Self {
        Self::new(factor_source_id, Quantities::default())
    }
}

impl QuantitiesForFactor {
    pub fn quantity_for_template(
        &self,
        derivation_template: DerivationTemplate,
    ) -> InstancesQuantity {
        self.quantities.quantity_for_template(derivation_template)
    }
}

// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct ToDerive;

// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct ToCache;

// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct ToUseDirectly;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuantitiesTripleForFactor {
    pub factor_source_id: FactorSourceIDFromHash,

    pub to_derive: Quantities,
    pub to_use_directly: Quantities,
    pub to_cache: Quantities,
}

impl QuantitiesForFactor {
    /// `FOR EACH derivation template FOR EACH factor source`
    /// (or: `FOR EACH factor source FOR EACH derivation template`)
    /// If
    /// CACHE_TARGET_SIZE: 30
    /// IN_CACHE: 25
    /// REQUESTED: 10,
    /// MORE_TO_DERIVE: ???
    /// We want to have: `CACHE_TARGET_SIZE` after `REQUESTED` are used,
    /// i.e. `IN_CACHE + MORE_TO_DERIVE = CACHE_TARGET_SIZE + REQUESTED`,
    /// thus `MORE_TO_DERIVE = CACHE_TARGET_SIZE + REQUESTED - IN_CACHE` =>
    /// thus `MORE_TO_DERIVE = 30 + 10 - 25 ==>
    /// thus `MORE_TO_DERIVE = 15` =>
    /// 15 > REQUESTED, so we can fulfill request, and 15 - REQUESTED = 5,
    /// which we will cache, so cache will contain 25 + 5 = 30 = CACHE_TARGET_SIZE.
    ///
    /// A simpler example:
    /// CACHE_TARGET_SIZE: 30
    /// IN_CACHE: 0
    /// REQUESTED: 1,
    /// MORE_TO_DERIVE = CACHE_TARGET_SIZE + REQUESTED - IN_CACHE` =>
    /// => 30 + 1 - 0 = 31.
    ///
    /// In case of NO CACHE, Mfa scenario (per factor):
    /// If
    /// CACHE_TARGET_SIZE: 30
    /// IN_CACHE: `NONE`, no cache
    /// REQUESTED: 10,
    /// MORE_TO_DERIVE: ???
    /// We just use the same calculation as above, but with `IN_CACHE` being 0.
    pub fn calculate_quantities_for_factor(
        self,
        requested: Quantities,
        in_cache: impl Into<Option<CollectionsOfFactorInstances>>,
    ) -> QuantitiesTripleForFactor {
        let in_cache = in_cache.into();

        let mut to_derive = Quantities::all(0);
        let mut to_use_directly = Quantities::all(0);
        let mut to_cache = Quantities::all(0);

        for template in enum_iterator::all::<DerivationTemplate>() {
            let r = requested.quantity_for_template(template).value;
            let from_cache = in_cache
                .as_ref()
                .map(|c| c.quantity_for_template(template))
                .unwrap_or_default();
            let more_to_derive = CACHE_SIZE + r - from_cache;

            println!(
                "🦧 template: {:?}, requested: {:?}, from_cache: {:?}, more_to_derive: {:?}",
                template, r, from_cache, more_to_derive
            );
            to_derive.set_quantity_for_template(template, more_to_derive);
            to_use_directly.set_quantity_for_template(template, r);
            assert_eq!(from_cache + more_to_derive - r, CACHE_SIZE);
            to_cache.set_quantity_for_template(template, CACHE_SIZE);
        }

        QuantitiesTripleForFactor {
            factor_source_id: self.factor_source_id,
            to_cache,
            to_use_directly,
            to_derive,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantitiesPerFactor {
    #[allow(dead_code)]
    hidden_constructor: HiddenConstructor,
    pub per_factor_source: IndexMap<FactorSourceIDFromHash, QuantitiesForFactor>,
}
impl QuantitiesPerFactor {
    pub fn just(item: QuantitiesForFactor) -> Self {
        Self {
            hidden_constructor: HiddenConstructor,
            per_factor_source: IndexMap::from_iter([(item.factor_source_id, item)]),
        }
    }
}
