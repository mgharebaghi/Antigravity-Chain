//! # Sharding Module
//!
//! Handles shard assignment and management for horizontal scaling.
//! The number of shards scales dynamically with validator count.

use super::node_state::NodeConsensusStatus;
use super::Consensus;
use sha2::{Digest, Sha256};

// =============================================================================
// Sharding Logic
// =============================================================================

impl Consensus {
    /// Calculates the number of active shards based on validator population.
    /// Formula: max(1, validators / 50)
    pub fn calculate_active_shards(&self) -> u16 {
        let validator_count = self.nodes.len();
        if validator_count < 50 {
            1
        } else {
            (validator_count / 50) as u16
        }
    }

    /// Deterministically assigns a peer to a specific shard.
    /// The assignment changes each epoch to balance load.
    pub fn get_assigned_shard(&self, peer_id: &str, epoch: u64) -> u16 {
        let active_shards = self.calculate_active_shards();
        let mut hasher = Sha256::new();
        hasher.update(peer_id.as_bytes());
        hasher.update(&epoch.to_le_bytes());
        let result = hasher.finalize();
        let hash_val = ((result[0] as u16) << 8) | (result[1] as u16);
        hash_val % active_shards
    }

    /// Generates the VDF challenge for a peer
    pub fn get_vdf_challenge(&self, peer_id: &String) -> String {
        // Challenge = SHA256(PeerID + "Patience")
        let mut hasher = Sha256::new();
        hasher.update(peer_id.as_bytes());
        hasher.update(b"Patience");
        hex::encode(hasher.finalize())
    }

    /// Gets comprehensive status for a node in the consensus
    pub fn get_node_status(&self, peer_id: &String) -> NodeConsensusStatus {
        let node = match self.nodes.get(peer_id) {
            Some(n) => n,
            None => {
                return NodeConsensusStatus {
                    state: "Connecting".to_string(),
                    queue_position: 0,
                    estimated_blocks: 0,
                    patience_progress: 0.0,
                    remaining_seconds: 0,
                    shard_id: 0,
                    is_slot_leader: false,
                }
            }
        };

        let current_slot = self.current_slot();
        let current_epoch = self.current_epoch();
        let assigned_shard = self.get_assigned_shard(peer_id, current_epoch);

        let shard_leader = self.get_shard_leader(assigned_shard, current_slot);
        let is_leader = shard_leader.as_ref() == Some(peer_id);

        let uptime = node.current_uptime();
        let quarantine_duration = self.get_quarantine_duration();
        let in_quarantine = uptime < quarantine_duration;
        let eligible = self.is_eligible_for_leadership(peer_id);

        if is_leader {
            NodeConsensusStatus {
                state: "Leader".to_string(),
                queue_position: 0,
                estimated_blocks: 0,
                patience_progress: 1.0,
                remaining_seconds: 0,
                shard_id: assigned_shard as u32,
                is_slot_leader: true,
            }
        } else if !eligible {
            // Patience Mode / Quarantine
            // Only if NOT active. If active, eligible returns true, so we don't go here.
            let progress = if quarantine_duration > 0 {
                (uptime as f64 / quarantine_duration as f64).min(1.0)
            } else {
                1.0
            };

            NodeConsensusStatus {
                state: "Patience".to_string(),
                queue_position: if in_quarantine { 999 } else { 500 },
                estimated_blocks: 0,
                patience_progress: progress as f32,
                remaining_seconds: quarantine_duration.saturating_sub(uptime),
                shard_id: assigned_shard as u32,
                is_slot_leader: false,
            }
        } else {
            // Verified Queue
            // Calculate real queue position
            let mut eligible_validators: Vec<String> = self
                .nodes
                .iter()
                .filter(|(pid, _)| {
                    self.get_assigned_shard(pid, current_epoch) == assigned_shard as u16
                        && self.is_eligible_for_leadership(pid)
                })
                .map(|(pid, _)| pid.clone())
                .collect();
            eligible_validators.sort();

            let my_index = eligible_validators
                .iter()
                .position(|x| x == peer_id)
                .unwrap_or(0);
            let total = eligible_validators.len();
            let current_mod = (current_slot as usize) % total;

            // Distance in slots
            let distance = if my_index >= current_mod {
                my_index - current_mod
            } else {
                (total - current_mod) + my_index
            };

            NodeConsensusStatus {
                state: "Queue".to_string(),
                queue_position: distance as u32,
                estimated_blocks: 0, // removed placeholder
                patience_progress: 1.0,
                remaining_seconds: distance as u64 * Self::SLOT_DURATION,
                shard_id: assigned_shard as u32,
                is_slot_leader: false,
            }
        }
    }
}
