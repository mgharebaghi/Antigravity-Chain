//! # Node State Module
//!
//! Represents the state of a validator node in the consensus mechanism.
//! Tracks activation status, trust scores, and mining eligibility.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// =============================================================================
// NodeState - Tracks validator state for consensus
// =============================================================================

/// Represents the state of a validator node in the network.
/// Once a node is activated (via VDF proof + quarantine), it remains eligible
/// for leadership unless explicitly slashed below the trust threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    /// Unique identifier for the peer
    pub peer_id: String,

    /// Unix timestamp when the node first joined the network
    pub join_time: u64,

    /// Trust score (0.0 - 1.0). Nodes below 0.01 lose active status.
    pub trust_score: f64,

    /// VDF proof submitted by the node for Proof of Patience
    pub vdf_proof: Option<String>,

    /// Whether the node has verified their VDF proof
    pub is_verified: bool,

    /// Whether the node is currently active in consensus
    pub is_active: bool,

    /// Unix timestamp when node was activated (None = never activated)
    /// Once set, the node remains eligible regardless of quarantine changes
    pub activated_at: Option<u64>,

    /// Number of slots this node has missed as leader
    pub missed_slots: u64,

    /// Multiaddresses for this peer
    pub addresses: Vec<String>,

    /// Whether mining is enabled for this node.
    /// If false, this node will not participate in leader election.
    pub mining_active: bool,
}

impl NodeState {
    /// Creates a new NodeState for a peer joining the network
    pub fn new(peer_id: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        NodeState {
            peer_id,
            join_time: now,
            trust_score: 0.1,
            vdf_proof: None,
            is_verified: false,
            is_active: false,
            activated_at: None,
            missed_slots: 0,
            addresses: Vec::new(),
            mining_active: true, // Default to ready for mining
        }
    }

    /// Returns how long this node has been online (in seconds)
    pub fn current_uptime(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now.saturating_sub(self.join_time)
    }

    /// Activates the node, recording the activation timestamp
    pub fn activate(&mut self) {
        if self.activated_at.is_none() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            self.activated_at = Some(now);
            self.is_active = true;
            log::info!("Node {} activated at timestamp {}", self.peer_id, now);
        }
    }

    /// Checks if this node was already activated (persisted eligibility)
    /// and has mining enabled
    pub fn is_permanently_eligible(&self) -> bool {
        self.activated_at.is_some() && self.trust_score >= 0.01 && self.mining_active
    }

    /// Checks if this node can participate in leader election
    pub fn can_be_leader(&self) -> bool {
        self.mining_active && self.is_active
    }
}

// =============================================================================
// NodeConsensusStatus - Status information for UI/API
// =============================================================================

/// Status information about a node's position in consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConsensusStatus {
    /// Current state: "Leader", "Queue", "Patience", "Connecting"
    pub state: String,

    /// Position in the leader queue (0 = current leader)
    pub queue_position: u32,

    /// Estimated number of blocks until leadership
    pub estimated_blocks: u32,

    /// Progress through patience/quarantine period (0.0 to 1.0)
    pub patience_progress: f32,

    /// Seconds remaining until eligible
    pub remaining_seconds: u64,

    /// Assigned shard ID
    pub shard_id: u32,

    /// Whether currently the slot leader
    pub is_slot_leader: bool,
}
