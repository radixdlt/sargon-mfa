use crate::prelude::*;

pub struct NextDerivationIndexAnalyzer {
    local_offsets: HashMap<UnquantifiedUnindexDerivationRequest, usize>,
    profile_snapshot: Option<Profile>,
}

impl NextDerivationIndexAnalyzer {
    pub fn new(profile_snapshot: impl Into<Option<Profile>>) -> Self {
        Self {
            profile_snapshot: profile_snapshot.into(),
            local_offsets: HashMap::new(),
        }
    }

    pub fn next(&self, _unindexed_request: UnquantifiedUnindexDerivationRequest) -> HDPathValue {
        todo!()
    }
}
