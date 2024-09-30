use crate::prelude::*;

/// A NonEmpty collection of Accounts all on the SAME Network
/// but mixed if they are securified or unsecurified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Accounts {
    pub network_id: NetworkID,
    accounts: IndexSet<Account>,
}

impl Accounts {
    pub fn just(account: Account) -> Self {
        Self {
            network_id: account.network_id(),
            accounts: IndexSet::just(account),
        }
    }

    pub fn new(network_id: NetworkID, accounts: IndexSet<Account>) -> Result<Self> {
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
}

impl IntoIterator for Accounts {
    type Item = Account;
    type IntoIter = <IndexSet<Account> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.accounts.clone().into_iter()
    }
}

impl Accounts {
    pub fn len(&self) -> usize {
        self.accounts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }

    pub fn network_id(&self) -> NetworkID {
        self.network_id
    }
}
