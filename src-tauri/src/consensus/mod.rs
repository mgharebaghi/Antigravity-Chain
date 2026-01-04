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
pub mod mempool;
pub mod vdf;

use self::vdf::CentichainVDF;

pub struct Consensus {
    pub nodes: HashMap<String, NodeState>,
    pub quarantine_duration: u64,
    pub vdf: CentichainVDF,
    pub local_peer_id: Option<String>,
}

impl Consensus {
    pub fn new() -> Self {
        Consensus {
            nodes: HashMap::new(),
            quarantine_duration: 72 * 3600,
            vdf: CentichainVDF::new(100_000), // Adjusted for demo (real would be higher)
            local_peer_id: None,
        }
    }

    pub fn set_local_peer_id(&mut self, peer_id: String) {
        // Also add self to nodes map so we are considered for leadership
        self.local_peer_id = Some(peer_id.clone());
        if !self.nodes.contains_key(&peer_id) {
            let mut node = NodeState::new(peer_id.clone());
            // local node is trusted by definition for *local* operations,
            // but for network consensus, it still follows rules (technically).
            // However, to start mining solo, we often boostrap:
            node.is_verified = true;
            node.trust_score = 1.0;
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
            log::warn!(
                "SLASHED Node {}: Missed Slots: {}, New Score: {}",
                peer_id,
                node.missed_slots,
                node.trust_score
            );
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
        let challenge = self.get_vdf_challenge(&peer_id);
        if self.vdf.verify(challenge.as_bytes(), &proof) {
            if let Some(node) = self.nodes.get_mut(&peer_id) {
                node.is_verified = true;
                node.vdf_proof = Some(proof);
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

    pub fn get_quarantine_duration(&self) -> u64 {
        let validator_count = self.nodes.len() as u64;
        if validator_count <= 1 {
            300 // 5 mins for solo/first peer
        } else {
            // +1 hour per validator, up to 72 hours
            (300 + (validator_count * 3600)).min(72 * 3600)
        }
    }

    // Check if a node is fully eligible to be a leader
    pub fn is_eligible_for_leadership(&self, peer_id: &String) -> bool {
        if let Some(node) = self.nodes.get(peer_id) {
            let uptime = node.current_uptime();
            let q_duration = self.get_quarantine_duration();

            // Special Case: Solo Validator
            // If there is only 1 node in the network (us), we skip the quarantine "Patience" requirement.
            // USER REQUIREMENT: "Without any preconditions" -> Return true immediately.
            if self.nodes.len() == 1 {
                return true;
            }

            // 1. Must be Verified (VDF Solved)
            // 2. Must be out of Quarantine (Uptime > Duration)
            // 3. Must have decent trust score (not banned)
            node.is_verified && uptime >= q_duration && node.trust_score >= 0.01
        } else {
            false
        }
    }

    // -------------------------------------------------------------------------
    // AHSP Consensus Logic: Epochs, Slots, and Sharding
    // -------------------------------------------------------------------------

    pub const EPOCH_DURATION: u64 = 600; // 10 Minutes for testing
    pub const SLOT_DURATION: u64 = 2; // 2 Seconds

    pub fn current_epoch(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now / Self::EPOCH_DURATION
    }

    pub fn current_slot(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now / Self::SLOT_DURATION
    }

    pub fn get_vdf_challenge(&self, peer_id: &String) -> String {
        // Challenge = SHA256(PeerID + "Patience")
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(peer_id.as_bytes());
        hasher.update(b"Patience");
        hex::encode(hasher.finalize())
    }

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
    pub fn get_assigned_shard(&self, peer_id: &str, epoch: u64) -> u16 {
        let active_shards = self.calculate_active_shards();
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(peer_id.as_bytes());
        hasher.update(&epoch.to_le_bytes());
        let result = hasher.finalize();
        let hash_val = ((result[0] as u16) << 8) | (result[1] as u16);
        hash_val % active_shards
    }

    /// Deterministic Round-Robin Leader Election
    /// Leaders = Sorted List of Eligible Validators in Shard
    /// Leader for Slot S = Leaders[S % Count]
    pub fn get_shard_leader(&self, shard_id: u16, slot: u64) -> Option<String> {
        let epoch = slot / (Self::EPOCH_DURATION / Self::SLOT_DURATION);

        // 1. Filter eligible validators for this shard
        let mut eligible_validators: Vec<String> = self
            .nodes
            .iter()
            .filter(|(pid, _)| {
                // strict check: Assigned to shard AND fully eligible (Verified + Patience)
                self.get_assigned_shard(pid, epoch) == shard_id
                    && self.is_eligible_for_leadership(pid)
            })
            .map(|(pid, _)| pid.clone())
            .collect();

        if eligible_validators.is_empty() {
            // Fallback for Genesis/Bootstrap:
            // If NO ONE is eligible (e.g. network just started, everyone is new),
            // allow ANY verified node to mine to keep liveness, if trust > 0.
            // But if specific nodes are failing quarantine, this might pause chain.
            // We allow a "Bootstrap Mode" if active nodes < 2
            if self.nodes.len() < 2 {
                eligible_validators = self.nodes.keys().cloned().collect();
            } else {
                return None; // No eligible leader = Skipped Slot
            }
        }

        if eligible_validators.is_empty() {
            return None;
        }

        // 2. Sort to ensure strict consensus on order
        eligible_validators.sort();

        // 3. Round-Robin Selection
        // Simple Modulo arithmetic ensures perfect rotation fairness.
        let index = (slot as usize) % eligible_validators.len();

        Some(eligible_validators[index].clone())
    }

    /// Identify and slash leaders who missed their slots between two blocks.
    /// Range is [start_slot, end_slot] inclusive.
    pub fn slash_missed_slots(
        &mut self,
        start_slot: u64,
        end_slot: u64,
        shard_id: u16,
    ) -> Vec<String> {
        let mut slashed_nodes = Vec::new();

        if start_slot > end_slot {
            return slashed_nodes;
        }

        for slot in start_slot..=end_slot {
            if let Some(expected_leader) = self.get_shard_leader(shard_id, slot) {
                // They missed their turn!
                self.slash_node(&expected_leader);
                slashed_nodes.push(expected_leader);
            }
        }
        slashed_nodes
    }

    pub fn get_future_leaders(
        &self,
        start_slot: u64,
        count: u64,
        shard_id: u16,
    ) -> Vec<(u64, Option<String>)> {
        let mut leaders = Vec::new();
        for i in 0..count {
            let slot = start_slot + i;
            leaders.push((slot, self.get_shard_leader(shard_id, slot)));
        }
        leaders
    }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConsensusStatus {
    pub state: String, // "Leader", "Queue", "Patience"
    pub queue_position: u32,
    pub estimated_blocks: u32,
    pub patience_progress: f32, // 0.0 to 1.0
    pub remaining_seconds: u64,
    pub shard_id: u32,
    pub is_slot_leader: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quarantine_enforcement() {
        let mut consensus = Consensus::new();
        // Lower difficulty for test to match solver
        consensus.vdf = CentichainVDF::new(100);

        let peer_id = "node1".to_string();

        // Register node
        consensus.register_node(peer_id.clone());

        // Add a second dummy node so 'peer_id' is NOT solo (and thus must follow quarantine)
        consensus.register_node("dummy_node".to_string());

        // Solve VDF
        let challenge = consensus.get_vdf_challenge(&peer_id);
        let vdf = CentichainVDF::new(100);
        let proof = vdf.solve(challenge.as_bytes());

        // This fails if difficulty doesn't match
        let verified = consensus.verify_peer(peer_id.clone(), proof);
        assert!(verified, "VDF verification failed");

        // Assert: Verified but NOT eligible (Time hasn't passed)
        assert!(consensus.nodes.get(&peer_id).unwrap().is_verified);
        assert!(!consensus.is_eligible_for_leadership(&peer_id));

        // Fast forward time check by uptime
        let uptime = consensus.nodes.get(&peer_id).unwrap().current_uptime();
        let duration = consensus.get_quarantine_duration();
        assert!(uptime < duration);
    }

    #[test]
    fn test_round_robin_selection() {
        let mut consensus = Consensus::new();

        // Mock 2 eligible nodes
        consensus.nodes.insert("nodeA".to_string(), {
            let mut n = NodeState::new("nodeA".to_string());
            n.is_verified = true;
            n.trust_score = 1.0;
            n.join_time = 0;
            n // Mock uptime > quarantine
        });
        consensus.nodes.insert("nodeB".to_string(), {
            let mut n = NodeState::new("nodeB".to_string());
            n.is_verified = true;
            n.trust_score = 1.0;
            n.join_time = 0;
            n
        });

        // Current epoch 0. Both assigned to same shard (likely 0 if <50 nodes)
        let shard = consensus.get_assigned_shard("nodeA", 0);

        // Note: With shard logic, different IDs might map to different shards if unlucky.
        // But with only 1 active shard (nodes<50), they are ALL in shard 0.
        // Let's verify active shards = 1.
        assert_eq!(consensus.calculate_active_shards(), 1);

        // Slot 0 -> nodeA or nodeB?
        // SortedIDs: [nodeA, nodeB]
        // Slot 0 % 2 = 0 -> nodeA
        let leader0 = consensus.get_shard_leader(shard, 0).unwrap();
        assert_eq!(leader0, "nodeA");

        // Slot 1 % 2 = 1 -> nodeB
        let leader1 = consensus.get_shard_leader(shard, 1).unwrap();
        assert_eq!(leader1, "nodeB");

        // Slot 2 % 2 = 0 -> nodeA
        let leader2 = consensus.get_shard_leader(shard, 2).unwrap();
        assert_eq!(leader2, "nodeA");
    }

    #[test]
    fn test_slashing() {
        let mut consensus = Consensus::new();
        // Setup nodeA as eligible
        consensus.nodes.insert("nodeA".to_string(), {
            let mut n = NodeState::new("nodeA".to_string());
            n.is_verified = true;
            n.trust_score = 1.0;
            n.join_time = 0;
            n
        });

        let initial_score = consensus.nodes.get("nodeA").unwrap().trust_score;
        let shard = consensus.get_assigned_shard("nodeA", 0);

        // Simulate missing slots 0 and 2.
        consensus.slash_missed_slots(0, 0, shard);

        let new_score = consensus.nodes.get("nodeA").unwrap().trust_score;
        assert!(new_score < initial_score);
        assert_eq!(consensus.nodes.get("nodeA").unwrap().missed_slots, 1);
    }

    #[test]
    fn test_solo_node_exemption() {
        let mut consensus = Consensus::new();
        let peer_id = "solo_node".to_string();

        // Add only ONE node (Solo)
        consensus.nodes.insert(peer_id.clone(), {
            let mut n = NodeState::new(peer_id.clone());
            // It solved VDF (Proof of Resources)
            n.is_verified = true;
            n
        });

        // Uptime is 0. Quarantine duration is 300s.
        let duration = consensus.get_quarantine_duration();
        assert!(consensus.nodes.get(&peer_id).unwrap().current_uptime() < duration);

        // Should be ELIGIBLE because nodes.len() == 1
        assert!(
            consensus.is_eligible_for_leadership(&peer_id),
            "Solo node should be eligible immediately"
        );

        // Add a second node
        consensus
            .nodes
            .insert("node2".to_string(), NodeState::new("node2".to_string()));

        // Now len() == 2. Solo node exemption should NOT apply.
        // It needs uptime now (Strict Quarantine applies to network > 1).
        assert!(
            !consensus.is_eligible_for_leadership(&peer_id),
            "Node should strictly follow quarantine once network > 1"
        );
    }
}
