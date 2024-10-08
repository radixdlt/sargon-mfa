use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct FactorInstancesProviderOutcomeForFactor {
    pub factor_source_id: FactorSourceIDFromHash,

    /// Might be empty
    pub to_cache: FactorInstances,
    /// Might be empty
    pub to_use_directly: FactorInstances,

    /// LESS IMPORTANT - for tests...
    /// might overlap with `to_use_directly`
    pub found_in_cache: FactorInstances,
    /// might overlap with `to_cache` and `to_use_directly`
    pub newly_derived: FactorInstances,
}
impl FactorInstancesProviderOutcomeForFactor {
    fn satisfied_by_cache(
        factor_source_id: FactorSourceIDFromHash,
        found_in_cache: FactorInstances,
    ) -> Self {
        Self {
            factor_source_id,
            found_in_cache: found_in_cache.clone(),
            to_use_directly: found_in_cache.clone(),
            to_cache: FactorInstances::default(),
            newly_derived: FactorInstances::default(),
        }
    }
}

pub struct FactorInstancesProviderOutcome {
    pub per_factor: IndexMap<FactorSourceIDFromHash, FactorInstancesProviderOutcomeForFactor>,
}
impl FactorInstancesProviderOutcome {
    pub fn new(
        per_factor: IndexMap<FactorSourceIDFromHash, FactorInstancesProviderOutcomeForFactor>,
    ) -> Self {
        Self { per_factor }
    }
    pub fn satisfied_by_cache(
        pf_found_in_cache: IndexMap<FactorSourceIDFromHash, FactorInstances>,
    ) -> Self {
        Self::new(
            pf_found_in_cache
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        FactorInstancesProviderOutcomeForFactor::satisfied_by_cache(k, v),
                    )
                })
                .collect(),
        )
    }
    pub fn transpose(
        pf_to_cache: IndexMap<FactorSourceIDFromHash, FactorInstances>,
        pf_to_use_directly: IndexMap<FactorSourceIDFromHash, FactorInstances>,
        pf_found_in_cache: IndexMap<FactorSourceIDFromHash, FactorInstances>,
        pf_newly_derived: IndexMap<FactorSourceIDFromHash, FactorInstances>,
    ) -> Self {
        struct Builder {
            factor_source_id: FactorSourceIDFromHash,

            /// Might be empty
            pub to_cache: IndexSet<HierarchicalDeterministicFactorInstance>,
            /// Might be empty
            pub to_use_directly: IndexSet<HierarchicalDeterministicFactorInstance>,

            /// LESS IMPORTANT - for tests...
            /// might overlap with `to_use_directly`
            pub found_in_cache: IndexSet<HierarchicalDeterministicFactorInstance>,
            /// might overlap with `to_cache` and `to_use_directly`
            pub newly_derived: IndexSet<HierarchicalDeterministicFactorInstance>,
        }
        impl Builder {
            fn build(self) -> FactorInstancesProviderOutcomeForFactor {
                FactorInstancesProviderOutcomeForFactor {
                    factor_source_id: self.factor_source_id,
                    to_cache: FactorInstances::from(self.to_cache),
                    to_use_directly: FactorInstances::from(self.to_use_directly),
                    found_in_cache: FactorInstances::from(self.found_in_cache),
                    newly_derived: FactorInstances::from(self.newly_derived),
                }
            }
            fn new(factor_source_id: FactorSourceIDFromHash) -> Self {
                Self {
                    factor_source_id,
                    to_cache: IndexSet::new(),
                    to_use_directly: IndexSet::new(),
                    found_in_cache: IndexSet::new(),
                    newly_derived: IndexSet::new(),
                }
            }
        }
        let mut builders = IndexMap::<FactorSourceIDFromHash, Builder>::new();

        for (factor_source_id, instances) in pf_found_in_cache {
            if let Some(builder) = builders.get_mut(&factor_source_id) {
                builder.found_in_cache.extend(instances.factor_instances());
            } else {
                let mut builder = Builder::new(factor_source_id);
                builder.found_in_cache.extend(instances.factor_instances());
                builders.insert(factor_source_id, builder);
            }
        }

        for (factor_source_id, instances) in pf_newly_derived {
            if let Some(builder) = builders.get_mut(&factor_source_id) {
                builder.newly_derived.extend(instances.factor_instances());
            } else {
                let mut builder = Builder::new(factor_source_id);
                builder.newly_derived.extend(instances.factor_instances());
                builders.insert(factor_source_id, builder);
            }
        }

        for (factor_source_id, instances) in pf_to_cache {
            if let Some(builder) = builders.get_mut(&factor_source_id) {
                builder.to_cache.extend(instances.factor_instances());
            } else {
                let mut builder = Builder::new(factor_source_id);
                builder.to_cache.extend(instances.factor_instances());
                builders.insert(factor_source_id, builder);
            }
        }

        for (factor_source_id, instances) in pf_to_use_directly {
            if let Some(builder) = builders.get_mut(&factor_source_id) {
                builder.to_use_directly.extend(instances.factor_instances());
            } else {
                let mut builder = Builder::new(factor_source_id);
                builder.to_use_directly.extend(instances.factor_instances());
                builders.insert(factor_source_id, builder);
            }
        }

        Self::new(
            builders
                .into_iter()
                .map(|(k, v)| (k, v.build()))
                .collect::<IndexMap<FactorSourceIDFromHash, FactorInstancesProviderOutcomeForFactor>>(),
        )
    }
}
