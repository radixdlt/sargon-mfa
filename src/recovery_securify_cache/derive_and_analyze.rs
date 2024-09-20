#![allow(unused)]

use crate::prelude::*;

/// Derivation of many keys differs between the following operations,
///
/// All operations ends with adding "ProbablyFree" FactorInstances to Cache.
///
/// "ProbablyFree" refers to keys in `KeySpace::Securified`, a Factor which
/// might have been used in the past for an AccessController, but might not
/// be active anymore - currently [2024-09-20] it does not look like Gateway
/// will index **past** public key hashes.
///
/// I will introduce some new concepts:
/// [VECI]: Virtual Entity Creating (Factor)Instance - previously (in meetings/
/// Slack) called "genesis factor instances", which is the FactorInstance which
/// created a virtual entity and formed its address. This is set in Profile on
/// said entity when it is securified to help with "is instance free" queries
/// during Profile analysis. It must be Optional since it might be unknown for
/// **recovered** securified entities - which can happen if the FactorSource of
/// the VECI was not provided by the user - or if not broad enough an index
/// space was scanned.
///
/// * [FSA] FactorSource Addition
///     CHARACTERISTICS:
///     - Derivation Indices Range Start: `0`
///     - Derivation Indices Range Size: Many, different for each factor source kind and key kind
///     - Cache Available: NO, not for this FactorSource
///     - Profile Available: YES
///     - FactorSource Addition: `Single`
///     - Entities Addition: YES - if new found - securified and unsecurified
///     - `VECI` Addition: YES - if new found
///     - Gateway Required: NO (but beneficial to use it if host is online to
///         analyze if FactorInstances are free.)
/// * [ARS] Account Recovery Scan - like `FSA` but for many FactorSources and
///     with Gateway Required.
///     CHARACTERISTICS:
///     - Derivation Indices Range Start: `0`
///     - Derivation Indices Range Size: Many, different for each factor source kind and key kind
///     - Cache Available: NO
///     - Profile Available: NO
///     - FactorSource Addition: `Many`
///     - Entities Addition: YES - securified and unsecurified
///     - `VECI` Addition: YES
///     - Gateway Required: YES
/// * [VECID] VECI Derivation
///     CHARACTERISTICS:
///     - Derivation Indices Range Start: Next Free According to Cache if available, else Profile analysis
///     - Derivation Indices Range Size: Single (OR Many if Cache needs to be filled)
///     - Cache Available: YES - if not deleted or inaccessible
///     - Profile Available: YES
///     - FactorSource Addition: NO
///     - Entities Addition: YES - unsecurified
///     - `VECI` Addition: NO
///     - Gateway Required: NO (but beneficial to use it if host is online to
///         analyze if FactorInstance are free.)
/// * [MOFID] MatrixOfFactor Instances Derivation (Securifying Entities)
///     CHARACTERISTICS:
///     - Derivation Indices Range Start: Next Free According to Cache if available, else Profile analysis
///     - Derivation Indices Range Size: Single PER FactorSource (OR Many if Cache needs to be filled)
///     - Cache Available: YES - if not deleted or inaccessible
///     - Profile Available: YES
///     - FactorSource Addition: NO
///     - Entities Addition: NO
///     - `VECI` Addition: NO
///     - Gateway Required: NO (but beneficial to use it if host is online to
///         analyze if FactorInstance are free.)
pub async fn derive_and_analyze(input: DeriveAndAnalyzeInput) -> Result<DerivationAndAnalysis> {
    error!("Using SAMPLE data in 'derive_and_analyze'!!!");
    let mut derived_instances = IndexSet::<HierarchicalDeterministicFactorInstance>::new();

    // To be fed into cache, NOT done by this function.
    let probably_free_instances = ProbablyFreeFactorInstances::sample();

    // Unsecurified entities that were recovered
    let recovered_unsecurified_entities = RecoveredUnsecurifiedEntities::sample();

    // Securified entities that were recovered
    let recovered_securified_entities = RecoveredSecurifiedEntities::sample();

    // Securified entities that were not recovered
    let unrecovered_securified_entities = UnrecoveredSecurifiedEntities::sample();

    let virtual_entity_creating_instances =
        IndexSet::<HierarchicalDeterministicFactorInstance>::new();

    // Used FactorSources which are not new - might be empty
    let old_factor_sources = input.old_factor_sources();

    /// Used FactorSource which are new - might be empty
    let new_factor_sources = input.new_factor_sources();

    Ok(DerivationAndAnalysis::new(
        probably_free_instances,
        recovered_unsecurified_entities,
        recovered_securified_entities,
        unrecovered_securified_entities,
        virtual_entity_creating_instances,
        old_factor_sources,
        new_factor_sources,
    ))
}
