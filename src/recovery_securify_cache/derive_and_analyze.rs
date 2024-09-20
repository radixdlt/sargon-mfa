#![allow(unused)]

use crate::prelude::*;

/// Derivation of many keys differs between the following operations,
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
/// All operations ends with adding "ProbablyFree" FactorInstances to Cache.
///
/// "ProbablyFree" refers to keys in `KeySpace::Securified`, a Factor which
/// might have been used in the past for an AccessController, but might not
/// be active anymore - currently [2024-09-20] it does not look like Gateway
/// will index **past** public key hashes.
///
/// If cache is empty/non-existent/inaccessible, we derive many keys for many
/// derivation index ranges to fill the cache and when possible/applicable we
/// match against entities in Profile to re-discovered VECIs and unsecurified
/// entities, also when possible/applicable we analyze if FactorInstances are
/// taken or free by using Gateway.
///
/// OPERATIONS:
///
/// * [FSA] FactorSource AdditionAddition
/// Does NOT add any new entities, for that, use `MARS`.`
///     CHARACTERISTICS:
///     - Derivation Indices Range Start: `0`
///     - Derivation Indices Range Size: Many, different for each factor source kind and key kind
///     - Cache Available: NO, not for this FactorSource
///     - Profile Available: YES
///     - FactorSource Addition: `Single`
///     - Entities Addition: NO
///     - `VECI` Addition: YES - if new found
///     - Gateway Required: NO, not used.
///
///
/// * [OARS] Onboarding Account Recovery Scan - like `FSA` but for many FactorSources and
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
///
///
/// * [MARS] Manual Account Recovery Scan
///     CHARACTERISTICS:
///     - Derivation Indices Range Start: `0`
///     - Derivation Indices Range Size: Many, different for each factor source kind and key kind
///     - Cache Available: NO
///     - Profile Available: NO
///     - FactorSource Addition: `Many`
///     - Entities Addition: YES - securified and unsecurified
///     - `VECI` Addition: YES
///     - Gateway Required: YES
///
///
/// * [NUVEC] New Unsecurified Virtual Entity Creation
///     CHARACTERISTICS:
///     - Derivation Indices Range Start: Next Free According to Cache if available, else start at Zero to fill cache and filter out take (by matching against entities in Profile)
///     - Derivation Indices Range Size: Single (OR Many if Cache needs to be filled)
///     - Cache Available: YES - if not deleted or inaccessible
///     - Profile Available: YES
///     - FactorSource Addition: NO
///     - Entities Addition: YES - unsecurified
///     - `VECI` Addition: NO - cannot happen, Cache should never contain keys which have been used to create Account/Identity addresses.
///     - Gateway Required: NO (but beneficial to use it if host is online to
///         analyze if FactorInstance are free.)
///
///
/// * [MOFIDs] `MatrixOfFactorSources` -> `Vec<MatrixOfFactorInstances>` Derivation (Securifying Entities)
///     CHARACTERISTICS:
///     - Derivation Indices Range Start: Next Free According to Cache if available, else start at Zero to fill cache and filter out take (by matching against entities in Profile)
///     - Derivation Indices Range Size: Single PER FactorSource PER Entity being securified (OR Many if Cache needs to be filled)
///     - Cache Available: YES - if not deleted or inaccessible
///     - Profile Available: YES
///     - FactorSource Addition: NO
///     - Entities Addition: NO
///     - `VECI` Addition: YES - if cache was empty, we might have re-discovered a VECI.
///     - Gateway Required: NO (but beneficial to use it if host is online to
///         analyze if FactorInstance are free.)
///
///
pub async fn derive_and_analyze(input: DeriveAndAnalyzeInput) -> Result<DerivationAndAnalysis> {
    let factor_instances = input.load_cached_or_derive_new_instances().await?;

    let analysis = input.analyze(factor_instances).await?;

    // To be fed into cache, NOT done by this function.
    let probably_free_instances = analysis.probably_free_instances;

    // Recovered Securified or Unsecurified entities, unrecovered Securified
    // Entities and virtual entity creating instances.
    let known_taken = analysis.known_taken;

    Ok(DerivationAndAnalysis::new(
        probably_free_instances,
        known_taken,
        input.old_factor_sources(),
        input.new_factor_sources(),
    ))
}
