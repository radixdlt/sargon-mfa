use std::sync::RwLock;

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OnChainAccountUnsecurified {
    account_address: AccountAddress,
    owner_keys: Vec<PublicKeyHash>,
}
impl OnChainAccountUnsecurified {
    pub fn new(account_address: AccountAddress, owner_keys: Vec<PublicKeyHash>) -> Self {
        Self {
            account_address,
            owner_keys,
        }
    }

    pub fn owner_keys(&self) -> HashSet<PublicKeyHash> {
        self.owner_keys.iter().cloned().collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OnChainAccountSecurified {
    account_address: AccountAddress,
    access_controller: AccessController,
    owner_keys: Vec<PublicKeyHash>,
}

impl OnChainAccountSecurified {
    pub fn new(
        account_address: AccountAddress,
        access_controller: AccessController,
        owner_keys: Vec<PublicKeyHash>,
    ) -> Self {
        Self {
            account_address,
            access_controller,
            owner_keys,
        }
    }
    pub fn owner_keys(&self) -> HashSet<PublicKeyHash> {
        self.owner_keys.iter().cloned().collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OnChainAccountState {
    Unsecurified(OnChainAccountUnsecurified),
    Securified(OnChainAccountSecurified),
}
impl OnChainAccountState {
    fn unsecurified(unsecurified: OnChainAccountUnsecurified) -> Self {
        Self::Unsecurified(unsecurified)
    }

    pub fn unsecurified_with(account_address: &AccountAddress, owner_key: PublicKeyHash) -> Self {
        Self::unsecurified(OnChainAccountUnsecurified::new(
            account_address.clone(),
            vec![owner_key],
        ))
    }

    fn securified(securified: OnChainAccountSecurified) -> Self {
        Self::Securified(securified)
    }

    pub fn securified_with(
        account_address: &AccountAddress,
        access_controller: AccessController,
        owner_keys: Vec<PublicKeyHash>,
    ) -> Self {
        Self::securified(OnChainAccountSecurified::new(
            account_address.clone(),
            access_controller.clone(),
            owner_keys,
        ))
    }
}

impl OnChainAccountState {
    #[allow(dead_code)]
    fn account_address(&self) -> AccountAddress {
        match self {
            OnChainAccountState::Unsecurified(account) => account.account_address.clone(),
            OnChainAccountState::Securified(account) => account.account_address.clone(),
        }
    }

    fn owner_keys(&self) -> HashSet<PublicKeyHash> {
        match self {
            OnChainAccountState::Unsecurified(account) => account.owner_keys(),
            OnChainAccountState::Securified(account) => account.owner_keys(),
        }
    }

    fn is_securified(&self) -> bool {
        matches!(self, OnChainAccountState::Securified(_))
    }
}

#[async_trait::async_trait]
pub trait GatewayReadonly {
    async fn get_account_address_of_by_public_key_hash(
        &self,
        hash: PublicKeyHash,
    ) -> Result<Option<AccountAddress>>;

    async fn get_on_chain_account(
        &self,
        account_address: &AccountAddress,
    ) -> Result<Option<OnChainAccountState>>;

    async fn get_owner_key_hashes(
        &self,
        account_address: &AccountAddress,
    ) -> Result<Option<HashSet<PublicKeyHash>>> {
        let on_chain_account = self.get_on_chain_account(account_address).await?;
        return Ok(on_chain_account.map(|account| account.owner_keys().clone()));
    }

    async fn is_securified(&self, account_address: &AccountAddress) -> Result<bool> {
        let account = self.get_on_chain_account(account_address).await?;
        Ok(account.map(|x| x.is_securified()).unwrap_or(false))
    }
}

#[async_trait::async_trait]
pub trait Gateway: GatewayReadonly {
    async fn set_unsecurified_account(
        &self,
        unsecurified: HierarchicalDeterministicFactorInstance,
        owner: &AccountAddress,
    ) -> Result<()>;
    async fn set_securified_account(
        &self,
        securified: SecurifiedEntityControl,
        owner: &AccountAddress,
    ) -> Result<()>;
}

pub async fn recover_accounts(
    _factors_sources: impl IntoIterator<Item = HDFactorSource>,
    _gateway: Arc<dyn GatewayReadonly>,
) -> Result<IndexSet<Account>> {
    todo!()
}

#[cfg(test)]
pub struct TestGateway {
    accounts: RwLock<HashMap<AccountAddress, OnChainAccountState>>,
}
#[cfg(test)]
impl Default for TestGateway {
    fn default() -> Self {
        Self {
            accounts: RwLock::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl GatewayReadonly for TestGateway {
    async fn get_account_address_of_by_public_key_hash(
        &self,
        hash: PublicKeyHash,
    ) -> Result<Option<AccountAddress>> {
        Ok(self
            .accounts
            .try_read()
            .unwrap()
            .values()
            .find(|account| account.owner_keys().contains(&hash))
            .map(|account| account.account_address()))
    }

    async fn get_on_chain_account(
        &self,
        account_address: &AccountAddress,
    ) -> Result<Option<OnChainAccountState>> {
        Ok(self
            .accounts
            .try_read()
            .unwrap()
            .get(account_address)
            .cloned())
    }
}
#[cfg(test)]
impl TestGateway {
    async fn assert_not_securified(&self, account_address: &AccountAddress) -> Result<()> {
        let is_already_securified = self.is_securified(account_address).await?;
        assert!(
            !is_already_securified,
            "Cannot unsecurify an already securified account"
        );
        Ok(())
    }

    fn contains(&self, account_address: &AccountAddress) -> bool {
        self.accounts
            .try_read()
            .unwrap()
            .contains_key(account_address)
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl Gateway for TestGateway {
    async fn set_unsecurified_account(
        &self,
        unsecurified: HierarchicalDeterministicFactorInstance,
        owner: &AccountAddress,
    ) -> Result<()> {
        self.assert_not_securified(owner).await?;

        let owner_key = unsecurified.public_key_hash();

        if self.contains(owner) {
            panic!("update not supported")
        } else {
            self.accounts.try_write().unwrap().insert(
                owner.clone(),
                OnChainAccountState::unsecurified_with(owner, owner_key),
            );
        }
        Ok(())
    }

    async fn set_securified_account(
        &self,
        securified: SecurifiedEntityControl,
        owner: &AccountAddress,
    ) -> Result<()> {
        self.assert_not_securified(owner).await?;

        let owner_keys = securified
            .matrix
            .all_factors()
            .iter()
            .map(|f| f.public_key_hash())
            .collect_vec();

        if self.contains(owner) {
            self.accounts.try_write().unwrap().remove(owner);
        }

        self.accounts.try_write().unwrap().insert(
            owner.clone(),
            OnChainAccountState::securified_with(
                owner,
                securified.access_controller.clone(),
                owner_keys,
            ),
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore = "stub"]
    #[actix_rt::test]
    async fn recovery_of_securified_accounts() {
        let all_factors = HDFactorSource::all();
        let gateway = Arc::new(TestGateway::default());

        let securified = Account::a7();
        gateway
            .set_securified_account(
                securified.security_state.as_securified().unwrap().clone(),
                &securified.entity_address(),
            )
            .await
            .unwrap();
        let recovered = recover_accounts(all_factors, gateway).await.unwrap();
        assert_eq!(recovered, IndexSet::just(securified))
    }
}
