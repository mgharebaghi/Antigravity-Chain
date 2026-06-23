//! # Consensus Module
//!
//! This module implements the AHSP (Adaptive Horizontal Sharded PoS) consensus mechanism.
//!
//! ## Structure
//!
//! The consensus module is organized into the following sub-modules:
//! - `node_state`: Validator node state and status tracking
//! - `leadership`: Leader election and eligibility logic
//! - `epoch`: Time-based consensus mechanics (epochs and slots)
//! - `sharding`: Horizontal scaling through dynamic sharding
//! - `mempool`: Transaction pool management
//! - `vdf`: Verifiable Delay Function for Proof of Patience
//!
//! ## Key Concepts
//!
//! - **Proof of Patience**: Nodes must solve a VDF and wait through quarantine before becoming validators
//! - **Sharding**: Network automatically scales by adding shards as validator count grows
//! - **Slot-based Production**: Time is divided into slots; each slot has one designated leader per shard
//! - **Trust Scores**: Nodes earn/lose trust based on block production performance

use std::collections::HashMap;

// Sub-modules
pub mod epoch;
pub mod leadership;
pub mod mempool;
pub mod node_state;
pub mod sharding;
pub mod vdf;

// Re-exports for convenience
pub use node_state::{NodeConsensusStatus, NodeState};
pub use vdf::CentichainVDF;

// =============================================================================
// Core Consensus Struct
// =============================================================================

/// Main consensus state manager
///
/// Tracks all validator nodes and their states, manages leader election,
/// and coordinates sharding decisions.
pub struct Consensus {
    /// Map of peer IDs to their node states
    pub nodes: HashMap<String, NodeState>,

    /// Base quarantine duration in seconds
    pub quarantine_duration: u64,

    /// VDF instance for Proof of Patience
    pub vdf: CentichainVDF,

    /// Local node's peer ID (if set)
    pub local_peer_id: Option<String>,
}

impl Consensus {
    /// Creates a new Consensus instance with default settings
    pub fn new() -> Self {
        Consensus {
            nodes: HashMap::new(),
            quarantine_duration: 72 * 3600,   // 72 hours base
            vdf: CentichainVDF::new(100_000), // Adjusted for demo (real would be higher)
            local_peer_id: None,
        }
    }

    /// Sets the local peer ID and adds self to the nodes map
    pub fn set_local_peer_id(&mut self, peer_id: String) {
        self.local_peer_id = Some(peer_id.clone());
        if !self.nodes.contains_key(&peer_id) {
            let mut node = NodeState::new(peer_id.clone());
            // Local node is trusted for local operations
            node.is_verified = true;
            node.trust_score = 1.0;
            // Note: is_active is NOT set here - must prove patience or be Genesis
            self.nodes.insert(peer_id, node);
        }
    }

    /// Force-activates the local node (used for Genesis creator)
    /// This grants immediate active status without quarantine.
    pub fn force_activate_local(&mut self) {
        if let Some(peer_id) = &self.local_peer_id {
            if let Some(node) = self.nodes.get_mut(peer_id) {
                node.activate();
                node.is_verified = true;
                node.trust_score = 1.0;
                log::info!("Consensus: Local node FORCE ACTIVATED (Genesis/Authoritative Mode)");
            }
        }
    }

    // =========================================================================
    // Node Management
    // =========================================================================

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

    /// Rewards a node for good behavior
    pub fn reward_node(&mut self, peer_id: &String) {
        if let Some(node) = self.nodes.get_mut(peer_id) {
            node.trust_score = (node.trust_score * 1.1).min(1.0);
            log::info!("REWARDED Node {}: New Score: {}", peer_id, node.trust_score);
        }
    }

    /// Verifies a peer's VDF proof
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

    /// Registers a new node in the consensus
    pub fn register_node(&mut self, peer_id: String) {
        if !self.nodes.contains_key(&peer_id) {
            self.nodes.insert(peer_id.clone(), NodeState::new(peer_id));
        }
    }

    /// Sets the mining status for a peer.
    /// When mining is disabled, the node will not be selected as leader.
    pub fn set_peer_mining_status(&mut self, peer_id: &String, mining_active: bool) -> bool {
        if let Some(node) = self.nodes.get_mut(peer_id) {
            let old_status = node.mining_active;
            node.mining_active = mining_active;

            if old_status != mining_active {
                log::info!(
                    "Consensus: Peer {} mining status changed: {} -> {}",
                    peer_id,
                    old_status,
                    mining_active
                );
            }
            return true;
        }
        false
    }

    /// Gets the mining status for a peer
    pub fn get_peer_mining_status(&self, peer_id: &String) -> Option<bool> {
        self.nodes.get(peer_id).map(|n| n.mining_active)
    }

    /// Calculates the current quarantine duration based on network size
    pub fn get_quarantine_duration(&self) -> u64 {
        let validator_count = self.nodes.len() as u64;
        if validator_count <= 1 {
            300 // 5 mins for solo/first peer
        } else {
            // +1 hour per validator, up to 72 hours
            (300 + (validator_count * 3600)).min(72 * 3600)
        }
    }
}

impl Default for Consensus {
    fn default() -> Self {
        Self::new()
    }
}

impl Consensus {
    /// Persisted snapshot of validator states (excluding ephemeral locks).
    pub fn export_nodes(&self) -> std::collections::HashMap<String, NodeState> {
        self.nodes.clone()
    }

    /// Restores validator states from disk (e.g. after app restart).
    pub fn import_nodes(&mut self, nodes: std::collections::HashMap<String, NodeState>) {
        self.nodes = nodes;
    }

    /// Saves nodes to storage; call after activation / slashing / verify.
    pub fn persist_to_storage(&self, storage: &crate::storage::Storage) {
        if let Err(e) = storage.save_consensus_nodes(&self.nodes) {
            log::warn!("Failed to persist consensus nodes: {}", e);
        }
    }

    /// Loads nodes from storage into this consensus instance.
    pub fn load_from_storage(&mut self, storage: &crate::storage::Storage) {
        match storage.load_consensus_nodes() {
            Ok(nodes) if !nodes.is_empty() => {
                log::info!("Loaded {} consensus node states from storage", nodes.len());
                self.import_nodes(nodes);
            }
            Ok(_) => {}
            Err(e) => log::warn!("Could not load consensus nodes: {}", e),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

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
            n.activate();
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
    fn test_block_author_registration_only() {
        let mut consensus = Consensus::new();
        let peer_id = "new_joiner".to_string();

        consensus.register_node(peer_id.clone());
        assert!(consensus
            .nodes
            .get(&peer_id)
            .unwrap()
            .activated_at
            .is_none());

        consensus.register_block_author(peer_id.clone());

        let node = consensus.nodes.get(&peer_id).unwrap();
        assert!(!node.is_active);
        assert!(!node.is_verified);
        assert!(node.activated_at.is_none());
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
