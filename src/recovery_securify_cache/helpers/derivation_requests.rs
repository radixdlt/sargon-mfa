use crate::prelude::*;
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
