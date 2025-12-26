use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// NodeState tracks the active time of a peer
#[derive(Debug, Clone)]
pub struct NodeState {
    pub peer_id: String,
    pub join_time: u64,
    pub trust_score: f64,
    pub vdf_proof: Option<String>,
    pub is_verified: bool,
    pub missed_slots: u64,      // Track missed blocks
    pub addresses: Vec<String>, // Technical Multiaddresses
}

impl NodeState {
    pub fn new(peer_id: String) -> Self {
        NodeState {
            peer_id,
            join_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            trust_score: 0.1,
            vdf_proof: None,
            is_verified: false,
            missed_slots: 0,
            addresses: Vec::new(),
        }
    }

    pub fn current_uptime(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if now > self.join_time {
            now - self.join_time
        } else {
            0
        }
    }
}

// Use current crate context
use crate::vdf::AntigravityVDF;

pub struct Consensus {
    pub nodes: HashMap<String, NodeState>,
    pub quarantine_duration: u64,
    pub vdf: AntigravityVDF,
    pub local_peer_id: Option<String>,
}

impl Consensus {
    pub fn new() -> Self {
        Consensus {
            nodes: HashMap::new(),
            quarantine_duration: 72 * 3600,
            vdf: AntigravityVDF::new(100_000), // Adjusted for demo (real would be higher)
            local_peer_id: None,
        }
    }

    pub fn set_local_peer_id(&mut self, peer_id: String) {
        // Also add self to nodes map so we are considered for leadership
        self.local_peer_id = Some(peer_id.clone());
        if !self.nodes.contains_key(&peer_id) {
            let mut node = NodeState::new(peer_id.clone());
            node.is_verified = true; // Trust self
            node.trust_score = 1.0; // Max trust for self
            self.nodes.insert(peer_id, node);
        }
    }

    pub fn slash_node(&mut self, peer_id: &String) {
        if let Some(node) = self.nodes.get_mut(peer_id) {
            node.missed_slots += 1;
            node.trust_score *= 0.5; // Halve the trust score
            if node.trust_score < 0.01 {
                node.trust_score = 0.01; // Floor
            }
            log::warn!("SLASHED Node {}: New Score: {}", peer_id, node.trust_score);
        }
    }

    pub fn reward_node(&mut self, peer_id: &String) {
        if let Some(node) = self.nodes.get_mut(peer_id) {
            // Gradual recovery
            node.trust_score = (node.trust_score * 1.1).min(1.0);
            log::info!("REWARDED Node {}: New Score: {}", peer_id, node.trust_score);
        }
    }

    pub fn verify_peer(&mut self, peer_id: String, proof: String) -> bool {
        if self.vdf.verify(peer_id.as_bytes(), &proof) {
            if let Some(node) = self.nodes.get_mut(&peer_id) {
                node.is_verified = true;
                node.vdf_proof = Some(proof);
                node.trust_score = 1.0; // Boost trust immediately upon VDF proof
                return true;
            }
        }
        false
    }

    pub fn register_node(&mut self, peer_id: String) {
        if !self.nodes.contains_key(&peer_id) {
            self.nodes.insert(peer_id.clone(), NodeState::new(peer_id));
        }
    }

    // 1.3 PoP Algorithm: Patience Weight = Uptime * Trust Score
    // Includes Sybil Resistance: If in quarantine, weight is drastically reduced.
    pub fn calculate_patience_weight(&self, peer_id: &String) -> f64 {
        if let Some(node) = self.nodes.get(peer_id) {
            let uptime = node.current_uptime();

            // Sybil Resistance: Quarantine & VDF
            // If verified by VDF, ignore quarantine.
            let effective_trust = if node.is_verified {
                node.trust_score
            } else if uptime < self.quarantine_duration {
                node.trust_score * 0.01 // Heavy penalty for unverified new nodes
            } else {
                node.trust_score
            };

            (uptime as f64) * effective_trust
        } else {
            0.0
        }
    }

    // Select leader for a specific round/timeslot (Beacon Chain Logic)
    pub fn select_beacon_leader(&self) -> Option<String> {
        let mut best_node = None;
        let mut max_weight = -1.0;

        for (peer_id, _) in &self.nodes {
            let weight = self.calculate_patience_weight(peer_id);
            // println!("Consensus: Node {} weight: {}", peer_id, weight);
            if weight > max_weight {
                max_weight = weight;
                best_node = Some(peer_id.clone());
            }
        }
        best_node
    }

    // AHSP Phase 1: Dynamic Sharding Calculations

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
    /// Uses SHA-256 for secure randomness to prevent "grinding" attacks.
    pub fn get_assigned_shard(&self, peer_id: &str, epoch_seed: u64) -> u16 {
        let active_shards = self.calculate_active_shards();

        // Use SHA-256 for secure checking
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(peer_id.as_bytes());
        hasher.update(&epoch_seed.to_le_bytes()); // Incorporate randomness
        let result = hasher.finalize();

        // Take first 2 bytes for u16 modulo
        let hash_val = ((result[0] as u16) << 8) | (result[1] as u16);

        // Modulo assignment
        hash_val % active_shards
    }

    pub fn get_node_status(&self, peer_id: &String) -> NodeConsensusStatus {
        // 1. Check if node exists
        let node = match self.nodes.get(peer_id) {
            Some(n) => n,
            None => {
                return NodeConsensusStatus {
                    state: "Connecting".to_string(),
                    queue_position: 0,
                    estimated_blocks: 0,
                    patience_progress: 0.0,
                    remaining_seconds: 0,
                }
            }
        };

        // 2. Determine Logic Source of Truth
        // Instead of recalculating sort, we ask who the consensus thinks is the leader RIGHT NOW.
        let actual_leader = self.select_beacon_leader();
        let is_leader = actual_leader.as_ref() == Some(peer_id);

        // Calculate uptime for patience check
        let uptime = node.current_uptime();

        // 3. Determine Queue Position (for fallback)
        // Calculate weights for all nodes to find rank
        let mut weights: Vec<(&String, f64)> = self
            .nodes
            .keys()
            .map(|pid| (pid, self.calculate_patience_weight(pid)))
            .collect();

        // Sort descending by weight
        weights.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Find my position
        let position = weights
            .iter()
            .position(|(pid, _)| *pid == peer_id)
            .unwrap_or(0);

        // 4. Status Decision Tree
        if is_leader {
            // Priority 1: If consensus says I am leader, I am Leader.
            NodeConsensusStatus {
                state: "Leader".to_string(),
                queue_position: 1, // Look good
                estimated_blocks: 0,
                patience_progress: 1.0,
                remaining_seconds: 0,
            }
        } else if !node.is_verified && uptime < self.quarantine_duration {
            // Priority 2: If I am not leader, and I am in quarantine, show Patience status
            let progress = uptime as f64 / self.quarantine_duration as f64;
            NodeConsensusStatus {
                state: "Patience".to_string(),
                queue_position: (position + 1) as u32,
                estimated_blocks: (self.quarantine_duration - uptime) as u32 / 2,
                patience_progress: progress as f32,
                remaining_seconds: (self.quarantine_duration - uptime),
            }
        } else {
            // Priority 3: Normal Queue
            NodeConsensusStatus {
                state: "Queue".to_string(),
                queue_position: (position + 1) as u32,
                estimated_blocks: position as u32, // Simplified: 1 block per person ahead
                patience_progress: 1.0,
                remaining_seconds: (position * 2) as u64, // Approx 2s per block
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConsensusStatus {
    pub state: String, // "Leader", "Queue", "Patience"
    pub queue_position: u32,
    pub estimated_blocks: u32,
    pub patience_progress: f32, // 0.0 to 1.0
    pub remaining_seconds: u64,
}
