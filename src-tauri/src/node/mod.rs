//! # Node Module
//!
//! Handles node operations including mining and block production.
//!
//! ## Structure
//!
//! - `mining`: Main mining loop and block production
//! - `relay`: Relay connection handling
//! - `network_init`: Network discovery and synchronization
//! - `helpers`: Block production helper functions
//! - `manager`: Node service management
//! - `vdf`: VDF solver and heartbeat

pub mod helpers;
pub mod manager;
pub mod mining;
pub mod network_init;
pub mod relay;
pub mod vdf;

// Re-exports for convenience
pub use helpers::{
    collect_shard_transactions, create_coinbase_tx, run_auto_pruning, slash_missed_slots,
};
pub use manager::start_node_service;
pub use mining::spawn_mining_loop;
pub use network_init::{
    create_genesis_block, initialize_network_state, sync_with_network, wait_for_peers,
    PEER_DISCOVERY_TIMEOUT, SYNC_TIMEOUT,
};
pub use relay::{emit_relay_error, wait_for_relay, RELAY_CONNECTION_TIMEOUT};
