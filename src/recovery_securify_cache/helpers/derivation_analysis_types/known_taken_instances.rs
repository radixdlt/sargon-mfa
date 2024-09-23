use crate::prelude::*;

/// A type used to hide a constructor for some other type, use
/// it like this:
///
/// ```rust
/// pub struct ValidatedName {
///     hiding_ctor: HiddenConstructor,
///     pub name: String,
///     pub name_appended_to_name: String // validated!
/// }
/// ```
///
/// Making it impossible to create `ValidatedName` with invalid value!
///
#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct HiddenConstructor;

// TODO figure out if we want this or `DerivedFactorInstances`? Or neither or both
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
