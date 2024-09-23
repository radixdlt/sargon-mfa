use std::hash::Hash;

use crate::prelude::*;

// TODO figure out if we want this or `KnownTakenInstances`? Or neither or both
/// A collection of newly derived or just loaded cached factor instances.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct DerivedFactorInstances {
    unsecurified_factor_instances: IndexSet<VirtualEntityCreatingInstance>,
    securified_matrices_of_factor_instances: IndexSet<MatrixOfFactorInstances>,
}

impl DerivedFactorInstances {
    pub fn unsecurified_accounts(&self, _network_id: NetworkID) -> IndexSet<UnsecurifiedEntity> {
        self.unsecurified_factor_instances()
            .into_iter()
            .map(|veci| UnsecurifiedEntity::with_veci(veci, None))
            .collect()
    }
    pub fn accounts_unsecurified(&self, network_id: NetworkID) -> IndexSet<Account> {
        self.unsecurified_accounts(network_id)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn unsecurified_factor_instances(&self) -> IndexSet<VirtualEntityCreatingInstance> {
        self.unsecurified_factor_instances.clone()
    }
}
