use crate::prelude::*;

/// Petition of signatures for a transaction.
/// Essentially a wrapper around `Iterator<Item = PetitionForEntity>`.
#[derive(derive_more::Debug, PartialEq, Eq)]
#[debug("{}", self.debug_str())]
pub(crate) struct PetitionForTransaction {
    /// Hash of transaction to sign
    pub intent_hash: IntentHash,

    pub for_entities: RefCell<HashMap<AddressOfAccountOrPersona, PetitionForEntity>>,
}

impl PetitionForTransaction {
    pub(crate) fn new(
        intent_hash: IntentHash,
        for_entities: HashMap<AddressOfAccountOrPersona, PetitionForEntity>,
    ) -> Self {
        Self {
            intent_hash,
            for_entities: RefCell::new(for_entities),
        }
    }

    /// Returns `(true, _)` if this transaction has been successfully signed by
    /// all required factor instances.
    ///
    /// Returns `(false, _)` if not enough factor instances have signed.
    ///
    /// The second value in the tuple `(_, IndexSet<HDSignature>, _)` contains all
    /// the signatures, even if it the transaction was failed, all signatures
    /// will be returned (which might be empty).
    ///
    /// The third value in the tuple `(_, _, IndexSet<FactorSourceIDFromHash>)` contains the
    /// id of all the factor sources which was skipped.
    pub fn outcome(self) -> PetitionTransactionOutcome {
        let for_entities = self
            .for_entities
            .into_inner()
            .values()
            .map(|x| x.to_owned())
            .collect_vec();

        let transaction_valid = for_entities
            .iter()
            .all(|b| b.has_signatures_requirement_been_fulfilled());

        let signatures = for_entities
            .iter()
            .flat_map(|x| x.all_signatures())
            .collect::<IndexSet<_>>();

        let neglected_factors = for_entities
            .iter()
            .flat_map(|x| x.all_neglected_factor_sources())
            .collect::<IndexSet<NeglectedFactor>>();

        PetitionTransactionOutcome::new(
            transaction_valid,
            self.intent_hash.clone(),
            signatures,
            neglected_factors,
        )
    }

    fn _all_factor_instances(&self) -> IndexSet<OwnedFactorInstance> {
        self.for_entities
            .borrow()
            .iter()
            .flat_map(|(_, petition)| petition.all_factor_instances())
            .collect()
    }

    pub fn has_tx_failed(&self) -> bool {
        self.for_entities.borrow().values().any(|p| p.has_failed())
    }

    pub fn all_relevant_factor_instances_of_source(
        &self,
        factor_source_id: &FactorSourceIDFromHash,
    ) -> IndexSet<OwnedFactorInstance> {
        assert!(!self.has_tx_failed());
        self.for_entities
            .borrow()
            .values()
            .filter(|&p| {
                if p.has_failed() {
                    debug!("OMITTING petition since it HAS failed: {:?}", p);
                    false
                } else {
                    debug!("INCLUDING petition since it has NOT failed: {:?}", p);
                    true
                }
            })
            .cloned()
            .flat_map(|petition| petition.all_factor_instances())
            .filter(|f| f.factor_source_id() == *factor_source_id)
            .collect()
    }

    pub fn add_signature(&self, signature: HDSignature) {
        let for_entities = self.for_entities.borrow_mut();
        let for_entity = for_entities
            .get(&signature.owned_factor_instance().owner)
            .unwrap();
        for_entity.add_signature(signature.clone());
    }

    pub fn neglect_factor_source(&self, neglected: NeglectedFactor) {
        let mut for_entities = self.for_entities.borrow_mut();
        for petition in for_entities.values_mut() {
            petition.neglect_if_referenced(neglected.clone()).unwrap()
        }
    }

    pub(crate) fn input_for_interactor(
        &self,
        factor_source_id: &FactorSourceIDFromHash,
    ) -> BatchKeySigningRequest {
        assert!(!self
            .should_neglect_factors_due_to_irrelevant(IndexSet::from_iter([*factor_source_id])));
        assert!(!self.has_tx_failed());
        BatchKeySigningRequest::new(
            self.intent_hash.clone(),
            *factor_source_id,
            self.all_relevant_factor_instances_of_source(factor_source_id),
        )
    }

    pub fn invalid_transactions_if_neglected_factors(
        &self,
        factor_source_ids: IndexSet<FactorSourceIDFromHash>,
    ) -> IndexSet<InvalidTransactionIfNeglected> {
        if self.has_tx_failed() {
            // No need to display already failed tx.
            return IndexSet::new();
        }
        self.for_entities
            .borrow()
            .iter()
            .flat_map(|(_, petition)| {
                petition.invalid_transactions_if_neglected_factors(factor_source_ids.clone())
            })
            .collect()
    }

    pub(crate) fn should_neglect_factors_due_to_irrelevant(
        &self,
        factor_source_ids: IndexSet<FactorSourceIDFromHash>,
    ) -> bool {
        self.for_entities
            .borrow()
            .values()
            .filter(|&p| p.references_any_factor_source(&factor_source_ids))
            .cloned()
            .all(|petition| {
                petition.should_neglect_factors_due_to_irrelevant(factor_source_ids.clone())
            })
    }

    #[allow(unused)]
    fn debug_str(&self) -> String {
        let entities = self
            .for_entities
            .borrow()
            .iter()
            .map(|p| format!("PetitionForEntity({:#?})", p.1))
            .join(", ");

        format!("PetitionForTransaction(for_entities: [{}])", entities)
    }
}

impl HasSampleValues for PetitionForTransaction {
    fn sample() -> Self {
        let intent_hash = IntentHash::sample();
        let entity = Account::sample_securified();
        Self::new(
            intent_hash.clone(),
            HashMap::from_iter([(
                entity.address(),
                PetitionForEntity::new(
                    intent_hash.clone(),
                    entity.address(),
                    PetitionForFactors::sample(),
                    PetitionForFactors::sample_other(),
                ),
            )]),
        )
    }

    fn sample_other() -> Self {
        let intent_hash = IntentHash::sample_other();
        let entity = Persona::sample_unsecurified();
        Self::new(
            intent_hash.clone(),
            HashMap::from_iter([(
                entity.address(),
                PetitionForEntity::new(
                    intent_hash.clone(),
                    entity.address(),
                    PetitionForFactors::sample_other(),
                    None,
                ),
            )]),
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    type Sut = PetitionForTransaction;

    #[test]
    fn equality() {
        assert_eq!(Sut::sample(), Sut::sample());
        assert_eq!(Sut::sample_other(), Sut::sample_other());
    }

    #[test]
    fn inequality() {
        assert_ne!(Sut::sample(), Sut::sample_other());
    }

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", Sut::sample()), "PetitionForTransaction(for_entities: [PetitionForEntity(intent_hash: TXID(\"dedede\"), entity: acco_Grace, \"threshold_factors PetitionForFactors(input: PetitionFactorsInput(factors: {\\n    factor_source_id: Device:de, derivation_path: 0/A/tx/0,\\n    factor_source_id: Ledger:1e, derivation_path: 0/A/tx/1,\\n}), state_snapshot: signatures: \\\"\\\", neglected: \\\"\\\")\"\"override_factors PetitionForFactors(input: PetitionFactorsInput(factors: {\\n    factor_source_id: Ledger:1e, derivation_path: 0/A/tx/1,\\n}), state_snapshot: signatures: \\\"\\\", neglected: \\\"\\\")\")])");
    }
}