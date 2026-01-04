//! # Transaction Types
//!
//! Transaction structure and related utilities.

use serde::{Deserialize, Serialize};

/// A blockchain transaction
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Transaction {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub shard_id: u16,
    pub timestamp: u64,
    pub signature: String,
}

impl Transaction {
    /// Checks if this transaction is independent of another (no shared state/accounts).
    /// Critical for Parallel Execution Engine.
    pub fn is_independent(&self, other: &Self) -> bool {
        self.sender != other.sender
            && self.sender != other.receiver
            && self.receiver != other.sender
            && self.receiver != other.receiver
    }
}

/// Calculates transaction fee (0.01%, minimum 0.001 AGT)
pub fn calculate_fee(amount: u64) -> u64 {
    let fee = (amount as f64 * 0.0001).ceil() as u64;
    fee.max(1_000)
}
