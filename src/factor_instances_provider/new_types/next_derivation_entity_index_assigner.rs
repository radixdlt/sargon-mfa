use crate::prelude::*;

pub struct NextDerivationEntityIndexProfileAnalyzingAssigner {
    network_id: NetworkID,
    /// might be empty
    accounts_on_network: IndexSet<Account>,
    /// might be empty
    personas_on_network: IndexSet<Persona>,
}
impl NextDerivationEntityIndexProfileAnalyzingAssigner {
    pub fn new(network_id: NetworkID, profile: Option<Profile>) -> Self {
        Self {
            network_id,
            accounts_on_network: profile
                .as_ref()
                .map(|p| p.accounts_on_network(network_id))
                .unwrap_or_default(),
            personas_on_network: profile
                .as_ref()
                .map(|p| p.personas_on_network(network_id))
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug)]
pub struct NextDerivationEntityIndexWithLocalOffsets {
    network_id: NetworkID,
    local_offsets: HashMap<FactorSourceIDFromHash, u32>,
}
impl NextDerivationEntityIndexWithLocalOffsets {
    pub fn empty(network_id: NetworkID) -> Self {
        Self {
            network_id,
            local_offsets: HashMap::new(),
        }
    }
}

pub struct NextDerivationEntityIndexAssigner {
    network_id: NetworkID,
    profile_analyzing: NextDerivationEntityIndexProfileAnalyzingAssigner,
    local_offsets: NextDerivationEntityIndexWithLocalOffsets,
}
impl NextDerivationEntityIndexAssigner {
    pub fn new(network_id: NetworkID, profile: Option<Profile>) -> Self {
        let profile_analyzing =
            NextDerivationEntityIndexProfileAnalyzingAssigner::new(network_id, profile);
        Self {
            network_id,
            profile_analyzing,
            local_offsets: NextDerivationEntityIndexWithLocalOffsets::empty(network_id),
        }
    }
    pub fn next_account_veci(&self, factor_source_id: FactorSourceIDFromHash) -> HDPathComponent {
        todo!()
    }
}
