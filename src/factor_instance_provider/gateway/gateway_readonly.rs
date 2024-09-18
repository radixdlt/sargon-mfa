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

    /// UNSURE IF WE CAN GET THIS. But we can ask our colleagues in #team-interface.
    async fn query_info_about_entities_by_factor_instances(
        &self,
        factor_instances: IndexSet<HierarchicalDeterministicFactorInstance>,
    ) -> Result<OnChainEntitiesInformation> {
        let public_key_to_instance_map = factor_instances
            .into_iter()
            .map(|fi| (fi.public_key_hash(), fi))
            .collect::<HashMap<_, _>>();

        let map_keyhash_to_address = self
            .get_entity_addresses_of_by_public_key_hashes(
                public_key_to_instance_map
                    .keys()
                    .cloned()
                    .collect::<HashSet<_>>(),
            )
            .await?;

        let mut info_by_factor_instances =
            IndexMap::<HierarchicalDeterministicFactorInstance, OnChainEntityInformation>::new();
        let mut probably_free = IndexSet::<HierarchicalDeterministicFactorInstance>::new();

        for (hash, addresses) in map_keyhash_to_address.into_iter() {
            for address in addresses {
                /* typically only one element */
                let factor_instance = public_key_to_instance_map.get(&hash).unwrap();
                let maybe_entity = self.get_on_chain_entity(address).await?;
                match maybe_entity {
                    Some(on_chain) => {
                        info_by_factor_instances.insert(
                            factor_instance.clone(),
                            OnChainEntityInformation::new(factor_instance.clone(), on_chain),
                        );
                    }
                    None => {
                        probably_free.insert(factor_instance.clone());
                    }
                };
            }
        }
        Ok(OnChainEntitiesInformation::new(
            info_by_factor_instances,
            probably_free,
        ))
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
