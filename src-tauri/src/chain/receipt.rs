//! # Cross-Shard Receipts
//!
//! Structures for cross-shard transaction receipts and cross-links.

use serde::{Deserialize, Serialize};

/// Status of a cross-shard transfer to ensure atomicity
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ReceiptStatus {
    Pending,
    Claimed,  // Successfully minted on target shard
    Reverted, // Target failed, funds returned on source shard
}

/// A Receipt proves that a transaction was executed on a Source Shard
/// and funds were burned/locked, allowing the Destination Shard to mint/unlock them.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Receipt {
    pub original_tx_id: String,
    pub source_shard: u16,
    pub target_shard: u16,
    pub amount: u64,
    pub receiver: String,
    pub block_hash: String,
    pub merkle_proof: Vec<String>,
    pub status: ReceiptStatus,
}

/// Cross-Link is a summary of a Shard's block header, signed by the shard's committee,
/// sent to the Beacon Chain for finalization.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrossLink {
    pub shard_id: u16,
    pub block_height: u64,
    pub block_hash: String,
    pub state_root: String,
    pub signature: String,
}

/// Message broadcast via P2P when a node solves the VDF
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VdfProofMessage {
    pub peer_id: String,
    pub proof: String,
    pub challenge: String,
}
