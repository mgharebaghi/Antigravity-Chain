pub mod api;
pub mod chain;
pub mod commands;
pub mod consensus;
pub mod network;
pub mod node;
pub mod state;
pub mod storage;
pub mod utils;
pub mod wallet;

use crate::consensus::{mempool::Mempool, Consensus};
use crate::state::{AppSettings, AppState, NodeType};
use crate::storage::Storage;
use crate::wallet::Wallet;
use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, Naming, WriteMode};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize Professional Logging
    Logger::try_with_str("info, centichain_lib=debug")
        .unwrap()
        .log_to_file(FileSpec::default().directory("logs").basename("centichain"))
        .write_mode(WriteMode::Async)
        .rotate(
            Criterion::Size(10 * 1024 * 1024), // 10MB
            Naming::Timestamps,
            Cleanup::KeepLogFiles(7),
        )
        .start()
        .expect("Failed to initialize logger");

    // Initialize DB
    let mut db_path = dirs::data_dir().unwrap_or_else(std::env::temp_dir);
    db_path.push("centichain");
    std::fs::create_dir_all(&db_path).ok();
    db_path.push("centichain.db");
    let storage = Storage::new(db_path.to_str().unwrap()).expect("Failed to create DB");
    let storage_arc = Arc::new(storage);

    // Initial load of settings
    let (initial_mining, initial_node_type) = match storage_arc.get_setting("app_settings") {
        Ok(Some(json)) => {
            let s = serde_json::from_str::<AppSettings>(&json).unwrap_or_default();
            (s.mining_enabled, s.node_type)
        }
        _ => (true, NodeType::Pruned),
    };

    // Initial metrics from DB
    let initial_height = storage_arc.get_latest_index().unwrap_or(0);

    // Attempt to derive address for initial count
    let initial_mined_count = if let Ok(Some(keys_json)) = storage_arc.get_wallet_keys() {
        if let Ok(kp_bytes) = serde_json::from_str::<Vec<u8>>(&keys_json) {
            if let Ok(kp) = libp2p::identity::Keypair::from_protobuf_encoding(&kp_bytes) {
                let addr = kp.public().to_peer_id().to_string();
                storage_arc.count_blocks_by_author(&addr).unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    let initial_wallet = if let Ok(Some(keys_json)) = storage_arc.get_wallet_keys() {
        if let Ok(kp_bytes) = serde_json::from_str::<Vec<u8>>(&keys_json) {
            if let Ok(kp) = libp2p::identity::Keypair::from_protobuf_encoding(&kp_bytes) {
                let addr = kp.public().to_peer_id().to_string();
                Some(Wallet {
                    start_timestamp: 0, // Not critical for display
                    address: addr,
                    alias: None,
                    keypair: kp_bytes,
                })
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    tauri::Builder::default()
        .manage(AppState {
            wallet: Arc::new(Mutex::new(initial_wallet)),
            consensus: Arc::new(Mutex::new(Consensus::new())),
            storage: storage_arc.clone(),
            mempool: {
                let m = Mempool::new(storage_arc.clone());
                if let Err(e) = m.load_from_db() {
                    log::error!("{}", e);
                }
                // Initial reconciliation
                if let Ok(count) = m.reconcile_with_chain() {
                    if count > 0 {
                        log::info!("Mempool reconciled: removed {} mined transactions.", count);
                    }
                }
                Arc::new(m)
            },
            is_synced: Arc::new(AtomicBool::new(false)),
            is_running: Arc::new(AtomicBool::new(false)),

            run_id: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            chain_index: Arc::new(std::sync::atomic::AtomicU64::new(initial_height)),
            mined_by_me_count: Arc::new(std::sync::atomic::AtomicU64::new(initial_mined_count)),
            peer_count: Arc::new(AtomicUsize::new(0)),
            validator_count: Arc::new(AtomicUsize::new(0)),
            relay_connected: Arc::new(AtomicBool::new(false)),
            tx_sender: Arc::new(Mutex::new(None)),
            receipt_sender: Arc::new(Mutex::new(None)),
            mining_enabled: Arc::new(AtomicBool::new(initial_mining)),
            node_type: Arc::new(Mutex::new(initial_node_type)),
            vdf_ips: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Wallet
            commands::wallet::create_wallet,
            commands::wallet::import_wallet,
            commands::wallet::get_wallet_info,
            commands::wallet::logout_wallet,
            // Node
            commands::node::start_node,
            commands::node::stop_node,
            // Block/Chain
            commands::chain::get_block,
            commands::chain::get_block_by_hash,
            commands::chain::get_transaction,
            commands::chain::get_latest_block,
            commands::chain::get_recent_blocks,
            commands::chain::get_blocks_paginated,
            commands::chain::get_chain_stats,
            commands::chain::get_mined_blocks_count,
            commands::chain::submit_transaction,
            commands::chain::get_mempool_transactions,
            commands::chain::reset_chain_data,
            commands::chain::get_tokenomics_info,
            commands::chain::get_consensus_status,
            // Network
            commands::network::get_network_info,
            commands::network::get_self_node_info,
            // General
            commands::general::greet,
            commands::general::get_app_settings,
            commands::general::save_app_settings,
            commands::general::exit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
