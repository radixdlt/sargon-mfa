use crate::prelude::*;

/// Essentially a map of `NextDerivationEntityIndexWithEphemeralOffsetsForFactorSource`
/// ephemeral offsets used by `NextDerivationEntityIndexAssigner`
/// to add ephemeral offsets to next index calculations.
#[derive(Default, Debug)]
pub struct NextDerivationEntityIndexWithEphemeralOffsets {
    ephemeral_offsets_per_factor_source: RwLock<
        HashMap<
            FactorSourceIDFromHash,
            NextDerivationEntityIndexWithEphemeralOffsetsForFactorSource,
        >,
    >,
}

impl NextDerivationEntityIndexWithEphemeralOffsets {
    /// Reserves the next ephemeral offset for `factor_source_id` for `agnostic_path`.
    /// Consecutive calls always returns a new value, which is `previous + 1` (given
    /// the same `factor_source_id, agnostic_path`)
    pub fn reserve(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        agnostic_path: IndexAgnosticPath,
    ) -> HDPathValue {
        let mut binding = self.ephemeral_offsets_per_factor_source.write().unwrap();
        if let Some(for_factor) = binding.get_mut(&factor_source_id) {
            for_factor.reserve(factor_source_id, agnostic_path)
        } else {
            let new = NextDerivationEntityIndexWithEphemeralOffsetsForFactorSource::empty(
                factor_source_id,
            );
            let next = new.reserve(factor_source_id, agnostic_path);
            binding.insert(factor_source_id, new);
            next
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = NextDerivationEntityIndexWithEphemeralOffsets;

    #[test]
    fn test_contiguous() {
        let sut = Sut::default();
        let n = 4;
        let indices = (0..n)
            .map(|_| {
                sut.reserve(
                    FactorSourceIDFromHash::fs0(),
                    DerivationPreset::AccountVeci
                        .index_agnostic_path_on_network(NetworkID::Mainnet),
                )
            })
            .collect::<IndexSet<_>>();
        assert_eq!(indices, IndexSet::<u32>::from_iter([0, 1, 2, 3]));
    }

    #[test]
    fn test_zero_for_each_factor_sources_first_time() {
        let sut = Sut::default();
        let fsids = HDFactorSource::all()
            .into_iter()
            .map(|f| f.factor_source_id())
            .collect_vec();
        let indices = fsids
            .clone()
            .into_iter()
            .map(|fsid| {
                sut.reserve(
                    fsid,
                    DerivationPreset::AccountVeci
                        .index_agnostic_path_on_network(NetworkID::Mainnet),
                )
            })
            .collect_vec();
        assert_eq!(indices, vec![0; fsids.len()]);
    }

    #[test]
    fn test_zero_for_each_derivation_preset() {
        let sut = Sut::default();
        let derivation_presets = DerivationPreset::all();
        let indices = derivation_presets
            .clone()
            .into_iter()
            .map(|preset| {
                sut.reserve(
                    FactorSourceIDFromHash::fs0(),
                    preset.index_agnostic_path_on_network(NetworkID::Mainnet),
                )
            })
            .collect_vec();
        assert_eq!(indices, vec![0; derivation_presets.len()]);
    }

    #[test]
    fn test_zero_for_each_network() {
        let sut = Sut::default();
        let network_ids = NetworkID::all();
        let indices = network_ids
            .clone()
            .into_iter()
            .map(|network_id| {
                sut.reserve(
                    FactorSourceIDFromHash::fs0(),
                    DerivationPreset::AccountMfa.index_agnostic_path_on_network(network_id),
                )
            })
            .collect_vec();
        assert_eq!(indices, vec![0; network_ids.len()]);
    }
}
