use crate::prelude::*;

/// The outcome of `derive_and_analyze` method. Contains:
/// * set of "probably free" FactorInstances
/// * set of recovered entities in spaces
///     - KeySpace::Unsecurified
///     - KeySpace::Securified
/// * Set of Unrecovered SecurifiedEntities
/// * set of discovered new VECIs (and the address of the entity)
/// * set of involved FactorSource divided into:
///     - Existing FactorSources
///     - NewFactorSources
///
/// The set of "probably free" FactorInstances will be used to fill
/// the PreDerivedKeysCache!
///
/// All new FactorSources and Entities should be added to Profile -
/// either existing or new!
///
/// All discovered VECI should be added into their matched securified entity
///
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DerivationAndAnalysis {
    /// To be fed into a PreDerivedKeysCache
    pub probably_free_instances: ProbablyFreeFactorInstances,

    /// Unsecurified entities that were recovered
    pub recovered_unsecurified_entities: RecoveredUnsecurifiedEntities,

    /// Securified entities that were recovered
    pub recovered_securified_entities: RecoveredSecurifiedEntities,

    /// Securified entities that were not recovered
    pub unrecovered_securified_entities: UnrecoveredSecurifiedEntities,

    virtual_entity_creating_instances: Vec<HierarchicalDeterministicFactorInstance>,

    /// Used FactorSources which are not new - might be empty
    old_factor_sources: Vec<HDFactorSource>,

    /// Used FactorSource which are new - might be empty
    new_factor_sources: Vec<HDFactorSource>,
}

impl DerivationAndAnalysis {
    /// # Panics
    /// Panics if `old_factor_sources` intersects with `new_factor_sources`
    pub fn new(
        probably_free_instances: ProbablyFreeFactorInstances,
        recovered_unsecurified_entities: RecoveredUnsecurifiedEntities,
        recovered_securified_entities: RecoveredSecurifiedEntities,
        unrecovered_securified_entities: UnrecoveredSecurifiedEntities,
        virtual_entity_creating_instances: IndexSet<HierarchicalDeterministicFactorInstance>,
        old_factor_sources: IndexSet<HDFactorSource>,
        new_factor_sources: IndexSet<HDFactorSource>,
    ) -> Self {
        assert!(
            old_factor_sources
                .intersection(&new_factor_sources)
                .collect::<IndexSet<_>>()
                .is_empty(),
            "Discrepancy! FactorSource found in old an new, this is a programmer error!"
        );
        Self {
            probably_free_instances,
            recovered_unsecurified_entities,
            recovered_securified_entities,
            unrecovered_securified_entities,
            virtual_entity_creating_instances: virtual_entity_creating_instances
                .into_iter()
                .collect(),
            old_factor_sources: old_factor_sources.into_iter().collect(),
            new_factor_sources: new_factor_sources.into_iter().collect(),
        }
    }

    pub fn virtual_entity_creating_instances(
        &self,
    ) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        self.virtual_entity_creating_instances
            .clone()
            .into_iter()
            .collect()
    }

    pub fn new_factor_sources(&self) -> IndexSet<HDFactorSource> {
        self.new_factor_sources.clone().into_iter().collect()
    }

    pub fn old_factor_sources(&self) -> IndexSet<HDFactorSource> {
        self.old_factor_sources.clone().into_iter().collect()
    }

    pub fn all_factor_sources(&self) -> IndexSet<HDFactorSource> {
        let mut set = self.new_factor_sources();
        set.extend(self.old_factor_sources());
        set
    }
}

impl HasSampleValues for DerivationAndAnalysis {
    fn sample() -> Self {
        Self::new(
            ProbablyFreeFactorInstances::sample(),
            RecoveredUnsecurifiedEntities::sample(),
            RecoveredSecurifiedEntities::sample(),
            UnrecoveredSecurifiedEntities::sample(),
            IndexSet::from_iter([
                HierarchicalDeterministicFactorInstance::sample(),
                HierarchicalDeterministicFactorInstance::sample_other(),
            ]),
            IndexSet::just(HDFactorSource::sample()),
            IndexSet::just(HDFactorSource::sample_other()),
        )
    }

    fn sample_other() -> Self {
        Self::new(
            ProbablyFreeFactorInstances::sample_other(),
            RecoveredUnsecurifiedEntities::sample_other(),
            RecoveredSecurifiedEntities::sample_other(),
            UnrecoveredSecurifiedEntities::sample_other(),
            IndexSet::just(HierarchicalDeterministicFactorInstance::sample_other()),
            IndexSet::just(HDFactorSource::sample_other()),
            IndexSet::just(HDFactorSource::sample()),
        )
    }
}
