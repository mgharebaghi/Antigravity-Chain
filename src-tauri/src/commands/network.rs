use crate::state::AppState;
use tauri::State;

#[derive(serde::Serialize, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub trust_score: f64,
    pub is_verified: bool,
    pub latency: u64,
    pub addresses: Vec<String>,
}

#[tauri::command]
pub fn get_network_info(state: State<'_, AppState>) -> Vec<PeerInfo> {
    let consensus = state.consensus.lock().unwrap();
    consensus
        .nodes
        .values()
        .map(|n| PeerInfo {
            peer_id: n.peer_id.clone(),
            trust_score: n.trust_score,
            is_verified: n.is_verified,
            latency: 0,
            addresses: n.addresses.clone(),
        })
        .collect()
}

#[derive(serde::Serialize)]
pub struct SelfNodeInfo {
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub shard_id: u16,
    pub total_shards: u16,
    pub shard_tps_limit: u64,
    pub global_tps_capacity: u64,
}

#[tauri::command]
pub fn get_self_node_info(state: State<'_, AppState>) -> Option<SelfNodeInfo> {
    let consensus = state.consensus.lock().unwrap();
    consensus.local_peer_id.as_ref().map(|id| {
        let addresses = consensus
            .nodes
            .get(id)
            .map(|n| n.addresses.clone())
            .unwrap_or_default();

        // AHSP Info
        let total_shards = consensus.calculate_active_shards();
        let shard_id = consensus.get_assigned_shard(id, 0);
        // TPS = Tx Per Block / Block Time
        let shard_tps_limit =
            crate::utils::constants::MAX_TXS_PER_BLOCK / crate::utils::constants::TARGET_BLOCK_TIME;
        let global_tps_capacity = total_shards as u64 * shard_tps_limit;

        SelfNodeInfo {
            peer_id: id.clone(),
            addresses,
            shard_id,
            total_shards,
            shard_tps_limit,
            global_tps_capacity,
        }
    })
}
