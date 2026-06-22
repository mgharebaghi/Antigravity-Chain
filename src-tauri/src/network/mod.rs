//! # Network Module
//!
//! This module handles all peer-to-peer networking functionality.
//!
//! ## Structure
//!
//! - `behaviour`: libp2p network behaviour definitions
//! - `commands`: Command types for controlling the P2P layer
//! - `startup`: Node startup state machine
//! - `p2p`: Main P2P node implementation

pub mod behaviour;
pub mod commands;
pub mod p2p;
pub mod startup;

// Re-exports for convenience
pub use behaviour::{message_id_fn, CentichainBehaviour, SYNC_PROTOCOL};
pub use commands::{P2PCommand, TopologyUpdate};
pub use p2p::start_p2p_node;
pub use startup::{NodeStartupState, StartupConfig};
