use crate::prelude::*;

pub struct NextDerivationIndexFromProfile {
    profile_snapshot: Profile,
}
impl NextDerivationIndexFromProfile {
    pub fn read_next(
        &self,
        unindexed_request: UnquantifiedUnindexDerivationRequest,
    ) -> HDPathValue {
        todo!()
    }
}

#[derive(Default)]
pub struct EphemeralLocalIndexOffsets {
    local_offsets: RwLock<HashMap<UnquantifiedUnindexDerivationRequest, usize>>,
}
impl EphemeralLocalIndexOffsets {
    pub fn write_next(&self, request: UnquantifiedUnindexDerivationRequest) -> usize {
        let mut write = self.local_offsets.try_write().unwrap();
        let entry = write.entry(request).or_insert(0);
        let next = entry.clone();
        *entry += 1;
        next
    }
}

pub struct NextIndexAssignerWithEphemeralLocalOffsets {
    ephemeral_local_offsets: EphemeralLocalIndexOffsets,
    profile_offsets: Option<NextDerivationIndexFromProfile>,
}

impl NextIndexAssignerWithEphemeralLocalOffsets {
    pub fn new(profile_snapshot: impl Into<Option<Profile>>) -> Self {
        let profile_snapshot = profile_snapshot.into();
        Self {
            profile_offsets: profile_snapshot.map(|p| NextDerivationIndexFromProfile {
                profile_snapshot: p,
            }),
            ephemeral_local_offsets: EphemeralLocalIndexOffsets::default(),
        }
    }

    pub fn next(&self, unindexed_request: UnquantifiedUnindexDerivationRequest) -> HDPathValue {
        let from_profile = self
            .profile_offsets
            .as_ref()
            .map(|p| p.read_next(unindexed_request.clone()))
            .unwrap_or(0);

        let from_local = self.ephemeral_local_offsets.write_next(unindexed_request);

        from_profile + from_local as u32
    }
}
