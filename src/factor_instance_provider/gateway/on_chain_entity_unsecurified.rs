use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OnChainEntityUnsecurified {
    pub address: AddressOfAccountOrPersona,
    pub owner_keys: Vec<PublicKeyHash>,
    /// TODO: we should read this...
    pub third_party_deposits: (),
}

impl OnChainEntityUnsecurified {
    pub fn new(
        address: impl Into<AddressOfAccountOrPersona>,
        owner_keys: Vec<PublicKeyHash>,
    ) -> Self {
        Self {
            address: address.into(),
            owner_keys,
            third_party_deposits: (),
        }
    }

    pub fn owner_keys(&self) -> HashSet<PublicKeyHash> {
        self.owner_keys.iter().cloned().collect()
    }
}
