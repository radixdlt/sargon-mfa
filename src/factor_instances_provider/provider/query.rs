use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstancesQuery {
    /// Uses the "next" derivation entity index for the derivation path
    /// The network is already known by the FactorInstancesProvider
    AccountVeci {
        /// The factor to use to derive the instance, typically the main BDFS.
        factor_source: HDFactorSource,
    },

    /// Uses a range of derivation paths, starting at the next, per factor source
    /// The network is already known by the FactorInstancesProvider
    ///
    /// N.B. we COULD have made this more advance/complex by passing a:
    /// `number_of_instances_for_each_factor_source: HashMap<HDFactorSource, usize>`
    /// but we don't need that complexity for now, we assume we want to get
    /// `number_of_instances_per_factor_source` for **each** factor source.
    ///
    /// `number_of_instances_per_factor_source` should be interpreted as
    /// `number_of_accounts_to_securify`.
    AccountMfa {
        number_of_instances_per_factor_source: usize,
        factor_sources: IndexSet<HDFactorSource>,
    },
    // PreDeriveKeysForFactorSource
}

impl InstancesQuery {
    pub fn factor_sources(&self) -> IndexSet<HDFactorSource> {
        match self {
            InstancesQuery::AccountVeci { factor_source } => IndexSet::just(factor_source.clone()),
            InstancesQuery::AccountMfa {
                factor_sources,
                number_of_instances_per_factor_source: _,
            } => factor_sources.clone(),
        }
    }
}
