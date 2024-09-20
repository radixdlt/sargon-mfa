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
///     - Cache Available: NO
///     - Profile Available: NO
///     - FactorSource Addition: `Many`
///     - Entities Addition: YES - securified and unsecurified
///     - `VECI` Addition: YES
///     - Gateway Required: YES
/// * [VECID] VECI Derivation
///     CHARACTERISTICS:
///     - Cache Available: YES - if not deleted or inaccessible
///     - Profile Available: YES
///     - FactorSource Addition: NO
///     - Entities Addition: YES - unsecurified
///     - `VECI` Addition: NO
///     - Gateway Required: NO (but beneficial to use it if host is online to
///         analyze if FactorInstance are free.)
/// * [MOFID] MatrixOfFactor Instances Derivation
///     CHARACTERISTICS:
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DerivationAndAnalysisAccountRecoveryScan {}

impl TryFrom<DerivationAndAnalysis> for DerivationAndAnalysisAccountRecoveryScan {
    type Error = CommonError;

    fn try_from(value: DerivationAndAnalysis) -> Result<Self> {
        warn!("ignored: {:?}", value);
        todo!()
    }
}

async fn _account_recovery_scan() -> Result<DerivationAndAnalysisAccountRecoveryScan> {
    let analysis = derive_and_analyze().await?;
    DerivationAndAnalysisAccountRecoveryScan::try_from(analysis)
}

#[derive(Debug)]
pub struct PreDerivedKeysCache;

#[derive(Debug)]
pub struct AccountRecoveryScanOutcome {
    pub cache: PreDerivedKeysCache,
    pub profile: Profile,
}
impl From<DerivationAndAnalysisAccountRecoveryScan> for AccountRecoveryScanOutcome {
    fn from(value: DerivationAndAnalysisAccountRecoveryScan) -> Self {
        warn!("ignored: {:?}", value);
        todo!()
    }
}

pub async fn account_recovery_scan() -> Result<AccountRecoveryScanOutcome> {
    let analysis = _account_recovery_scan().await?;
    Ok(AccountRecoveryScanOutcome::from(analysis))
}
