#![cfg(test)]
#![allow(unused)]

use crate::prelude::*;

pub struct TestGateway {
    has_internet_connection: bool,
    /// contains only current state for each entity
    entities: RwLock<HashMap<AddressOfAccountOrPersona, OnChainEntityState>>,

    /// contains historic state, we only ever add to this set, never remove.
    known_hashes: RwLock<HashSet<PublicKeyHash>>,
}

impl TestGateway {
    pub fn clone_snapshot(&self) -> Self {
        Self {
            has_internet_connection: self.has_internet_connection,
            known_hashes: RwLock::new(self.known_hashes.try_read().unwrap().clone()),
            entities: RwLock::new(self.entities.try_read().unwrap().clone()),
        }
    }
    pub fn new(has_internet_connection: bool) -> Self {
        Self {
            has_internet_connection,
            known_hashes: RwLock::new(HashSet::new()),
            entities: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for TestGateway {
    fn default() -> Self {
        Self::new(true)
    }
}

impl TestGateway {
    #[allow(unused)]
    pub fn debug_print(&self) {
        println!(
            "⛩️ known_hashes: {:?}",
            self.known_hashes.try_read().unwrap()
        );
        println!("⛩️ entities: {:?}", self.entities.try_read().unwrap().keys());
    }
}

#[async_trait::async_trait]
impl GatewayReadonly for TestGateway {
    async fn has_internet_connection(&self) -> bool {
        self.has_internet_connection
    }

    async fn is_key_hash_known(&self, hash: PublicKeyHash) -> Result<bool> {
        let is_known = self.known_hashes.try_read().unwrap().contains(&hash);
        Ok(is_known)
    }
    async fn get_entity_addresses_of_by_public_key_hashes(
        &self,
        hashes: HashSet<PublicKeyHash>,
    ) -> Result<HashMap<PublicKeyHash, HashSet<AddressOfAccountOrPersona>>> {
        let entities = self.entities.try_read().unwrap();
        let states = entities.values();

        Ok(hashes
            .iter()
            .filter_map(|k| {
                // N.B. we want this to always be single element (Axiom 1).
                let mut entities_references_hash = HashSet::<AddressOfAccountOrPersona>::new();
                for state in states.clone().filter(|x| x.owner_keys().contains(k)) {
                    entities_references_hash.insert(state.address());
                }
                if entities_references_hash.is_empty() {
                    None
                } else {
                    Some((k.clone(), entities_references_hash))
                }
            })
            .collect::<HashMap<PublicKeyHash, HashSet<AddressOfAccountOrPersona>>>())
    }

    async fn get_on_chain_entity(
        &self,
        address: AddressOfAccountOrPersona,
    ) -> Result<Option<OnChainEntityState>> {
        Ok(self.entities.try_read().unwrap().get(&address).cloned())
    }
}

impl TestGateway {
    async fn assert_not_securified(&self, address: &AddressOfAccountOrPersona) -> Result<()> {
        let is_already_securified = self.is_securified(address.clone()).await?;
        assert!(
            !is_already_securified,
            "Cannot unsecurify an already securified entity"
        );
        Ok(())
    }

    fn contains(&self, address: &AddressOfAccountOrPersona) -> bool {
        self.entities.try_read().unwrap().contains_key(address)
    }
}

#[async_trait::async_trait]
impl Gateway for TestGateway {
    async fn simulate_network_activity_for(&self, owner: AddressOfAccountOrPersona) -> Result<()> {
        self.assert_not_securified(&owner).await?;

        let owner_key = owner.public_key_hash();

        if self.contains(&owner) || self.known_hashes.try_read().unwrap().contains(&owner_key) {
            panic!("update not supported")
        } else {
            self.entities.try_write().unwrap().insert(
                owner.clone(),
                OnChainEntityState::unsecurified(owner, owner_key.clone()),
            );
            self.known_hashes.try_write().unwrap().insert(owner_key);
        }
        Ok(())
    }

    async fn set_securified_entity(
        &self,
        securified: SecurifiedEntityControl,
        owner: AddressOfAccountOrPersona,
    ) -> Result<()> {
        self.assert_not_securified(&owner).await?;

        let owner_keys = securified
            .matrix
            .all_factors()
            .iter()
            .map(|f| f.public_key_hash())
            .collect::<IndexSet<_>>();

        if self.contains(&owner) {
            self.entities.try_write().unwrap().remove(&owner);
        }

        self.known_hashes
            .try_write()
            .unwrap()
            .extend(owner_keys.clone());

        self.entities.try_write().unwrap().insert(
            owner.clone(),
            OnChainEntityState::securified(owner, securified.access_controller.clone(), owner_keys),
        );

        Ok(())
    }
}

mod tests {

    use super::*;

    type Sut = TestGateway;

    #[actix_rt::test]
    async fn test_has_internet_connection() {
        let has_internet_connection = true;
        let sut = Sut::new(has_internet_connection);
        let does_have_internet = sut.has_internet_connection().await;
        assert_eq!(does_have_internet, has_internet_connection);

        let has_internet_connection = false;
        let sut = Sut::new(has_internet_connection);
        let does_have_internet = sut.has_internet_connection().await;
        assert_eq!(does_have_internet, has_internet_connection);
    }

    #[actix_rt::test]
    async fn test_set_securified_account() {
        let sut = Sut::new(true);
        let address = AccountAddress::sample();
        let value = SecurifiedEntityControl::sample();
        sut.set_securified_account(value.clone(), &address)
            .await
            .unwrap();
        let on_chain = sut.get_on_chain_account(&address).await.unwrap();
        let is_securified = sut.is_securified(address.clone().into()).await.unwrap();
        assert!(is_securified);
        assert_eq!(
            on_chain
                .unwrap()
                .access_controller
                .unwrap()
                .metadata
                .unwrap()
                .scrypto_access_rules,
            ScryptoAccessRule::from(value.matrix)
        );
        let is_pub_key_known = sut
            .query_public_key_hash_is_known(IndexSet::just(address.public_key_hash()))
            .await
            .unwrap();
        assert_eq!(
            is_pub_key_known.get(&address.public_key_hash()).unwrap(),
            &false // veci is not retained...
        );
    }

    #[actix_rt::test]
    async fn test_set_securified_persona() {
        let sut = Sut::new(true);
        let address = IdentityAddress::sample();
        let value = SecurifiedEntityControl::sample();
        sut.set_securified_persona(value.clone(), &address)
            .await
            .unwrap();
        let on_chain = sut
            .get_on_chain_entity(AddressOfAccountOrPersona::Identity(address.clone()))
            .await
            .unwrap();
        assert_eq!(
            on_chain
                .unwrap()
                .access_controller
                .unwrap()
                .metadata
                .unwrap()
                .scrypto_access_rules,
            ScryptoAccessRule::from(value.matrix)
        );
    }
}
