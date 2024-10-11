use crate::prelude::*;

/// An assigner of derivation entity indices, used by the FactorInstancesProvider
/// to map `IndexAgnosticPath` -> `DerivationPath` for some FactorSource on
/// some NetworkID.
///
/// This assigner works with the:
/// * cache (indirectly, via the `OffsetFromCache` parameter on `next` [should probably clean up])
/// * profile
/// * local offsets
///
/// More specifically the assigner's `next` method performs approximately this
/// operation:
///
/// ```ignore
/// pub fn next(
///    &mut self,
///    fs_id: FactorSourceIDFromHash,
///    path: IndexAgnosticPath,
///    cache_offset: OffsetFromCache,
/// ) -> Result<HDPathComponent> {
///     let next_from_cache = offset_from_cache.next(fs_id, path).unwrap_or(0);
///     let next_from_profile = self.profile_analyzing.next(fs_id, path).unwrap_or(0);
///     
///     let max_index = std::cmp::max(next_from_profile, next_from_cache);
///     let ephemeral_offset = self.ephemeral_offsets.reserve()
///
///     max_index + ephemeral_offset
/// ```
pub struct NextDerivationEntityIndexAssigner {
    #[allow(dead_code)]
    network_id: NetworkID,
    profile_analyzing: NextDerivationEntityIndexProfileAnalyzingAssigner,
    cache_analyzing: NextDerivationEntityIndexCacheAnalyzingAssigner,
    ephemeral_offsets: NextDerivationEntityIndexWithEphemeralOffsets,
}

pub struct NextDerivationEntityIndexCacheAnalyzingAssigner {
    cache: FactorInstancesCache,
}
impl NextDerivationEntityIndexCacheAnalyzingAssigner {
    pub fn cache(&self) -> FactorInstancesCache {
        self.cache.clone()
    }

    pub fn new(cache: FactorInstancesCache) -> Self {
        Self { cache }
    }
    pub fn next(
        &self,
        agnostic_path: IndexAgnosticPath,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Result<Option<HDPathComponent>> {
        let cache = self.cache.max_index_for(agnostic_path, factor_source_id);
        Ok(cache)
    }
}

impl NextDerivationEntityIndexAssigner {
    pub fn new(
        network_id: NetworkID,
        profile: Option<Profile>,
        cache: FactorInstancesCache,
    ) -> Self {
        let profile_analyzing =
            NextDerivationEntityIndexProfileAnalyzingAssigner::new(network_id, profile);
        let cache_analyzing = NextDerivationEntityIndexCacheAnalyzingAssigner::new(cache);
        let ephemeral_offsets = NextDerivationEntityIndexWithEphemeralOffsets::default();
        Self {
            network_id,
            profile_analyzing,
            cache_analyzing,
            ephemeral_offsets,
        }
    }

    pub fn cache(&self) -> FactorInstancesCache {
        self.cache_analyzing.cache()
    }

    pub fn next(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        index_agnostic_path: IndexAgnosticPath,
    ) -> Result<HDPathComponent> {
        let default_index = HDPathComponent::new_with_key_space_and_base_index(
            index_agnostic_path.key_space,
            U30::new(0).unwrap(),
        );

        let maybe_next_from_cache = self
            .cache_analyzing
            .next(index_agnostic_path, factor_source_id)?;

        let next_from_cache = maybe_next_from_cache.unwrap_or(default_index);
        let local = self
            .ephemeral_offsets
            .reserve(factor_source_id, index_agnostic_path);

        let maybe_next_from_profile = self
            .profile_analyzing
            .next(index_agnostic_path, factor_source_id)?;

        let next_from_profile = maybe_next_from_profile.unwrap_or(default_index);

        let max_index = std::cmp::max(next_from_profile, next_from_cache);

        max_index.add_n(local)
    }
}
