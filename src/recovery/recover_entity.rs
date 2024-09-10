use std::{hash::Hash, sync::RwLock};

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

#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumAsInner)]
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
}

#[async_trait::async_trait]
pub trait GatewayReadonly: Sync {
    async fn get_account_addresses_of_by_public_key_hashes(
        &self,
        hashes: HashSet<PublicKeyHash>,
    ) -> Result<HashMap<PublicKeyHash, HashSet<AccountAddress>>>;

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

const RECOVERY_BATCH_SIZE_DERIVATION_ENTITY_INDEX: HDPathValue = 50;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UncoveredAccount {
    pub on_chain: OnChainAccountState,
    pub key_hash_to_factor_instances:
        HashMap<PublicKeyHash, HierarchicalDeterministicFactorInstance>,
}
impl UncoveredAccount {
    pub fn new(
        on_chain: OnChainAccountState,
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
pub struct AccountRecoveryOutcome {
    pub recovered_unsecurified: IndexSet<Account>,
    pub recovered_securified: IndexSet<Account>,
    pub unrecovered: Vec<UncoveredAccount>, // want set
}
impl AccountRecoveryOutcome {
    pub fn new(
        recovered_unsecurified: impl IntoIterator<Item = Account>,
        recovered_securified: impl IntoIterator<Item = Account>,
        unrecovered: impl IntoIterator<Item = UncoveredAccount>,
    ) -> Self {
        Self {
            recovered_unsecurified: recovered_unsecurified.into_iter().collect(),
            recovered_securified: recovered_securified.into_iter().collect(),
            unrecovered: unrecovered.into_iter().collect(),
        }
    }
}

/// A first implementation of Recovery of Securified accounts, working POC using
/// Entity Indexing heuristics - `CEI - strategy 1` - Canonical Entity Indexing,
/// as described in [doc].
///
/// N.B. This is a simplified version of the algorithm, which does not allow
/// users to trigger "Scan More" action - for which we will continue to derive
/// more PublicKeys for each factor source at "next batch" of derivation indices.
/// Also note that this implementation is for Account, which can and SHOULD be
/// generalized to support Personas - which is easily done (adding EntityKind as input).
///
/// Here follows an executive summary of the algorithm:
/// 0a. Define `const RECOVERY_BATCH_SIZE_DERIVATION_ENTITY_INDEX = 0`
/// 0b. Define `const BIP32_SECURIFIED_HALF = 2^30`
/// 1. Input is (Vec<FactorSource>, NetorkID>), used to accounts - either securified or unsecurified.
/// 2. Create `index_range = (0, RECOVERY_BATCH_SIZE_DERIVATION_ENTITY_INDEX)`
/// 3. Create **two** HDPathComponent sets, one which maps the index to the unsecurified half of
/// the Derivation Entity Index space and the other to the securified half (adding `BIP32_SECURIFIED_HALF` to
/// each item in the set).
/// 4. Merge the two sets into a single set `all_indices`.
/// 5. Create set `all_paths` which is a set of DerivationPaths by mapping each index ind `all_indices` to
/// a DerivationPath created with said index, NetworkID, `CAP26KeyKind::Transaction` & CAP26EntityKind::Account.
///
/// [doc]: https://radixdlt.atlassian.net/wiki/spaces/AT/pages/3640655873/Yet+Another+Page+about+Derivation+Indices
pub async fn recover_accounts(
    network_id: NetworkID,
    factor_sources: impl IntoIterator<Item = HDFactorSource>,
    key_derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    gateway: Arc<dyn GatewayReadonly>,
) -> Result<AccountRecoveryOutcome> {
    let factor_sources = factor_sources.into_iter().collect::<IndexSet<_>>();
    let index_range = 0..RECOVERY_BATCH_SIZE_DERIVATION_ENTITY_INDEX;
    let make_paths =
        |make_entity_index: fn(HDPathValue) -> HDPathComponent| -> IndexSet<DerivationPath> {
            index_range
                .clone()
                .map(make_entity_index)
                .map(|i| DerivationPath::account_tx(network_id, i))
                .collect::<IndexSet<_>>()
        };

    let paths_unsecurified_accounts = make_paths(HDPathComponent::unsecurified);
    let paths_securified_accounts = make_paths(HDPathComponent::securified);
    let mut all_paths = IndexSet::<DerivationPath>::new();
    all_paths.extend(paths_securified_accounts);
    all_paths.extend(paths_unsecurified_accounts);

    let mut map_paths = IndexMap::<FactorSourceIDFromHash, IndexSet<DerivationPath>>::new();
    for factor_source in factor_sources.iter() {
        map_paths.insert(factor_source.factor_source_id(), all_paths.clone());
    }

    let keys_collector =
        KeysCollector::new(factor_sources, map_paths, key_derivation_interactors).unwrap();

    let factor_instances = keys_collector.collect_keys().await.all_factors();

    let map_hash_to_factor = factor_instances
        .into_iter()
        .map(|f| (f.public_key_hash(), f.clone()))
        .collect::<HashMap<PublicKeyHash, HierarchicalDeterministicFactorInstance>>();

    let account_addresses_per_hash = gateway
        .get_account_addresses_of_by_public_key_hashes(
            map_hash_to_factor.keys().cloned().collect::<HashSet<_>>(),
        )
        .await?;

    let mut unsecurified_accounts_addresses = HashSet::<AccountAddress>::new();
    let mut securified_accounts_addresses = HashSet::<AccountAddress>::new();

    let mut account_address_to_factor_instances_map =
        HashMap::<AccountAddress, HashSet<HierarchicalDeterministicFactorInstance>>::new();

    for (hash, account_addresses) in account_addresses_per_hash.iter() {
        if account_addresses.is_empty() {
            unreachable!("We should never create empty sets");
        }
        if account_addresses.len() > 1 {
            panic!("Violation of Axiom 1: same key is used in many accounts")
        }
        let account_address = account_addresses.iter().last().unwrap();

        let factor_instance = map_hash_to_factor.get(hash).unwrap();
        if let Some(existing) = account_address_to_factor_instances_map.get_mut(account_address) {
            existing.insert(factor_instance.clone());
        } else {
            account_address_to_factor_instances_map.insert(
                account_address.clone(),
                HashSet::just(factor_instance.clone()),
            );
        }

        let is_securified = gateway.is_securified(account_address).await?;

        if is_securified {
            securified_accounts_addresses.insert(account_address.clone());
        } else {
            unsecurified_accounts_addresses.insert(account_address.clone());
        }
    }

    let unsecurified_accounts = unsecurified_accounts_addresses
        .into_iter()
        .map(|a| {
            let factor_instances = account_address_to_factor_instances_map.get(&a).unwrap();
            assert_eq!(
                factor_instances.len(),
                1,
                "Expected single factor since unsecurified"
            );
            let factor_instance = factor_instances.iter().last().unwrap();
            let security_state = EntitySecurityState::Unsecured(factor_instance.clone());
            Account::new(format!("Recovered Unsecurified: {}", a), security_state)
        })
        .collect::<HashSet<_>>();

    let mut securified_accounts = HashSet::new();
    let mut unrecovered_accounts = Vec::new();
    for a in securified_accounts_addresses {
        let _factor_instances = account_address_to_factor_instances_map.get(&a).unwrap();
        let on_chain_account = gateway
            .get_on_chain_account(&a)
            .await
            .unwrap()
            .unwrap()
            .as_securified()
            .unwrap()
            .clone();

        let mut fail = || {
            let unrecovered_account = UncoveredAccount::new(
                OnChainAccountState::Securified(on_chain_account.clone()),
                HashMap::new(), // TODO: fill this
            );
            warn!("Could not recover account: {:?}", unrecovered_account);
            unrecovered_accounts.push(unrecovered_account);
        };

        let Ok(matrix_of_hashes) = MatrixOfKeyHashes::try_from(
            on_chain_account
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
            on_chain_account.access_controller,
        );
        let security_state = EntitySecurityState::Securified(sec);
        let recoverd_securified_account =
            Account::new(format!("Recovered Unsecurified: {}", a), security_state);
        assert!(securified_accounts.insert(recoverd_securified_account));
    }

    Ok(AccountRecoveryOutcome::new(
        unsecurified_accounts,
        securified_accounts,
        Vec::new(),
    ))
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
    async fn get_account_addresses_of_by_public_key_hashes(
        &self,
        hashes: HashSet<PublicKeyHash>,
    ) -> Result<HashMap<PublicKeyHash, HashSet<AccountAddress>>> {
        let accounts = self.accounts.try_read().unwrap();
        let states = accounts.values();

        Ok(hashes
            .iter()
            .filter_map(|k| {
                // N.B. we want this to always be single element (Axiom 1).
                let mut accounts_references_hash = HashSet::<AccountAddress>::new();
                for state in states.clone().filter(|x| x.owner_keys().contains(k)) {
                    accounts_references_hash.insert(state.account_address());
                }
                if accounts_references_hash.is_empty() {
                    None
                } else {
                    Some((k.clone(), accounts_references_hash))
                }
            })
            .collect::<HashMap<PublicKeyHash, HashSet<AccountAddress>>>())
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

    #[actix_rt::test]
    async fn recovery_of_unsecurified_accounts() {
        let all_factors = HDFactorSource::all();
        let gateway = Arc::new(TestGateway::default());

        let interactors = Arc::new(TestDerivationInteractors::default());

        let unsecurified = Account::a0();
        gateway
            .set_unsecurified_account(
                unsecurified.security_state.as_unsecured().unwrap().clone(),
                &unsecurified.entity_address(),
            )
            .await
            .unwrap();
        let recovered = recover_accounts(NetworkID::Mainnet, all_factors, interactors, gateway)
            .await
            .unwrap();
        let recovered_unsecurified_accounts = recovered.recovered_unsecurified;
        assert_eq!(recovered_unsecurified_accounts.len(), 1);
        let recoverd = recovered_unsecurified_accounts.first().unwrap();
        assert_eq!(recoverd.security_state(), unsecurified.security_state())
    }

    #[actix_rt::test]
    async fn recovery_of_securified_accounts() {
        let all_factors = HDFactorSource::all();
        let gateway = Arc::new(TestGateway::default());

        let interactors = Arc::new(TestDerivationInteractors::default());

        let securified = Account::a7();
        gateway
            .set_securified_account(
                securified.security_state.as_securified().unwrap().clone(),
                &securified.entity_address(),
            )
            .await
            .unwrap();
        let recovered = recover_accounts(NetworkID::Mainnet, all_factors, interactors, gateway)
            .await
            .unwrap();

        let recovered_unsecurified_accounts = recovered.recovered_unsecurified;
        assert_eq!(recovered_unsecurified_accounts.len(), 0);

        let recovered_securified_accounts = recovered.recovered_securified;
        assert_eq!(recovered_securified_accounts.len(), 1);

        let recoverd = recovered_securified_accounts.first().unwrap();
        assert_eq!(recoverd.security_state(), securified.security_state())
    }
}
