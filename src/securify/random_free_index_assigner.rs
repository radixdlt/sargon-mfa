use crate::prelude::*;

use rand::Rng;
use sha2::{Digest, Sha256, Sha512};

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
                EntitySecurityState::Securified(sec) => {
                    sec.access_controller.metadata.derivation_index == index
                }
                _ => false,
            }
        }) {
            index = self.generate_path_component()
        }

        assert!(index.is_securified());
        index
    }
}

#[cfg(test)]
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
