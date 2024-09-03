#![cfg(test)]

impl Profile {
    fn account_and_others(&self, address: AccountAddress) -> Result<(Account, HashSet<Account>)> {
        let account = self.account_by_address(address)?;

        let mut other_accounts = self
            .accounts
            .values()
            .cloned()
            .collect::<HashSet<Account>>();

        other_accounts.remove(&account);

        Ok((account, other_accounts))
    }
}

use crate::{derivation, prelude::*};
use rand::Rng;
use sha2::{Digest, Sha256, Sha512};

pub trait DerivationIndexWhenSecurifiedAssigner {
    fn assign_derivation_index(
        &self,
        account: Account,
        other_accounts: HashSet<Account>,
    ) -> HDPathComponent;
}

pub struct RandomFreeIndexAssigner;
impl DerivationIndexWhenSecurifiedAssigner for RandomFreeIndexAssigner {
    fn assign_derivation_index(
        &self,
        _account: Account,
        other_accounts: HashSet<Account>,
    ) -> HDPathComponent {
        let mut rng = rand::thread_rng();
        let mut index = HDPathComponent::securified(BIP32_SECURIFIED_HALF);

        while other_accounts.iter().any(|a| match a.security_state() {
            EntitySecurityState::Securified {
                matrix: _,
                access_controller,
            } => access_controller.metadata.derivation_index == index,
            _ => false,
        }) {
            index =
                HDPathComponent::securified(rng.gen_range(BIP32_SECURIFIED_HALF..BIP32_HARDENED));
        }

        index
    }
}

impl KeysCollector {
    pub fn securifying(
        matrix: MatrixOfFactorSources,
        derivation_path: DerivationPath,
        profile: &Profile,
    ) -> Result<Self> {
        KeysCollector::new(
            profile.factor_sources.clone(),
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
        )
    }
}

pub async fn securify(
    address: AccountAddress,
    matrix: MatrixOfFactorSources,
    profile: &Profile,
    derivation_index_assigner: impl DerivationIndexWhenSecurifiedAssigner,
) -> Result<AccessController> {
    let (account, other_accounts) = profile.account_and_others(address)?;

    let derivation_index =
        derivation_index_assigner.assign_derivation_index(account, other_accounts);
    let derivation_path = DerivationPath::account_tx(NetworkID::Mainnet, derivation_index);

    let keys_collector = KeysCollector::securifying(matrix, derivation_path, profile)?;

    let factor_instances = keys_collector.collect_keys().await.all_factors();

    let component_metadata = ComponentMetadata::new(factor_instances, derivation_index);

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
