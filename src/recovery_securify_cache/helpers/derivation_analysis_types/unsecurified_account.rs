use crate::prelude::*;

/// The HDFactorInstance, address and possibly third party deposit state of some
/// unsecurified entity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnsecurifiedAccount {
    account_address: AccountAddress,
    veci: VirtualEntityCreatingInstance,

    /// If we found this UnsecurifiedEntity while scanning OnChain using
    /// Gateway, we might have been able to read out the third party deposit
    /// settings.
    third_party_deposit: Option<ThirdPartyDepositPreference>,
}
impl UnsecurifiedAccount {
    pub fn address(&self) -> AccountAddress {
        self.account_address.clone()
    }

    pub fn factor_instance(&self) -> HierarchicalDeterministicFactorInstance {
        self.veci.factor_instance()
    }

    pub fn veci(&self) -> VirtualEntityCreatingInstance {
        self.veci.clone()
    }

    pub fn third_party_deposit(&self) -> Option<ThirdPartyDepositPreference> {
        self.third_party_deposit
    }
}
