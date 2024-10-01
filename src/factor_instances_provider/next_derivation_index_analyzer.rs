use crate::prelude::*;

impl HierarchicalDeterministicFactorInstance {
    fn matches(&self, request: &UnquantifiedUnindexDerivationRequest) -> bool {
        self.factor_source_id() == request.factor_source_id
            && self.derivation_path().matches(request)
    }
}
impl DerivationPath {
    fn matches(&self, request: &UnquantifiedUnindexDerivationRequest) -> bool {
        self.network_id == request.network_id
            && self.entity_kind == request.entity_kind
            && self.key_kind == request.key_kind
            && self.index.key_space() == request.key_space
    }
}

impl MatrixOfFactorInstances {
    fn highest_derivation_path_index(
        &self,
        request: &UnquantifiedUnindexDerivationRequest,
    ) -> Option<HDPathComponent> {
        self.all_factors()
            .into_iter()
            .filter(|f| f.matches(request))
            .map(|f| f.derivation_path().index)
            .max()
    }
}
impl SecurifiedEntityControl {
    fn highest_derivation_path_index(
        &self,
        request: &UnquantifiedUnindexDerivationRequest,
    ) -> Option<HDPathComponent> {
        self.matrix.highest_derivation_path_index(request)
    }
}
impl SecurifiedEntity {
    fn highest_derivation_path_index(
        &self,
        request: &UnquantifiedUnindexDerivationRequest,
    ) -> Option<HDPathComponent> {
        self.securified_entity_control()
            .highest_derivation_path_index(request)
    }
}

pub struct NextDerivationIndexFromProfile {
    profile_snapshot: Profile,
}
impl NextDerivationIndexFromProfile {
    pub fn read_next(
        &self,
        unindexed_request: UnquantifiedUnindexDerivationRequest,
    ) -> Option<HDPathComponent> {
        let entity_kind = unindexed_request.entity_kind;
        let network_id = unindexed_request.network_id;
        let key_space = unindexed_request.key_space;
        let max_or_none = match key_space {
            KeySpace::Securified => {
                let max_or_none: Option<HDPathComponent> = {
                    let all_securified_in_profile = self
                        .profile_snapshot
                        .get_securified_entities_of_kind_on_network(entity_kind, network_id);
                    all_securified_in_profile
                        .into_iter()
                        .flat_map(|e: SecurifiedEntity| {
                            e.highest_derivation_path_index(&unindexed_request)
                        })
                        .max()
                };
                max_or_none
            }
            KeySpace::Unsecurified => {
                let all_unsecurified_in_profile = self
                    .profile_snapshot
                    .get_unsecurified_entities_of_kind_on_network(entity_kind, network_id);

                all_unsecurified_in_profile
                    .into_iter()
                    .map(|x: UnsecurifiedEntity| x.veci().factor_instance())
                    .filter(|fi| fi.matches(&unindexed_request))
                    .map(|fi| fi.derivation_path().index)
                    .max()
            }
        };

        max_or_none.map(|i| i.add_one())
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
        let next = *entry;
        *entry += 1;
        next
    }

    fn increase_offset(&self, with_range: &DerivationRequestWithRange) {
        let mut write = self.local_offsets.try_write().unwrap();
        let key = UnquantifiedUnindexDerivationRequest::from(with_range.clone());
        let entry = write.entry(key.clone()).or_insert(0);
        let offset = with_range.end_base_index() as usize;
        println!("🐥👻 set offset to: {}, for request: {:?}", offset, key);
        *entry = offset;
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

    pub fn increase_local_offset(&self, with_range: &DerivationRequestWithRange) {
        self.ephemeral_local_offsets.increase_offset(with_range)
    }

    pub fn next(&self, unindexed_request: UnquantifiedUnindexDerivationRequest) -> HDPathComponent {
        let default = match unindexed_request.key_space {
            KeySpace::Securified => HDPathComponent::securifying_base_index(0),
            KeySpace::Unsecurified => HDPathComponent::unsecurified_hardening_base_index(0),
        };
        let from_profile = self
            .profile_offsets
            .as_ref()
            .and_then(|p| p.read_next(unindexed_request.clone()))
            .unwrap_or(default);

        let from_local = self
            .ephemeral_local_offsets
            .write_next(unindexed_request.clone());

        let next = from_profile.add_n(from_local as HDPathValue);
        println!(
            "🐥 next: {:?}, for request: {:?}, local: {}, from_profile: {}",
            next, unindexed_request, from_local, from_profile
        );
        next
    }
}
