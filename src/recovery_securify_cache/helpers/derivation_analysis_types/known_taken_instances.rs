use crate::prelude::*;

// TODO figure out if we want this or `DerivedFactorInstances`? Or neither or both
/// A collection of collections of FactorInstances, all collections are disjoint,
/// i.e. no FactorInstance is present in more than one collection.
///
/// All FactorInstances are known to be not free, i.e. they are taken, meaning
/// they are already used by some Securified or Unsecurified entity, which we
/// know by having matched them against Profile or Gateway or both.
#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct EntitiesFromAnalysis {
    hiding_ctor: HiddenConstructor,
    /// Unsecurified entities that were recovered
    pub recovered_unsecurified_entities: RecoveredUnsecurifiedEntities,

    /// Securified entities that were recovered
    pub recovered_securified_entities: RecoveredSecurifiedEntities,

    /// Securified entities that were not recovered
    pub unrecovered_securified_entities: UnrecoveredSecurifiedEntities,

    pub virtual_entity_creating_instances: VirtualEntityCreatingInstances,
}

impl IsFactorInstanceCollectionBase for EntitiesFromAnalysis {
    fn factor_instances(&self) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        let mut set = self.recovered_unsecurified_entities.factor_instances();
        set.extend(self.recovered_securified_entities.factor_instances());
        set.extend(self.unrecovered_securified_entities.factor_instances());
        set.extend(self.virtual_entity_creating_instances.factor_instances());
        set
    }
}

impl HasSampleValues for EntitiesFromAnalysis {
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

impl EntitiesFromAnalysis {
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

    pub fn recovered_entities(&self) -> IndexSet<AccountOrPersona> {
        let mut set = self.recovered_unsecurified_entities.entities();
        set.extend(self.recovered_securified_entities.entities());
        set
    }

    pub fn recovered_unsecurified_entities(&self) -> IndexSet<AccountOrPersona> {
        self.recovered_unsecurified_entities.entities()
    }

    pub fn recovered_unsecurified_accounts(&self) -> IndexSet<Account> {
        self.recovered_unsecurified_entities()
            .into_iter()
            .filter_map(|e| e.as_account_entity().cloned())
            .collect()
    }

    pub fn recovered_securified_entities(&self) -> IndexSet<AccountOrPersona> {
        self.recovered_securified_entities.entities()
    }

    pub fn recovered_securified_accounts(&self) -> IndexSet<Account> {
        self.recovered_securified_entities()
            .into_iter()
            .filter_map(|e| e.as_account_entity().cloned())
            .collect()
    }
}
