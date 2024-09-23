use std::hash::Hash;

use crate::prelude::*;

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

    // pub fn account_addresses_of_securified(&self) -> IndexSet<AccountAddress> {
    //     self.securified_factor_instances
    //         .iter()
    //         .map(|f| AccountAddress::new(f.clone(), self.network_id))
    //         .collect()
    // }
    // pub fn all_account_addresses(&self) -> IndexSet<AccountAddress> {
    //     let mut addresses = IndexSet::new();
    //     addresses.extend(self.account_addresses_of_unsecurified());
    //     addresses.extend(self.account_addresses_of_securified());
    //     addresses
    // }
}
