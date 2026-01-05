use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// NodeState - Tracks validator state for consensus
// ============================================================================

/// Represents the state of a validator node in the network.
/// Once a node is activated (via VDF proof + quarantine), it remains eligible
/// for leadership unless explicitly slashed below the trust threshold.
#[derive(Debug, Clone)]
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
    pub fn is_permanently_eligible(&self) -> bool {
        self.activated_at.is_some() && self.trust_score >= 0.01
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
            // node.is_active = true; // REMOVED: Auto-active disabled. Must prove patience or be Genesis.
            self.nodes.insert(peer_id, node);
        }
    }

    /// Force-activates the local node (used for Genesis creator)
    /// This grants immediate active status without quarantine.
    pub fn force_activate_local(&mut self) {
        if let Some(peer_id) = &self.local_peer_id {
            if let Some(node) = self.nodes.get_mut(peer_id) {
                node.activate(); // Use the new activate method
                node.is_verified = true; // Genesis creator is always verified
                node.trust_score = 1.0;
                log::info!("Consensus: Local node FORCE ACTIVATED (Genesis/Authoritative Mode)");
            }
        }
    }

    /// Slashes a node for misbehavior (missing slots)
    /// Trust score is halved. If it falls below 0.01, active status is revoked.
    pub fn slash_node(&mut self, peer_id: &String) {
        if let Some(node) = self.nodes.get_mut(peer_id) {
            node.missed_slots += 1;
            node.trust_score *= 0.5; // Halve the trust score

            if node.trust_score < 0.01 {
                node.trust_score = 0.01; // Floor at minimum
                node.is_active = false; // Revoke active status
                node.activated_at = None; // Remove permanent eligibility
                log::warn!("Node {} DEACTIVATED due to low trust score", peer_id);
            }

            log::warn!(
                "SLASHED Node {}: Missed Slots: {}, Trust: {:.3}, Active: {}",
                peer_id,
                node.missed_slots,
                node.trust_score,
                node.is_active
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

    // =========================================================================
    // Leadership Eligibility
    // =========================================================================

    /// Checks if a node is eligible for block production leadership.
    ///
    /// A node is eligible if:
    /// 1. Already permanently activated (activated_at is set) with good trust
    /// 2. Solo node (only node in network) - bootstrap exception
    /// 3. Verified + completed quarantine + good trust score
    pub fn is_eligible_for_leadership(&self, peer_id: &String) -> bool {
        let Some(node) = self.nodes.get(peer_id) else {
            return false;
        };

        // Rule 1: Permanently activated nodes stay eligible (Grandfather Clause)
        // This is the KEY fix - once activated, a node doesn't need to re-qualify
        if node.is_permanently_eligible() {
            return true;
        }

        // Rule 2: Solo node bootstrap exception
        if self.nodes.len() == 1 {
            return true;
        }

        // Rule 3: Fresh node must complete VDF + quarantine
        let uptime = node.current_uptime();
        let q_duration = self.get_quarantine_duration();

        if node.is_verified && uptime >= q_duration && node.trust_score >= 0.01 {
            return true;
        }

        false
    }

    /// Periodically updates active status for all nodes.
    /// Promotes eligible nodes and demotes nodes with low trust.
    ///
    /// Called every iteration of the mining loop.
    pub fn update_active_status(&mut self) {
        let node_count = self.nodes.len();
        let q_duration = self.get_quarantine_duration();

        for (_, node) in self.nodes.iter_mut() {
            // Demote nodes with critically low trust
            if node.trust_score < 0.01 {
                if node.is_active {
                    node.is_active = false;
                    node.activated_at = None;
                    log::warn!("Node {} DEMOTED due to low trust", node.peer_id);
                }
                continue;
            }

            // Skip already activated nodes - they stay active
            if node.activated_at.is_some() {
                continue;
            }

            // Check if node qualifies for activation
            let should_activate = if node_count == 1 {
                // Solo node: immediate activation
                true
            } else {
                // Network node: must be verified and complete quarantine
                node.is_verified && node.current_uptime() >= q_duration
            };

            if should_activate {
                node.activate();
                log::info!(
                    "Node {} PROMOTED to Active Validator (uptime: {}s, quarantine: {}s)",
                    node.peer_id,
                    node.current_uptime(),
                    q_duration
                );
            }
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

        if eligible_validators.is_empty() {
            return None;
        }

        // 2. Sort to ensure strict consensus on order
        eligible_validators.sort();

        // 3. Deterministic Randomness (Weighted by Slot + Epoch)
        // SHA256(Epoch + Slot) % Count
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(shard_id.to_be_bytes());
        hasher.update(epoch.to_be_bytes());
        hasher.update(slot.to_be_bytes());
        let result = hasher.finalize();
        // Use first 8 bytes for randomness index
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&result[0..8]);
        let rand_val = u64::from_le_bytes(bytes);

        let index = (rand_val as usize) % eligible_validators.len();

        Some(eligible_validators[index].clone())
    }

    /// Automatically activates a peer who authored a valid block.
    ///
    /// When we receive a valid block from a peer, we trust they went through
    /// proper verification on other nodes. This ensures new joiners correctly
    /// recognize existing chain authors as active validators.
    pub fn mark_peer_active(&mut self, peer_id: String) {
        if let Some(node) = self.nodes.get_mut(&peer_id) {
            if node.activated_at.is_none() {
                log::info!(
                    "Consensus: Peer {} authored valid block - granting active status",
                    peer_id
                );
                node.activate();
                node.is_verified = true;
                node.trust_score = (node.trust_score * 1.05).min(1.0);
            }
        } else {
            // Unknown peer authored a block - register and activate
            let mut node = NodeState::new(peer_id.clone());
            node.activate();
            node.is_verified = true;
            node.trust_score = 1.0;
            self.nodes.insert(peer_id.clone(), node);
            log::info!(
                "Consensus: New peer {} registered and activated via block authorship",
                peer_id
            );
        }
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

    // test_round_robin_selection removed (replaced by test_deterministic_randomness)

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

        // CRITICAL: Activate the node WHILE it's still solo
        // This simulates real behavior - update_active_status is called every loop iteration
        consensus.update_active_status();

        // Verify activation happened
        assert!(
            consensus
                .nodes
                .get(&peer_id)
                .unwrap()
                .activated_at
                .is_some(),
            "Solo node should have been activated"
        );

        // NOW add a second node
        consensus
            .nodes
            .insert("node2".to_string(), NodeState::new("node2".to_string()));

        // KEY BEHAVIOR: Once activated, the first node STAYS activated
        // even when more nodes join (grandfather clause)
        assert!(
            consensus.is_eligible_for_leadership(&peer_id),
            "Activated node should stay eligible when more nodes join"
        );
    }

    #[test]
    fn test_deterministic_randomness() {
        let mut consensus = Consensus::new();
        // Setup 3 nodes with proper activation
        for i in 0..3 {
            let pid = format!("node{}", i);
            let mut n = NodeState::new(pid.clone());
            n.activate(); // Use new method
            n.trust_score = 1.0;
            n.is_verified = true;
            consensus.nodes.insert(pid, n);
        }

        let leader_slot_10 = consensus.get_shard_leader(0, 10).unwrap();
        let leader_slot_10_again = consensus.get_shard_leader(0, 10).unwrap();

        // Determinism Check
        assert_eq!(leader_slot_10, leader_slot_10_again);

        let leader_slot_11 = consensus.get_shard_leader(0, 11).unwrap();
        // Likely different, but not guaranteed (1/3 chance)
        println!("Slot 10: {}, Slot 11: {}", leader_slot_10, leader_slot_11);
    }

    #[test]
    fn test_implicit_activation() {
        let mut consensus = Consensus::new();
        let peer_id = "new_joiner".to_string();

        // Node joins
        consensus.register_node(peer_id.clone());
        assert!(consensus
            .nodes
            .get(&peer_id)
            .unwrap()
            .activated_at
            .is_none());

        // Produces valid block (simulated reception)
        consensus.mark_peer_active(peer_id.clone());

        assert!(consensus.nodes.get(&peer_id).unwrap().is_active);
        assert!(consensus.nodes.get(&peer_id).unwrap().is_verified);
        assert!(consensus
            .nodes
            .get(&peer_id)
            .unwrap()
            .activated_at
            .is_some());
    }

    #[test]
    fn test_persistent_eligibility() {
        // This test verifies the KEY fix: once activated, nodes stay eligible
        let mut consensus = Consensus::new();

        // Setup first node as activated
        let node1 = "node1".to_string();
        let mut n1 = NodeState::new(node1.clone());
        n1.activate();
        n1.is_verified = true;
        n1.trust_score = 1.0;
        consensus.nodes.insert(node1.clone(), n1);

        // Verify node1 is eligible
        assert!(consensus.is_eligible_for_leadership(&node1));

        // Add many more nodes (this increases quarantine duration)
        for i in 2..10 {
            let pid = format!("node{}", i);
            consensus
                .nodes
                .insert(pid, NodeState::new(format!("node{}", i)));
        }

        // Node1 should STILL be eligible (grandfather clause)
        assert!(
            consensus.is_eligible_for_leadership(&node1),
            "Activated node should remain eligible regardless of quarantine changes"
        );

        // New nodes should NOT be eligible (haven't done quarantine)
        assert!(
            !consensus.is_eligible_for_leadership(&"node2".to_string()),
            "New node should not be eligible without completing quarantine"
        );
    }
}
