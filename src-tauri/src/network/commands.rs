//! # P2P Commands Module
//!
//! Defines commands that can be sent to the P2P network layer.

use serde::{Deserialize, Serialize};

/// Commands that can be sent to the P2P network from other parts of the application
#[derive(Debug)]
pub enum P2PCommand {
    /// Trigger network synchronization
    SyncWithNetwork,

    /// Broadcast mining status change to network
    BroadcastMiningStatus { mining_active: bool },
}

/// Topology update message for network graph visualization
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopologyUpdate {
    /// Source peer ID
    pub source: String,
    /// List of connected peer IDs
    pub connections: Vec<String>,
    /// Unix timestamp
    pub timestamp: u64,
}

impl TopologyUpdate {
    /// Creates a new topology update
    pub fn new(source: String, connections: Vec<String>) -> Self {
        Self {
            source,
            connections,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}
