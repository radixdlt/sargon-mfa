use crate::prelude::*;

/// Petition of signatures from an entity in a transaction.
/// Essentially a wrapper around a tuple
/// `{ threshold: PetitionFactors, override: PetitionFactors }`
#[derive(Clone, PartialEq, Eq, derive_more::Debug)]
#[debug("{}", self.debug_str())]
pub struct PetitionEntity {
    /// The owner of these factors
    pub entity: AddressOfAccountOrPersona,

    /// Index and hash of transaction
    pub intent_hash: IntentHash,

    /// Petition with threshold factors
    pub threshold_factors: Option<RefCell<PetitionFactors>>,

    /// Petition with override factors
    pub override_factors: Option<RefCell<PetitionFactors>>,
}

impl PetitionEntity {
    pub fn new(
        intent_hash: IntentHash,
        entity: AddressOfAccountOrPersona,
        threshold_factors: impl Into<Option<PetitionFactors>>,
        override_factors: impl Into<Option<PetitionFactors>>,
    ) -> Self {
        let threshold_factors = threshold_factors.into();
        let override_factors = override_factors.into();
        if threshold_factors.is_none() && override_factors.is_none() {
            panic!("Programmer error! Must have at least one factors list.");
        }
        Self {
            entity,
            intent_hash,
            threshold_factors: threshold_factors.map(RefCell::new),
            override_factors: override_factors.map(RefCell::new),
        }
    }

    pub fn new_securified(
        intent_hash: IntentHash,
        entity: AddressOfAccountOrPersona,
        matrix: MatrixOfFactorInstances,
    ) -> Self {
        Self::new(
            intent_hash,
            entity,
            PetitionFactors::new_threshold(matrix.threshold_factors, matrix.threshold as i8),
            PetitionFactors::new_override(matrix.override_factors),
        )
    }

    pub fn new_unsecurified(
        intent_hash: IntentHash,
        entity: AddressOfAccountOrPersona,
        instance: HierarchicalDeterministicFactorInstance,
    ) -> Self {
        Self::new(
            intent_hash,
            entity,
            PetitionFactors::new_unsecurified(instance),
            None,
        )
    }

    /// Returns `true` signatures requirement has been fulfilled, either by
    /// override factors or by threshold factors
    pub fn has_signatures_requirement_been_fulfilled(&self) -> bool {
        self.status() == PetitionFactorsStatus::Finished(PetitionFactorsStatusFinished::Success)
    }

    fn union_of<F, T>(&self, map: F) -> IndexSet<T>
    where
        T: Eq + std::hash::Hash + Clone,
        F: Fn(&PetitionFactors) -> IndexSet<T>,
    {
        self.both(
            |l| map(l),
            |t, o| {
                t.unwrap_or_default()
                    .union(&o.unwrap_or_default())
                    .cloned()
                    .collect::<IndexSet<T>>()
            },
        )
    }

    pub fn all_factor_instances(&self) -> IndexSet<OwnedFactorInstance> {
        self.union_of(|l| l.factor_instances())
            .into_iter()
            .map(|f| OwnedFactorInstance::owned_factor_instance(self.entity.clone(), f.clone()))
            .collect::<IndexSet<_>>()
    }

    pub fn all_neglected_factor_instances(&self) -> IndexSet<NeglectedFactorInstance> {
        self.union_of(|f| f.all_neglected())
    }

    pub fn all_neglected_factor_sources(&self) -> IndexSet<NeglectedFactor> {
        self.all_neglected_factor_instances()
            .into_iter()
            .map(|n| n.as_neglected_factor())
            .collect::<IndexSet<_>>()
    }

    pub fn all_signatures(&self) -> IndexSet<HDSignature> {
        self.union_of(|f| f.all_signatures())
    }

    fn with_list<F, T>(list: &Option<RefCell<PetitionFactors>>, map: F) -> Option<T>
    where
        F: Fn(&PetitionFactors) -> T,
    {
        list.as_ref().map(|refcell| map(&refcell.borrow()))
    }

    fn on_list<F, R>(&self, kind: FactorListKind, r#do: &F) -> Option<R>
    where
        F: Fn(&PetitionFactors) -> R,
    {
        match kind {
            FactorListKind::Threshold => Self::with_list(&self.threshold_factors, r#do),
            FactorListKind::Override => Self::with_list(&self.override_factors, r#do),
        }
    }

    fn both<F, C, T, R>(&self, r#do: F, combine: C) -> R
    where
        F: Fn(&PetitionFactors) -> T,
        C: Fn(Option<T>, Option<T>) -> R,
    {
        let t = self.on_list(FactorListKind::Threshold, &r#do);
        let o = self.on_list(FactorListKind::Override, &r#do);
        combine(t, o)
    }

    fn both_void<F, R>(&self, r#do: F)
    where
        F: Fn(&PetitionFactors) -> R,
    {
        self.both(r#do, |_, _| ())
    }

    /// # Panics
    /// Panics if this factor source has already been neglected or signed with.
    ///
    /// Or panics if the factor source is not known to this petition.
    pub fn add_signature(&self, signature: HDSignature) {
        self.both(|l| l.add_signature_if_relevant(&signature), |t, o| {
            match (t, o) {
                (Some(true), Some(true)) => {
                    unreachable!("Matrix of FactorInstances does not allow for a factor to be present in both threshold and override list, thus this will never happen.")
                }
                (Some(false), Some(false)) => panic!("Factor source not found in any of the lists."),
                (None, None) => panic!("Programmer error! Must have at least one factors list."), 
                _ => (),
            }
        })
    }

    pub(crate) fn should_neglect_factors_due_to_irrelevant(
        &self,
        factor_source_ids: IndexSet<FactorSourceIDFromHash>,
    ) -> bool {
        assert!(self.references_any_factor_source(&factor_source_ids));
        match self.status() {
            PetitionFactorsStatus::Finished(PetitionFactorsStatusFinished::Fail) => true,
            PetitionFactorsStatus::Finished(PetitionFactorsStatusFinished::Success) => false, // unsure about this...
            PetitionFactorsStatus::InProgress => false,
        }
    }

    pub fn invalid_transactions_if_neglected_factors(
        &self,
        factor_source_ids: IndexSet<FactorSourceIDFromHash>,
    ) -> IndexSet<InvalidTransactionIfNeglected> {
        let status_if_neglected = self.status_if_neglected_factors(factor_source_ids);
        match status_if_neglected {
            PetitionFactorsStatus::Finished(finished_reason) => match finished_reason {
                PetitionFactorsStatusFinished::Fail => {
                    let intent_hash = self.intent_hash.clone();
                    let invalid_transaction =
                        InvalidTransactionIfNeglected::new(intent_hash, vec![self.entity.clone()]);
                    IndexSet::from_iter([invalid_transaction])
                }
                PetitionFactorsStatusFinished::Success => IndexSet::new(),
            },
            PetitionFactorsStatus::InProgress => IndexSet::new(),
        }
    }

    pub fn status_if_neglected_factors(
        &self,
        factor_source_ids: IndexSet<FactorSourceIDFromHash>,
    ) -> PetitionFactorsStatus {
        let simulation = self.clone();
        for factor_source_id in factor_source_ids.iter() {
            simulation
                .neglect_if_referenced(NeglectedFactor::new(
                    NeglectFactorReason::Simulation,
                    *factor_source_id,
                ))
                .unwrap();
        }
        simulation.status()
    }

    pub fn references_any_factor_source(
        &self,
        factor_source_ids: &IndexSet<FactorSourceIDFromHash>,
    ) -> bool {
        factor_source_ids
            .iter()
            .any(|f| self.references_factor_source_with_id(f))
    }

    pub fn references_factor_source_with_id(&self, id: &FactorSourceIDFromHash) -> bool {
        self.both(
            |p| p.references_factor_source_with_id(id),
            |a, b| a.unwrap_or(false) || b.unwrap_or(false),
        )
    }

    pub fn neglect_if_referenced(&self, neglected: NeglectedFactor) -> Result<()> {
        self.both_void(|p| p.neglect_if_referenced(neglected.clone()));
        Ok(())
    }

    pub fn status(&self) -> PetitionFactorsStatus {
        use PetitionFactorsStatus::*;
        use PetitionFactorsStatusFinished::*;

        let maybe_threshold = self.threshold_factors.as_ref().map(|t| t.borrow().status());
        let maybe_override = self.override_factors.as_ref().map(|o| o.borrow().status());
        if let Some(t) = &maybe_threshold {
            trace!("Threshold factor status: {:?}", t);
        }
        if let Some(o) = &maybe_override {
            trace!("Override factor status: {:?}", o);
        }
        match (maybe_threshold, maybe_override) {
            (None, None) => panic!("Programmer error! Should have at least one factors list."),
            (Some(threshold), None) => threshold,
            (None, Some(r#override)) => r#override,
            (Some(threshold), Some(r#override)) => match (threshold, r#override) {
                (InProgress, InProgress) => PetitionFactorsStatus::InProgress,
                (Finished(Fail), InProgress) => PetitionFactorsStatus::InProgress,
                (InProgress, Finished(Fail)) => PetitionFactorsStatus::InProgress,
                (Finished(Fail), Finished(Fail)) => PetitionFactorsStatus::Finished(Fail),
                (Finished(Success), _) => PetitionFactorsStatus::Finished(Success),
                (_, Finished(Success)) => PetitionFactorsStatus::Finished(Success),
            },
        }
    }

    #[allow(unused)]
    fn debug_str(&self) -> String {
        let thres: String = self
            .threshold_factors
            .clone()
            .map(|f| format!("threshold_factors {:#?}", f.borrow()))
            .unwrap_or_default();

        let overr: String = self
            .override_factors
            .clone()
            .map(|f| format!("override_factors {:#?}", f.borrow()))
            .unwrap_or_default();

        format!(
            "intent_hash: {:#?}, entity: {:#?}, {:#?}{:#?}",
            self.intent_hash, self.entity, thres, overr
        )
    }
}

impl PetitionEntity {
    fn from_entity(entity: impl Into<AccountOrPersona>, intent_hash: IntentHash) -> Self {
        let entity = entity.into();
        match entity.security_state() {
            EntitySecurityState::Securified(matrix) => {
                Self::new_securified(intent_hash, entity.address(), matrix)
            }
            EntitySecurityState::Unsecured(factor) => {
                Self::new_unsecurified(intent_hash, entity.address(), factor)
            }
        }
    }
}

impl HasSampleValues for PetitionEntity {
    fn sample() -> Self {
        Self::from_entity(Account::sample_securified(), IntentHash::sample())
    }

    fn sample_other() -> Self {
        Self::from_entity(Account::sample_unsecurified(), IntentHash::sample_other())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type Sut = PetitionEntity;

    #[test]
    fn multiple_device_as_override_skipped_both_is_invalid() {
        let d0 = HDFactorSource::fs0();
        let d1 = HDFactorSource::fs10();
        assert_eq!(d0.factor_source_kind(), FactorSourceKind::Device);
        assert_eq!(d1.factor_source_kind(), FactorSourceKind::Device);

        let matrix =
            MatrixOfFactorInstances::override_only([d0.clone(), d1.clone()].into_iter().map(|f| {
                HierarchicalDeterministicFactorInstance::mainnet_tx_account(
                    HDPathComponent::securified(0),
                    f.factor_source_id(),
                )
            }));
        let entity = AddressOfAccountOrPersona::Account(AccountAddress::sample());
        let tx = IntentHash::sample_third();
        let sut = Sut::new_securified(tx.clone(), entity.clone(), matrix);
        let invalid = sut.invalid_transactions_if_neglected_factors(IndexSet::from_iter([
            d0.factor_source_id(),
            d1.factor_source_id(),
        ]));
        assert_eq!(
            invalid
                .clone()
                .into_iter()
                .map(|t| t.intent_hash)
                .collect_vec(),
            vec![tx]
        );
        assert_eq!(
            invalid
                .into_iter()
                .flat_map(|t| t.entities_which_would_fail_auth().into_iter().collect_vec())
                .collect_vec(),
            vec![entity]
        );
    }

    #[test]
    fn multiple_device_as_override_skipped_one_is_valid() {
        let d0 = HDFactorSource::fs0();
        let d1 = HDFactorSource::fs10();
        assert_eq!(d0.factor_source_kind(), FactorSourceKind::Device);
        assert_eq!(d1.factor_source_kind(), FactorSourceKind::Device);

        let matrix =
            MatrixOfFactorInstances::override_only([d0.clone(), d1.clone()].into_iter().map(|f| {
                HierarchicalDeterministicFactorInstance::mainnet_tx_account(
                    HDPathComponent::securified(0),
                    f.factor_source_id(),
                )
            }));
        let entity = AddressOfAccountOrPersona::Account(AccountAddress::sample());
        let tx = IntentHash::sample_third();
        let sut = Sut::new_securified(tx.clone(), entity.clone(), matrix);
        let invalid = sut.invalid_transactions_if_neglected_factors(IndexSet::from_iter([
            d0.factor_source_id()
        ]));
        assert!(invalid.is_empty());
    }

    #[test]
    fn multiple_device_as_threshold_skipped_both_is_invalid() {
        let d0 = HDFactorSource::fs0();
        let d1 = HDFactorSource::fs10();
        assert_eq!(d0.factor_source_kind(), FactorSourceKind::Device);
        assert_eq!(d1.factor_source_kind(), FactorSourceKind::Device);

        let matrix = MatrixOfFactorInstances::threshold_only(
            [d0.clone(), d1.clone()].into_iter().map(|f| {
                HierarchicalDeterministicFactorInstance::mainnet_tx_account(
                    HDPathComponent::securified(0),
                    f.factor_source_id(),
                )
            }),
            2,
        );

        let entity = AddressOfAccountOrPersona::Account(AccountAddress::sample());
        let tx = IntentHash::sample_third();
        let sut = Sut::new_securified(tx.clone(), entity.clone(), matrix);
        let invalid = sut.invalid_transactions_if_neglected_factors(IndexSet::from_iter([
            d0.factor_source_id(),
            d1.factor_source_id(),
        ]));
        assert_eq!(
            invalid
                .clone()
                .into_iter()
                .map(|t| t.intent_hash)
                .collect_vec(),
            vec![tx]
        );
        assert_eq!(
            invalid
                .into_iter()
                .flat_map(|t| t.entities_which_would_fail_auth().into_iter().collect_vec())
                .collect_vec(),
            vec![entity]
        );
    }

    #[test]
    fn two_device_as_threshold_of_2_skipped_one_is_invalid() {
        let d0 = HDFactorSource::fs0();
        let d1 = HDFactorSource::fs10();
        assert_eq!(d0.factor_source_kind(), FactorSourceKind::Device);
        assert_eq!(d1.factor_source_kind(), FactorSourceKind::Device);

        let matrix = MatrixOfFactorInstances::threshold_only(
            [d0.clone(), d1.clone()].into_iter().map(|f| {
                HierarchicalDeterministicFactorInstance::mainnet_tx_account(
                    HDPathComponent::securified(0),
                    f.factor_source_id(),
                )
            }),
            2,
        );

        let entity = AddressOfAccountOrPersona::Account(AccountAddress::sample());
        let tx = IntentHash::sample_third();
        let sut = Sut::new_securified(tx.clone(), entity.clone(), matrix);

        let invalid = sut.invalid_transactions_if_neglected_factors(IndexSet::from_iter([
            d1.factor_source_id()
        ]));

        assert_eq!(
            invalid
                .clone()
                .into_iter()
                .map(|t| t.intent_hash)
                .collect_vec(),
            vec![tx]
        );
        assert_eq!(
            invalid
                .into_iter()
                .flat_map(|t| t.entities_which_would_fail_auth().into_iter().collect_vec())
                .collect_vec(),
            vec![entity]
        );
    }

    #[test]
    fn two_device_as_threshold_of_1_skipped_one_is_valid() {
        let d0 = HDFactorSource::fs0();
        let d1 = HDFactorSource::fs10();
        assert_eq!(d0.factor_source_kind(), FactorSourceKind::Device);
        assert_eq!(d1.factor_source_kind(), FactorSourceKind::Device);

        let matrix = MatrixOfFactorInstances::threshold_only(
            [d0.clone(), d1.clone()].into_iter().map(|f| {
                HierarchicalDeterministicFactorInstance::mainnet_tx_account(
                    HDPathComponent::securified(0),
                    f.factor_source_id(),
                )
            }),
            1,
        );

        let entity = AddressOfAccountOrPersona::Account(AccountAddress::sample());
        let tx = IntentHash::sample_third();
        let sut = Sut::new_securified(tx.clone(), entity.clone(), matrix);

        let invalid = sut.invalid_transactions_if_neglected_factors(IndexSet::from_iter([
            d1.factor_source_id()
        ]));

        assert!(invalid.is_empty());
    }

    #[test]
    fn debug() {
        pretty_assertions::assert_eq!(format!("{:?}", Sut::sample()), "intent_hash: TXID(\"dedede\"), entity: acco_Grace, \"threshold_factors PetitionFactors(input: PetitionFactorsInput(factors: {\\n    factor_source_id: Device:00, derivation_path: 0/A/tx/6,\\n    factor_source_id: Arculus:03, derivation_path: 0/A/tx/6,\\n    factor_source_id: Yubikey:05, derivation_path: 0/A/tx/6,\\n}), state_snapshot: signatures: \\\"\\\", neglected: \\\"\\\")\"\"override_factors PetitionFactors(input: PetitionFactorsInput(factors: {\\n    factor_source_id: Ledger:01, derivation_path: 0/A/tx/6,\\n    factor_source_id: Arculus:04, derivation_path: 0/A/tx/6,\\n}), state_snapshot: signatures: \\\"\\\", neglected: \\\"\\\")\"");
    }

    #[test]
    #[should_panic(expected = "Programmer error! Must have at least one factors list.")]
    fn invalid_empty_factors() {
        Sut::new(
            IntentHash::sample(),
            AddressOfAccountOrPersona::sample(),
            None,
            None,
        );
    }

    #[test]
    #[should_panic(expected = "Factor source not found in any of the lists.")]
    fn cannot_add_unrelated_signature() {
        let sut = Sut::sample();
        sut.add_signature(HDSignature::sample());
    }

    #[test]
    #[should_panic(expected = "A factor MUST NOT be present in both threshold AND override list.")]
    fn factor_should_not_be_used_in_both_lists() {
        Account::securified_mainnet(0, "Jane Doe", |idx| {
            let fi = HierarchicalDeterministicFactorInstance::f(CAP26EntityKind::Account, idx);
            MatrixOfFactorInstances::new(
                [FactorSourceIDFromHash::fs0()].map(&fi),
                1,
                [FactorSourceIDFromHash::fs0()].map(&fi),
            )
        });
    }

    #[test]
    #[should_panic]
    fn cannot_add_same_signature_twice() {
        let intent_hash = IntentHash::sample();
        let entity = Account::securified_mainnet(0, "Jane Doe", |idx| {
            let fi = HierarchicalDeterministicFactorInstance::f(CAP26EntityKind::Account, idx);
            MatrixOfFactorInstances::new(
                [FactorSourceIDFromHash::fs0()].map(&fi),
                1,
                [FactorSourceIDFromHash::fs1()].map(&fi),
            )
        });
        let sut = Sut::from_entity(entity.clone(), intent_hash.clone());
        let sign_input = HDSignatureInput::new(
            intent_hash,
            OwnedFactorInstance::new(
                entity.address(),
                HierarchicalDeterministicFactorInstance::mainnet_tx_account(
                    HDPathComponent::non_hardened(0),
                    FactorSourceIDFromHash::fs0(),
                ),
            ),
        );
        let signature = HDSignature::produced_signing_with_input(sign_input);

        sut.add_signature(signature.clone());
        sut.add_signature(signature.clone());
    }

    #[test]
    fn invalid_transactions_if_neglected_success() {
        let sut = Sut::sample();
        sut.add_signature(HDSignature::produced_signing_with_input(
            HDSignatureInput::new(
                sut.intent_hash.clone(),
                OwnedFactorInstance::new(
                    sut.entity.clone(),
                    HierarchicalDeterministicFactorInstance::mainnet_tx_account(
                        HDPathComponent::non_hardened(6),
                        FactorSourceIDFromHash::fs1(),
                    ),
                ),
            ),
        ));
        let can_skip = |f: FactorSourceIDFromHash| {
            assert!(sut
                // Already signed with override factor `FactorSourceIDFromHash::fs1()`. Thus
                // can skip
                .invalid_transactions_if_neglected_factors(IndexSet::from_iter([f]))
                .is_empty())
        };
        can_skip(FactorSourceIDFromHash::fs0());
        can_skip(FactorSourceIDFromHash::fs3());
        can_skip(FactorSourceIDFromHash::fs4());
        can_skip(FactorSourceIDFromHash::fs5());
    }

    #[test]
    fn inequality() {
        assert_ne!(Sut::sample(), Sut::sample_other())
    }

    #[test]
    fn equality() {
        assert_eq!(Sut::sample(), Sut::sample());
        assert_eq!(Sut::sample_other(), Sut::sample_other());
    }
}
