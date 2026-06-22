//! # Network Messages
//!
//! Message types for network communication between nodes.

use serde::{Deserialize, Serialize};

// =============================================================================
// Node Status Update Message
// =============================================================================

/// Message broadcast when a node's mining status changes.
///
/// When mining is disabled, the node broadcasts this to inform the network
/// that it should not be counted for leader election.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeStatusUpdate {
    /// The peer ID of the node
    pub peer_id: String,

    /// Whether the node is available for block production
    pub mining_active: bool,

    /// Timestamp of the status change
    pub timestamp: u64,

    /// Signature to verify authenticity (peer signs their own status)
    pub signature: String,
}

impl NodeStatusUpdate {
    /// Creates a new status update message
    pub fn new(peer_id: String, mining_active: bool) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            peer_id,
            mining_active,
            timestamp,
            signature: String::new(), // Will be filled by caller if needed
        }
    }
}
