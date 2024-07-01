use crate::prelude::*;

/// A collection of factor sources to use to sign, transactions with multiple keys
/// (derivations paths).
pub struct ParallelBatchSigningRequest {
    /// Per factor source, a set of transactions to sign, with
    /// multiple derivations paths.
    pub per_factor_source: IndexMap<FactorSourceID, BatchTXBatchKeySigningRequest>,
    /// A collection of transactions which would be invalid if the user skips
    /// signing with this factor source.
    pub invalid_transactions_if_skipped: IndexSet<InvalidTransactionIfSkipped>,
}
impl ParallelBatchSigningRequest {
    pub fn new(
        per_factor_source: IndexMap<FactorSourceID, BatchTXBatchKeySigningRequest>,
        invalid_transactions_if_skipped: IndexSet<InvalidTransactionIfSkipped>,
    ) -> Self {
        Self {
            per_factor_source,
            invalid_transactions_if_skipped,
        }
    }
    pub fn factor_source_ids(&self) -> IndexSet<FactorSourceID> {
        self.per_factor_source.keys().into_iter().cloned().collect()
    }
}

#[async_trait::async_trait]
pub trait IsUseFactorSourcesDriver {
    async fn did_fail_ask_if_retry(&self, factor_source_ids: IndexSet<FactorSourceID>) -> bool;
}

#[async_trait::async_trait]
pub trait IsTestUseFactorSourcesDriver: IsUseFactorSourcesDriver + Sync {
    fn simulated_user(&self) -> SimulatedUser;

    async fn should_simulate_failure(&self, factor_source_ids: IndexSet<FactorSourceID>) -> bool {
        self.simulated_user()
            .simulate_failure_if_needed(factor_source_ids)
    }
}

#[async_trait::async_trait]
impl<T> IsUseFactorSourcesDriver for T
where
    T: IsTestUseFactorSourcesDriver,
{
    async fn did_fail_ask_if_retry(&self, factor_source_ids: IndexSet<FactorSourceID>) -> bool {
        self.simulated_user().retry_if_needed(factor_source_ids)
    }
}

/// A driver for a factor source kind which supports *Batch* usage of
/// multiple factor sources in parallel.
///
/// Most FactorSourceKinds does in fact NOT support parallel usage,
/// e.g. signing using multiple factors sources at once, but some do,
/// typically the DeviceFactorSource does, i.e. we can load multiple
/// mnemonics from secure storage in one go and sign with all of them
/// "in parallel".
///
/// This is a bit of a misnomer, as we don't actually use them in parallel,
/// but rather we iterate through all mnemonics and derive public keys/
/// or sign a payload with each of them in sequence
///
/// The user does not have the ability to SKIP a certain factor source,
/// instead either ALL factor sources are used to sign the transactions
/// or none.
///
/// Example of a Parallel Batch Signing Driver is that for DeviceFactorSource.
#[async_trait::async_trait]
pub trait ParallelBatchUseFactorSourcesDriver: IsUseFactorSourcesDriver {
    async fn sign(
        &self,
        request: ParallelBatchSigningRequest,
    ) -> Result<SignWithFactorSourceOrSourcesOutcome<BatchSigningResponse>>;
}
