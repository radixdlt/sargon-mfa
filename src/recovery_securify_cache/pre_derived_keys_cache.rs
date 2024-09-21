use crate::{prelude::*, recovery_securify_cache::factor_instances};

#[derive(Debug, Default)]
pub struct PreDerivedKeysCache;

impl PreDerivedKeysCache {
    pub fn new(probably_free_factor_instances: ProbablyFreeFactorInstances) -> Self {
        warn!(
            "TODO: Implement PreDerivedKeysCache::new, IGNORED {:?}",
            probably_free_factor_instances
        );
        Self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct DerivationRequests {
    hidden: HiddenConstructor,
    requests: Vec<DerivationRequest>,
}

impl FromIterator<DerivationRequest> for DerivationRequests {
    fn from_iter<I: IntoIterator<Item = DerivationRequest>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl IntoIterator for DerivationRequests {
    type Item = DerivationRequest;
    type IntoIter = <IndexSet<DerivationRequest> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.requests().into_iter()
    }
}

impl DerivationRequests {
    pub fn new(requests: IndexSet<DerivationRequest>) -> Self {
        Self {
            hidden: HiddenConstructor,
            requests: requests.into_iter().collect(),
        }
    }
    pub fn requests(&self) -> IndexSet<DerivationRequest> {
        self.requests.clone().into_iter().collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CacheOutcome {
    #[allow(dead_code)]
    hidden_constructor: HiddenConstructor,
    /// The requests
    pub requests: DerivationRequests,
    /// If the cache has contains more free instances after having satisfied the requests.
    pub has_spare_capacity: bool,
    pub fully_satisfied: bool,
    pub factor_instances: FactorInstances,
}
impl CacheOutcome {
    pub fn empty(requests: &DerivationRequests) -> Self {
        Self {
            hidden_constructor: HiddenConstructor,
            requests: requests.clone(),
            has_spare_capacity: false,
            fully_satisfied: false,
            factor_instances: FactorInstances::default(),
        }
    }
    pub fn partial(
        requests: &DerivationRequests,
        factor_instances: impl Into<FactorInstances>,
    ) -> Self {
        let factor_instances = factor_instances.into();
        assert!(requests.partially_satisfied_by(&factor_instances));
        Self {
            hidden_constructor: HiddenConstructor,
            requests: requests.clone(),
            has_spare_capacity: false,
            fully_satisfied: false,
            factor_instances,
        }
    }
    fn full_specifying_spare_capacity(
        has_spare_capacity: bool,
        requests: &DerivationRequests,
        factor_instances: impl Into<FactorInstances>,
    ) -> Self {
        let factor_instances = factor_instances.into();
        assert!(requests.fully_satisfied_by(&factor_instances));
        Self {
            hidden_constructor: HiddenConstructor,
            requests: requests.clone(),
            fully_satisfied: true,
            has_spare_capacity,
            factor_instances,
        }
    }
    pub fn full_last(
        requests: &DerivationRequests,
        factor_instances: impl Into<FactorInstances>,
    ) -> Self {
        Self::full_specifying_spare_capacity(false, requests, factor_instances)
    }
    pub fn full_with_spare(
        requests: &DerivationRequests,
        factor_instances: impl Into<FactorInstances>,
    ) -> Self {
        Self::full_specifying_spare_capacity(true, requests, factor_instances)
    }
}

impl PreDerivedKeysCache {
    pub async fn load(&self, _requests: &DerivationRequests) -> Result<CacheOutcome> {
        todo!()
    }
}
