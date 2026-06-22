//! # Startup State Module
//!
//! Defines the state machine for node startup phases.

use std::time::{Duration, Instant};

/// States for node startup process
#[derive(Debug, PartialEq)]
pub enum NodeStartupState {
    /// Connecting to relay server
    ConnectingToRelay { start_time: Instant },

    /// Discovering peers on the network
    DiscoveringPeers { start_time: Instant },

    /// Normal operation mode
    Running,

    /// Relay connection failed
    RelayConnectionFailed,
}

impl NodeStartupState {
    /// Creates a new ConnectingToRelay state
    pub fn new_connecting() -> Self {
        Self::ConnectingToRelay {
            start_time: Instant::now(),
        }
    }

    /// Creates a new DiscoveringPeers state
    pub fn new_discovering() -> Self {
        Self::DiscoveringPeers {
            start_time: Instant::now(),
        }
    }

    /// Returns true if in Running state
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Returns true if relay connection failed
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::RelayConnectionFailed)
    }
}

/// Configuration for startup timeouts
pub struct StartupConfig {
    /// Time to wait for relay connection
    pub relay_timeout: Duration,
    /// Time to wait for peer discovery
    pub discovery_duration: Duration,
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            relay_timeout: Duration::from_secs(15),
            discovery_duration: Duration::from_secs(5),
        }
    }
}
