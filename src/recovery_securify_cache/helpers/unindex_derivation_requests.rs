use crate::prelude::*;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct UnindexDerivationRequests {
    hidden: HiddenConstructor,
    requests: Vec<UnindexDerivationRequest>,
}

impl FromIterator<UnindexDerivationRequest> for UnindexDerivationRequests {
    fn from_iter<I: IntoIterator<Item = UnindexDerivationRequest>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl IntoIterator for UnindexDerivationRequests {
    type Item = UnindexDerivationRequest;
    type IntoIter = <IndexSet<UnindexDerivationRequest> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.requests().into_iter()
    }
}

impl UnindexDerivationRequests {
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }
    pub fn new(requests: IndexSet<UnindexDerivationRequest>) -> Self {
        Self {
            hidden: HiddenConstructor,
            requests: requests.into_iter().collect(),
        }
    }

    pub fn requests(&self) -> IndexSet<UnindexDerivationRequest> {
        self.requests.clone().into_iter().collect()
    }
}
