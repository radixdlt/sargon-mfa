use crate::prelude::*;

/// A kind of factor list, either threshold, or override kind.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum FactorListKind {
    Threshold,
    Override,
}

pub trait HasFactorInstances {
    fn unique_factor_instances(&self) -> IndexSet<FactorInstance>;
}

/// TODO move to Sargon!!!!
pub trait HasFactorSourceKindObjectSafe {
    fn get_factor_source_kind(&self) -> FactorSourceKind;
}
impl HasFactorSourceKindObjectSafe for FactorSourceID {
    fn get_factor_source_kind(&self) -> FactorSourceKind {
        match self {
            FactorSourceID::Hash { value } => value.kind,
            FactorSourceID::Address { value } => value.kind,
        }
    }
}

#[allow(dead_code)]
// TODO REMOVE once migrated to sargon
pub trait SampleValues: Sized {
    fn sample_device() -> Self;
    fn sample_device_other() -> Self;
    fn sample_ledger() -> Self;
    fn sample_ledger_other() -> Self;
    fn sample_arculus() -> Self;
    fn sample_arculus_other() -> Self;
    fn sample_password() -> Self;
    fn sample_password_other() -> Self;
    fn sample_passphrase() -> Self;
    fn sample_passphrase_other() -> Self;
    fn sample_security_questions() -> Self;

    fn sample_security_questions_other() -> Self;
    fn sample_trusted_contact() -> Self;
    fn sample_trusted_contact_other() -> Self;
}

impl SampleValues for FactorSourceID {
    fn sample_device() -> Self {
        FactorSourceIDFromHash::sample_device().into()
    }
    fn sample_ledger() -> Self {
        FactorSourceIDFromHash::sample_ledger().into()
    }
    fn sample_ledger_other() -> Self {
        FactorSourceIDFromHash::sample_ledger_other().into()
    }
    fn sample_arculus() -> Self {
        FactorSourceIDFromHash::sample_arculus().into()
    }
    fn sample_arculus_other() -> Self {
        FactorSourceIDFromHash::sample_arculus_other().into()
    }

    fn sample_password() -> Self {
        FactorSourceIDFromHash::sample_password().into()
    }

    fn sample_password_other() -> Self {
        FactorSourceIDFromHash::sample_password_other().into()
    }

    /// Matt calls `off_device_mnemonic` "passphrase"
    fn sample_passphrase() -> Self {
        FactorSourceIDFromHash::sample_off_device().into()
    }
    /// Matt calls `off_device_mnemonic` "passphrase"
    fn sample_passphrase_other() -> Self {
        FactorSourceIDFromHash::sample_off_device_other().into()
    }
    fn sample_security_questions() -> Self {
        FactorSourceIDFromHash::sample_security_questions().into()
    }
    fn sample_device_other() -> Self {
        FactorSourceIDFromHash::sample_device_other().into()
    }
    fn sample_security_questions_other() -> Self {
        FactorSourceIDFromHash::sample_security_questions_other().into()
    }
    fn sample_trusted_contact() -> Self {
        sargon::FactorSource::sample_trusted_contact_frank().id()
    }
    fn sample_trusted_contact_other() -> Self {
        sargon::FactorSource::sample_trusted_contact_grace().id()
    }
}

use assert_json_diff::assert_json_include;
use core::fmt::Debug;
use pretty_assertions::assert_eq;
use sargon::{DerivationPath, FactorInstancesCache, NetworkID, NextDerivationEntityIndexAssigner};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::str::FromStr;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum TestingError {
    #[error("File contents is not valid JSON '{0}'")]
    FailedDoesNotContainValidJSON(String),

    #[error("Failed to JSON deserialize string")]
    FailedToDeserialize(serde_json::Error),
}

/// `name` is file name without extension, assuming it is json file

pub fn fixture_and_json<'a, T>(vector: &str) -> Result<(T, serde_json::Value), TestingError>
where
    T: for<'de> Deserialize<'de>,
{
    let json = serde_json::Value::from_str(vector)
        .map_err(|_| TestingError::FailedDoesNotContainValidJSON(vector.to_owned()))?;

    serde_json::from_value::<T>(json.clone())
        .map_err(TestingError::FailedToDeserialize)
        .map(|v| (v, json))
}

/// `name` is file name without extension, assuming it is json file

#[allow(unused)]
pub fn fixture<'a, T>(vector: &str) -> Result<T, TestingError>
where
    T: for<'de> Deserialize<'de>,
{
    fixture_and_json(vector).map(|t| t.0)
}

fn base_assert_equality_after_json_roundtrip<T>(model: &T, json: Value, expect_eq: bool)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let serialized = serde_json::to_value(model).unwrap();
    let deserialized: T = serde_json::from_value(json.clone()).unwrap();
    if expect_eq {
        pretty_assertions::assert_eq!(&deserialized, model, "Expected `model: T` and `T` deserialized from `json_string`, to be equal, but they were not.");
        assert_json_include!(actual: serialized, expected: json);
    } else {
        pretty_assertions::assert_ne!(model, &deserialized);
        pretty_assertions::assert_ne!(&deserialized, model, "Expected difference between `model: T` and `T` deserialized from `json_string`, but they were unexpectedly equal.");
        pretty_assertions::assert_ne!(serialized, json, "Expected difference between `json` (string) and json serialized from `model`, but they were unexpectedly equal.");
    }
}

/// Asserts that (pseudocode) `model.to_json() == json_string` (serialization)
/// and also asserts the associative property:
/// `Model::from_json(json_string) == model` (deserialization)
pub fn assert_eq_after_json_roundtrip<T>(model: &T, json_string: &str)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let json = json_string.parse::<serde_json::Value>().unwrap();
    base_assert_equality_after_json_roundtrip(model, json, true)
}

pub fn print_json<T>(model: &T)
where
    T: Serialize,
{
    println!(
        "{}",
        serde_json::to_string_pretty(model)
            .expect("Should be able to JSON serialize passed in serializable model.")
    );
}

/// Asserts that (pseudocode) `model.to_json() == json` (serialization)
/// and also asserts the associative property:
/// `Model::from_json(json) == model` (deserialization)

pub fn assert_json_value_eq_after_roundtrip<T>(model: &T, json: Value)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    base_assert_equality_after_json_roundtrip(model, json, true)
}

/// Asserts that (pseudocode) `model.to_json() != json_string` (serialization)
/// and also asserts the associative property:
/// `Model::from_json(json_string) != model` (deserialization)

pub fn assert_ne_after_json_roundtrip<T>(model: &T, json_string: &str)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let json = json_string.parse::<serde_json::Value>().unwrap();
    base_assert_equality_after_json_roundtrip(model, json, false)
}

/// Asserts that (pseudocode) `model.to_json() != json` (serialization)
/// and also asserts the associative property:
/// `Model::from_json(json) != model` (deserialization)

pub fn assert_json_value_ne_after_roundtrip<T>(model: &T, json: Value)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    base_assert_equality_after_json_roundtrip(model, json, false)
}

/// Asserts that (pseudocode) `Model::from_json(model.to_json()) == model`,
/// i.e. that a model after JSON roundtripping remain unchanged.

pub fn assert_json_roundtrip<T>(model: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let serialized = serde_json::to_value(model).unwrap();
    let deserialized: T = serde_json::from_value(serialized.clone()).unwrap();
    assert_eq!(model, &deserialized);
}

/// Creates JSON from `json_str` and tries to decode it, then encode the decoded,
/// value and compare it to the JSON value of the json_str.

pub fn assert_json_str_roundtrip<T>(json_str: &str)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let value = serde_json::Value::from_str(json_str).unwrap();
    let deserialized: T = serde_json::from_value(value.clone()).unwrap();
    let serialized = serde_json::to_value(&deserialized).unwrap();
    assert_eq!(value, serialized);
}

pub fn assert_json_value_fails<T>(json: Value)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let result = serde_json::from_value::<T>(json.clone());

    if let Ok(t) = result {
        panic!(
            "Expected JSON serialization to fail, but it did not, deserialized into: {:?},\n\nFrom JSON: {}",
            t,
            serde_json::to_string(&json).unwrap()
        );
    }
    // all good, expected fail.
}

pub fn assert_json_fails<T>(json_string: &str)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let json = json_string.parse::<serde_json::Value>().unwrap();
    assert_json_value_fails::<T>(json)
}

pub fn assert_json_eq_ignore_whitespace(json1: &str, json2: &str) {
    let value1: Value = serde_json::from_str(json1).expect("Invalid JSON in json1");
    let value2: Value = serde_json::from_str(json2).expect("Invalid JSON in json2");
    assert_eq!(value1, value2, "JSON strings do not match");
}

pub trait MnemonicWithPassphraseSamples: Sized {
    fn sample_device() -> Self;

    fn sample_device_other() -> Self;

    fn sample_device_12_words() -> Self;

    fn sample_device_12_words_other() -> Self;

    fn sample_ledger() -> Self;

    fn sample_ledger_other() -> Self;

    fn sample_off_device() -> Self;

    fn sample_off_device_other() -> Self;

    fn sample_arculus() -> Self;

    fn sample_arculus_other() -> Self;

    fn sample_security_questions() -> Self;

    fn sample_security_questions_other() -> Self;

    fn sample_password() -> Self;

    fn sample_password_other() -> Self;

    fn all_samples() -> Vec<Self> {
        vec![
            Self::sample_device(),
            Self::sample_device_other(),
            Self::sample_device_12_words(),
            Self::sample_device_12_words_other(),
            Self::sample_ledger(),
            Self::sample_ledger_other(),
            Self::sample_off_device(),
            Self::sample_off_device_other(),
            Self::sample_arculus(),
            Self::sample_arculus_other(),
            Self::sample_security_questions(),
            Self::sample_security_questions_other(),
            Self::sample_password(),
            Self::sample_password_other(),
        ]
    }

    fn derive_instances_for_factor_sources(
        network_id: NetworkID,
        quantity_per_factor: usize,
        derivation_presets: impl IntoIterator<Item = DerivationPreset>,
        sources: impl IntoIterator<Item = FactorSource>,
    ) -> IndexMap<FactorSourceIDFromHash, FactorInstances> {
        let next_index_assigner = NextDerivationEntityIndexAssigner::new(
            network_id,
            None,
            FactorInstancesCache::default(),
        );

        let derivation_presets = derivation_presets.into_iter().collect::<Vec<_>>();

        sources
            .into_iter()
            .map(|fs| {
                let fsid = fs.id_from_hash();
                let mwp = fsid.sample_associated_mnemonic();

                let paths = derivation_presets
                    .clone()
                    .into_iter()
                    .map(|dp| (dp, quantity_per_factor))
                    .collect::<IndexMap<DerivationPreset, usize>>();

                let paths = paths
                    .into_iter()
                    .flat_map(|(derivation_preset, qty)| {
                        // `qty` many paths
                        (0..qty)
                            .map(|_| {
                                let index_agnostic_path =
                                    derivation_preset.index_agnostic_path_on_network(network_id);

                                next_index_assigner
                                    .next(fsid, index_agnostic_path)
                                    .map(|index| DerivationPath::from((index_agnostic_path, index)))
                                    .unwrap()
                            })
                            .collect::<IndexSet<DerivationPath>>()
                    })
                    .collect::<IndexSet<DerivationPath>>();

                let instances = mwp
                    .derive_public_keys(paths)
                    .into_iter()
                    .map(|public_key| {
                        HierarchicalDeterministicFactorInstance::new(fsid, public_key)
                    })
                    .collect::<FactorInstances>();

                (fsid, instances)
            })
            .collect::<IndexMap<FactorSourceIDFromHash, FactorInstances>>()
    }
}

use once_cell::sync::Lazy;

#[allow(dead_code)]
pub(crate) static ALL_FACTOR_SOURCE_ID_SAMPLES_INC_NON_HD: Lazy<[FactorSourceID; 14]> =
    Lazy::new(|| {
        [
            FactorSourceID::sample_device(),
            FactorSourceID::sample_ledger(),
            FactorSourceID::sample_ledger_other(),
            FactorSourceID::sample_arculus(),
            FactorSourceID::sample_arculus_other(),
            FactorSourceID::sample_password(),
            FactorSourceID::sample_password_other(),
            FactorSourceID::sample_passphrase(),
            FactorSourceID::sample_passphrase_other(),
            FactorSourceID::sample_security_questions(),
            FactorSourceID::sample_device_other(),
            FactorSourceID::sample_security_questions_other(),
            FactorSourceID::sample_trusted_contact(),
            FactorSourceID::sample_trusted_contact_other(),
        ]
    });

pub(crate) static MNEMONIC_BY_ID_MAP: Lazy<
    IndexMap<FactorSourceIDFromHash, MnemonicWithPassphrase>,
> = Lazy::new(|| {
    IndexMap::from_iter([
        (
            FactorSourceIDFromHash::sample_device(),
            MnemonicWithPassphrase::sample_device(),
        ),
        (
            FactorSourceIDFromHash::sample_ledger(),
            MnemonicWithPassphrase::sample_ledger(),
        ),
        (
            FactorSourceIDFromHash::sample_ledger_other(),
            MnemonicWithPassphrase::sample_ledger_other(),
        ),
        (
            FactorSourceIDFromHash::sample_arculus(),
            MnemonicWithPassphrase::sample_arculus(),
        ),
        (
            FactorSourceIDFromHash::sample_arculus_other(),
            MnemonicWithPassphrase::sample_arculus_other(),
        ),
        (
            FactorSourceIDFromHash::sample_password(),
            MnemonicWithPassphrase::sample_password(),
        ),
        (
            FactorSourceIDFromHash::sample_off_device(),
            MnemonicWithPassphrase::sample_off_device_other(),
        ),
        (
            FactorSourceIDFromHash::sample_off_device(),
            MnemonicWithPassphrase::sample_off_device(),
        ),
        (
            FactorSourceIDFromHash::sample_off_device_other(),
            MnemonicWithPassphrase::sample_off_device_other(),
        ),
        (
            FactorSourceIDFromHash::sample_security_questions(),
            MnemonicWithPassphrase::sample_security_questions(),
        ),
        (
            FactorSourceIDFromHash::sample_security_questions_other(),
            MnemonicWithPassphrase::sample_security_questions_other(),
        ),
        (
            FactorSourceIDFromHash::sample_device_other(),
            MnemonicWithPassphrase::sample_device_other(),
        ),
        (
            FactorSourceIDFromHash::sample_device_12_words(),
            MnemonicWithPassphrase::sample_device_12_words(),
        ),
        (
            FactorSourceIDFromHash::sample_device_12_words_other(),
            MnemonicWithPassphrase::sample_device_12_words_other(),
        ),
    ])
});

pub trait MnemonicLookup {
    fn sample_associated_mnemonic(&self) -> MnemonicWithPassphrase;
}

impl MnemonicLookup for FactorSourceIDFromHash {
    fn sample_associated_mnemonic(&self) -> MnemonicWithPassphrase {
        MNEMONIC_BY_ID_MAP.get(self).cloned().unwrap()
    }
}

impl MnemonicWithPassphraseSamples for MnemonicWithPassphrase {
    fn sample_device() -> Self {
        Self::with_passphrase(Mnemonic::sample_device(), BIP39Passphrase::default())
    }

    fn sample_device_other() -> Self {
        Self::with_passphrase(Mnemonic::sample_device_other(), BIP39Passphrase::default())
    }

    fn sample_device_12_words() -> Self {
        Self::with_passphrase(
            Mnemonic::sample_device_12_words(),
            BIP39Passphrase::default(),
        )
    }

    fn sample_device_12_words_other() -> Self {
        Self::with_passphrase(
            Mnemonic::sample_device_12_words_other(),
            BIP39Passphrase::new("Olympia rules!"),
        )
    }

    fn sample_ledger() -> Self {
        Self::with_passphrase(Mnemonic::sample_ledger(), BIP39Passphrase::default())
    }

    fn sample_ledger_other() -> Self {
        Self::with_passphrase(
            Mnemonic::sample_ledger_other(),
            BIP39Passphrase::new("Mellon"),
        )
    }

    fn sample_off_device() -> Self {
        Self::with_passphrase(Mnemonic::sample_off_device(), BIP39Passphrase::default())
    }

    fn sample_off_device_other() -> Self {
        Self::with_passphrase(
            Mnemonic::sample_off_device_other(),
            BIP39Passphrase::new("open sesame"),
        )
    }

    fn sample_arculus() -> Self {
        Self::with_passphrase(Mnemonic::sample_arculus(), BIP39Passphrase::default())
    }

    fn sample_arculus_other() -> Self {
        Self::with_passphrase(
            Mnemonic::sample_arculus_other(),
            BIP39Passphrase::new("Leonidas"),
        )
    }

    fn sample_security_questions() -> Self {
        Self::with_passphrase(
            Mnemonic::sample_security_questions(),
            BIP39Passphrase::default(),
        )
    }

    fn sample_security_questions_other() -> Self {
        Self::with_passphrase(
            Mnemonic::sample_security_questions_other(),
            BIP39Passphrase::default(),
        )
    }

    fn sample_password() -> Self {
        Self::with_passphrase(Mnemonic::sample_password(), BIP39Passphrase::default())
    }

    fn sample_password_other() -> Self {
        Self::with_passphrase(
            Mnemonic::sample_password_other(),
            BIP39Passphrase::default(),
        )
    }
}
