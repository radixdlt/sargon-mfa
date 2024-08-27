use crate::prelude::*;

pub struct KeysCollectorState {
    pub(super) keyrings: RefCell<IndexMap<FactorSourceIDFromHash, Keyring>>,
}

impl KeysCollectorState {
    pub fn new(
        derivation_paths: IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>,
    ) -> Self {
        let keyrings = derivation_paths
            .into_iter()
            .map(|(factor_source_id, derivation_paths)| {
                (
                    factor_source_id,
                    Keyring::new(factor_source_id, derivation_paths),
                )
            })
            .collect::<IndexMap<FactorSourceIDFromHash, Keyring>>();
        Self {
            keyrings: RefCell::new(keyrings),
        }
    }

    pub fn outcome(self) -> KeyDerivationOutcome {
        let key_rings = self.keyrings.into_inner();
        KeyDerivationOutcome::new(
            key_rings
                .into_iter()
                .map(|(k, v)| (k, v.factors()))
                .collect(),
        )
    }

    pub fn keyring_for(&self, factor_source_id: &FactorSourceIDFromHash) -> Result<Keyring> {
        self.keyrings
            .borrow()
            .get(factor_source_id)
            .cloned()
            .inspect(|k| assert_eq!(k.factor_source_id, *factor_source_id))
            .ok_or(CommonError::UnknownFactorSource)
    }

    pub(crate) fn process_batch_response(&self, response: BatchDerivationResponse) {
        for (factor_source_id, factors) in response.per_factor_source.into_iter() {
            let mut rings = self.keyrings.borrow_mut();
            let keyring = rings.get_mut(&factor_source_id).unwrap();
            keyring.process_response(factors)
        }
    }
}
