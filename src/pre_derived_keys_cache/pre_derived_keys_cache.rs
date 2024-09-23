use crate::prelude::*;

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

impl DerivationRequests {
    pub fn next(&self) -> Self {
        todo!("figure this out");
    }
    pub fn derivation_paths(&self) -> IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>> {
        todo!("figure this out");
    }
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
    pub async fn load(&self, requests: &DerivationRequests) -> Result<CacheOutcome> {
        Ok(CacheOutcome::empty(requests))
    }
}

/*

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct CachedFactorInstances(pub IndexMap<DerivationRequestInKeySpace, FactorInstances>);

#[derive(Debug, Default)]
pub struct PreDerivedKeysCache {
    factor_instances_for_requests: RwLock<CachedFactorInstances>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheLoadOutcome {
    pub requests: IndexSet<DerivationRequestInKeySpace>,
    /// Response to `requests`
    pub factor_instances: CachedFactorInstances,
    /// If `factor_instances` response satisfies all `requests`
    pub is_satisfying_all_requests: bool,
    /// If we should derive more factor instances after this response, either
    /// because we consumed the last factor instance in the cache or because
    /// we did not fully satisfy all requests.
    pub should_derive_more: bool,
}
impl CacheLoadOutcome {
    pub fn is_empty(&self) -> bool {
        self.factor_instances.0.is_empty()
    }
}

impl PreDerivedKeysCache {
    fn get(&self, key: &DerivationRequestInKeySpace) -> Option<FactorInstances> {
        self.factor_instances_for_requests
            .try_read()
            .unwrap()
            .0
            .get(key)
            .cloned()
    }

    pub async fn load(
        &self,
        requests: IndexSet<DerivationRequestInKeySpace>,
    ) -> Result<CacheLoadOutcome> {
        let mut found = CachedFactorInstances::default();
        let mut failure = false;
        for key in requests.iter() {
            let Some(loaded) = self.get(key) else {
                failure = true;
                continue;
            };
            found.0.insert(key.clone(), loaded);
        }

        Ok(CacheLoadOutcome {
            requests,
            factor_instances: found,
            is_satisfying_all_requests: !failure,
            should_derive_more: failure,
        })
    }
}
impl PreDerivedKeysCache {
    fn with_map(map: IndexMap<DerivationRequestInKeySpace, FactorInstances>) -> Self {
        Self {
            factor_instances_for_requests: RwLock::new(CachedFactorInstances(map)),
        }
    }
    pub fn new(probably_free_factor_instances: ProbablyFreeFactorInstances) -> Self {
        let map = probably_free_factor_instances
            .0
            .into_iter()
            .into_group_map_by(|x| x.derivation_in_key_space())
            .into_iter()
            .map(|(k, v)| (k, FactorInstances::from(v)))
            .collect::<IndexMap<DerivationRequestInKeySpace, FactorInstances>>();

        Self::with_map(map)
    }
    pub fn empty() -> Self {
        Self::with_map(IndexMap::default())
    }
}

*/
