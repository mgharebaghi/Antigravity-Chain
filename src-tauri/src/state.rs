use crate::chain::Transaction;
use crate::consensus::mempool::Mempool;
use crate::consensus::Consensus;
use crate::storage::Storage;
use crate::wallet::Wallet;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(serde::Serialize, Clone)]
pub struct VdfStatus {
    pub iterations_per_second: u64,
    pub difficulty: u64,
    pub is_active: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
pub enum NodeType {
    Full,
    Pruned,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AppSettings {
    pub node_name: String,
    pub relay_addresses: Vec<String>, // Multiple relays for failover
    pub allow_relay_free_mode: bool,  // Allow operation without relay if DHT has peers
    pub mining_enabled: bool,
    pub max_peers: u32,
    pub node_type: NodeType,
}

impl Default for AppSettings {
    fn default() -> Self {
        use crate::utils::constants::{DEFAULT_MAX_PEERS, RELAY_ADDRESSES};
        Self {
            node_name: "Centichain-Node-01".to_string(),
            relay_addresses: RELAY_ADDRESSES
                .iter()
                .map(|s: &&str| s.to_string())
                .collect(),
            allow_relay_free_mode: true, // Enable DHT-only fallback
            mining_enabled: true,
            max_peers: DEFAULT_MAX_PEERS,
            node_type: NodeType::Pruned, // Default to home-user friendly
        }
    }
}

// Shared state
pub struct AppState {
    pub wallet: Arc<Mutex<Option<Wallet>>>,
    pub consensus: Arc<Mutex<Consensus>>,
    pub storage: Arc<Storage>,
    pub mempool: Arc<Mempool>,
    pub is_synced: Arc<AtomicBool>,
    pub is_running: Arc<AtomicBool>, // New flag for controlling the loop
    pub run_id: Arc<std::sync::atomic::AtomicU64>, // Generation counter
    pub chain_index: Arc<std::sync::atomic::AtomicU64>,
    pub mined_by_me_count: Arc<std::sync::atomic::AtomicU64>,
    pub peer_count: Arc<AtomicUsize>,
    pub validator_count: Arc<AtomicUsize>,
    pub relay_connected: Arc<AtomicBool>, // Shared relay status
    pub tx_sender: Arc<Mutex<Option<tokio::sync::mpsc::Sender<Transaction>>>>,
    pub receipt_sender: Arc<Mutex<Option<tokio::sync::mpsc::Sender<crate::chain::Receipt>>>>,
    pub mining_enabled: Arc<AtomicBool>,
    pub node_type: Arc<Mutex<NodeType>>,
    pub vdf_ips: Arc<std::sync::atomic::AtomicU64>,
}
