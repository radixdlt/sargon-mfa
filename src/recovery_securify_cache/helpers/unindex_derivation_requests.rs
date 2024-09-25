use crate::prelude::*;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct QuantifiedUnindexDerivationRequests {
    hidden: HiddenConstructor,
    requests: Vec<QuantifiedUnindexDerivationRequest>,
}

impl FromIterator<QuantifiedUnindexDerivationRequest> for QuantifiedUnindexDerivationRequests {
    fn from_iter<I: IntoIterator<Item = QuantifiedUnindexDerivationRequest>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl IntoIterator for QuantifiedUnindexDerivationRequests {
    type Item = QuantifiedUnindexDerivationRequest;
    type IntoIter = <IndexSet<QuantifiedUnindexDerivationRequest> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.requests().into_iter()
    }
}

impl QuantifiedUnindexDerivationRequests {
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }
    pub fn new(requests: IndexSet<QuantifiedUnindexDerivationRequest>) -> Self {
        Self {
            hidden: HiddenConstructor,
            requests: requests.into_iter().collect(),
        }
    }

    pub fn requests(&self) -> IndexSet<QuantifiedUnindexDerivationRequest> {
        self.requests.clone().into_iter().collect()
    }
}
