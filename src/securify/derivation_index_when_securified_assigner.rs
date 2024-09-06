use crate::prelude::*;

pub trait DerivationIndexWhenSecurifiedAssigner {
    fn assign_derivation_index(&self, profile: &Profile, network_id: NetworkID) -> HDPathComponent;
}
