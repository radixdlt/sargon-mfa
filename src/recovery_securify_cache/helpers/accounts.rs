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

    /// Should never be true, since we do not allow empty.
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }

    pub fn network_id(&self) -> NetworkID {
        self.network_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = Accounts;
    type Item = Account;

    #[test]
    fn empty_throws() {
        assert!(matches!(
            Sut::new(NetworkID::Mainnet, IndexSet::new()),
            Err(CommonError::EmptyCollection)
        ));
    }

    #[test]
    fn wrong_network_single() {
        assert!(matches!(
            Sut::new(NetworkID::Stokenet, IndexSet::just(Item::sample())),
            Err(CommonError::WrongNetwork)
        ));
    }

    #[test]
    fn wrong_network_two() {
        assert!(matches!(
            Sut::new(
                NetworkID::Stokenet,
                IndexSet::from_iter([Account::sample_other(), Item::sample(),])
            ),
            Err(CommonError::WrongNetwork)
        ));
    }

    #[test]
    fn ok_new() {
        let sut = Sut::new(NetworkID::Mainnet, IndexSet::just(Item::sample())).unwrap();
        assert!(!sut.is_empty());
    }
}
