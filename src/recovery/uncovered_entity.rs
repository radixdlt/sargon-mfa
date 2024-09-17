use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UncoveredEntity {
    pub on_chain: OnChainEntityState,
    pub key_hash_to_factor_instances:
        HashMap<PublicKeyHash, HierarchicalDeterministicFactorInstance>,
}
impl UncoveredEntity {
    pub fn new(
        on_chain: OnChainEntityState,
        key_hash_to_factor_instances: HashMap<
            PublicKeyHash,
            HierarchicalDeterministicFactorInstance,
        >,
    ) -> Self {
        Self {
            on_chain,
            key_hash_to_factor_instances,
        }
    }
}
