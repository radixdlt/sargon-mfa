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
    pub factor_to_txid: HashMap<FactorSourceIDFromHash, IndexSet<IntentHash>>,

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
        factor_to_txid: HashMap<FactorSourceIDFromHash, IndexSet<IntentHash>>,
        txid_to_petition: IndexMap<IntentHash, PetitionTransaction>,
    ) -> Self {
        Self {
            factor_to_txid,
            txid_to_petition: RefCell::new(txid_to_petition),
        }
    }

    pub fn outcome(self) -> SignaturesOutcome {
        let txid_to_petition = self.txid_to_petition.into_inner();
        let mut failed_transactions = MaybeSignedTransactions::empty();
        let mut successful_transactions = MaybeSignedTransactions::empty();
        let mut skipped_factor_sources = IndexSet::<_>::new();
        for (txid, petition_of_transaction) in txid_to_petition.into_iter() {
            let (successful, signatures, skipped) = petition_of_transaction.outcome();
            if successful {
                successful_transactions.add_signatures(txid, signatures);
            } else {
                failed_transactions.add_signatures(txid, signatures);
            }
            skipped_factor_sources.extend(skipped)
        }

        SignaturesOutcome::new(
            successful_transactions,
            failed_transactions,
            skipped_factor_sources,
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

    pub fn invalid_transactions_if_skipped_factors(
        &self,
        factor_source_ids: IndexSet<FactorSourceIDFromHash>,
    ) -> IndexSet<InvalidTransactionIfSkipped> {
        factor_source_ids
            .clone()
            .iter()
            .flat_map(|f| {
                self.factor_to_txid.get(f).unwrap().iter().flat_map(|txid| {
                    let binding = self.txid_to_petition.borrow();
                    let value = binding.get(txid).unwrap();
                    value.invalid_transactions_if_skipped_factors(factor_source_ids.clone())
                })
            })
            .collect::<IndexSet<_>>()
    }

    pub(crate) fn input_for_interactor(
        &self,
        factor_source_id: &FactorSourceIDFromHash,
    ) -> BatchTXBatchKeySigningRequest {
        let txids = self.factor_to_txid.get(factor_source_id).unwrap();
        let per_transaction = txids
            .into_iter()
            .map(|txid| {
                let binding = self.txid_to_petition.borrow();
                let petition = binding.get(txid).unwrap();
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

    fn skip_factor_source_with_id(&self, skipped_factor_source_id: &FactorSourceIDFromHash) {
        let binding = self.txid_to_petition.borrow();
        let txids = self.factor_to_txid.get(skipped_factor_source_id).unwrap();
        txids.into_iter().for_each(|txid| {
            let petition = binding.get(txid).unwrap();
            petition.skipped_factor_source(skipped_factor_source_id)
        });
    }

    pub(crate) fn process_batch_response(&self, response: SignWithFactorSourceOrSourcesOutcome) {
        match response {
            SignWithFactorSourceOrSourcesOutcome::Signed {
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
            SignWithFactorSourceOrSourcesOutcome::Skipped {
                ids_of_skipped_factors_sources,
            } => {
                for skipped_factor_source_id in ids_of_skipped_factors_sources.iter() {
                    info!("Skipped {}", skipped_factor_source_id);
                    self.skip_factor_source_with_id(skipped_factor_source_id)
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
        pretty_assertions::assert_eq!(format!("{:?}", Sut::sample()), "Petitions(TXID(\"dedede\"): PetitionTransaction(for_entities: [PetitionEntity(intent_hash: TXID(\"dedede\"), entity: acco_Grace, \"threshold_factors PetitionFactors(input: PetitionFactorsInput(factors: {\\n    factor_source_id: Device:de, derivation_path: 0/A/tx/0,\\n    factor_source_id: Ledger:1e, derivation_path: 0/A/tx/1,\\n}), state_snapshot: signatures: \\\"\\\", skipped: \\\"\\\")\"\"override_factors PetitionFactors(input: PetitionFactorsInput(factors: {\\n    factor_source_id: Ledger:1e, derivation_path: 0/A/tx/1,\\n}), state_snapshot: signatures: \\\"\\\", skipped: \\\"\\\")\")]))");
    }
}
