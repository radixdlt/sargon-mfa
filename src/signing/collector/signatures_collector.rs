use crate::prelude::*;

use super::{
    signatures_collector_dependencies::*, signatures_collector_preprocessor::*,
    signatures_collector_state::*,
};

/// A coordinator which gathers signatures from several factor sources of different
/// kinds, in increasing friction order, for many transactions and for
/// potentially multiple entities and for many factor instances (derivation paths)
/// for each transaction.
///
/// By increasing friction order we mean, the quickest and easiest to use FactorSourceKind
/// is last; namely `DeviceFactorSource`, and the most tedious FactorSourceKind is
/// first; namely `LedgerFactorSource`, which user might also lack access to.
pub struct SignaturesCollector {
    /// Stateless immutable values used by the collector to gather signatures
    /// from factor sources.
    dependencies: SignaturesCollectorDependencies,

    /// Mutable internal state of the collector which builds up the list
    /// of signatures from each used factor source.
    state: RefCell<SignaturesCollectorState>,
}

impl SignaturesCollector {
    /// Used by our tests. But Sargon will typically wanna use `SignaturesCollector::new` and passing
    /// it a
    pub(crate) fn with(
        finish_early_strategy: SigningFinishEarlyStrategy,
        all_factor_sources_in_profile: IndexSet<HDFactorSource>,
        transactions: IndexSet<TXToSign>,
        interactors: Arc<dyn SignatureCollectingInteractors>,
    ) -> Self {
        debug!("Init SignaturesCollector");
        let preprocessor = SignaturesCollectorPreprocessor::new(transactions);
        let (petitions, factors) = preprocessor.preprocess(all_factor_sources_in_profile);

        let dependencies =
            SignaturesCollectorDependencies::new(finish_early_strategy, interactors, factors);
        let state = SignaturesCollectorState::new(petitions);

        Self {
            dependencies,
            state: RefCell::new(state),
        }
    }

    pub fn with_signers_extraction<F>(
        finish_early_strategy: SigningFinishEarlyStrategy,
        all_factor_sources_in_profile: IndexSet<HDFactorSource>,
        transactions: IndexSet<TransactionIntent>,
        interactors: Arc<dyn SignatureCollectingInteractors>,
        extract_signers: F,
    ) -> Result<Self>
    where
        F: Fn(TransactionIntent) -> Result<TXToSign>,
    {
        let transactions = transactions
            .into_iter()
            .map(extract_signers)
            .collect::<Result<IndexSet<TXToSign>>>()?;

        let collector = Self::with(
            finish_early_strategy,
            all_factor_sources_in_profile,
            transactions,
            interactors,
        );

        Ok(collector)
    }

    pub fn new(
        finish_early_strategy: SigningFinishEarlyStrategy,
        transactions: IndexSet<TransactionIntent>,
        interactors: Arc<dyn SignatureCollectingInteractors>,
        profile: &Profile,
    ) -> Result<Self> {
        Self::with_signers_extraction(
            finish_early_strategy,
            profile.factor_sources.clone(),
            transactions,
            interactors,
            |i| TXToSign::extracting_from_intent_and_profile(&i, profile),
        )
    }
}

// === PUBLIC ===
impl SignaturesCollector {
    pub async fn collect_signatures(self) -> SignaturesOutcome {
        _ = self
            .sign_with_factors() // in decreasing "friction order"
            .await
            .inspect_err(|e| error!("Failed to use factor sources: {:#?}", e));

        self.outcome()
    }
}

use SignaturesCollectingContinuation::*;

// === PRIVATE ===
impl SignaturesCollector {
    /// Returning `Continue` means that we should continue collecting signatures.
    ///
    /// Returning `FinishEarly` if it is meaningless to continue collecting signatures,
    /// either since all transactions are valid and this collector is configured
    /// to finish early in that case, or if some transaction is invalid and this
    /// collector is configured to finish early in that case.
    ///
    /// N.B. this method does not concern itself with how many or which
    /// factor sources are left to sign with, that is handled by the main loop,
    /// i.e. this might return `Continue` even though there is not factor sources
    /// left to sign with.
    fn continuation(&self) -> SignaturesCollectingContinuation {
        let finish_early_strategy = self.dependencies.finish_early_strategy;
        let when_all_transactions_are_valid =
            finish_early_strategy.when_all_transactions_are_valid.0;
        let when_some_transaction_is_invalid =
            finish_early_strategy.when_some_transaction_is_invalid.0;

        let petitions_status = self.state.borrow().petitions.borrow().status();

        if petitions_status.are_all_valid() {
            if when_all_transactions_are_valid == FinishEarly {
                info!("All valid && should finish early => finish early");
                return FinishEarly;
            } else {
                debug!(
                    "All valid, BUT the collector is configured to NOT finish early => Continue"
                );
            }
        } else if petitions_status.is_some_invalid() {
            if when_some_transaction_is_invalid == FinishEarly {
                info!("Some invalid && should finish early => finish early");
                return FinishEarly;
            } else {
                debug!("Some transactions invalid, BUT the collector is configured to NOT finish early in case of failures => Continue");
            }
        }

        Continue
    }

    fn neglected_factors_due_to_irrelevant(
        &self,
        factor_sources_of_kind: &FactorSourcesOfKind,
    ) -> bool {
        false
    }

    async fn sign_with_factors_of_kind(&self, factor_sources_of_kind: &FactorSourcesOfKind) {
        info!(
            "Use(?) #{:?} factors of kind: {:?}",
            &factor_sources_of_kind.factor_sources().len(),
            &factor_sources_of_kind.kind
        );

        let interactor = self
            .dependencies
            .interactors
            .interactor_for(factor_sources_of_kind.kind);
        let factor_sources = factor_sources_of_kind.factor_sources();
        match interactor {
            // Parallel Interactor: Many Factor Sources at once
            SigningInteractor::Parallel(interactor) => {
                // Prepare the request for the interactor
                debug!("Creating parallel request for interactor");
                let request = self.request_for_parallel_interactor(
                    factor_sources
                        .into_iter()
                        .map(|f| f.factor_source_id())
                        .collect(),
                );
                if !request.invalid_transactions_if_neglected.is_empty() {
                    info!(
                        "If factors {:?} are neglected, invalid TXs: {:?}",
                        request.per_factor_source.keys(),
                        request.invalid_transactions_if_neglected
                    )
                }
                debug!("Dispatching parallel request to interactor: {:?}", request);
                let response = interactor.sign(request).await;
                debug!("Got response from parallel interactor: {:?}", response);
                self.process_batch_response(response);
            }

            // Serial Interactor: One Factor Sources at a time
            // After each factor source we pass the result to the collector
            // updating its internal state so that we state about being able
            // to skip the next factor source or not.
            SigningInteractor::Serial(interactor) => {
                for factor_source in factor_sources {
                    // Prepare the request for the interactor
                    debug!("Creating serial request for interactor");
                    let request =
                        self.request_for_serial_interactor(&factor_source.factor_source_id());

                    if !request.invalid_transactions_if_neglected.is_empty() {
                        info!(
                            "If factor {:?} are neglected, invalid TXs: {:?}",
                            request.input.factor_source_id,
                            request.invalid_transactions_if_neglected
                        )
                    }

                    debug!("Dispatching serial request to interactor: {:?}", request);
                    // Produce the results from the interactor
                    let response = interactor.sign(request).await;
                    debug!("Got response from serial interactor: {:?}", response);

                    // Report the results back to the collector
                    self.process_batch_response(response);

                    if self.continuation() == FinishEarly {
                        break;
                    }
                }
            }
        }
    }

    /// In decreasing "friction order"
    async fn sign_with_factors(&self) -> Result<()> {
        let factors_of_kind = self.dependencies.factors_of_kind.clone();
        for factor_sources_of_kind in factors_of_kind.iter() {
            if self.continuation() == FinishEarly {
                break;
            }
            if self.neglected_factors_due_to_irrelevant(factor_sources_of_kind) {
                continue;
            }
            self.sign_with_factors_of_kind(factor_sources_of_kind).await;
        }
        info!("FINISHED WITH ALL FACTORS");
        Ok(())
    }

    fn input_for_interactor(
        &self,
        factor_source_id: &FactorSourceIDFromHash,
    ) -> BatchTXBatchKeySigningRequest {
        self.state
            .borrow()
            .petitions
            .borrow()
            .input_for_interactor(factor_source_id)
    }

    fn request_for_serial_interactor(
        &self,
        factor_source_id: &FactorSourceIDFromHash,
    ) -> SerialBatchSigningRequest {
        let batch_signing_request = self.input_for_interactor(factor_source_id);

        SerialBatchSigningRequest::new(
            batch_signing_request,
            self.invalid_transactions_if_neglected_factor_sources(IndexSet::from_iter([
                *factor_source_id,
            ]))
            .into_iter()
            .collect_vec(),
        )
    }

    fn request_for_parallel_interactor(
        &self,
        factor_source_ids: IndexSet<FactorSourceIDFromHash>,
    ) -> ParallelBatchSigningRequest {
        let per_factor_source = factor_source_ids
            .clone()
            .iter()
            .map(|fid| (*fid, self.input_for_interactor(fid)))
            .collect::<IndexMap<FactorSourceIDFromHash, BatchTXBatchKeySigningRequest>>();

        let invalid_transactions_if_neglected =
            self.invalid_transactions_if_neglected_factor_sources(factor_source_ids);

        info!(
            "Invalid if neglected: {:?}",
            invalid_transactions_if_neglected
        );

        // Prepare the request for the interactor
        ParallelBatchSigningRequest::new(per_factor_source, invalid_transactions_if_neglected)
    }

    fn invalid_transactions_if_neglected_factor_sources(
        &self,
        factor_source_ids: IndexSet<FactorSourceIDFromHash>,
    ) -> IndexSet<InvalidTransactionIfNeglected> {
        self.state
            .borrow()
            .petitions
            .borrow()
            .invalid_transactions_if_neglected_factors(factor_source_ids)
    }

    fn process_batch_response(&self, response: SignWithFactorsOutcome) {
        let state = self.state.borrow_mut();
        let petitions = state.petitions.borrow_mut();
        petitions.process_batch_response(response)
    }

    fn outcome(self) -> SignaturesOutcome {
        let expected_number_of_transactions;
        {
            let state = self.state.borrow_mut();
            let petitions = state.petitions.borrow_mut();
            expected_number_of_transactions = petitions.txid_to_petition.borrow().len();
        }
        let outcome = self.state.into_inner().petitions.into_inner().outcome();
        assert_eq!(
            outcome.failed_transactions().len() + outcome.successful_transactions().len(),
            expected_number_of_transactions
        );
        if !outcome.successful() {
            warn!(
                "Failed to sign, invalid tx: {:?}, petition",
                outcome.failed_transactions()
            )
        }
        outcome
    }
}

#[cfg(test)]
mod tests {

    use std::iter;

    use super::*;

    impl SignaturesCollector {
        /// Used by tests
        pub(crate) fn petitions(self) -> Petitions {
            self.state.into_inner().petitions.into_inner()
        }
    }

    #[test]
    fn invalid_profile_unknown_account() {
        let res = SignaturesCollector::new(
            SigningFinishEarlyStrategy::default(),
            IndexSet::from_iter([TransactionIntent::new([Account::a0().entity_address()], [])]),
            Arc::new(TestSignatureCollectingInteractors::new(
                SimulatedUser::prudent_no_fail(),
            )),
            &Profile::new(IndexSet::new(), [], []),
        );
        assert!(matches!(res, Err(CommonError::UnknownAccount)));
    }

    #[test]
    fn invalid_profile_unknown_persona() {
        let res = SignaturesCollector::new(
            SigningFinishEarlyStrategy::default(),
            IndexSet::from_iter([TransactionIntent::new([], [Persona::p0().entity_address()])]),
            Arc::new(TestSignatureCollectingInteractors::new(
                SimulatedUser::prudent_no_fail(),
            )),
            &Profile::new(IndexSet::new(), [], []),
        );
        assert!(matches!(res, Err(CommonError::UnknownPersona)));
    }

    #[actix_rt::test]
    async fn valid_profile() {
        let factors_sources = HDFactorSource::all();
        let persona = Persona::p0();
        let collector = SignaturesCollector::new(
            SigningFinishEarlyStrategy::default(),
            IndexSet::from_iter([TransactionIntent::new([], [persona.entity_address()])]),
            Arc::new(TestSignatureCollectingInteractors::new(
                SimulatedUser::prudent_no_fail(),
            )),
            &Profile::new(factors_sources, [], [&persona]),
        )
        .unwrap();
        let outcome = collector.collect_signatures().await;
        assert!(outcome.successful())
    }

    #[actix_rt::test]
    async fn continues_even_with_failed_tx_when_configured_to() {
        let factor_sources = &HDFactorSource::all();
        let a0 = &Account::a0();
        let a1 = &Account::a1();

        let t0 = TransactionIntent::address_of([a1], []);
        let t1 = TransactionIntent::address_of([a0], []);

        let profile = Profile::new(factor_sources.clone(), [a0, a1], []);

        let collector = SignaturesCollector::new(
            SigningFinishEarlyStrategy::new(
                WhenAllTransactionsAreValid(FinishEarly),
                WhenSomeTransactionIsInvalid(Continue),
            ),
            IndexSet::<TransactionIntent>::from_iter([t0.clone(), t1.clone()]),
            Arc::new(TestSignatureCollectingInteractors::new(
                SimulatedUser::prudent_with_failures(SimulatedFailures::with_simulated_failures([
                    FactorSourceIDFromHash::fs1(),
                ])),
            )),
            &profile,
        )
        .unwrap();

        let outcome = collector.collect_signatures().await;
        assert!(!outcome.successful());
        assert_eq!(outcome.failed_transactions().len(), 1);
        assert_eq!(outcome.successful_transactions().len(), 1);
    }

    #[actix_rt::test]
    async fn continues_even_when_all_valid_if_configured_to() {
        sensible_env_logger::safe_init!();
        let test = async move |when_all_valid: WhenAllTransactionsAreValid,
                               expected_sig_count: usize| {
            let factor_sources = &HDFactorSource::all();
            let a5 = &Account::a5();

            let t0 = TransactionIntent::address_of([a5], []);

            let profile = Profile::new(factor_sources.clone(), [a5], []);

            let collector = SignaturesCollector::new(
                SigningFinishEarlyStrategy::new(
                    when_all_valid,
                    WhenSomeTransactionIsInvalid::default(),
                ),
                IndexSet::<TransactionIntent>::from_iter([t0.clone()]),
                Arc::new(TestSignatureCollectingInteractors::new(
                    SimulatedUser::prudent_no_fail(),
                )),
                &profile,
            )
            .unwrap();

            let outcome = collector.collect_signatures().await;
            assert!(outcome.successful());
            assert_eq!(
                outcome.signatures_of_successful_transactions().len(),
                expected_sig_count
            );
        };

        test(WhenAllTransactionsAreValid(FinishEarly), 1).await;
        test(WhenAllTransactionsAreValid(Continue), 2).await;
    }

    #[test]
    fn test_profile() {
        let factor_sources = &HDFactorSource::all();
        let a0 = &Account::a0();
        let a1 = &Account::a1();
        let a2 = &Account::a2();
        let a6 = &Account::a6();

        let p0 = &Persona::p0();
        let p1 = &Persona::p1();
        let p2 = &Persona::p2();
        let p6 = &Persona::p6();

        let t0 = TransactionIntent::address_of([a0, a1], [p0, p1]);
        let t1 = TransactionIntent::address_of([a0, a1, a2], []);
        let t2 = TransactionIntent::address_of([], [p0, p1, p2]);
        let t3 = TransactionIntent::address_of([a6], [p6]);

        let profile = Profile::new(factor_sources.clone(), [a0, a1, a2, a6], [p0, p1, p2, p6]);

        let collector = SignaturesCollector::new(
            SigningFinishEarlyStrategy::default(),
            IndexSet::<TransactionIntent>::from_iter([
                t0.clone(),
                t1.clone(),
                t2.clone(),
                t3.clone(),
            ]),
            Arc::new(TestSignatureCollectingInteractors::new(
                SimulatedUser::prudent_no_fail(),
            )),
            &profile,
        )
        .unwrap();

        let petitions = collector.petitions();

        assert_eq!(petitions.txid_to_petition.borrow().len(), 4);

        {
            let petitions_ref = petitions.txid_to_petition.borrow();
            let petition = petitions_ref.get(&t3.intent_hash).unwrap();
            let for_entities = petition.for_entities.borrow().clone();
            let pet6 = for_entities.get(&a6.address()).unwrap();

            let paths6 = pet6
                .all_factor_instances()
                .iter()
                .map(|f| f.factor_instance().derivation_path())
                .collect_vec();

            pretty_assertions::assert_eq!(
                paths6,
                iter::repeat_n(
                    DerivationPath::new(
                        NetworkID::Mainnet,
                        CAP26EntityKind::Account,
                        CAP26KeyKind::T9n,
                        HDPathComponent::non_hardened(6)
                    ),
                    5
                )
                .collect_vec()
            );
        }

        let assert_petition = |t: &TransactionIntent,
                               threshold_factors: HashMap<
            AddressOfAccountOrPersona,
            HashSet<FactorSourceIDFromHash>,
        >,
                               override_factors: HashMap<
            AddressOfAccountOrPersona,
            HashSet<FactorSourceIDFromHash>,
        >| {
            let petitions_ref = petitions.txid_to_petition.borrow();
            let petition = petitions_ref.get(&t.intent_hash).unwrap();
            assert_eq!(petition.intent_hash, t.intent_hash);

            let mut addresses = threshold_factors.keys().collect::<HashSet<_>>();
            addresses.extend(override_factors.keys().collect::<HashSet<_>>());

            assert_eq!(
                petition
                    .for_entities
                    .borrow()
                    .keys()
                    .collect::<HashSet<_>>(),
                addresses
            );

            assert!(petition
                .for_entities
                .borrow()
                .iter()
                .all(|(a, p)| { p.entity == *a }));

            assert!(petition
                .for_entities
                .borrow()
                .iter()
                .all(|(_, p)| { p.intent_hash == t.intent_hash }));

            for (k, v) in petition.for_entities.borrow().iter() {
                let threshold = threshold_factors.get(k);
                if let Some(actual_threshold) = &v.threshold_factors {
                    let threshold = threshold.unwrap().clone();
                    assert_eq!(
                        actual_threshold
                            .borrow()
                            .factor_instances()
                            .into_iter()
                            .map(|f| f.factor_source_id)
                            .collect::<HashSet<_>>(),
                        threshold
                    );
                } else {
                    assert!(threshold.is_none());
                }

                let override_ = override_factors.get(k);
                if let Some(actual_override) = &v.override_factors {
                    let override_ = override_.unwrap().clone();
                    assert_eq!(
                        actual_override
                            .borrow()
                            .factor_instances()
                            .into_iter()
                            .map(|f| f.factor_source_id)
                            .collect::<HashSet<_>>(),
                        override_
                    );
                } else {
                    assert!(override_.is_none());
                }
            }
        };
        assert_petition(
            &t0,
            HashMap::from_iter([
                (
                    a0.address(),
                    HashSet::from_iter([FactorSourceIDFromHash::fs0()]),
                ),
                (
                    a1.address(),
                    HashSet::from_iter([FactorSourceIDFromHash::fs1()]),
                ),
                (
                    p0.address(),
                    HashSet::from_iter([FactorSourceIDFromHash::fs0()]),
                ),
                (
                    p1.address(),
                    HashSet::from_iter([FactorSourceIDFromHash::fs1()]),
                ),
            ]),
            HashMap::new(),
        );

        assert_petition(
            &t1,
            HashMap::from_iter([
                (
                    a0.address(),
                    HashSet::from_iter([FactorSourceIDFromHash::fs0()]),
                ),
                (
                    a1.address(),
                    HashSet::from_iter([FactorSourceIDFromHash::fs1()]),
                ),
                (
                    a2.address(),
                    HashSet::from_iter([FactorSourceIDFromHash::fs0()]),
                ),
            ]),
            HashMap::new(),
        );

        assert_petition(
            &t2,
            HashMap::from_iter([
                (
                    p0.address(),
                    HashSet::from_iter([FactorSourceIDFromHash::fs0()]),
                ),
                (
                    p1.address(),
                    HashSet::from_iter([FactorSourceIDFromHash::fs1()]),
                ),
                (
                    p2.address(),
                    HashSet::from_iter([FactorSourceIDFromHash::fs0()]),
                ),
            ]),
            HashMap::new(),
        );

        assert_petition(
            &t3,
            HashMap::from_iter([
                (
                    a6.address(),
                    HashSet::from_iter([
                        FactorSourceIDFromHash::fs0(),
                        FactorSourceIDFromHash::fs3(),
                        FactorSourceIDFromHash::fs5(),
                    ]),
                ),
                (
                    p6.address(),
                    HashSet::from_iter([
                        FactorSourceIDFromHash::fs0(),
                        FactorSourceIDFromHash::fs3(),
                        FactorSourceIDFromHash::fs5(),
                    ]),
                ),
            ]),
            HashMap::from_iter([
                (
                    a6.address(),
                    HashSet::from_iter([
                        FactorSourceIDFromHash::fs1(),
                        FactorSourceIDFromHash::fs4(),
                    ]),
                ),
                (
                    p6.address(),
                    HashSet::from_iter([
                        FactorSourceIDFromHash::fs1(),
                        FactorSourceIDFromHash::fs4(),
                    ]),
                ),
            ]),
        );
    }
}
