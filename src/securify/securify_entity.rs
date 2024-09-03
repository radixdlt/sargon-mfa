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

pub struct RandomFreeIndexAssigner {
    generate: Box<dyn Fn() -> HDPathValue>,
}
impl RandomFreeIndexAssigner {
    fn new(generate: impl Fn() -> HDPathValue + 'static) -> Self {
        Self {
            generate: Box::new(generate),
        }
    }
    pub fn live() -> Self {
        Self::new(|| {
            let mut rng = rand::thread_rng();
            rng.gen_range(BIP32_SECURIFIED_HALF..BIP32_HARDENED)
        })
    }

    #[cfg(test)]
    pub fn test(hardcoded: HDPathValue) -> Self {
        Self::new(move || hardcoded + BIP32_SECURIFIED_HALF)
    }

    fn generate_path_component(&self) -> HDPathComponent {
        HDPathComponent::non_hardened((self.generate)())
    }
}
impl Default for RandomFreeIndexAssigner {
    fn default() -> Self {
        Self::live()
    }
}

impl DerivationIndexWhenSecurifiedAssigner for RandomFreeIndexAssigner {
    /// # Panics
    /// Panics after 5 failed attempts, if there are no free derivation indexes,
    /// which should never happen in practice. The probabiltiy of this happening is
    /// (1/2^30)^5 = 1/2^250, which about the same probabiltiy as guessing someones
    /// mnemonic.
    fn assign_derivation_index(
        &self,
        _account: Account,
        other_accounts: HashSet<Account>,
    ) -> HDPathComponent {
        let mut index = self.generate_path_component();

        let mut attempts = 0;
        while other_accounts.iter().any(|a| {
            attempts += 1;
            if attempts > 5 {
                panic!("Incorrect implementation, 'generate' function is not random.");
            }
            match a.security_state() {
                EntitySecurityState::Securified {
                    matrix: _,
                    access_controller,
                } => access_controller.metadata.derivation_index == index,
                _ => false,
            }
        }) {
            index = self.generate_path_component()
        }

        assert!(index.value >= BIP32_SECURIFIED_HALF);
        index
    }
}

mod test_random_free_index_assigner {

    use super::*;

    type Sut = RandomFreeIndexAssigner;

    #[test]
    #[should_panic(expected = "Incorrect implementation, 'generate' function is not random.")]
    fn test_panics_after_too_many_failed_attempts() {
        let sut = Sut::test(6);
        let account = Account::sample_unsecurified();
        let other_accounts = HashSet::<Account>::from_iter([Account::sample_securified()]);
        let _ = sut.assign_derivation_index(account, other_accounts);
    }

    #[test]
    fn works() {
        let expected = 5;
        let sut = Sut::test(expected);
        let account = Account::sample_unsecurified();
        let other_accounts = HashSet::<Account>::from_iter([Account::sample_securified()]);
        let actual = sut.assign_derivation_index(account, other_accounts);
        assert_eq!(actual, HDPathComponent::securified(expected));
    }

    #[test]
    fn live() {
        let account = Account::sample_unsecurified();
        let other_accounts = HashSet::<Account>::from_iter([Account::sample_securified()]);
        let n = 100;
        let sut = Sut::live();
        let indices = (0..n)
            .map(|_| sut.assign_derivation_index(account.clone(), other_accounts.clone()))
            .collect::<HashSet<_>>();
        assert_eq!(indices.len(), n);
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
