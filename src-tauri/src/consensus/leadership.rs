//! # Leadership Module
//!
//! Handles leader election and eligibility logic for the consensus mechanism.
//! This module contains the core algorithms for determining who can produce blocks.

use super::node_state::NodeState;
use super::Consensus;

// =============================================================================
// Leadership Eligibility Logic
// =============================================================================

impl Consensus {
    /// Checks if a node is eligible for block production leadership.
    ///
    /// A node is eligible if:
    /// 1. Already permanently activated (activated_at is set) with good trust + mining enabled
    /// 2. Solo node (only node in network) - but MUST have solved VDF or be first node
    /// 3. Verified + completed quarantine + good trust score + mining enabled
    pub fn is_eligible_for_leadership(&self, peer_id: &String) -> bool {
        let Some(node) = self.nodes.get(peer_id) else {
            log::debug!("Eligibility check for {}: NOT FOUND in nodes", peer_id);
            return false;
        };

        // DEBUG: Log node state for troubleshooting
        log::debug!(
            "Eligibility check for {}: activated_at={:?}, is_verified={}, mining_active={}, trust={:.2}, nodes_count={}",
            peer_id,
            node.activated_at,
            node.is_verified,
            node.mining_active,
            node.trust_score,
            self.nodes.len()
        );

        // CRITICAL: Mining must be enabled for ANY eligibility
        if !node.mining_active {
            log::debug!(
                "Eligibility check for {}: FAILED - mining_active=false",
                peer_id
            );
            return false;
        }

        // Rule 1: Permanently activated nodes stay eligible (Grandfather Clause)
        // This is the KEY fix - once activated, a node doesn't need to re-qualify
        if node.is_permanently_eligible() {
            log::debug!(
                "Eligibility check for {}: PASSED - permanently eligible",
                peer_id
            );
            return true;
        }

        // Rule 2: Solo node bootstrap exception
        // IMPORTANT: Solo node must still be verified (VDF solved) OR be the genesis creator
        // A new joining node should NOT qualify here if there are other nodes
        if self.nodes.len() == 1 {
            // For solo node, require at least verified status OR activated_at from genesis
            // If it's truly alone and verified (solved VDF), allow it
            if node.is_verified || node.activated_at.is_some() {
                log::debug!(
                    "Eligibility check for {}: PASSED - solo node exception",
                    peer_id
                );
                return true;
            }
            // If not verified and not activated, this is a brand new node waiting for VDF
            log::debug!(
                "Eligibility check for {}: FAILED - solo but not verified/activated",
                peer_id
            );
            return false;
        }

        // Rule 3: Fresh node in network must complete VDF + quarantine
        let uptime = node.current_uptime();
        let q_duration = self.get_quarantine_duration();

        if node.is_verified && uptime >= q_duration && node.trust_score >= 0.01 {
            log::debug!(
                "Eligibility check for {}: PASSED - completed quarantine",
                peer_id
            );
            return true;
        }

        log::debug!(
            "Eligibility check for {}: FAILED - not yet qualified (uptime={}, quarantine={}, verified={})",
            peer_id,
            uptime,
            q_duration,
            node.is_verified
        );
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
                // Solo node: MUST be verified (VDF solved) to qualify
                // This prevents new nodes from auto-activating before discovering peers
                if node.is_verified {
                    true
                } else {
                    log::debug!(
                        "Solo node {} not verified yet - waiting for VDF proof",
                        node.peer_id
                    );
                    false
                }
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

    /// Deterministic Round-Robin Leader Election
    /// Leaders = Sorted List of Eligible Validators in Shard
    /// Leader for Slot S = Leaders[S % Count]
    pub fn get_shard_leader(&self, shard_id: u16, slot: u64) -> Option<String> {
        let epoch = slot / (Self::EPOCH_DURATION / Self::SLOT_DURATION);

        // DEBUG: Print all node states
        println!(
            "[LEADER_ELECTION] Slot {} - Checking {} nodes:",
            slot,
            self.nodes.len()
        );
        for (pid, node) in &self.nodes {
            println!(
                "  - {} | activated_at={:?} | verified={} | mining={} | trust={:.2}",
                pid, node.activated_at, node.is_verified, node.mining_active, node.trust_score
            );
        }

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

        println!(
            "[LEADER_ELECTION] Eligible validators: {:?}",
            eligible_validators
        );

        if eligible_validators.is_empty() {
            // Fallback for Genesis/Bootstrap:
            // If NO ONE is eligible (e.g. network just started, everyone is new),
            // allow ANY verified node to mine to keep liveness, if trust > 0.
            // But if specific nodes are failing quarantine, this might pause chain.
            // We allow a "Bootstrap Mode" if active nodes < 2
            if self.nodes.len() < 2 {
                eligible_validators = self.nodes.keys().cloned().collect();
                println!(
                    "[LEADER_ELECTION] Using fallback - all nodes: {:?}",
                    eligible_validators
                );
            } else {
                println!("[LEADER_ELECTION] NO ELIGIBLE LEADER! Returning None.");
                return None; // No eligible leader = Skipped Slot
            }
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

    /// Registers a peer who produced a valid block — does NOT bypass PoP quarantine.
    pub fn register_block_author(&mut self, peer_id: String) {
        if !self.nodes.contains_key(&peer_id) {
            log::info!(
                "Consensus: Registering block author {} (PoP still required)",
                peer_id
            );
            self.nodes.insert(peer_id.clone(), NodeState::new(peer_id));
        }
    }

    /// @deprecated Phase 1 — block authorship no longer grants activation.
    pub fn mark_peer_active(&mut self, peer_id: String) {
        self.register_block_author(peer_id);
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

    /// Get leaders for upcoming slots
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
}
