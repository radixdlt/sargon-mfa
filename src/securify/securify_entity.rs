#![cfg(test)]

use crate::prelude::*;
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

pub async fn securify(account: Account, matrix: MatrixOfFactorSources) -> Result<AccessController> {
    let keys_collector = KeysCollector::new(
        matrix.all_factors(),
        IndexMap::new(),
        Arc::new(TestDerivationInteractors::default()),
    )?;
    let factor_instances = keys_collector.collect_keys().await.all_factors();
    let component_metadata = ComponentMetadata::new(
        factor_instances
            .iter()
            .map(|fi| fi.public_key.clone().public_key)
            .collect(),
        None,
    );
    Ok(AccessController {
        address: AccessControllerAddress::generate(),
        metadata: component_metadata,
    })
}

pub async fn securify_with_matrix_of_instances(
    account: Account,
    matrix: MatrixOfFactorInstances,
) -> Result<AccessController> {
    todo!()
}
