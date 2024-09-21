use crate::prelude::*;

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct HiddenConstructor;

/// A collection of collections of FactorInstances, all collections are disjoint,
/// i.e. no FactorInstance is present in more than one collection.
///
/// All FactorInstances are known to be not free, i.e. they are taken, meaning
/// they are already used by some Securified or Unsecurified entity, which we
/// know by having matched them against Profile or Gateway or both.
#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct KnownTakenInstances {
    hiding_ctor: HiddenConstructor,
    /// Unsecurified entities that were recovered
    pub recovered_unsecurified_entities: RecoveredUnsecurifiedEntities,

    /// Securified entities that were recovered
    pub recovered_securified_entities: RecoveredSecurifiedEntities,

    /// Securified entities that were not recovered
    pub unrecovered_securified_entities: UnrecoveredSecurifiedEntities,

    pub virtual_entity_creating_instances: VirtualEntityCreatingInstances,
}

impl IsFactorInstanceCollectionBase for KnownTakenInstances {
    fn factor_instances(&self) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        let mut set = self.recovered_unsecurified_entities.factor_instances();
        set.extend(self.recovered_securified_entities.factor_instances());
        set.extend(self.unrecovered_securified_entities.factor_instances());
        set.extend(self.virtual_entity_creating_instances.factor_instances());
        set
    }
}

impl HasSampleValues for KnownTakenInstances {
    fn sample() -> Self {
        Self::new(
            RecoveredUnsecurifiedEntities::sample(),
            RecoveredSecurifiedEntities::sample(),
            UnrecoveredSecurifiedEntities::sample(),
            VirtualEntityCreatingInstances::sample(),
        )
    }

    fn sample_other() -> Self {
        Self::new(
            RecoveredUnsecurifiedEntities::sample_other(),
            RecoveredSecurifiedEntities::sample_other(),
            UnrecoveredSecurifiedEntities::sample_other(),
            VirtualEntityCreatingInstances::sample(),
        )
    }
}

impl KnownTakenInstances {
    /// # Panics
    /// Panics if the collections of factor instances are not disjoint
    pub fn new(
        recovered_unsecurified_entities: RecoveredUnsecurifiedEntities,
        recovered_securified_entities: RecoveredSecurifiedEntities,
        unrecovered_securified_entities: UnrecoveredSecurifiedEntities,
        virtual_entity_creating_instances: VirtualEntityCreatingInstances,
    ) -> Self {
        assert_are_factor_instance_collections_disjoint(vec![
            &recovered_unsecurified_entities,
            &recovered_securified_entities,
            &unrecovered_securified_entities,
            &virtual_entity_creating_instances,
        ]);
        Self {
            hiding_ctor: HiddenConstructor,
            recovered_unsecurified_entities,
            recovered_securified_entities,
            unrecovered_securified_entities,
            virtual_entity_creating_instances,
        }
    }

    pub fn merge(self, other: Self) -> Self {
        Self::new(
            self.recovered_unsecurified_entities
                .merge(other.recovered_unsecurified_entities),
            self.recovered_securified_entities
                .merge(other.recovered_securified_entities),
            self.unrecovered_securified_entities
                .merge(other.unrecovered_securified_entities),
            self.virtual_entity_creating_instances
                .merge(other.virtual_entity_creating_instances),
        )
    }
}

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

    pub known_taken_instances: KnownTakenInstances,

    /// Used FactorSources which are not new - might be empty
    old_factor_sources: Vec<HDFactorSource>,

    /// Used FactorSource which are new - might be empty
    new_factor_sources: Vec<HDFactorSource>,
}

impl DerivationAndAnalysis {
    /// # Panics
    /// Panics if `old_factor_sources` intersects with `new_factor_sources`
    ///
    /// Panics if the collections of factor instances are not disjoint
    pub fn new(
        probably_free_instances: ProbablyFreeFactorInstances,
        known_taken_instances: KnownTakenInstances,
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
        assert_are_factor_instance_collections_disjoint(vec![
            &probably_free_instances,
            &known_taken_instances,
        ]);
        Self {
            probably_free_instances,
            known_taken_instances,
            old_factor_sources: old_factor_sources.into_iter().collect(),
            new_factor_sources: new_factor_sources.into_iter().collect(),
        }
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
            KnownTakenInstances::sample(),
            IndexSet::just(HDFactorSource::sample()),
            IndexSet::just(HDFactorSource::sample_other()),
        )
    }

    fn sample_other() -> Self {
        Self::new(
            ProbablyFreeFactorInstances::sample_other(),
            KnownTakenInstances::sample_other(),
            IndexSet::just(HDFactorSource::sample_other()),
            IndexSet::just(HDFactorSource::sample()),
        )
    }
}
