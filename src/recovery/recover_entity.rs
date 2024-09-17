use std::{hash::Hash, ops::Add, sync::RwLock};

use crate::prelude::*;

impl Profile {
    async fn new_entity<E: IsEntity + std::fmt::Debug + std::hash::Hash + Eq>(
        &mut self,
        network_id: NetworkID,
        name: impl AsRef<str>,
        factor_source_id: FactorSourceIDFromHash,
        factor_instance_provider: &FactorInstanceProvider,
    ) -> Result<E> {
        assert!(self
            .factor_sources
            .iter()
            .map(|f| f.factor_source_id())
            .contains(&factor_source_id));

        let genesis_factor = factor_instance_provider
            .provide_genesis_factor_for(factor_source_id, E::kind(), network_id, self)
            .await?;

        let address = E::Address::by_hashing(network_id, genesis_factor.clone());

        let entity = E::new(
            name,
            address,
            EntitySecurityState::Unsecured(genesis_factor),
        );

        let erased = Into::<AccountOrPersona>::into(entity.clone());

        match erased {
            AccountOrPersona::AccountEntity(account) => {
                self.accounts.insert(account.entity_address(), account);
            }
            AccountOrPersona::PersonaEntity(persona) => {
                self.personas.insert(persona.entity_address(), persona);
            }
        };

        Ok(entity)
    }

    pub async fn new_account(
        &mut self,
        network_id: NetworkID,
        name: impl AsRef<str>,
        factor_source_id: FactorSourceIDFromHash,
        factor_instance_provider: &FactorInstanceProvider,
    ) -> Result<Account> {
        self.new_entity(network_id, name, factor_source_id, factor_instance_provider)
            .await
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OnChainEntityUnsecurified {
    address: AddressOfAccountOrPersona,
    owner_keys: Vec<PublicKeyHash>,
}
impl OnChainEntityUnsecurified {
    pub fn new(
        address: impl Into<AddressOfAccountOrPersona>,
        owner_keys: Vec<PublicKeyHash>,
    ) -> Self {
        Self {
            address: address.into(),
            owner_keys,
        }
    }

    pub fn owner_keys(&self) -> HashSet<PublicKeyHash> {
        self.owner_keys.iter().cloned().collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OnChainEntitySecurified {
    address: AddressOfAccountOrPersona,
    access_controller: AccessController,
    owner_keys: Vec<PublicKeyHash>,
}

impl OnChainEntitySecurified {
    pub fn new(
        address: impl Into<AddressOfAccountOrPersona>,
        access_controller: AccessController,
        owner_keys: Vec<PublicKeyHash>,
    ) -> Self {
        Self {
            address: address.into(),
            access_controller,
            owner_keys,
        }
    }
    pub fn owner_keys(&self) -> HashSet<PublicKeyHash> {
        self.owner_keys.iter().cloned().collect()
    }
}

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
    fn address(&self) -> AddressOfAccountOrPersona {
        match self {
            OnChainEntityState::Unsecurified(account) => account.address.clone(),
            OnChainEntityState::Securified(account) => account.address.clone(),
        }
    }

    fn owner_keys(&self) -> HashSet<PublicKeyHash> {
        match self {
            OnChainEntityState::Unsecurified(account) => account.owner_keys(),
            OnChainEntityState::Securified(account) => account.owner_keys(),
        }
    }
}

#[async_trait::async_trait]
pub trait GatewayReadonly: Sync + Send {
    async fn is_key_hash_known(&self, hash: PublicKeyHash) -> Result<bool>;

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

#[async_trait::async_trait]
pub trait Gateway: GatewayReadonly {
    async fn simulate_network_activity_for(&self, owner: AddressOfAccountOrPersona) -> Result<()>;

    async fn set_securified_entity(
        &self,
        securified: SecurifiedEntityControl,
        owner: AddressOfAccountOrPersona,
    ) -> Result<()>;

    async fn set_securified_account(
        &self,
        securified: SecurifiedEntityControl,
        owner: &AccountAddress,
    ) -> Result<()> {
        self.set_securified_entity(securified, owner.clone().into())
            .await
    }

    async fn set_securified_persona(
        &self,
        securified: SecurifiedEntityControl,
        owner: &IdentityAddress,
    ) -> Result<()> {
        self.set_securified_entity(securified, owner.clone().into())
            .await
    }
}

const RECOVERY_BATCH_SIZE_DERIVATION_ENTITY_INDEX: HDPathValue = 50;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UncoveredEntity {
    pub on_chain: OnChainEntityState,
    pub key_hash_to_factor_instances:
        HashMap<PublicKeyHash, HierarchicalDeterministicFactorInstance>,
}
impl UncoveredEntity {
    pub fn new(
        on_chain: OnChainEntityState,
        key_hash_to_factor_instances: HashMap<
            PublicKeyHash,
            HierarchicalDeterministicFactorInstance,
        >,
    ) -> Self {
        Self {
            on_chain,
            key_hash_to_factor_instances,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntityRecoveryOutcome<E>
where
    E: IsEntity + Hash + Eq,
{
    pub recovered_unsecurified: IndexSet<E>,
    pub recovered_securified: IndexSet<E>,
    pub unrecovered: Vec<UncoveredEntity>, // want `IndexSet` but is not `Hash`
}
impl<E: IsEntity + Hash + Eq> EntityRecoveryOutcome<E> {
    pub fn new(
        recovered_unsecurified: impl IntoIterator<Item = E>,
        recovered_securified: impl IntoIterator<Item = E>,
        unrecovered: impl IntoIterator<Item = UncoveredEntity>,
    ) -> Self {
        Self {
            recovered_unsecurified: recovered_unsecurified.into_iter().collect(),
            recovered_securified: recovered_securified.into_iter().collect(),
            unrecovered: unrecovered.into_iter().collect(),
        }
    }
}

/// A first implementation of Recovery of Securified entities, working POC using
/// Entity Indexing heuristics - `CEI - strategy 1` - Canonical Entity Indexing,
/// as described in [doc].
///
/// N.B. This is a simplified version of the algorithm, which does not allow
/// users to trigger "Scan More" action - for which we will continue to derive
/// more PublicKeys for each factor source at "next batch" of derivation indices.
///
/// Here follows an executive summary of the algorithm:
/// A. User inputs a list of FactorSources
/// B. Create a set of derivation paths, both for securified and unsecurified entities
/// C. For each factor source we derive PublicKey's at **all paths**
/// D. Create PublicKeyHash'es for each PublicKey
/// E. Ensure to retain which (FactorSource, DerivationPath) tuple was for each PublicKeyHash
/// F. Query gateway for EntityAddress referencing each PublicKeyHash
/// G. Query gateway with each EntityAddress to get: AccessController's ScryptoAccessRule or single `owner_key hash
/// H. "Play a match making game" between locally calculated PublicKeyHash'es and the ones downloaded from Gateway
/// I. For each EntityAddress with single `owner_key` create an Unsecurified Entity
/// J. for each with ScryptoAccessRule try to map the `ScryptoAccessRule` into a `MatrixOfPublicKeyHashes`, then try to map that
/// into a `MatrixOfFactorInstances` by looking up the locally derived factor instances (PublicKeys).
/// K. For each EntityAddress which we failed to match all PublicKeyHashes, ask user if should would like to
/// continue the search, by deriving keys using another batch of derivation paths.
/// L. Return the results, which is three sets: recovered_unsecurified, recovered_securified, unrecovered
///
/// [doc]: https://radixdlt.atlassian.net/wiki/spaces/AT/pages/3640655873/Yet+Another+Page+about+Derivation+Indices
pub async fn recover_entity<E: IsEntity + Sync + Hash + Eq>(
    network_id: NetworkID,
    factor_sources: impl IntoIterator<Item = HDFactorSource>,
    key_derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    gateway: Arc<dyn GatewayReadonly>,
) -> Result<EntityRecoveryOutcome<E>> {
    // A. User inputs a list of FactorSources
    let entity_kind = E::kind();
    let factor_sources = factor_sources.into_iter().collect::<IndexSet<_>>();

    // B. Create a set of derivation paths, both for securified and unsecurified entities
    let map_paths = {
        let index_range = 0..RECOVERY_BATCH_SIZE_DERIVATION_ENTITY_INDEX;
        let make_paths =
            |make_entity_index: fn(HDPathValue) -> HDPathComponent| -> IndexSet<DerivationPath> {
                index_range
                    .clone()
                    .map(make_entity_index)
                    .map(|i| {
                        DerivationPath::new(
                            network_id,
                            entity_kind,
                            CAP26KeyKind::TransactionSigning,
                            i,
                        )
                    })
                    .collect::<IndexSet<_>>()
            };

        let paths_unsecurified = make_paths(HDPathComponent::unsecurified);
        let paths_securified = make_paths(HDPathComponent::securified);
        let mut all_paths = IndexSet::<DerivationPath>::new();
        all_paths.extend(paths_unsecurified);
        all_paths.extend(paths_securified);

        let mut map_paths = IndexMap::<FactorSourceIDFromHash, IndexSet<DerivationPath>>::new();
        for factor_source in factor_sources.iter() {
            map_paths.insert(factor_source.factor_source_id(), all_paths.clone());
        }
        map_paths
    };

    let (addresses_per_hash, map_hash_to_factor) = {
        // C. For each factor source we derive PublicKey's at **all paths**
        let keys_collector =
            KeysCollector::new(factor_sources, map_paths, key_derivation_interactors).unwrap();

        let factor_instances = keys_collector.collect_keys().await.all_factors();

        // D. Create PublicKeyHash'es for each PublicKey
        let map_hash_to_factor = factor_instances
            .into_iter()
            .map(|f| (f.public_key_hash(), f.clone()))
            .collect::<HashMap<PublicKeyHash, HierarchicalDeterministicFactorInstance>>();

        // E. Ensure to retain which (FactorSource, DerivationPath) tuple was for each PublicKeyHash
        // F. Query gateway for EntityAddress referencing each PublicKeyHash
        let untyped_addresses_per_hash = gateway
            .get_entity_addresses_of_by_public_key_hashes(
                map_hash_to_factor.keys().cloned().collect::<HashSet<_>>(),
            )
            .await?;

        let addresses_per_hash = untyped_addresses_per_hash
            .into_iter()
            .map(|(k, v)| {
                let typed_address = v
                    .into_iter()
                    .map(|a| {
                        E::Address::try_from(a).map_err(|_| CommonError::AddressConversionError)
                    })
                    .collect::<Result<HashSet<E::Address>>>()?;

                Ok((k, typed_address))
            })
            .collect::<Result<HashMap<PublicKeyHash, HashSet<E::Address>>>>()?;

        (addresses_per_hash, map_hash_to_factor)
    };

    // G. Query gateway with each EntityAddress to get: AccessController's ScryptoAccessRule or single `owner_key hash
    let (address_to_factor_instances_map, unsecurified_addresses, securified_addresses) = {
        let mut unsecurified_addresses = HashSet::<E::Address>::new();
        let mut securified_addresses = HashSet::<E::Address>::new();

        let mut address_to_factor_instances_map =
            HashMap::<E::Address, HashSet<HierarchicalDeterministicFactorInstance>>::new();

        for (hash, addresses) in addresses_per_hash.iter() {
            if addresses.is_empty() {
                unreachable!("We should never create empty sets");
            }
            if addresses.len() > 1 {
                panic!("Violation of Axiom 1: same key is used in many entities")
            }
            let address = addresses.iter().last().unwrap();

            let factor_instance = map_hash_to_factor.get(hash).unwrap();
            if let Some(existing) = address_to_factor_instances_map.get_mut(address) {
                existing.insert(factor_instance.clone());
            } else {
                address_to_factor_instances_map
                    .insert(address.clone(), HashSet::just(factor_instance.clone()));
            }

            let is_securified = gateway.is_securified(address.clone().into()).await?;

            if is_securified {
                securified_addresses.insert(address.clone());
            } else {
                unsecurified_addresses.insert(address.clone());
            }
        }

        (
            address_to_factor_instances_map,
            unsecurified_addresses,
            securified_addresses,
        )
    };

    // H. "Play a match making game" between locally calculated PublicKeyHash'es and the ones downloaded from Gateway

    // I. For each EntityAddress with single `owner_key` create an Unsecurified Entity
    let unsecurified_entities = unsecurified_addresses
        .into_iter()
        .map(|a| {
            let factor_instances = address_to_factor_instances_map.get(&a).unwrap();
            assert_eq!(
                factor_instances.len(),
                1,
                "Expected single factor since unsecurified"
            );
            let factor_instance = factor_instances.iter().last().unwrap();
            let security_state = EntitySecurityState::Unsecured(factor_instance.clone());
            E::new(
                format!("Recovered Unsecurified: {:?}", a),
                a,
                security_state,
            )
        })
        .collect::<HashSet<_>>();

    let mut securified_entities = HashSet::<E>::new();
    let mut unrecovered_entities = Vec::<UncoveredEntity>::new();

    // J. for each with ScryptoAccessRule try to map the `ScryptoAccessRule` into a `MatrixOfPublicKeyHashes`, then try to map that
    // into a `MatrixOfFactorInstances` by looking up the locally derived factor instances (PublicKeys).
    for a in securified_addresses {
        let on_chain_entity = gateway
            .get_on_chain_entity(a.clone().into())
            .await
            .unwrap()
            .unwrap()
            .as_securified()
            .unwrap()
            .clone();

        // K. [NOT IMPLEMENTED YET] For each EntityAddress which we failed to match all PublicKeyHashes, ask user if should would like to
        // continue the search, by deriving keys using another batch of derivation paths.

        let mut fail = || {
            let unrecovered_entity = UncoveredEntity::new(
                OnChainEntityState::Securified(on_chain_entity.clone()),
                HashMap::new(), // TODO: fill this
            );
            warn!("Could not recover entity: {:?}", unrecovered_entity);
            unrecovered_entities.push(unrecovered_entity);
        };

        let Ok(matrix_of_hashes) = MatrixOfKeyHashes::try_from(
            on_chain_entity
                .clone()
                .access_controller
                .metadata
                .scrypto_access_rules,
        ) else {
            fail();
            continue;
        };

        let mut threshold_factor_instances = IndexSet::new();
        let mut override_factor_instances = IndexSet::new();

        for threshold_factor_hash in matrix_of_hashes.threshold_factors.iter() {
            let Some(factor_instance) = map_hash_to_factor.get(threshold_factor_hash) else {
                warn!(
                    "Missing THRESHOLD factor instance for hash: {:?}",
                    threshold_factor_hash
                );
                continue;
            };
            threshold_factor_instances.insert(factor_instance.clone());
        }

        for override_factor_hash in matrix_of_hashes.override_factors.iter() {
            let Some(factor_instance) = map_hash_to_factor.get(override_factor_hash) else {
                warn!(
                    "Missing OVERRIDE factor instance for hash: {:?}",
                    override_factor_hash
                );
                continue;
            };
            override_factor_instances.insert(factor_instance.clone());
        }

        if threshold_factor_instances.len() < matrix_of_hashes.threshold as usize {
            warn!("Not enough threshold factors");
            fail();
            continue;
        }

        let sec = SecurifiedEntityControl::new(
            MatrixOfFactorInstances::new(
                threshold_factor_instances,
                matrix_of_hashes.threshold,
                override_factor_instances,
            ),
            on_chain_entity.access_controller,
        );
        let security_state = EntitySecurityState::Securified(sec);
        let recovered_securified_entity =
            E::new(format!("Recovered Securified: {:?}", a), a, security_state);
        assert!(securified_entities.insert(recovered_securified_entity));
    }

    // L. Return the results, which is three sets: recovered_unsecurified, recovered_securified, unrecovered
    Ok(EntityRecoveryOutcome::<E>::new(
        unsecurified_entities,
        securified_entities,
        Vec::new(),
    ))
}

pub async fn recover_accounts(
    network_id: NetworkID,
    factor_sources: impl IntoIterator<Item = HDFactorSource>,
    key_derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    gateway: Arc<dyn GatewayReadonly>,
) -> Result<EntityRecoveryOutcome<Account>> {
    recover_entity(
        network_id,
        factor_sources,
        key_derivation_interactors,
        gateway,
    )
    .await
}

pub async fn recover_personas(
    network_id: NetworkID,
    factor_sources: impl IntoIterator<Item = HDFactorSource>,
    key_derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    gateway: Arc<dyn GatewayReadonly>,
) -> Result<EntityRecoveryOutcome<Persona>> {
    recover_entity(
        network_id,
        factor_sources,
        key_derivation_interactors,
        gateway,
    )
    .await
}

#[cfg(test)]
pub struct TestGateway {
    /// contains only current state for each entity
    entities: RwLock<HashMap<AddressOfAccountOrPersona, OnChainEntityState>>,

    /// contains historic state, we only ever add to this set, never remove.
    known_hashes: RwLock<HashSet<PublicKeyHash>>,
}
#[cfg(test)]
impl Default for TestGateway {
    fn default() -> Self {
        Self {
            known_hashes: RwLock::new(HashSet::new()),
            entities: RwLock::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
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

#[cfg(test)]
#[async_trait::async_trait]
impl GatewayReadonly for TestGateway {
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
#[cfg(test)]
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

#[cfg(test)]
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
                OnChainEntityState::unsecurified_with(owner, owner_key.clone()),
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
            .collect_vec();

        if self.contains(&owner) {
            self.entities.try_write().unwrap().remove(&owner);
        }

        self.known_hashes
            .try_write()
            .unwrap()
            .extend(owner_keys.clone());

        self.entities.try_write().unwrap().insert(
            owner.clone(),
            OnChainEntityState::securified_with(
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
    use std::{
        borrow::BorrowMut,
        future::{Future, IntoFuture},
    };

    use futures::future::BoxFuture;

    use super::*;

    #[test]
    fn public_key_hash_is_unique() {
        let f = &FactorSourceIDFromHash::fs0();
        type PK = HierarchicalDeterministicPublicKey;
        let n: usize = 10;
        let pub_keys = HashSet::<PK>::from_iter(
            (0..n as HDPathValue)
                .map(|i| {
                    DerivationPath::account_tx(NetworkID::Mainnet, HDPathComponent::unsecurified(i))
                })
                .map(|p| PK::mocked_with(p, f)),
        );
        assert_eq!(pub_keys.len(), n);
        let hashes = pub_keys.iter().map(|k| k.hash()).collect::<HashSet<_>>();

        assert_eq!(hashes.len(), n);
    }

    async fn do_test<
        E: IsEntity + Hash + Eq + Sync,
        Fut: Future<Output = IndexSet<E>>,
        F: FnOnce(Arc<dyn Gateway>) -> Fut,
    >(
        network_id: NetworkID,
        all_factors: IndexSet<HDFactorSource>,
        setup: F,
        assert: impl FnOnce(IndexSet<E>, EntityRecoveryOutcome<E>) + 'static,
    ) {
        let gateway = Arc::new(TestGateway::default());

        let interactors = Arc::new(TestDerivationInteractors::default());

        let entities = setup(gateway.clone()).await;

        let recovered = recover_entity::<E>(network_id, all_factors, interactors, gateway)
            .await
            .unwrap();

        assert(entities, recovered);
    }

    #[actix_rt::test]
    async fn recovery_of_single_many_securified_accounts() {
        let all_factors = HDFactorSource::all();

        do_test(
            NetworkID::Mainnet,
            all_factors,
            |gateway| {
                Box::pin(async move {
                    let securified_accounts = IndexSet::<Account>::from_iter([
                        Account::a2(),
                        Account::a3(),
                        Account::a4(),
                        Account::a5(),
                        Account::a6(),
                        Account::a7(),
                    ]);

                    for account in securified_accounts.iter() {
                        gateway
                            .set_securified_account(
                                account.security_state.as_securified().unwrap().clone(),
                                &account.entity_address(),
                            )
                            .await
                            .unwrap();
                    }

                    securified_accounts
                })
            },
            |known, recovered| {
                let recovered_unsecurified_accounts = recovered.recovered_unsecurified;
                assert_eq!(recovered_unsecurified_accounts.len(), 0);

                let recovered_securified_accounts = recovered.recovered_securified;
                assert_eq!(recovered_securified_accounts.len(), known.len());

                assert_eq!(
                    recovered_securified_accounts
                        .iter()
                        .map(|a| a.security_state())
                        .collect::<IndexSet<_>>(),
                    known
                        .iter()
                        .map(|a| a.security_state())
                        .collect::<IndexSet<_>>(),
                );
            },
        )
        .await;
    }

    #[actix_rt::test]
    async fn recovery_of_unsecurified_accounts_only() {
        let all_factors = HDFactorSource::all();

        do_test(
            NetworkID::Mainnet,
            all_factors,
            |gateway| {
                Box::pin(async move {
                    let securified_accounts =
                        IndexSet::<Account>::from_iter([Account::a0(), Account::a1()]);

                    for account in securified_accounts.iter() {
                        gateway
                            .simulate_network_activity_for(account.address())
                            .await
                            .unwrap();
                    }

                    securified_accounts
                })
            },
            |known, recovered| {
                assert_eq!(recovered.recovered_securified.len(), 0);

                let recovered_unsecurified_accounts = recovered.recovered_unsecurified;
                assert_eq!(recovered_unsecurified_accounts.len(), known.len());

                assert_eq!(
                    recovered_unsecurified_accounts
                        .iter()
                        .map(|a| a.security_state())
                        .collect::<IndexSet<_>>(),
                    known
                        .iter()
                        .map(|a| a.security_state())
                        .collect::<IndexSet<_>>(),
                );
            },
        )
        .await;
    }

    #[actix_rt::test]
    async fn recovery_of_single_many_securified_personas() {
        let all_factors = HDFactorSource::all();

        do_test(
            NetworkID::Mainnet,
            all_factors.clone(),
            |gateway| {
                Box::pin(async move {
                    let securified_personas = IndexSet::<Persona>::from_iter([
                        Persona::p2(),
                        Persona::p3(),
                        Persona::p4(),
                        Persona::p5(),
                        Persona::p6(),
                        Persona::p7(),
                    ]);

                    for persona in securified_personas.iter() {
                        gateway
                            .set_securified_persona(
                                persona.security_state.as_securified().unwrap().clone(),
                                &persona.entity_address(),
                            )
                            .await
                            .unwrap();
                    }
                    securified_personas
                })
            },
            |known: IndexSet<Persona>, recovered| {
                assert_eq!(recovered.recovered_unsecurified.len(), 0);

                let recovered_securified_personas = recovered.recovered_securified;
                assert_eq!(recovered_securified_personas.len(), known.len());

                assert_eq!(
                    recovered_securified_personas
                        .iter()
                        .map(|a| a.security_state())
                        .collect::<IndexSet<_>>(),
                    known
                        .iter()
                        .map(|a| a.security_state())
                        .collect::<IndexSet<_>>(),
                );
            },
        )
        .await;
    }

    #[actix_rt::test]
    async fn recovery_of_accounts_mixed_securified_and_non() {
        let all_factors = HDFactorSource::all();

        do_test(
            NetworkID::Mainnet,
            all_factors.clone(),
            |gateway| {
                Box::pin(async move {
                    let mut profile = Profile::new(all_factors.clone(), [], []);

                    let keys_cache = Arc::new(InMemoryPreDerivedKeysCache::default());
                    let interactors = Arc::new(TestDerivationInteractors::default());

                    let factor_instance_provider =
                        FactorInstanceProvider::new(gateway.clone(), interactors, keys_cache);

                    let alice_address = profile
                        .new_account(
                            NetworkID::Mainnet,
                            "alice",
                            fs_id_at(0),
                            &factor_instance_provider,
                        )
                        .await
                        .unwrap()
                        .entity_address();

                    securify(
                        alice_address.clone(),
                        MatrixOfFactorSources::new(
                            [fs_at(0), fs_at(1), fs_at(2), fs_at(3)],
                            3,
                            [fs_at(6)],
                        ),
                        &mut profile,
                        &factor_instance_provider,
                    )
                    .await
                    .unwrap();

                    let bob_address = profile
                        .new_account(
                            NetworkID::Mainnet,
                            "bob",
                            fs_id_at(1),
                            &factor_instance_provider,
                        )
                        .await
                        .unwrap()
                        .entity_address();

                    securify(
                        bob_address.clone(),
                        MatrixOfFactorSources::new([fs_at(1), fs_at(3)], 2, [fs_at(7)]),
                        &mut profile,
                        &factor_instance_provider,
                    )
                    .await
                    .unwrap();

                    let charlie_address = profile
                        .new_account(
                            NetworkID::Mainnet,
                            "charlie",
                            fs_id_at(1),
                            &factor_instance_provider,
                        )
                        .await
                        .unwrap()
                        .entity_address();

                    let accounts: IndexSet<Account> = profile.get_accounts();

                    assert_eq!(accounts.len(), 3);

                    let alice = profile.account_by_address(alice_address).unwrap();
                    let bob = profile.account_by_address(bob_address).unwrap();
                    let charlie = profile.account_by_address(charlie_address).unwrap();

                    gateway
                        .simulate_network_activity_for(charlie.address())
                        .await
                        .unwrap();

                    assert!(alice.is_securified());
                    assert!(bob.is_securified());
                    assert!(!charlie.is_securified());

                    assert_eq!(
                        charlie
                            .security_state
                            .into_unsecured()
                            .unwrap()
                            .derivation_path()
                            .index
                            .index(),
                        1 // second time that factor source was used.
                    );
                    accounts
                })
            },
            move |known: IndexSet<Account>, recovered| {
                assert!(recovered.unrecovered.is_empty());
                assert_eq!(
                    known.len(),
                    recovered.recovered_securified.len() + recovered.recovered_unsecurified.len()
                );
            },
        )
        .await;
    }
}
