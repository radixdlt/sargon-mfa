use crate::prelude::*;
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FullDerivationRequests {
    hidden: HiddenConstructor,
    requests: Vec<DerivationPath>,
}

impl From<FullDerivationRequests> for IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>> {
    fn from(_val: FullDerivationRequests) -> Self {
        todo!()
    }
}

impl FromIterator<DerivationPath> for FullDerivationRequests {
    fn from_iter<I: IntoIterator<Item = DerivationPath>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl IntoIterator for FullDerivationRequests {
    type Item = DerivationPath;
    type IntoIter = <IndexSet<DerivationPath> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.requests().into_iter()
    }
}

impl FullDerivationRequests {
    pub fn new(requests: IndexSet<DerivationPath>) -> Self {
        Self {
            hidden: HiddenConstructor,
            requests: requests.into_iter().collect(),
        }
    }

    pub fn requests(&self) -> IndexSet<DerivationPath> {
        self.requests.clone().into_iter().collect()
    }
}
