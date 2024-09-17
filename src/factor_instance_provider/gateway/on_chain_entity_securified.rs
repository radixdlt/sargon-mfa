use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OnChainEntitySecurified {
    pub address: AddressOfAccountOrPersona,
    pub access_controller: AccessController,
    pub owner_keys: Vec<PublicKeyHash>,
}

impl OnChainEntitySecurified {
    pub fn new(
        address: impl Into<AddressOfAccountOrPersona>,
        access_controller: AccessController,
        owner_keys: Vec<PublicKeyHash>,
    ) -> Self {
        Self {
            address: address.into(),
            access_controller,
            owner_keys,
        }
    }
    pub fn owner_keys(&self) -> HashSet<PublicKeyHash> {
        self.owner_keys.iter().cloned().collect()
    }
}
