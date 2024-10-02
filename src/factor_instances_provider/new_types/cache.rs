use std::sync::RwLock;

use crate::prelude::*;

/// On one specific network
#[derive(Debug)]
pub struct FactorInstancesForSpecificNetworkCache {
    #[allow(dead_code)]
    hidden_constructor: HiddenConstructor,
    pub network_id: NetworkID,
    per_factor_source: RwLock<IndexMap<FactorSourceIDFromHash, CollectionsOfFactorInstances>>,
}
impl FactorInstancesForSpecificNetworkCache {
    pub fn cloned_snapshot(&self) -> Self {
        Self {
            hidden_constructor: HiddenConstructor,
            network_id: self.network_id,
            per_factor_source: RwLock::new(self.per_factor_source.read().unwrap().clone()),
        }
    }
    pub fn append_for_factor(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        instances: ToCache,
    ) -> Result<()> {
        assert_eq!(self.network_id, instances.0.network);
        assert_eq!(factor_source_id, instances.0.factor_source_id);
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FactorInstanceFromCache {
    #[allow(dead_code)]
    hidden_constructor: HiddenConstructor,
    pub instance: HierarchicalDeterministicFactorInstance,
    /// if this was the last instance in the collection of instances, if it is,
    /// we SHOULD derive more!
    pub was_last_used: bool,
}
impl FactorInstanceFromCache {
    pub fn new(instance: HierarchicalDeterministicFactorInstance, was_last_used: bool) -> Self {
        Self {
            hidden_constructor: HiddenConstructor,
            instance,
            was_last_used,
        }
    }
}

impl FactorInstancesForSpecificNetworkCache {
    pub fn empty(network: NetworkID) -> Self {
        Self {
            hidden_constructor: HiddenConstructor,
            network_id: network,
            per_factor_source: RwLock::new(IndexMap::new()),
        }
    }

    /// Mutates self, consumes the next account veci if any, else returns None
    pub fn consume_account_veci(
        &self,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Option<FactorInstanceFromCache> {
        let mut default = CollectionsOfFactorInstances::empty(self.network_id, factor_source_id);
        let mut binding = self.per_factor_source.write().unwrap();
        let collections = binding.get_mut(&factor_source_id).unwrap_or(&mut default);
        if let Some(first) = collections.take_first_account_veci() {
            Some(FactorInstanceFromCache::new(
                first.instance(),
                collections.unsecurified_accounts.is_empty(),
            ))
        } else {
            None
        }
    }

    /// Does NOT mutate self
    pub fn peek_all_instances_for_factor_source(
        &self,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Option<CollectionsOfFactorInstances> {
        todo!()
    }
}

impl CollectionsOfFactorInstances {
    pub fn take_first_account_veci(&mut self) -> Option<AccountVeci> {
        self.unsecurified_accounts.swap_remove_index(0)
    }
}

#[derive(Default, Debug)]
pub struct FactorInstancesForEachNetworkCache {
    #[allow(dead_code)]
    hidden_constructor: HiddenConstructor,
    pub networks: HashMap<NetworkID, FactorInstancesForSpecificNetworkCache>,
}
impl FactorInstancesForEachNetworkCache {
    pub fn clone_for_network_or_empty(
        &self,
        network_id: NetworkID,
    ) -> FactorInstancesForSpecificNetworkCache {
        self.clone_for_network(network_id)
            .unwrap_or(FactorInstancesForSpecificNetworkCache::empty(network_id))
    }
    pub fn clone_for_network(
        &self,
        network_id: NetworkID,
    ) -> Option<FactorInstancesForSpecificNetworkCache> {
        self.networks.get(&network_id).map(|x| x.cloned_snapshot())
    }
    pub fn merge(&self, on_network: FactorInstancesForSpecificNetworkCache) -> Result<()> {
        todo!()
    }
}
