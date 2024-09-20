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
async fn derive_and_analyze() -> Result<DerivationAndAnalysis> {
    todo!()
}

async fn _account_recovery_scan() -> Result<DerivationAndAnalysisAccountRecoveryScan> {
    let analysis = derive_and_analyze().await?;
    DerivationAndAnalysisAccountRecoveryScan::try_from(analysis)
}

pub async fn account_recovery_scan() -> Result<AccountRecoveryScanOutcome> {
    let analysis = _account_recovery_scan().await?;
    Ok(AccountRecoveryScanOutcome::from(analysis))
}
