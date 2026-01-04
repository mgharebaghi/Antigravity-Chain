//! # Node Control Commands
//!
//! Commands for controlling node lifecycle and getting status.

use crate::AppState;
use std::sync::atomic::Ordering;
use tauri::State;

#[derive(serde::Serialize, Clone)]
pub struct ConsensusStateResponse {
    pub node_status: crate::consensus::NodeConsensusStatus,
    pub next_leaders: Vec<(u64, Option<String>)>,
}

#[tauri::command]
pub fn cmd_get_consensus_state(
    state: State<'_, AppState>,
) -> Result<ConsensusStateResponse, String> {
    let consensus = state.consensus.lock().map_err(|e| e.to_string())?;
    let local_peer_id = consensus.local_peer_id.clone().unwrap_or_default();
    let mut status = consensus.get_node_status(&local_peer_id);

    // Force 'Connecting' only if relay is not up
    if !state.relay_connected.load(Ordering::Relaxed) && status.state == "Connecting" {
        status.state = "Connecting".to_string();
    }

    let current_slot = consensus.current_slot();
    let shard_id = status.shard_id as u16;
    let leaders = consensus.get_future_leaders(current_slot, 10, shard_id);

    Ok(ConsensusStateResponse {
        node_status: status,
        next_leaders: leaders,
    })
}

#[tauri::command]
pub fn cmd_reset_chain_data(state: State<'_, AppState>) -> Result<(), String> {
    state.storage.reset_blocks().map_err(|e| e.to_string())?;
    state.storage.start_new_run().map_err(|e| e.to_string())?;
    state.mempool.clear();
    state.chain_index.store(0, Ordering::Relaxed);
    state.mined_by_me_count.store(0, Ordering::Relaxed);

    match state.consensus.lock() {
        Ok(mut c) => {
            c.nodes.clear();
            if let Some(pid) = &c.local_peer_id {
                let pid_clone = pid.clone();
                c.register_node(pid_clone);
            }
        }
        Err(e) => return Err(e.to_string()),
    }

    let _ = state.is_synced.store(false, Ordering::Relaxed);
    Ok(())
}
