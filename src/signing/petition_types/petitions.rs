#![allow(clippy::non_canonical_partial_ord_impl)]

use crate::prelude::*;

#[derive(derive_more::Debug, PartialEq, Eq)]
#[debug("{}", self.debug_str())]
pub(crate) struct Petitions {
    /// Lookup from factor to TXID.
    ///
    ///
    /// The same HDFactorSource might be required by many payloads
    /// and per payload might be required by many entities, e.g. transactions
    /// `t0` and `t1`, where
    /// `t0` is signed by accounts: A and B
    /// `t1` is signed by accounts: A, C and D,
    ///
    /// Where A, B, C and D, all use the factor source, e.g. some arculus
    /// card which the user has setup as a factor (source) for all these accounts.
    pub factor_source_to_intent_hash: HashMap<FactorSourceIDFromHash, IndexSet<IntentHash>>,

    /// Lookup from TXID to signatures builders, sorted according to the order of
    /// transactions passed to the SignaturesBuilder.
    pub txid_to_petition: RefCell<IndexMap<IntentHash, PetitionTransaction>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PetitionsStatus {
    InProgressNoneInvalid,
    AllAreValid,
    SomeIsInvalid,
}
impl PetitionsStatus {
    // pub fn are_all_done(&self) -> bool {
    //     matches!(self, Self::Done { .. })
    // }
    pub fn are_all_valid(&self) -> bool {
        matches!(self, Self::AllAreValid)
    }

    pub fn is_some_invalid(&self) -> bool {
        matches!(self, Self::SomeIsInvalid)
    }
}

impl Petitions {
    pub(crate) fn new(
        factor_source_to_intent_hash: HashMap<FactorSourceIDFromHash, IndexSet<IntentHash>>,
        txid_to_petition: IndexMap<IntentHash, PetitionTransaction>,
    ) -> Self {
        Self {
            factor_source_to_intent_hash,
            txid_to_petition: RefCell::new(txid_to_petition),
        }
    }

    pub fn outcome(self) -> SignaturesOutcome {
        let txid_to_petition = self.txid_to_petition.into_inner();
        let mut failed_transactions = MaybeSignedTransactions::empty();
        let mut successful_transactions = MaybeSignedTransactions::empty();
        let mut neglected_factor_sources = IndexSet::<NeglectedFactor>::new();
        for (intent_hash, petition_of_transaction) in txid_to_petition.into_iter() {
            let outcome = petition_of_transaction.outcome();
            let signatures = outcome.signatures;

            if outcome.transaction_valid {
                successful_transactions.add_signatures(intent_hash, signatures);
            } else {
                failed_transactions.add_signatures(intent_hash, signatures);
            }
            neglected_factor_sources.extend(outcome.neglected_factors)
        }

        SignaturesOutcome::new(
            successful_transactions,
            failed_transactions,
            neglected_factor_sources,
        )
    }

    pub fn status(&self) -> PetitionsStatus {
        let statuses = self
            .txid_to_petition
            .borrow()
            .iter()
            .flat_map(|(_, petition)| {
                petition
                    .for_entities
                    .borrow()
                    .iter()
                    .map(|(_, petition)| petition.status())
                    .collect_vec()
            })
            .collect::<Vec<PetitionFactorsStatus>>();

        let are_all_valid = statuses.iter().all(|s| {
            matches!(
                s,
                PetitionFactorsStatus::Finished(PetitionFactorsStatusFinished::Success)
            )
        });
        if are_all_valid {
            return PetitionsStatus::AllAreValid;
        }

        let is_some_invalid = statuses.iter().any(|s| {
            matches!(
                s,
                PetitionFactorsStatus::Finished(PetitionFactorsStatusFinished::Fail)
            )
        });
        if is_some_invalid {
            return PetitionsStatus::SomeIsInvalid;
        }
        PetitionsStatus::InProgressNoneInvalid
    }

    pub fn invalid_transactions_if_neglected_factors(
        &self,
        factor_source_ids: IndexSet<FactorSourceIDFromHash>,
    ) -> IndexSet<InvalidTransactionIfNeglected> {
        factor_source_ids
            .clone()
            .iter()
            .flat_map(|f| {
                self.factor_source_to_intent_hash
                    .get(f)
                    .unwrap()
                    .iter()
                    .flat_map(|intent_hash| {
                        let binding = self.txid_to_petition.borrow();
                        let value = binding.get(intent_hash).unwrap();
                        value.invalid_transactions_if_neglected_factors(factor_source_ids.clone())
                    })
            })
            .collect::<IndexSet<_>>()
    }

    pub(crate) fn should_neglect_factors_due_to_irrelevant(
        &self,
        factor_sources_of_kind: &FactorSourcesOfKind,
    ) -> bool {
        let factor_source_ids = factor_sources_of_kind
            .factor_sources()
            .iter()
            .map(|f| f.factor_source_id())
            .collect::<IndexSet<_>>();
        factor_sources_of_kind
            .factor_sources()
            .iter()
            .map(|f| f.factor_source_id())
            .all(|f| {
                self.factor_source_to_intent_hash
                    .get(&f)
                    .unwrap()
                    .iter()
                    .all(|intent_hash| {
                        let binding = self.txid_to_petition.borrow();
                        let value = binding.get(intent_hash).unwrap();
                        value.should_neglect_factors_due_to_irrelevant(factor_source_ids.clone())
                    })
            })
    }

    /// # Panics
    /// Panics if no petition deem usage of `FactorSource` with id
    /// `factor_source_id` relevant. We SHOULD have checked this already with
    /// `should_neglect_factors_due_to_irrelevant` from SignatureCollector main
    /// loop, i.e. we should not have called this method from SignaturesCollector
    /// if `should_neglect_factors_due_to_irrelevant` returned true.
    pub(crate) fn input_for_interactor(
        &self,
        factor_source_id: &FactorSourceIDFromHash,
    ) -> BatchTXBatchKeySigningRequest {
        let intent_hashes = self
            .factor_source_to_intent_hash
            .get(factor_source_id)
            .unwrap();

        let per_transaction = intent_hashes
            .into_iter()
            .map(|intent_hash| {
                let binding = self.txid_to_petition.borrow();
                let petition = binding.get(intent_hash).unwrap();
                petition.input_for_interactor(factor_source_id)
            })
            .collect::<IndexSet<BatchKeySigningRequest>>();

        BatchTXBatchKeySigningRequest::new(*factor_source_id, per_transaction)
    }

    fn add_signature(&self, signature: &HDSignature) {
        let binding = self.txid_to_petition.borrow();
        let petition = binding.get(signature.intent_hash()).unwrap();
        petition.add_signature(signature.clone())
    }

    fn neglect_factor_source_with_id(&self, neglected: NeglectedFactor) {
        let binding = self.txid_to_petition.borrow();
        let intent_hashes = self
            .factor_source_to_intent_hash
            .get(&neglected.factor_source_id())
            .unwrap();
        intent_hashes.into_iter().for_each(|intent_hash| {
            let petition = binding.get(intent_hash).unwrap();
            petition.neglect_factor_source(neglected.clone())
        });
    }

    pub(crate) fn process_batch_response(&self, response: SignWithFactorsOutcome) {
        match response {
            SignWithFactorsOutcome::Signed {
                produced_signatures,
            } => {
                for (k, v) in produced_signatures.signatures.clone().iter() {
                    info!("Signed with {} (#{} signatures)", k, v.len());
                }
                produced_signatures
                    .signatures
                    .values()
                    .flatten()
                    .for_each(|s| self.add_signature(s));
            }
            SignWithFactorsOutcome::Neglected(neglected_factors) => {
                let reason = neglected_factors.reason;
                for neglected_factor_source_id in neglected_factors.content.iter() {
                    info!("Neglected {}", neglected_factor_source_id);
                    self.neglect_factor_source_with_id(NeglectedFactor::new(
                        reason,
                        *neglected_factor_source_id,
                    ))
                }
            }
        }
    }

    #[allow(unused)]
    fn debug_str(&self) -> String {
        self.txid_to_petition
            .borrow()
            .iter()
            .map(|p| format!("Petitions({:#?}: {:#?})", p.0, p.1))
            .join(" + ")
    }
}

impl HasSampleValues for Petitions {
    fn sample() -> Self {
        let p0 = PetitionTransaction::sample();
        Self::new(
            HashMap::from_iter([(
                FactorSourceIDFromHash::fs0(),
                IndexSet::from_iter([p0.intent_hash.clone()]),
            )]),
            IndexMap::from_iter([(p0.intent_hash.clone(), p0)]),
        )
    }

    fn sample_other() -> Self {
        let p1 = PetitionTransaction::sample();
        Self::new(
            HashMap::from_iter([(
                FactorSourceIDFromHash::fs1(),
                IndexSet::from_iter([p1.intent_hash.clone()]),
            )]),
            IndexMap::from_iter([(p1.intent_hash.clone(), p1)]),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = Petitions;

    #[test]
    fn equality_of_samples() {
        assert_eq!(Sut::sample(), Sut::sample());
        assert_eq!(Sut::sample_other(), Sut::sample_other());
    }

    #[test]
    fn inequality_of_samples() {
        assert_ne!(Sut::sample(), Sut::sample_other());
    }

    #[test]
    fn debug() {
        pretty_assertions::assert_eq!(format!("{:?}", Sut::sample()), "Petitions(TXID(\"dedede\"): PetitionTransaction(for_entities: [PetitionEntity(intent_hash: TXID(\"dedede\"), entity: acco_Grace, \"threshold_factors PetitionFactors(input: PetitionFactorsInput(factors: {\\n    factor_source_id: Device:de, derivation_path: 0/A/tx/0,\\n    factor_source_id: Ledger:1e, derivation_path: 0/A/tx/1,\\n}), state_snapshot: signatures: \\\"\\\", neglected: \\\"\\\")\"\"override_factors PetitionFactors(input: PetitionFactorsInput(factors: {\\n    factor_source_id: Ledger:1e, derivation_path: 0/A/tx/1,\\n}), state_snapshot: signatures: \\\"\\\", neglected: \\\"\\\")\")]))");
    }
}
