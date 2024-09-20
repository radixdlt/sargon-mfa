use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DerivationAndAnalysisAccountRecoveryScan {
    /// All factor sources input to account recovery scan, used or not.
    factor_sources: Vec<HDFactorSource>,

    /// All recovered accounts from account recovery scan
    recovered_accounts: Vec<Account>,

    /// Unrecovered securified accounts from account recovery scan
    pub unrecovered_securified_accounts: UnrecoveredSecurifiedEntities,

    pub probably_free_instances: ProbablyFreeFactorInstances,
}
impl DerivationAndAnalysisAccountRecoveryScan {
    pub fn new(
        factor_sources: IndexSet<HDFactorSource>,
        recovered_accounts: IndexSet<Account>,
        unrecovered_securified_accounts: UnrecoveredSecurifiedEntities,
        probably_free_instances: ProbablyFreeFactorInstances,
    ) -> Self {
        Self {
            factor_sources: factor_sources.into_iter().collect(),
            recovered_accounts: recovered_accounts.into_iter().collect(),
            unrecovered_securified_accounts,
            probably_free_instances,
        }
    }

    pub fn factor_sources(&self) -> IndexSet<HDFactorSource> {
        self.factor_sources.clone().into_iter().collect()
    }

    pub fn recovered_accounts(&self) -> IndexSet<Account> {
        self.recovered_accounts.clone().into_iter().collect()
    }
}

impl TryFrom<DerivationAndAnalysis> for DerivationAndAnalysisAccountRecoveryScan {
    type Error = CommonError;

    fn try_from(value: DerivationAndAnalysis) -> Result<Self> {
        let factor_sources = value.all_factor_sources();
        let recovered_entities = value
            .known_taken_instances
            .recovered_unsecurified_entities
            .merge_with_securified(value.known_taken_instances.recovered_securified_entities);

        let recovered_accounts = recovered_entities
            .into_iter()
            .map(Account::try_from)
            .collect::<Result<IndexSet<_>>>()?;

        Ok(Self::new(
            factor_sources,
            recovered_accounts,
            value.known_taken_instances.unrecovered_securified_entities,
            value.probably_free_instances,
        ))
    }
}
