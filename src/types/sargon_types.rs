use std::iter::Step;
use std::marker::PhantomData;

use indexmap::map::Keys;

use crate::prelude::*;

use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::ops::AddAssign;
use std::sync::Mutex;

/// An UNSAFE IDStepper, which `next` returns the consecutive next ID,
/// should only be used by tests and sample value creation.
pub struct IDStepper<T: From<Uuid>> {
    ctr: Arc<Mutex<u64>>,
    phantom: PhantomData<T>,
}
pub type UuidStepper = IDStepper<Uuid>;

impl<T: From<Uuid>> IDStepper<T> {
    pub fn starting_at(ctr: u64) -> Self {
        Self {
            ctr: Arc::new(Mutex::new(ctr)),
            phantom: PhantomData,
        }
    }

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::starting_at(0)
    }

    /// ONLY Use this in a test or when creating sample (preview) values.
    ///
    /// # Safety
    /// This is completely unsafe, it does not generate a random UUID, it creates
    /// the consecutive "next" ID.
    pub fn _next(&self) -> T {
        let n = Uuid::from_u64_pair(0, **self.ctr.lock().unwrap().borrow());
        self.ctr.lock().unwrap().borrow_mut().add_assign(1);
        n.into()
    }
}

fn take_last_n(str: impl AsRef<str>, n: usize) -> String {
    let str = str.as_ref();
    if str.len() >= n {
        str[str.len() - n..].to_owned()
    } else {
        "".to_owned()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, std::hash::Hash, derive_more::Display, derive_more::Debug)]
#[display("{kind}:{}", take_last_n(self.id.to_string(), 2))]
#[debug("{}", self.to_string())]
pub struct FactorSourceIDFromHash {
    pub kind: FactorSourceKind,
    pub id: Uuid,
}

impl FactorSourceIDFromHash {
    fn with_details(kind: FactorSourceKind, id: Uuid) -> Self {
        Self { kind, id }
    }
    pub fn new(kind: FactorSourceKind) -> Self {
        Self::with_details(kind, IDStepper::next())
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.id.as_bytes().to_vec()
    }

    pub fn sample_third() -> Self {
        Self::with_details(FactorSourceKind::Arculus, Uuid::from_bytes([0xaa; 16]))
    }

    pub fn sample_fourth() -> Self {
        Self::with_details(
            FactorSourceKind::SecurityQuestions,
            Uuid::from_bytes([0x5e; 16]),
        )
    }
}

impl HasSampleValues for FactorSourceIDFromHash {
    fn sample() -> Self {
        Self::with_details(FactorSourceKind::Device, Uuid::from_bytes([0xde; 16]))
    }
    fn sample_other() -> Self {
        Self::with_details(FactorSourceKind::Ledger, Uuid::from_bytes([0x1e; 16]))
    }
}

#[derive(Clone, PartialEq, Eq, std::hash::Hash, derive_more::Debug)]
#[debug("{:#?}", id)]
pub struct HDFactorSource {
    pub last_used: SystemTime,
    id: FactorSourceIDFromHash,
}

impl HDFactorSource {
    pub fn factor_source_id(&self) -> FactorSourceIDFromHash {
        self.id
    }
    pub fn factor_source_kind(&self) -> FactorSourceKind {
        self.id.kind
    }
    pub fn new(kind: FactorSourceKind) -> Self {
        Self {
            id: FactorSourceIDFromHash::new(kind),
            last_used: SystemTime::UNIX_EPOCH,
        }
    }
    pub fn arculus() -> Self {
        Self::new(FactorSourceKind::Arculus)
    }
    pub fn ledger() -> Self {
        Self::new(FactorSourceKind::Ledger)
    }
    pub fn device() -> Self {
        Self::new(FactorSourceKind::Device)
    }
    pub fn yubikey() -> Self {
        Self::new(FactorSourceKind::Yubikey)
    }
    pub fn off_device() -> Self {
        Self::new(FactorSourceKind::OffDeviceMnemonic)
    }
    pub fn security_question() -> Self {
        Self::new(FactorSourceKind::SecurityQuestions)
    }
}

impl PartialOrd for HDFactorSource {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for HDFactorSource {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.factor_source_kind().cmp(&other.factor_source_kind()) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.last_used.cmp(&other.last_used) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        core::cmp::Ordering::Equal
    }
}

pub trait Just<Item> {
    fn just(item: Item) -> Self;
}
impl<T: std::hash::Hash + Eq> Just<T> for IndexSet<T> {
    fn just(item: T) -> Self {
        Self::from_iter([item])
    }
}
impl<T: std::hash::Hash + Eq> Just<T> for HashSet<T> {
    fn just(item: T) -> Self {
        Self::from_iter([item])
    }
}
impl<K: std::hash::Hash + Eq, V> Just<(K, V)> for IndexMap<K, V> {
    fn just(item: (K, V)) -> Self {
        Self::from_iter([item])
    }
}
impl<K: std::hash::Hash + Eq, V> Just<(K, V)> for HashMap<K, V> {
    fn just(item: (K, V)) -> Self {
        Self::from_iter([item])
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, std::hash::Hash, PartialOrd, Ord, strum::Display)]
pub enum FactorSourceKind {
    Ledger,
    Arculus,
    Yubikey,
    SecurityQuestions,
    OffDeviceMnemonic,
    Device,
}

impl HasSampleValues for FactorSourceKind {
    fn sample() -> Self {
        FactorSourceKind::Device
    }
    fn sample_other() -> Self {
        FactorSourceKind::Ledger
    }
}

pub type HDPathValue = u32;

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, derive_more::Display, derive_more::Debug,
)]
#[display("{value}")]
#[debug("{value}")]
pub struct HDPathComponent {
    pub value: HDPathValue,
}
pub const BIP32_SECURIFIED_HALF: u32 = 0x4000_0000;
pub(crate) const BIP32_HARDENED: u32 = 0x8000_0000;

impl Step for HDPathComponent {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        Some((end.index() - start.index()) as usize)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        start.add_n_checked(count as u32)
    }

    fn backward_checked(_start: Self, _count: usize) -> Option<Self> {
        unreachable!("not needed, use (N..M) instead of (M..N) when M > N.")
    }
}

impl HDPathComponent {
    fn hardening(value: HDPathValue) -> Self {
        assert!(
            value < BIP32_HARDENED,
            "Passed value was hardened, expected it to not be."
        );
        Self {
            value: value + BIP32_HARDENED,
        }
    }
    pub fn unsecurified(value: HDPathValue) -> Self {
        Self::hardening(value)
    }
    pub fn key_space(&self) -> KeySpace {
        if self.is_securified() {
            KeySpace::Securified
        } else {
            KeySpace::Unsecurified
        }
    }
    pub fn is_in_key_space(&self, key_space: KeySpace) -> bool {
        self.key_space() == key_space
    }
    pub fn new_in_key_space(value: HDPathValue, key_space: KeySpace) -> Self {
        match key_space {
            KeySpace::Unsecurified => Self::unsecurified(value),
            KeySpace::Securified => Self::securified(value),
        }
    }
    pub fn securified(value: HDPathValue) -> Self {
        Self::hardening(value + BIP32_SECURIFIED_HALF)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.value.to_be_bytes().to_vec()
    }

    pub(crate) fn is_hardened(&self) -> bool {
        self.value >= BIP32_HARDENED
    }

    /// # Panics
    /// Panics if self would overflow within its keyspace.
    pub fn add_n_checked(&self, n: HDPathValue) -> Option<Self> {
        use std::panic;
        panic::catch_unwind(|| self.add_n(n)).ok()
    }

    /// # Panics
    /// Panics if self would overflow within its keyspace.
    pub fn add_n(&self, n: HDPathValue) -> Self {
        let index = self.index();
        if self.is_securified() {
            assert!(
                index < BIP32_HARDENED - n,
                "Index would overflow beyond BIP32_HARDENED if incremented with {:?}.",
                n,
            )
        } else {
            assert!(index < BIP32_SECURIFIED_HALF - n, "Index would overflow beyond BIP32_SECURIFIED_HALF if incremented with {:?}, which is not allowed for unsecurified indexes.", n)
        }
        Self {
            value: self.value + n,
        }
    }

    /// # Panics
    /// Panics if self would overflow within its keyspace.
    pub fn add_one(&self) -> Self {
        self.add_n(1)
    }

    #[allow(unused)]
    pub(crate) fn is_securified(&self) -> bool {
        if self.index() < BIP32_SECURIFIED_HALF {
            return false;
        }
        true
    }

    pub(crate) fn index(&self) -> HDPathValue {
        if self.is_hardened() {
            self.value - BIP32_HARDENED
        } else {
            self.value
        }
    }

    #[allow(unused)]
    pub(crate) fn securified_index(&self) -> Option<HDPathValue> {
        if !self.is_securified() {
            return None;
        }
        Some(self.index() - BIP32_SECURIFIED_HALF)
    }
}
impl HasSampleValues for HDPathComponent {
    fn sample() -> Self {
        Self::unsecurified(0)
    }
    fn sample_other() -> Self {
        Self::securified(1)
    }
}

#[cfg(test)]
mod tests_hdpathcomp {

    use super::*;

    type Sut = HDPathComponent;

    #[test]
    fn add_one_successful() {
        let t = |value: Sut, expected_index: HDPathValue| {
            let actual = value.add_one();
            assert_eq!(actual.index(), expected_index)
        };
        t(Sut::unsecurified(0), 1);
        t(Sut::unsecurified(5), 6);
        t(
            Sut::unsecurified(BIP32_SECURIFIED_HALF - 2),
            BIP32_SECURIFIED_HALF - 1,
        );

        t(Sut::securified(0), 1 + BIP32_SECURIFIED_HALF);
        t(Sut::securified(5), 6 + BIP32_SECURIFIED_HALF);
        t(
            Sut::securified(BIP32_SECURIFIED_HALF - 3),
            BIP32_SECURIFIED_HALF - 2 + BIP32_SECURIFIED_HALF,
        );

        t(
            Sut::securified(BIP32_SECURIFIED_HALF - 2),
            BIP32_SECURIFIED_HALF - 1 + BIP32_SECURIFIED_HALF,
        );
    }

    #[test]
    #[should_panic]
    fn add_one_unsecurified_max_panics() {
        let sut = Sut::unsecurified(BIP32_SECURIFIED_HALF - 1);
        _ = sut.add_one()
    }

    #[test]
    #[should_panic]
    fn add_one_securified_max_panics() {
        let sut = Sut::securified(BIP32_SECURIFIED_HALF - 1);
        _ = sut.add_one()
    }

    #[test]
    fn index_if_securified() {
        let i = 5;
        let sut = Sut::securified(i);
        assert_eq!(sut.index(), i + BIP32_SECURIFIED_HALF);
        assert_eq!(sut.securified_index(), Some(i));
    }
}

#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::Display, derive_more::Debug)]
pub enum CAP26KeyKind {
    /// For a key to be used for signing transactions.
    /// The value is the ascii sum of `"TRANSACTION_SIGNING"`
    #[display("tx")]
    #[debug("tx")]
    TransactionSigning = 1460,

    /// For a key to be used for signing authentication..
    /// The value is the ascii sum of `"AUTHENTICATION_SIGNING"`
    #[display("rola")]
    #[debug("rola")]
    AuthenticationSigning = 1678,
}

impl CAP26KeyKind {
    fn discriminant(&self) -> u16 {
        core::intrinsics::discriminant_value(self)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::Display, derive_more::Debug)]
pub enum NetworkID {
    #[display("Mainnet")]
    #[debug("0")]
    Mainnet,

    #[display("Stokenet")]
    #[debug("1")]
    Stokenet,
}

impl NetworkID {
    fn discriminant(&self) -> u8 {
        core::intrinsics::discriminant_value(self)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::Display, derive_more::Debug)]
pub enum CAP26EntityKind {
    #[display("Account")]
    #[debug("A")]
    Account,

    #[display("Identity")]
    #[debug("I")]
    Identity,
}

impl CAP26EntityKind {
    fn discriminant(&self) -> u8 {
        core::intrinsics::discriminant_value(self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, derive_more::Display, derive_more::Debug)]
#[display("{}/{}/{}/{}", network_id, entity_kind, key_kind, index)]
#[debug("{:?}/{:?}/{:?}/{:?}", network_id, entity_kind, key_kind, index)]
pub struct DerivationPath {
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_kind: CAP26KeyKind,
    pub index: HDPathComponent,
}

impl DerivationPath {
    pub fn new(
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        key_kind: CAP26KeyKind,
        index: HDPathComponent,
    ) -> Self {
        Self {
            network_id,
            entity_kind,
            key_kind,
            index,
        }
    }
    pub fn unsecurified(
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        key_kind: CAP26KeyKind,
        index: HDPathValue,
    ) -> Self {
        Self::new(
            network_id,
            entity_kind,
            key_kind,
            HDPathComponent::unsecurified(index),
        )
    }
    pub fn account_tx(network_id: NetworkID, index: HDPathComponent) -> Self {
        Self::new(
            network_id,
            CAP26EntityKind::Account,
            CAP26KeyKind::TransactionSigning,
            index,
        )
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.push(self.network_id.discriminant());
        vec.push(self.entity_kind.discriminant());
        vec.extend(self.key_kind.discriminant().to_be_bytes());
        vec.extend(self.index.to_bytes());
        vec
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PublicKey {
    /// this emulates the mnemonic
    factor_source_id: FactorSourceIDFromHash,
    /// this emulates the node in the HD tree
    derivation_path: DerivationPath,
}
impl PublicKey {
    pub fn new(factor_source_id: FactorSourceIDFromHash, derivation_path: DerivationPath) -> Self {
        Self {
            factor_source_id,
            derivation_path,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.factor_source_id.to_bytes();
        bytes.extend(self.derivation_path.to_bytes());
        bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HierarchicalDeterministicPublicKey {
    /// The expected public key of the private key derived at `derivationPath`
    pub public_key: PublicKey,

    /// The HD derivation path for the key pair which produces virtual badges (signatures).
    pub derivation_path: DerivationPath,
}
impl HierarchicalDeterministicPublicKey {
    pub fn new(derivation_path: DerivationPath, public_key: PublicKey) -> Self {
        Self {
            derivation_path,
            public_key,
        }
    }

    pub fn mocked_with(
        derivation_path: DerivationPath,
        factor_source_id: &FactorSourceIDFromHash,
    ) -> Self {
        Self::new(
            derivation_path.clone(),
            PublicKey::new(*factor_source_id, derivation_path),
        )
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [self.public_key.to_bytes(), self.derivation_path.to_bytes()].concat()
    }
}

#[derive(Clone, PartialEq, Eq, std::hash::Hash, derive_more::Debug)]
#[debug("{}", self.debug_str())]
pub struct HierarchicalDeterministicFactorInstance {
    pub factor_source_id: FactorSourceIDFromHash,
    pub public_key: HierarchicalDeterministicPublicKey,
}

impl HierarchicalDeterministicFactorInstance {
    #[allow(unused)]
    fn debug_str(&self) -> String {
        format!(
            "factor_source_id: {:#?}, derivation_path: {:#?}",
            self.factor_source_id, self.public_key.derivation_path
        )
    }

    pub fn new(
        public_key: HierarchicalDeterministicPublicKey,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Self {
        Self {
            public_key,
            factor_source_id,
        }
    }

    pub fn derivation_path(&self) -> DerivationPath {
        self.public_key.derivation_path.clone()
    }

    pub fn mocked_with(
        derivation_path: DerivationPath,
        factor_source_id: &FactorSourceIDFromHash,
    ) -> Self {
        Self::new(
            HierarchicalDeterministicPublicKey::mocked_with(derivation_path, factor_source_id),
            *factor_source_id,
        )
    }

    pub fn tx_on_network(
        entity_kind: CAP26EntityKind,
        network_id: NetworkID,
        index: HDPathComponent,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Self {
        let derivation_path = DerivationPath::new(
            network_id,
            entity_kind,
            CAP26KeyKind::TransactionSigning,
            index,
        );
        let public_key = PublicKey::new(factor_source_id, derivation_path.clone());
        let hd_public_key = HierarchicalDeterministicPublicKey::new(derivation_path, public_key);
        Self::new(hd_public_key, factor_source_id)
    }

    pub fn mainnet_tx(
        entity_kind: CAP26EntityKind,
        index: HDPathComponent,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Self {
        Self::tx_on_network(entity_kind, NetworkID::Mainnet, index, factor_source_id)
    }

    pub fn mainnet_tx_account(
        index: HDPathComponent,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Self {
        Self::mainnet_tx(CAP26EntityKind::Account, index, factor_source_id)
    }

    pub fn mainnet_tx_identity(
        index: HDPathComponent,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Self {
        Self::mainnet_tx(CAP26EntityKind::Identity, index, factor_source_id)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [self.public_key.to_bytes(), self.factor_source_id.to_bytes()].concat()
    }
}

impl HasSampleValues for HierarchicalDeterministicFactorInstance {
    fn sample() -> Self {
        Self::mainnet_tx_account(HDPathComponent::sample(), FactorSourceIDFromHash::sample())
    }
    fn sample_other() -> Self {
        Self::mainnet_tx_account(
            HDPathComponent::sample_other(),
            FactorSourceIDFromHash::sample_other(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct Hash {
    id: Uuid,
}
impl Hash {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.id.as_bytes().to_vec()
    }
    fn new(id: Uuid) -> Self {
        Self { id }
    }
    pub fn generate() -> Self {
        Self::new(Uuid::new_v4())
    }
    pub fn sample_third() -> Self {
        Self::new(Uuid::from_bytes([0x11; 16]))
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), 16); // mock
        Self::new(Uuid::from_slice(bytes).unwrap())
    }
}
impl HasSampleValues for Hash {
    fn sample() -> Self {
        Self::new(Uuid::from_bytes([0xde; 16]))
    }
    fn sample_other() -> Self {
        Self::new(Uuid::from_bytes([0xab; 16]))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct SecurifiedEntityControl {
    pub matrix: MatrixOfFactorInstances,
    pub access_controller: AccessController,
}
impl SecurifiedEntityControl {
    pub fn new(matrix: MatrixOfFactorInstances, access_controller: AccessController) -> Self {
        Self {
            matrix,
            access_controller,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash, EnumAsInner)]
pub enum EntitySecurityState {
    Unsecured(HierarchicalDeterministicFactorInstance),
    Securified(SecurifiedEntityControl),
}
impl EntitySecurityState {
    pub fn all_factor_instances(&self) -> IndexSet<HierarchicalDeterministicFactorInstance> {
        match self {
            Self::Securified(sec) => {
                let matrix = sec.matrix.clone();
                let mut set = IndexSet::new();
                set.extend(matrix.threshold_factors.clone());
                set.extend(matrix.override_factors.clone());
                set
            }
            Self::Unsecured(fi) => IndexSet::from_iter([fi.clone()]),
        }
    }
}

#[derive(Clone, PartialEq, Eq, std::hash::Hash, derive_more::Display, derive_more::Debug)]
#[display("{}_{:?}_{:?}", self.kind(), network_id, public_key_hash)]
#[debug("{}_{:?}_{:?}", self.kind(), network_id, public_key_hash)]
pub struct AbstractAddress<T: EntityKindSpecifier> {
    phantom: PhantomData<T>,
    pub network_id: NetworkID,
    pub public_key_hash: PublicKeyHash,
}
impl<T: EntityKindSpecifier> AbstractAddress<T> {
    fn kind(&self) -> String {
        T::entity_kind().to_string().to_lowercase()[0..4].to_owned()
    }
}
impl<T: EntityKindSpecifier> IsEntityAddress for AbstractAddress<T> {
    fn new(network_id: NetworkID, public_key_hash: PublicKeyHash) -> Self {
        Self {
            phantom: PhantomData,
            network_id,
            public_key_hash,
        }
    }
    fn network_id(&self) -> NetworkID {
        self.network_id
    }
    fn public_key_hash(&self) -> PublicKeyHash {
        self.public_key_hash.clone()
    }
}
impl<T: EntityKindSpecifier> AbstractAddress<T> {
    pub fn entity_kind() -> CAP26EntityKind {
        T::entity_kind()
    }
}
impl<T: EntityKindSpecifier> AbstractAddress<T> {
    pub fn sample_0() -> Self {
        Self::new(NetworkID::Mainnet, PublicKeyHash::sample_1())
    }
    pub fn sample_1() -> Self {
        Self::new(NetworkID::Mainnet, PublicKeyHash::sample_1())
    }
    pub fn sample_2() -> Self {
        Self::new(NetworkID::Mainnet, PublicKeyHash::sample_2())
    }
    pub fn sample_3() -> Self {
        Self::new(NetworkID::Mainnet, PublicKeyHash::sample_3())
    }
    pub fn sample_4() -> Self {
        Self::new(NetworkID::Mainnet, PublicKeyHash::sample_4())
    }
    pub fn sample_5() -> Self {
        Self::new(NetworkID::Mainnet, PublicKeyHash::sample_5())
    }
    pub fn sample_6() -> Self {
        Self::new(NetworkID::Mainnet, PublicKeyHash::sample_6())
    }
    pub fn sample_7() -> Self {
        Self::new(NetworkID::Mainnet, PublicKeyHash::sample_7())
    }
    pub fn sample_8() -> Self {
        Self::new(NetworkID::Mainnet, PublicKeyHash::sample_8())
    }
    pub fn sample_9() -> Self {
        Self::new(NetworkID::Mainnet, PublicKeyHash::sample_9())
    }
}
impl<T: EntityKindSpecifier> HasSampleValues for AbstractAddress<T> {
    fn sample() -> Self {
        Self::sample_0()
    }
    fn sample_other() -> Self {
        Self::sample_1()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct AccountAddressTag;

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct IdentityAddressTag;

pub trait EntityKindSpecifier {
    fn entity_kind() -> CAP26EntityKind;
}
impl EntityKindSpecifier for AccountAddressTag {
    fn entity_kind() -> CAP26EntityKind {
        CAP26EntityKind::Account
    }
}
impl EntityKindSpecifier for IdentityAddressTag {
    fn entity_kind() -> CAP26EntityKind {
        CAP26EntityKind::Identity
    }
}

impl<T: EntityKindSpecifier> EntityKindSpecifier for AbstractAddress<T> {
    fn entity_kind() -> CAP26EntityKind {
        T::entity_kind()
    }
}

pub type AccountAddress = AbstractAddress<AccountAddressTag>;
pub type IdentityAddress = AbstractAddress<IdentityAddressTag>;

#[derive(Clone, PartialEq, Eq, std::hash::Hash, derive_more::Display)]
pub enum AddressOfAccountOrPersona {
    Account(AccountAddress),
    Identity(IdentityAddress),
}
impl AddressOfAccountOrPersona {
    pub fn network_id(&self) -> NetworkID {
        match self {
            Self::Account(a) => a.network_id(),
            Self::Identity(i) => i.network_id(),
        }
    }
    pub fn public_key_hash(&self) -> PublicKeyHash {
        match self {
            Self::Account(a) => a.public_key_hash(),
            Self::Identity(i) => i.public_key_hash(),
        }
    }
}
impl TryFrom<AddressOfAccountOrPersona> for AccountAddress {
    type Error = CommonError;

    fn try_from(value: AddressOfAccountOrPersona) -> Result<Self> {
        match value {
            AddressOfAccountOrPersona::Account(a) => Ok(a),
            AddressOfAccountOrPersona::Identity(_) => Err(CommonError::Failure),
        }
    }
}
impl TryFrom<AddressOfAccountOrPersona> for IdentityAddress {
    type Error = CommonError;

    fn try_from(value: AddressOfAccountOrPersona) -> Result<Self> {
        match value {
            AddressOfAccountOrPersona::Identity(a) => Ok(a),
            AddressOfAccountOrPersona::Account(_) => Err(CommonError::Failure),
        }
    }
}
impl std::fmt::Debug for AddressOfAccountOrPersona {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}
impl HasSampleValues for AddressOfAccountOrPersona {
    fn sample() -> Self {
        Self::Account(AccountAddress::sample())
    }
    fn sample_other() -> Self {
        Self::Identity(IdentityAddress::sample())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub enum AccountOrPersona {
    AccountEntity(Account),
    PersonaEntity(Persona),
}
impl AccountOrPersona {
    pub fn network_id(&self) -> NetworkID {
        match self {
            AccountOrPersona::AccountEntity(a) => a.network_id(),
            AccountOrPersona::PersonaEntity(p) => p.network_id(),
        }
    }

    pub fn matches_key_space(&self, key_space: KeySpace) -> bool {
        match key_space {
            KeySpace::Securified => self.is_securified(),
            KeySpace::Unsecurified => !self.is_securified(),
        }
    }

    pub fn is_securified(&self) -> bool {
        self.security_state().is_securified()
    }
}

pub trait IsEntityAddress: Sized {
    fn new(network_id: NetworkID, public_key_hash: PublicKeyHash) -> Self;
    fn network_id(&self) -> NetworkID;
    fn public_key_hash(&self) -> PublicKeyHash;

    fn by_hashing(network_id: NetworkID, key: impl Into<PublicKeyHash>) -> Self {
        Self::new(network_id, key.into())
    }
}

pub trait IsEntity: Into<AccountOrPersona> + TryFrom<AccountOrPersona> + Clone {
    type Address: IsEntityAddress
        + HasSampleValues
        + Clone
        + Into<AddressOfAccountOrPersona>
        + TryFrom<AddressOfAccountOrPersona>
        + EntityKindSpecifier
        + std::hash::Hash
        + Eq
        + std::fmt::Debug;

    fn new(
        name: impl AsRef<str>,
        address: Self::Address,
        security_state: impl Into<EntitySecurityState>,
    ) -> Self;

    fn unsecurified_mainnet(
        name: impl AsRef<str>,
        genesis_factor_instance: HierarchicalDeterministicFactorInstance,
    ) -> Self {
        let address = Self::Address::new(
            NetworkID::Mainnet,
            genesis_factor_instance.public_key_hash(),
        );
        Self::new(
            name,
            address,
            EntitySecurityState::Unsecured(genesis_factor_instance),
        )
    }

    fn securified_mainnet(
        name: impl AsRef<str>,
        address: Self::Address,
        make_matrix: impl Fn() -> MatrixOfFactorInstances,
    ) -> Self {
        let matrix = make_matrix();
        let access_controller = AccessController::new(
            AccessControllerAddress::new(address.clone()),
            ComponentMetadata::new(matrix.clone()),
        );

        Self::new(
            name,
            address,
            EntitySecurityState::Securified(SecurifiedEntityControl::new(
                matrix,
                access_controller,
            )),
        )
    }

    fn network_id(&self) -> NetworkID {
        match self.security_state() {
            EntitySecurityState::Securified(sec) => {
                sec.matrix
                    .all_factors()
                    .iter()
                    .last()
                    .unwrap()
                    .public_key
                    .derivation_path
                    .network_id
            }
            EntitySecurityState::Unsecured(fi) => fi.public_key.derivation_path.network_id,
        }
    }
    fn all_factor_instances(&self) -> HashSet<HierarchicalDeterministicFactorInstance> {
        self.security_state()
            .all_factor_instances()
            .into_iter()
            .collect()
    }

    fn is_securified(&self) -> bool {
        match self.security_state() {
            EntitySecurityState::Securified(_) => true,
            EntitySecurityState::Unsecured(_) => false,
        }
    }
    fn entity_address(&self) -> Self::Address;

    fn name(&self) -> String;
    fn kind() -> CAP26EntityKind {
        Self::Address::entity_kind()
    }
    fn security_state(&self) -> EntitySecurityState;
    fn address(&self) -> AddressOfAccountOrPersona {
        self.entity_address().clone().into()
    }
    fn e0() -> Self;
    fn e1() -> Self;
    fn e2() -> Self;
    fn e3() -> Self;
    fn e4() -> Self;
    fn e5() -> Self;
    fn e6() -> Self;
    fn e7() -> Self;
}

#[derive(Clone, PartialEq, Eq, std::hash::Hash, derive_more::Debug)]
#[debug("{}", self.address())]
pub struct AbstractEntity<A: Clone + Into<AddressOfAccountOrPersona> + EntityKindSpecifier> {
    address: A,
    pub name: String,
    pub security_state: EntitySecurityState,
}
pub type Account = AbstractEntity<AccountAddress>;

impl IsEntity for Account {
    fn new(
        name: impl AsRef<str>,
        address: Self::Address,
        security_state: impl Into<EntitySecurityState>,
    ) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            address,
            security_state: security_state.into(),
        }
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    type Address = AccountAddress;
    fn security_state(&self) -> EntitySecurityState {
        self.security_state.clone()
    }
    fn entity_address(&self) -> Self::Address {
        self.address.clone()
    }
    fn e0() -> Self {
        Self::a0()
    }
    fn e1() -> Self {
        Self::a1()
    }
    fn e2() -> Self {
        Self::a2()
    }
    fn e3() -> Self {
        Self::a3()
    }
    fn e4() -> Self {
        Self::a4()
    }
    fn e5() -> Self {
        Self::a5()
    }
    fn e6() -> Self {
        Self::a6()
    }
    fn e7() -> Self {
        Self::a7()
    }
}

pub type Persona = AbstractEntity<IdentityAddress>;
impl IsEntity for Persona {
    fn new(
        name: impl AsRef<str>,
        address: IdentityAddress,
        security_state: impl Into<EntitySecurityState>,
    ) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            address,
            security_state: security_state.into(),
        }
    }
    type Address = IdentityAddress;
    fn security_state(&self) -> EntitySecurityState {
        self.security_state.clone()
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    fn entity_address(&self) -> Self::Address {
        self.address.clone()
    }
    fn e0() -> Self {
        Self::p0()
    }
    fn e1() -> Self {
        Self::p1()
    }
    fn e2() -> Self {
        Self::p2()
    }
    fn e3() -> Self {
        Self::p3()
    }
    fn e4() -> Self {
        Self::p4()
    }
    fn e5() -> Self {
        Self::p5()
    }
    fn e6() -> Self {
        Self::p6()
    }
    fn e7() -> Self {
        Self::p7()
    }
}

impl<T: Clone + Into<AddressOfAccountOrPersona> + EntityKindSpecifier> EntityKindSpecifier
    for AbstractEntity<T>
{
    fn entity_kind() -> CAP26EntityKind {
        T::entity_kind()
    }
}

impl<T: Clone + Into<AddressOfAccountOrPersona> + EntityKindSpecifier> AbstractEntity<T> {
    pub fn address(&self) -> AddressOfAccountOrPersona {
        self.address.clone().into()
    }
}

impl From<Account> for AccountOrPersona {
    fn from(value: Account) -> Self {
        Self::AccountEntity(value)
    }
}

impl TryFrom<AccountOrPersona> for Account {
    type Error = CommonError;

    fn try_from(value: AccountOrPersona) -> Result<Self> {
        match value {
            AccountOrPersona::AccountEntity(a) => Ok(a),
            AccountOrPersona::PersonaEntity(_) => Err(CommonError::Failure),
        }
    }
}

impl TryFrom<AccountOrPersona> for Persona {
    type Error = CommonError;

    fn try_from(value: AccountOrPersona) -> Result<Self> {
        match value {
            AccountOrPersona::PersonaEntity(p) => Ok(p),
            AccountOrPersona::AccountEntity(_) => Err(CommonError::Failure),
        }
    }
}

impl From<Persona> for AccountOrPersona {
    fn from(value: Persona) -> Self {
        Self::PersonaEntity(value)
    }
}

impl From<AccountAddress> for AddressOfAccountOrPersona {
    fn from(value: AccountAddress) -> Self {
        Self::Account(value)
    }
}

impl From<IdentityAddress> for AddressOfAccountOrPersona {
    fn from(value: IdentityAddress) -> Self {
        Self::Identity(value)
    }
}

impl HasSampleValues for Account {
    fn sample() -> Self {
        Self::sample_unsecurified()
    }
    fn sample_other() -> Self {
        Self::sample_securified()
    }
}

impl HasSampleValues for Persona {
    fn sample() -> Self {
        Self::sample_unsecurified()
    }
    fn sample_other() -> Self {
        Self::sample_securified()
    }
}

impl<
        T: IsEntityAddress
            + Clone
            + Into<AddressOfAccountOrPersona>
            + HasSampleValues
            + EntityKindSpecifier,
    > AbstractEntity<T>
where
    Self: IsEntity,
{
    /// mainnet
    pub(crate) fn sample_unsecurified() -> Self {
        <Self as IsEntity>::unsecurified_mainnet(
            "Sample Unsec",
            HierarchicalDeterministicFactorInstance::fi0(T::entity_kind()),
        )
    }

    /// mainnet
    pub(crate) fn sample_securified() -> Self {
        <Self as IsEntity>::securified_mainnet(
            "Grace",
            <AbstractEntity<T> as IsEntity>::Address::sample_other(),
            || {
                let idx = HDPathComponent::securified(6);
                MatrixOfFactorInstances::m6(HierarchicalDeterministicFactorInstance::f(
                    Self::entity_kind(),
                    idx,
                ))
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct MatrixOfFactors<F> {
    pub threshold_factors: Vec<F>,
    pub threshold: u8,
    pub override_factors: Vec<F>,
}

impl<F> MatrixOfFactors<F>
where
    F: std::hash::Hash + std::cmp::Eq + Clone,
{
    /// # Panics
    /// Panics if threshold > threshold_factor.len()
    ///
    /// Panics if the same factor is present in both lists
    pub fn new(
        threshold_factors: impl IntoIterator<Item = F>,
        threshold: u8,
        override_factors: impl IntoIterator<Item = F>,
    ) -> Self {
        let threshold_factors = threshold_factors.into_iter().collect_vec();

        assert!(threshold_factors.len() >= threshold as usize);

        let override_factors = override_factors.into_iter().collect_vec();

        assert!(
            HashSet::<F>::from_iter(threshold_factors.clone())
                .intersection(&HashSet::<F>::from_iter(override_factors.clone()))
                .collect_vec()
                .is_empty(),
            "A factor MUST NOT be present in both threshold AND override list."
        );

        Self {
            threshold_factors,
            threshold,
            override_factors: override_factors.into_iter().collect_vec(),
        }
    }

    pub fn override_only(factors: impl IntoIterator<Item = F>) -> Self {
        Self::new([], 0, factors)
    }

    pub fn single_override(factor: F) -> Self {
        Self::override_only([factor])
    }

    pub fn threshold_only(factors: impl IntoIterator<Item = F>, threshold: u8) -> Self {
        Self::new(factors, threshold, [])
    }

    pub fn all_factors(&self) -> IndexSet<F> {
        let mut set = IndexSet::new();
        set.extend(self.threshold_factors.clone());
        set.extend(self.override_factors.clone());
        set
    }

    pub fn single_threshold(factor: F) -> Self {
        Self::threshold_only([factor], 1)
    }
}

pub type MatrixOfFactorInstances = MatrixOfFactors<HierarchicalDeterministicFactorInstance>;

impl MatrixOfFactorInstances {
    pub fn fulfilling_matrix_of_factor_sources_with_instances(
        instances: impl IntoIterator<Item = HierarchicalDeterministicFactorInstance>,
        matrix_of_factor_sources: MatrixOfFactorSources,
    ) -> Result<Self> {
        let instances = instances.into_iter().collect_vec();

        let get_factors =
            |required: Vec<HDFactorSource>| -> Result<Vec<HierarchicalDeterministicFactorInstance>> {
                required
                    .iter()
                    .map(|f| {
                        instances
                            .iter()
                            .find(|i| i.factor_source_id() == f.factor_source_id())
                            .cloned()
                            .ok_or(CommonError::Failure)
                        })
                    .collect::<Result<Vec<HierarchicalDeterministicFactorInstance>>>()
            };

        let threshold_factors = get_factors(matrix_of_factor_sources.threshold_factors)?;
        let override_factors = get_factors(matrix_of_factor_sources.override_factors)?;

        Ok(Self::new(
            threshold_factors,
            matrix_of_factor_sources.threshold,
            override_factors,
        ))
    }
}

pub type MatrixOfFactorSources = MatrixOfFactors<HDFactorSource>;

/// For unsecurified entities we map single factor -> single threshold factor.
/// Which is used by ROLA.
impl From<HierarchicalDeterministicFactorInstance> for MatrixOfFactorInstances {
    fn from(value: HierarchicalDeterministicFactorInstance) -> Self {
        Self {
            threshold: 1,
            threshold_factors: vec![value],
            override_factors: Vec::new(),
        }
    }
}

pub trait HasSampleValues {
    fn sample() -> Self;
    fn sample_other() -> Self;
}

#[derive(Clone, PartialEq, Eq, std::hash::Hash, Getters, derive_more::Debug)]
#[debug("TXID({:#?})", hash.id.to_string()[..6].to_owned())]
pub struct IntentHash {
    hash: Hash,
}

impl IntentHash {
    fn new(hash: Hash) -> Self {
        Self { hash }
    }
    pub fn generate() -> Self {
        Self::new(Hash::generate())
    }
    pub fn sample_third() -> Self {
        Self::new(Hash::sample_third())
    }
}

impl HasSampleValues for IntentHash {
    fn sample() -> Self {
        Self::new(Hash::sample())
    }
    fn sample_other() -> Self {
        Self::new(Hash::sample_other())
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct TransactionManifest {
    addresses_of_accounts_requiring_auth: Vec<AccountAddress>,
    addresses_of_personas_requiring_auth: Vec<IdentityAddress>,
}

impl TransactionManifest {
    pub fn new(
        addresses_of_accounts_requiring_auth: impl IntoIterator<Item = AccountAddress>,
        addresses_of_personas_requiring_auth: impl IntoIterator<Item = IdentityAddress>,
    ) -> Self {
        Self {
            addresses_of_accounts_requiring_auth: addresses_of_accounts_requiring_auth
                .into_iter()
                .collect_vec(),
            addresses_of_personas_requiring_auth: addresses_of_personas_requiring_auth
                .into_iter()
                .collect_vec(),
        }
    }
    pub fn summary(&self) -> ManifestSummary {
        ManifestSummary::new(
            self.addresses_of_accounts_requiring_auth.clone(),
            self.addresses_of_personas_requiring_auth.clone(),
        )
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct TransactionIntent {
    pub intent_hash: IntentHash,
    pub(crate) manifest: TransactionManifest,
}

impl TransactionIntent {
    fn with(manifest: TransactionManifest) -> Self {
        Self {
            manifest,
            intent_hash: IntentHash::generate(),
        }
    }
    pub fn new(
        addresses_of_accounts_requiring_auth: impl IntoIterator<Item = AccountAddress>,
        addresses_of_personas_requiring_auth: impl IntoIterator<Item = IdentityAddress>,
    ) -> Self {
        Self::with(TransactionManifest::new(
            addresses_of_accounts_requiring_auth,
            addresses_of_personas_requiring_auth,
        ))
    }
    pub fn address_of<'a, 'p>(
        accounts_requiring_auth: impl IntoIterator<Item = &'a Account>,
        personas_requiring_auth: impl IntoIterator<Item = &'p Persona>,
    ) -> Self {
        Self::new(
            accounts_requiring_auth
                .into_iter()
                .map(|a| a.entity_address())
                .collect_vec(),
            personas_requiring_auth
                .into_iter()
                .map(|a| a.entity_address())
                .collect_vec(),
        )
    }
}

pub struct ManifestSummary {
    pub addresses_of_accounts_requiring_auth: Vec<AccountAddress>,
    pub addresses_of_personas_requiring_auth: Vec<IdentityAddress>,
}

impl ManifestSummary {
    pub fn new(
        addresses_of_accounts_requiring_auth: impl IntoIterator<Item = AccountAddress>,
        addresses_of_personas_requiring_auth: impl IntoIterator<Item = IdentityAddress>,
    ) -> Self {
        Self {
            addresses_of_accounts_requiring_auth: addresses_of_accounts_requiring_auth
                .into_iter()
                .collect_vec(),
            addresses_of_personas_requiring_auth: addresses_of_personas_requiring_auth
                .into_iter()
                .collect_vec(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Profile {
    pub factor_sources: IndexSet<HDFactorSource>,
    pub accounts: HashMap<AccountAddress, Account>,
    pub personas: HashMap<IdentityAddress, Persona>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct SecurifiedEntity {
    pub entity: AccountOrPersona,
    pub control: SecurifiedEntityControl,
}

impl Profile {
    pub fn get_entities_erased(&self, entity_kind: CAP26EntityKind) -> IndexSet<AccountOrPersona> {
        match entity_kind {
            CAP26EntityKind::Account => self
                .accounts
                .values()
                .cloned()
                .map(AccountOrPersona::from)
                .collect::<IndexSet<_>>(),
            CAP26EntityKind::Identity => self
                .personas
                .values()
                .cloned()
                .map(AccountOrPersona::from)
                .collect::<IndexSet<_>>(),
        }
    }
    pub fn get_entities<E: IsEntity + std::hash::Hash + Eq>(&self) -> IndexSet<E> {
        self.get_entities_erased(E::kind())
            .into_iter()
            .map(|e| E::try_from(e).ok().unwrap())
            .collect()
    }

    pub fn get_entities_of_kind_on_network_in_key_space(
        &self,
        entity_kind: CAP26EntityKind,
        network_id: NetworkID,
        key_space: KeySpace,
    ) -> IndexSet<AccountOrPersona> {
        self.get_entities_erased(entity_kind)
            .into_iter()
            .filter(|e| e.network_id() == network_id)
            .filter(|e| e.matches_key_space(key_space))
            .collect()
    }

    pub fn get_securified_entities_of_kind_on_network(
        &self,
        entity_kind: CAP26EntityKind,
        network_id: NetworkID,
    ) -> IndexSet<SecurifiedEntity> {
        self.get_entities_of_kind_on_network_in_key_space(
            entity_kind,
            network_id,
            KeySpace::Securified,
        )
        .into_iter()
        .map(|e: AccountOrPersona| {
            let control = match e.security_state() {
                EntitySecurityState::Securified(control) => control,
                _ => unreachable!(),
            };
            SecurifiedEntity { entity: e, control }
        })
        .collect()
    }

    pub fn get_accounts(&self) -> IndexSet<Account> {
        self.get_entities()
    }
    pub fn new<'a, 'p>(
        factor_sources: impl IntoIterator<Item = HDFactorSource>,
        accounts: impl IntoIterator<Item = &'a Account>,
        personas: impl IntoIterator<Item = &'p Persona>,
    ) -> Self {
        let factor_sources = factor_sources.into_iter().collect::<IndexSet<_>>();
        Self {
            factor_sources,
            accounts: accounts
                .into_iter()
                .map(|a| (a.entity_address(), a.clone()))
                .collect::<HashMap<_, _>>(),
            personas: personas
                .into_iter()
                .map(|p| (p.entity_address(), p.clone()))
                .collect::<HashMap<_, _>>(),
        }
    }
    pub fn account_by_address(&self, address: AccountAddress) -> Result<Account> {
        self.accounts
            .get(&address)
            .ok_or(CommonError::UnknownAccount)
            .cloned()
    }
    pub fn update_account(&mut self, account: Account) {
        assert!(self
            .accounts
            .insert(account.entity_address(), account)
            .is_some());
    }
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct Signature([u8; 64]);
impl Signature {
    pub fn new_with_hex(s: impl AsRef<str>) -> Result<Self> {
        hex::decode(s.as_ref())
            .map_err(|_| CommonError::Failure)
            .and_then(|b| b.try_into().map_err(|_| CommonError::Failure))
            .map(Self)
    }
}
impl HasSampleValues for Signature {
    fn sample() -> Self {
        Self::new_with_hex("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef").unwrap()
    }
    fn sample_other() -> Self {
        Self::new_with_hex("fadecafefadecafefadecafefadecafefadecafefadecafefadecafefadecafefadecafefadecafefadecafefadecafefadecafefadecafefadecafefadecafe").unwrap()
    }
}

#[cfg(test)]
mod signature_tests {
    use super::*;

    type Sut = Signature;

    #[test]
    fn eq() {
        assert_eq!(Sut::sample(), Sut::sample());
        assert_eq!(Sut::sample_other(), Sut::sample_other());
        assert_ne!(Sut::sample(), Sut::sample_other());
    }
}

impl Signature {
    /// Emulates the signing of `intent_hash` with `factor_instance` - in a
    /// deterministic manner.
    pub fn produced_by(
        intent_hash: IntentHash,
        factor_instance: impl Into<HierarchicalDeterministicFactorInstance>,
    ) -> Self {
        let factor_instance = factor_instance.into();

        let intent_hash_bytes = intent_hash.hash().to_bytes();
        let factor_instance_bytes = factor_instance.to_bytes();
        let input_bytes = [intent_hash_bytes, factor_instance_bytes].concat();
        let mut hasher = sha2::Sha512::new();
        hasher.update(input_bytes);
        Self(hasher.finalize().into())
    }

    /// Emulates signing using `input`.
    pub fn produced_by_input(input: &HDSignatureInput) -> Self {
        Self::produced_by(
            input.intent_hash.clone(),
            input.owned_factor_instance.clone(),
        )
    }
}

pub type Result<T, E = CommonError> = std::result::Result<T, E>;

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
pub enum CommonError {
    #[error("Unknown factor source")]
    UnknownFactorSource,

    #[error("Failed")]
    Failure,

    #[error("Invalid factor source kind")]
    InvalidFactorSourceKind,

    #[error("Empty FactorSources list")]
    FactorSourcesOfKindEmptyFactors,

    #[error("Unknown account")]
    UnknownAccount,

    #[error("Unknown persona")]
    UnknownPersona,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccessControllerAddress(pub String);
impl AccessControllerAddress {
    pub fn new<A: IsEntityAddress>(a: A) -> Self {
        Self(format!(
            "access_controller_{:?}_{:?}",
            a.network_id(),
            a.public_key_hash()
        ))
    }
    // pub fn generate() -> Self {
    //     Self::new(Uuid::new_v4().to_string())
    // }
}

#[derive(Clone, PartialEq, Eq, Hash, derive_more::Debug)]
#[debug("{}", hex::encode(&self.0[28..32]))]
pub struct PublicKeyHash([u8; 32]);

impl PublicKeyHash {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    pub fn repeat(byte: u8) -> Self {
        Self::new([byte; 32])
    }
    pub fn sample_0() -> Self {
        Self::repeat(0x50)
    }
    pub fn sample_1() -> Self {
        Self::repeat(0x51)
    }
    pub fn sample_2() -> Self {
        Self::repeat(0x52)
    }
    pub fn sample_3() -> Self {
        Self::repeat(0x53)
    }
    pub fn sample_4() -> Self {
        Self::repeat(0x54)
    }
    pub fn sample_5() -> Self {
        Self::repeat(0x55)
    }
    pub fn sample_6() -> Self {
        Self::repeat(0x56)
    }
    pub fn sample_7() -> Self {
        Self::repeat(0x57)
    }
    pub fn sample_8() -> Self {
        Self::repeat(0x58)
    }
    pub fn sample_9() -> Self {
        Self::repeat(0x59)
    }
}
impl HasSampleValues for PublicKeyHash {
    fn sample() -> Self {
        Self::sample_0()
    }
    fn sample_other() -> Self {
        Self::sample_1()
    }
}

impl PublicKey {
    pub fn hash(&self) -> PublicKeyHash {
        let mut hasher = Sha256::new();
        hasher.update(self.to_bytes());
        let digest = hasher.finalize().into();
        PublicKeyHash(digest)
    }
}

impl HierarchicalDeterministicPublicKey {
    pub fn hash(&self) -> PublicKeyHash {
        self.public_key.hash()
    }
}
impl HierarchicalDeterministicFactorInstance {
    pub fn public_key_hash(&self) -> PublicKeyHash {
        self.public_key.hash()
    }
}
impl From<PublicKey> for PublicKeyHash {
    fn from(value: PublicKey) -> Self {
        value.hash()
    }
}
impl From<HierarchicalDeterministicPublicKey> for PublicKeyHash {
    fn from(value: HierarchicalDeterministicPublicKey) -> Self {
        value.hash()
    }
}
impl From<HierarchicalDeterministicFactorInstance> for PublicKeyHash {
    fn from(value: HierarchicalDeterministicFactorInstance) -> Self {
        value.public_key_hash()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumAsInner)]
pub enum ScryptoResourceOrNonFungible {
    PublicKeyHash(PublicKeyHash),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumAsInner)]
pub enum ScryptoProofRule {
    AnyOf(Vec<ScryptoResourceOrNonFungible>),
    CountOf(usize, Vec<ScryptoResourceOrNonFungible>),
    // AllOf
    // Require
    // AmountOf
}
impl ScryptoProofRule {
    pub fn any_of(values: Vec<ScryptoResourceOrNonFungible>) -> Self {
        Self::AnyOf(values)
    }
    pub fn count_of(count: usize, values: Vec<ScryptoResourceOrNonFungible>) -> Self {
        Self::CountOf(count, values)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumAsInner)]
pub enum ScryptoAccessRuleNode {
    ProofRule(ScryptoProofRule),
    AnyOf(Vec<ScryptoAccessRuleNode>),
    AllOf(Vec<ScryptoAccessRuleNode>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumAsInner)]
pub enum ScryptoAccessRule {
    Protected(ScryptoAccessRuleNode),
    // AllowAll
    // DenyAll
}
impl ScryptoAccessRule {
    pub fn protected(rule: ScryptoAccessRuleNode) -> Self {
        Self::Protected(rule)
    }
    pub fn with_threshold(
        count: usize,
        threshold_factors: impl IntoIterator<Item = impl Into<PublicKeyHash>>,
        override_factors: impl IntoIterator<Item = impl Into<PublicKeyHash>>,
    ) -> Self {
        Self::protected(ScryptoAccessRuleNode::AnyOf(vec![
            ScryptoAccessRuleNode::ProofRule(ScryptoProofRule::CountOf(
                count,
                threshold_factors
                    .into_iter()
                    .map(Into::into)
                    .map(ScryptoResourceOrNonFungible::PublicKeyHash)
                    .collect_vec(),
            )),
            ScryptoAccessRuleNode::ProofRule(ScryptoProofRule::AnyOf(
                override_factors
                    .into_iter()
                    .map(Into::into)
                    .map(ScryptoResourceOrNonFungible::PublicKeyHash)
                    .collect_vec(),
            )),
        ]))
    }
}

pub type MatrixOfKeyHashes = MatrixOfFactors<PublicKeyHash>;
impl From<MatrixOfFactorInstances> for ScryptoAccessRule {
    fn from(value: MatrixOfFactorInstances) -> Self {
        Self::with_threshold(
            value.threshold as usize,
            value.threshold_factors,
            value.override_factors,
        )
    }
}
impl From<MatrixOfKeyHashes> for ScryptoAccessRule {
    fn from(value: MatrixOfKeyHashes) -> Self {
        Self::with_threshold(
            value.threshold as usize,
            value.threshold_factors,
            value.override_factors,
        )
    }
}
impl TryFrom<ScryptoAccessRule> for MatrixOfKeyHashes {
    type Error = CommonError;

    fn try_from(value: ScryptoAccessRule) -> Result<Self> {
        let protected = value.into_protected().map_err(|_| CommonError::Failure)?;
        let root_any_of = protected.into_any_of().map_err(|_| CommonError::Failure)?;
        if root_any_of.len() != 2 {
            return Err(CommonError::Failure);
        }
        let rule_0 = root_any_of[0]
            .clone()
            .into_proof_rule()
            .map_err(|_| CommonError::Failure)?;

        let rule_1 = root_any_of[1]
            .clone()
            .into_proof_rule()
            .map_err(|_| CommonError::Failure)?;

        let threshold_rule = rule_0.into_count_of().map_err(|_| CommonError::Failure)?;
        let override_rule = rule_1.into_any_of().map_err(|_| CommonError::Failure)?;

        let threshold = threshold_rule.0;
        let threshold_hashes = threshold_rule
            .1
            .into_iter()
            .map(|r| r.into_public_key_hash().map_err(|_| CommonError::Failure))
            .collect::<Result<Vec<PublicKeyHash>>>()?;

        let override_hashes = override_rule
            .into_iter()
            .map(|r| r.into_public_key_hash().map_err(|_| CommonError::Failure))
            .collect::<Result<Vec<PublicKeyHash>>>()?;

        Ok(Self::new(
            threshold_hashes,
            threshold as u8,
            override_hashes,
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComponentMetadata {
    pub scrypto_access_rules: ScryptoAccessRule,
}

impl ComponentMetadata {
    pub fn new(scrypto_access_rules: impl Into<ScryptoAccessRule>) -> Self {
        Self {
            scrypto_access_rules: scrypto_access_rules.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccessController {
    pub address: AccessControllerAddress,
    pub metadata: ComponentMetadata,
}

impl AccessController {
    pub fn new(address: AccessControllerAddress, metadata: ComponentMetadata) -> Self {
        Self { address, metadata }
    }
}
