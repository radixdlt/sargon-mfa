use crate::prelude::*;

use rand::Rng;
use sha2::{Digest, Sha256, Sha512};

pub struct NextFreeIndexAssigner {
    next: Box<dyn Fn(&Profile, NetworkID) -> HDPathValue>,
}
impl NextFreeIndexAssigner {
    fn new(next: impl Fn(&Profile, NetworkID) -> HDPathValue + 'static) -> Self {
        Self {
            next: Box::new(next),
        }
    }
    pub fn live() -> Self {
        Self::new(|profile, network| {
            profile
                .accounts
                .values()
                .filter(|a| a.address() == network)
                .map(|a| a.derivation_path)
                .max()
                .unwrap_or(HDPathValue::default())
        })
    }

    #[cfg(test)]
    pub fn test(hardcoded: HDPathValue) -> Self {
        Self::new(move |_, _| hardcoded + BIP32_SECURIFIED_HALF)
    }

    fn next_path_component(&self, profile: &Profile, network_id: NetworkID) -> HDPathComponent {
        let component = HDPathComponent::non_hardened((self.next)(profile, network_id));
        assert!(component.is_securified());
        component
    }
}
impl Default for NextFreeIndexAssigner {
    fn default() -> Self {
        Self::live()
    }
}

impl DerivationIndexWhenSecurifiedAssigner for NextFreeIndexAssigner {
    /// mnemonic.
    fn assign_derivation_index(&self, profile: &Profile, network_id: NetworkID) -> HDPathComponent {
        self.next_path_component(profile, network_id)
    }
}

#[cfg(test)]
mod test_next_free_index_assigner {

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
