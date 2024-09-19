use std::ops::Index;

use crate::prelude::*;

#[async_trait::async_trait]
pub trait GatewayReadonly: Sync + Send {
    async fn has_internet_connection(&self) -> bool;
    async fn is_key_hash_known(&self, hash: PublicKeyHash) -> Result<bool>;

    async fn query_public_key_hash_is_known(
        &self,
        hashes: IndexSet<PublicKeyHash>,
    ) -> Result<HashMap<PublicKeyHash, bool>> {
        let mut is_known_map = HashMap::<PublicKeyHash, bool>::new();
        for hash in hashes.into_iter() {
            let is_known = self.is_key_hash_known(hash.clone()).await?;
            is_known_map.insert(hash, is_known);
        }
        Ok(is_known_map)
    }

    async fn get_entity_addresses_of_by_public_key_hashes(
        &self,
        hashes: HashSet<PublicKeyHash>,
    ) -> Result<HashMap<PublicKeyHash, HashSet<AddressOfAccountOrPersona>>>;

    async fn get_on_chain_entity(
        &self,
        address: AddressOfAccountOrPersona,
    ) -> Result<Option<OnChainEntityState>>;

    async fn get_on_chain_account(
        &self,
        account_address: &AccountAddress,
    ) -> Result<Option<OnChainEntityState>> {
        self.get_on_chain_entity(account_address.clone().into())
            .await
    }

    async fn get_owner_key_hashes(
        &self,
        address: AddressOfAccountOrPersona,
    ) -> Result<Option<HashSet<PublicKeyHash>>> {
        let on_chain_account = self.get_on_chain_entity(address).await?;
        return Ok(on_chain_account.map(|account| account.owner_keys().clone()));
    }

    async fn is_securified(&self, address: AddressOfAccountOrPersona) -> Result<bool> {
        let entity = self.get_on_chain_entity(address).await?;
        Ok(entity.map(|x| x.is_securified()).unwrap_or(false))
    }
}
