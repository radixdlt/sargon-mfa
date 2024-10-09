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
#[cfg(test)]
mod tests {
    use super::*;
    type Sut = SecurifiedAccounts;
    type Item = SecurifiedAccount;
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
                IndexSet::from_iter([Item::sample_other(), Item::sample(),])
            ),
            Err(CommonError::WrongNetwork)
        ));
    }
    #[test]
    fn ok_new() {
        let network_id = NetworkID::Mainnet;
        let sut = Sut::new(network_id, IndexSet::just(Item::sample())).unwrap();
        assert!(!sut.is_empty());
        assert_eq!(sut.len(), 1);
        assert!(!sut.is_empty());
        assert_eq!(sut.network_id(), network_id);
    }
}
