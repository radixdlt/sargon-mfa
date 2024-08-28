use crate::prelude::*;

impl SignaturesCollector {
    /// Used by our tests. But Sargon will typically wanna use `SignaturesCollector::new` and passing
    /// it a
    pub fn new_test_with(
        finish_early_strategy: SigningFinishEarlyStrategy,
        all_factor_sources_in_profile: IndexSet<HDFactorSource>,
        transactions: IndexSet<TXToSign>,
        interactors: Arc<dyn SignatureCollectingInteractors>,
    ) -> Self {
        sensible_env_logger::safe_init!();
        Self::with(
            finish_early_strategy,
            all_factor_sources_in_profile,
            transactions,
            interactors,
        )
    }
    pub fn new_test(
        finish_early_strategy: SigningFinishEarlyStrategy,
        all_factor_sources_in_profile: impl IntoIterator<Item = HDFactorSource>,
        transactions: impl IntoIterator<Item = TXToSign>,
        simulated_user: SimulatedUser,
    ) -> Self {
        Self::new_test_with(
            finish_early_strategy,
            all_factor_sources_in_profile.into_iter().collect(),
            transactions.into_iter().collect(),
            Arc::new(TestSignatureCollectingInteractors::new(simulated_user)),
        )
    }

    pub fn test_prudent_with_factors_and_finish_early(
        finish_early_strategy: SigningFinishEarlyStrategy,
        all_factor_sources_in_profile: impl IntoIterator<Item = HDFactorSource>,
        transactions: impl IntoIterator<Item = TXToSign>,
    ) -> Self {
        Self::new_test(
            finish_early_strategy,
            all_factor_sources_in_profile,
            transactions,
            SimulatedUser::prudent_no_fail(),
        )
    }

    pub fn test_prudent_with_finish_early(
        finish_early_strategy: SigningFinishEarlyStrategy,
        transactions: impl IntoIterator<Item = TXToSign>,
    ) -> Self {
        Self::test_prudent_with_factors_and_finish_early(
            finish_early_strategy,
            HDFactorSource::all(),
            transactions,
        )
    }

    pub fn test_prudent(transactions: impl IntoIterator<Item = TXToSign>) -> Self {
        Self::test_prudent_with_finish_early(SigningFinishEarlyStrategy::default(), transactions)
    }

    pub fn test_prudent_with_failures(
        transactions: impl IntoIterator<Item = TXToSign>,
        simulated_failures: SimulatedFailures,
    ) -> Self {
        Self::new_test(
            SigningFinishEarlyStrategy::default(),
            HDFactorSource::all(),
            transactions,
            SimulatedUser::prudent_with_failures(simulated_failures),
        )
    }

    pub fn test_lazy_sign_minimum_no_failures_with_factors(
        all_factor_sources_in_profile: impl IntoIterator<Item = HDFactorSource>,
        transactions: impl IntoIterator<Item = TXToSign>,
    ) -> Self {
        Self::new_test(
            SigningFinishEarlyStrategy::default(),
            all_factor_sources_in_profile,
            transactions,
            SimulatedUser::lazy_sign_minimum([]),
        )
    }

    pub fn test_lazy_sign_minimum_no_failures(
        transactions: impl IntoIterator<Item = TXToSign>,
    ) -> Self {
        Self::test_lazy_sign_minimum_no_failures_with_factors(HDFactorSource::all(), transactions)
    }

    pub fn test_lazy_always_skip_with_factors(
        all_factor_sources_in_profile: impl IntoIterator<Item = HDFactorSource>,
        transactions: impl IntoIterator<Item = TXToSign>,
    ) -> Self {
        Self::new_test(
            SigningFinishEarlyStrategy::default(),
            all_factor_sources_in_profile,
            transactions,
            SimulatedUser::lazy_always_skip_no_fail(),
        )
    }

    pub fn test_lazy_always_skip(transactions: impl IntoIterator<Item = TXToSign>) -> Self {
        Self::test_lazy_always_skip_with_factors(HDFactorSource::all(), transactions)
    }
}
