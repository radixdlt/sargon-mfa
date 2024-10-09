use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct FactorInstancesProviderOutcomeForFactorFinal {
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
    pub debug_was_cached: FactorInstances,

    /// FactorInstances which was found in the cache before the operation was
    /// executed.
    ///
    /// Might be empty
    ///
    /// Useful for unit tests.
    ///
    /// Might overlap with `to_use_directly`
    pub debug_found_in_cache: FactorInstances,

    /// FactorInstances which was derived.
    ///
    /// Might be empty
    ///
    /// Useful for unit tests.
    ///
    /// Might overlap with `to_cache` and `to_use_directly`
    pub debug_was_derived: FactorInstances,
}

impl From<FactorInstancesProviderOutcomeForFactorNonFinal>
    for FactorInstancesProviderOutcomeForFactorFinal
{
    fn from(value: FactorInstancesProviderOutcomeForFactorNonFinal) -> Self {
        Self {
            hidden: HiddenConstructor,
            factor_source_id: value.factor_source_id,
            to_use_directly: value.to_use_directly,
            debug_was_cached: value.to_cache,
            debug_found_in_cache: value.found_in_cache,
            debug_was_derived: value.newly_derived,
        }
    }
}

impl From<FactorInstancesProviderOutcomeNonFinal> for FactorInstancesProviderOutcomeFinal {
    fn from(value: FactorInstancesProviderOutcomeNonFinal) -> Self {
        Self {
            per_factor: value
                .per_factor
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FactorInstancesProviderOutcomeForFactorNonFinal {
    #[allow(dead_code)]
    hidden: HiddenConstructor,

    /// The FactorSourceID of all the factor instances of this type.
    pub factor_source_id: FactorSourceIDFromHash,

    /// FactorInstances which are saved into the cache
    ///
    /// Might be empty
    pub to_cache: FactorInstances,

    /// FactorInstances which are not saved into the cache.
    ///
    /// Might be empty
    pub to_use_directly: FactorInstances,

    /// FactorInstances which was found in the cache before the operation was
    /// executed.
    ///
    /// Might be empty
    ///
    /// Useful for unit tests.
    ///
    /// Might overlap with `to_use_directly`
    pub found_in_cache: FactorInstances,

    /// FactorInstances which was newly derived.
    ///
    /// Might be empty
    ///
    /// Useful for unit tests.
    ///
    /// Might overlap with `to_cache` and `to_use_directly`
    pub newly_derived: FactorInstances,
}
impl FactorInstancesProviderOutcomeForFactorNonFinal {
    pub fn new(
        factor_source_id: FactorSourceIDFromHash,
        to_cache: FactorInstances,
        to_use_directly: FactorInstances,
        found_in_cache: FactorInstances,
        newly_derived: FactorInstances,
    ) -> Self {
        let assert_factor = |xs: &FactorInstances| {
            assert!(
                xs.factor_instances()
                    .iter()
                    .all(|x| x.factor_source_id() == factor_source_id),
                "Discrepancy factor source id"
            );
        };
        assert_factor(&to_cache);
        assert_factor(&to_use_directly);
        assert_factor(&found_in_cache);
        assert_factor(&newly_derived);

        Self {
            hidden: HiddenConstructor,
            factor_source_id,
            to_cache,
            to_use_directly,
            found_in_cache,
            newly_derived,
        }
    }

    fn satisfied_by_cache(
        factor_source_id: FactorSourceIDFromHash,
        found_in_cache: FactorInstances,
    ) -> Self {
        let to_use_directly = found_in_cache.clone();

        // nothing to cache
        let to_cache = FactorInstances::default();

        // nothing was derived
        let newly_derived = FactorInstances::default();

        Self::new(
            factor_source_id,
            to_cache,
            to_use_directly,
            found_in_cache,
            newly_derived,
        )
    }
}

#[derive(Clone, Debug)]
pub struct FactorInstancesProviderOutcomeFinal {
    pub per_factor: IndexMap<FactorSourceIDFromHash, FactorInstancesProviderOutcomeForFactorFinal>,
}

#[cfg(test)]
impl FactorInstancesProviderOutcomeFinal {
    pub fn newly_derived_instances_from_all_factor_sources(&self) -> FactorInstances {
        self.per_factor
            .values()
            .flat_map(|x| x.debug_was_derived.factor_instances())
            .collect()
    }

    pub fn total_number_of_newly_derived_instances(&self) -> usize {
        self.newly_derived_instances_from_all_factor_sources().len()
    }

    pub fn derived_any_new_instance_for_any_factor_source(&self) -> bool {
        self.total_number_of_newly_derived_instances() > 0
    }

    pub fn instances_found_in_cache_from_all_factor_sources(&self) -> FactorInstances {
        self.per_factor
            .values()
            .flat_map(|x| x.debug_found_in_cache.factor_instances())
            .collect()
    }

    pub fn total_number_of_instances_found_in_cache(&self) -> usize {
        self.instances_found_in_cache_from_all_factor_sources()
            .len()
    }

    pub fn found_any_instances_in_cache_for_any_factor_source(&self) -> bool {
        self.total_number_of_instances_found_in_cache() > 0
    }
}

#[derive(Clone, Debug)]
pub struct FactorInstancesProviderOutcomeNonFinal {
    pub per_factor:
        IndexMap<FactorSourceIDFromHash, FactorInstancesProviderOutcomeForFactorNonFinal>,
}

impl FactorInstancesProviderOutcomeNonFinal {
    pub fn new(
        per_factor: IndexMap<
            FactorSourceIDFromHash,
            FactorInstancesProviderOutcomeForFactorNonFinal,
        >,
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
                        FactorInstancesProviderOutcomeForFactorNonFinal::satisfied_by_cache(k, v),
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
            fn build(self) -> FactorInstancesProviderOutcomeForFactorNonFinal {
                let to_cache = FactorInstances::from(self.to_cache);
                let to_use_directly = FactorInstances::from(self.to_use_directly);
                let found_in_cache = FactorInstances::from(self.found_in_cache);
                let newly_derived = FactorInstances::from(self.newly_derived);
                FactorInstancesProviderOutcomeForFactorNonFinal::new(
                    self.factor_source_id,
                    to_cache,
                    to_use_directly,
                    found_in_cache,
                    newly_derived,
                )
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
                .collect::<IndexMap<FactorSourceIDFromHash, FactorInstancesProviderOutcomeForFactorNonFinal>>(),
        )
    }
}
