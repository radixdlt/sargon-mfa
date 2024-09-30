use crate::prelude::*;

/// A NonEmpty collection of Accounts all on the SAME Network and all verified
/// to be Securified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecurifiedAccounts {
    network_id: NetworkID,
    accounts: IndexSet<SecurifiedAccount>,
}

impl IntoIterator for SecurifiedAccounts {
    type Item = SecurifiedAccount;
    type IntoIter = <IndexSet<SecurifiedAccount> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.accounts.clone().into_iter()
    }
}

impl SecurifiedAccounts {
    pub fn new(network_id: NetworkID, accounts: IndexSet<SecurifiedAccount>) -> Result<Self> {
        if accounts.is_empty() {
            return Err(CommonError::EmptyCollection);
        }
        if !accounts.iter().all(|a| a.network_id() == network_id) {
            return Err(CommonError::WrongNetwork);
        }
        Ok(Self {
            network_id,
            accounts,
        })
    }
    pub fn network_id(&self) -> NetworkID {
        self.network_id
    }
    pub fn len(&self) -> usize {
        self.accounts.len()
    }
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }
}
