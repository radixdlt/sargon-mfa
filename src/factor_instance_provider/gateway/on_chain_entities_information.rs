use crate::prelude::*;

/// "Probably" since we might not have all the information to be sure, since
/// Gateway might not keep track of past FactorInstances, some of the FactorInstances
/// in KeySpace::Securified might in fact have been used in the past for some entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbablyFreeFactorInstances(IndexSet<HierarchicalDeterministicFactorInstance>);
impl ProbablyFreeFactorInstances {
    pub fn new(instances: IndexSet<HierarchicalDeterministicFactorInstance>) -> Self {
        Self(instances)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OnChainEntityInformation {
    /// Locally known only
    factor_instance: HierarchicalDeterministicFactorInstance,
    on_chain_entity: OnChainEntityState,
}
impl OnChainEntityInformation {
    pub fn new(
        factor_instance: HierarchicalDeterministicFactorInstance,
        on_chain_entity: OnChainEntityState,
    ) -> Self {
        Self {
            factor_instance,
            on_chain_entity,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnChainEntitiesInformation {
    /// We have to ensure Sargon never sets the same key on two different accounts,
    /// so that the values here can be single item, rather than `IndexSet<OnChainEntityInformation>`
    /// per factor instance.
    info_by_factor_instances:
        IndexMap<HierarchicalDeterministicFactorInstance, OnChainEntityInformation>,
    probably_free: ProbablyFreeFactorInstances,
}
impl OnChainEntitiesInformation {
    pub fn new(
        info_by_factor_instances: IndexMap<
            HierarchicalDeterministicFactorInstance,
            OnChainEntityInformation,
        >,
        probably_free: IndexSet<HierarchicalDeterministicFactorInstance>,
    ) -> Self {
        Self {
            info_by_factor_instances,
            probably_free: ProbablyFreeFactorInstances::new(probably_free),
        }
    }
}
