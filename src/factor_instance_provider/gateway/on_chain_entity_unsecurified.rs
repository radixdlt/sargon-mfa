use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OnChainEntityUnsecurified {
    pub address: AddressOfAccountOrPersona,
    pub owner_keys: Vec<PublicKeyHash>,
}

impl OnChainEntityUnsecurified {
    pub fn new(
        address: impl Into<AddressOfAccountOrPersona>,
        owner_keys: Vec<PublicKeyHash>,
    ) -> Self {
        Self {
            address: address.into(),
            owner_keys,
        }
    }

    pub fn owner_keys(&self) -> HashSet<PublicKeyHash> {
        self.owner_keys.iter().cloned().collect()
    }
}
