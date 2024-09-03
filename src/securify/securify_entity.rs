#![cfg(test)]

use crate::{derivation, prelude::*};
use sha2::{Digest, Sha256, Sha512};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccessControllerAddress(pub String);
impl AccessControllerAddress {
    pub fn generate() -> Self {
        Self(format!("access_controller_{}", Uuid::new_v4()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComponentMetadata {
    /// Empty if not securified
    pub public_key_hashes: Vec<[u8; 32]>,
    /// None if not securified
    pub derivation_index: Option<HDPathValue>,
}
impl ComponentMetadata {
    pub fn new(public_keys: Vec<PublicKey>, derivation_index: Option<HDPathValue>) -> Self {
        Self {
            public_key_hashes: public_keys
                .into_iter()
                .map(|pk| pk.to_bytes())
                .map(|b| {
                    let mut hasher = Sha256::new();
                    hasher.update(b);
                    hasher.finalize().into()
                })
                .collect(),
            derivation_index,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccessController {
    pub address: AccessControllerAddress,
    pub metadata: ComponentMetadata,
}

pub trait DerivationIndexWhenSecurifiedAssigner {
    fn assign_derivation_index(
        &self,
        account: Account,
        other_accounts: HashSet<Account>,
    ) -> HDPathValue;
}

pub async fn securify(
    address: AccountAddress,
    matrix: MatrixOfFactorSources,
    profile: Profile,
    derivation_index_assigner: impl DerivationIndexWhenSecurifiedAssigner,
) -> Result<AccessController> {
    let account = profile.account_by_address(address)?;

    let mut other_accounts = profile
        .accounts
        .values()
        .cloned()
        .collect::<HashSet<Account>>();

    other_accounts.remove(&account);

    let derivation_index =
        derivation_index_assigner.assign_derivation_index(account, other_accounts);
    let network_id = NetworkID::Mainnet; // TODO read from address...

    let derivation_path = DerivationPath::new(
        network_id,
        CAP26EntityKind::Account,
        CAP26KeyKind::T9n,
        HDPathComponent::securified(derivation_index),
    );

    let keys_collector = KeysCollector::new(
        profile.factor_sources,
        matrix
            .all_factors()
            .clone()
            .into_iter()
            .map(|f| {
                (
                    f.factor_source_id(),
                    IndexSet::just(derivation_path.clone()),
                )
            })
            .collect::<IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>>(),
        Arc::new(TestDerivationInteractors::default()),
    )?;

    let factor_instances = keys_collector.collect_keys().await.all_factors();

    let component_metadata = ComponentMetadata::new(
        factor_instances
            .iter()
            .map(|fi| fi.public_key.clone().public_key)
            .collect(),
        Some(derivation_index),
    );

    Ok(AccessController {
        address: AccessControllerAddress::generate(),
        metadata: component_metadata,
    })
}

pub async fn securify_with_matrix_of_instances(
    _account: Account,
    _matrix: MatrixOfFactorInstances,
) -> Result<AccessController> {
    todo!()
}
