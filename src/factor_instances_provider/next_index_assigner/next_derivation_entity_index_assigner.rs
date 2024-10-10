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
    ephemeral_offsets: NextDerivationEntityIndexWithEphemeralOffsets,
}

impl NextDerivationEntityIndexAssigner {
    pub fn new(network_id: NetworkID, profile: Option<Profile>) -> Self {
        let profile_analyzing =
            NextDerivationEntityIndexProfileAnalyzingAssigner::new(network_id, profile);
        Self {
            network_id,
            profile_analyzing,
            ephemeral_offsets: NextDerivationEntityIndexWithEphemeralOffsets::default(),
        }
    }

    pub fn next(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        index_agnostic_path: IndexAgnosticPath,
        cache_offset: OffsetFromCache,
    ) -> Result<HDPathComponent> {
        let default_index = HDPathComponent::new_with_key_space_and_base_index(
            index_agnostic_path.key_space,
            U30::new(0).unwrap(),
        );

        // Must update local offset based on values found in cache.
        // Imagine we are securifying 3 accounts with a single FactorSource
        // `L` to keep things simple, profile already contains 28 securified
        // accounts controlled by `L`, with the highest entity index is `27^`
        // We look for keys in the cache for `L` and we find 2, with entity
        // indices `[28^, 29^]`, so we need to derive 2 (+CACHE_FILLING_QUANTITY)
        // more keys. The next index assigner will correctly use a profile based offset
        // of 28^ for `L`, since it found the max value `28^` in Profile controlled by `L`.
        // If we would use `next` now, the index would be `next = max + 1`, and
        // `max = offset_from_profile + ephemeral_offset` = `28^ + 0^` = 28^.
        // Which is wrong! Since the cache contains `28^` and `29^`, we should
        // derive `2 (+CACHE_FILLING_QUANTITY)` starting at `30^`.
        let maybe_next_from_cache = cache_offset.next(factor_source_id, index_agnostic_path)?;

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
