use crate::prelude::*;

/// A coordinator which gathers public keys from several factor sources of different
/// kinds, in increasing friction order, for many transactions and for
/// potentially multiple entities and for many factor instances (derivation paths)
/// for each transaction.
///
/// By increasing friction order we mean, the quickest and easiest to use FactorSourceKind
/// is last; namely `DeviceFactorSource`, and the most tedious FactorSourceKind is
/// first; namely `LedgerFactorSource`, which user might also lack access to.
pub struct KeysCollector {
    /// Stateless immutable values used by the collector to gather public keys
    /// from factor sources.
    dependencies: KeysCollectorDependencies,

    /// Mutable internal state of the collector which builds up the list
    /// of public keys from each used factor source.
    state: RefCell<KeysCollectorState>,
}

impl KeysCollector {
    pub fn new(
        all_factor_sources_in_profile: impl IntoIterator<Item = HDFactorSource>,
        derivation_paths: IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<Self> {
        let preprocessor = KeysCollectorPreprocessor::new(derivation_paths);
        Self::with_preprocessor(
            all_factor_sources_in_profile
                .into_iter()
                .collect::<IndexSet<_>>(),
            interactors,
            preprocessor,
        )
    }

    fn with_preprocessor(
        all_factor_sources_in_profile: impl Into<IndexSet<HDFactorSource>>,
        interactors: Arc<dyn KeysDerivationInteractors>,
        preprocessor: KeysCollectorPreprocessor,
    ) -> Result<Self> {
        debug!("Init KeysCollector");
        let all_factor_sources_in_profile = all_factor_sources_in_profile.into();
        let (state, factors) = preprocessor.preprocess(all_factor_sources_in_profile)?;

        let dependencies = KeysCollectorDependencies::new(interactors, factors);

        Ok(Self {
            dependencies,
            state: RefCell::new(state),
        })
    }
}

// === PUBLIC ===
impl KeysCollector {
    #[allow(unused)]
    pub async fn collect_keys(self) -> KeyDerivationOutcome {
        _ = self
            .derive_with_factors() // in decreasing "friction order"
            .await
            .inspect_err(|e| eprintln!("Failed to use factor sources: {:#?}", e));
        self.state.into_inner().outcome()
    }
}

// === PRIVATE ===
impl KeysCollector {
    async fn use_factor_sources(&self, factor_sources_of_kind: &FactorSourcesOfKind) -> Result<()> {
        let interactor = self
            .dependencies
            .interactors
            .interactor_for(factor_sources_of_kind.kind);
        let factor_sources = factor_sources_of_kind.factor_sources();
        match interactor {
            KeyDerivationInteractor::PolyFactor(interactor) => {
                // Prepare the request for the interactor
                debug!("Creating poly request for interactor");
                let request = self.request_for_parallel_interactor(
                    factor_sources
                        .into_iter()
                        .map(|f| f.factor_source_id())
                        .collect(),
                )?;
                debug!("Dispatching poly request to interactor: {:?}", request);
                let response = interactor.derive(request).await?;
                self.process_batch_response(response)?;
            }

            KeyDerivationInteractor::MonoFactor(interactor) => {
                for factor_source in factor_sources {
                    // Prepare the request for the interactor
                    debug!("Creating mono request for interactor");
                    let request =
                        self.request_for_serial_interactor(&factor_source.factor_source_id())?;

                    debug!("Dispatching mono request to interactor: {:?}", request);
                    // Produce the results from the interactor
                    let response = interactor.derive(request).await?;

                    // Report the results back to the collector
                    self.process_batch_response(response)?;
                }
            }
        }
        Ok(())
    }

    /// In decreasing "friction order"
    async fn derive_with_factors(&self) -> Result<()> {
        for factor_sources_of_kind in self.dependencies.factors_of_kind.iter() {
            info!(
                "Use(?) #{:?} factors of kind: {:?}",
                &factor_sources_of_kind.factor_sources().len(),
                &factor_sources_of_kind.kind
            );
            self.use_factor_sources(factor_sources_of_kind).await?;
        }
        Ok(())
    }

    fn input_for_interactor(
        &self,
        factor_source_id: &FactorSourceIDFromHash,
    ) -> Result<MonoFactorKeyDerivationRequest> {
        let keyring = self.state.borrow().keyring_for(factor_source_id)?;
        assert_eq!(keyring.factors().len(), 0);
        let paths = keyring.paths.clone();
        Ok(MonoFactorKeyDerivationRequest::new(
            *factor_source_id,
            paths,
        ))
    }

    fn request_for_parallel_interactor(
        &self,
        factor_sources_ids: IndexSet<FactorSourceIDFromHash>,
    ) -> Result<PolyFactorKeyDerivationRequest> {
        let per_factor_source = factor_sources_ids
            .into_iter()
            .map(|f| self.input_for_interactor(&f))
            .collect::<Result<Vec<MonoFactorKeyDerivationRequest>>>()?;
        Ok(PolyFactorKeyDerivationRequest::new(
            per_factor_source
                .into_iter()
                .map(|r| (r.factor_source_id, r))
                .collect(),
        ))
    }

    fn request_for_serial_interactor(
        &self,
        factor_source_id: &FactorSourceIDFromHash,
    ) -> Result<MonoFactorKeyDerivationRequest> {
        self.input_for_interactor(factor_source_id)
    }

    fn process_batch_response(&self, response: KeyDerivationResponse) -> Result<()> {
        self.state.borrow_mut().process_batch_response(response)
    }
}
