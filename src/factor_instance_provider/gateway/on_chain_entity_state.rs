use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumAsInner)]
pub enum OnChainEntityState {
    Unsecurified(OnChainEntityUnsecurified),
    Securified(OnChainEntitySecurified),
}

impl OnChainEntityState {
    fn unsecurified(unsecurified: OnChainEntityUnsecurified) -> Self {
        Self::Unsecurified(unsecurified)
    }

    pub fn unsecurified_with(
        address: impl Into<AddressOfAccountOrPersona>,
        owner_key: PublicKeyHash,
    ) -> Self {
        Self::unsecurified(OnChainEntityUnsecurified::new(address, vec![owner_key]))
    }

    fn securified(securified: OnChainEntitySecurified) -> Self {
        Self::Securified(securified)
    }

    pub fn securified_with(
        address: impl Into<AddressOfAccountOrPersona>,
        access_controller: AccessController,
        owner_keys: Vec<PublicKeyHash>,
    ) -> Self {
        Self::securified(OnChainEntitySecurified::new(
            address,
            access_controller.clone(),
            owner_keys,
        ))
    }
}

impl OnChainEntityState {
    #[allow(unused)]
    pub fn address(&self) -> AddressOfAccountOrPersona {
        match self {
            OnChainEntityState::Unsecurified(account) => account.address.clone(),
            OnChainEntityState::Securified(account) => account.address.clone(),
        }
    }

    pub fn owner_keys(&self) -> HashSet<PublicKeyHash> {
        match self {
            OnChainEntityState::Unsecurified(account) => account.owner_keys(),
            OnChainEntityState::Securified(account) => account.owner_keys(),
        }
    }
}
