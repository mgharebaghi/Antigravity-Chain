pub mod chain;
pub mod consensus;
pub mod mempool;
pub mod p2p;
pub mod storage;
pub mod vdf;
pub mod wallet;

use crate::vdf::AntigravityVDF;
use chain::Transaction;
use consensus::Consensus;
use mempool::Mempool;
use rand::RngCore;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use storage::Storage;
use tauri::{Emitter, State};
use wallet::Wallet;

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
    pub relay_address: String,
    pub mining_enabled: bool,
    pub max_peers: u32,
    pub node_type: NodeType,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            node_name: "Antigravity-Node-01".to_string(),
            relay_address: "/ip4/127.0.0.1/tcp/9090".to_string(),
            mining_enabled: true,
            max_peers: 50,
            node_type: NodeType::Pruned, // Default to home-user friendly
        }
    }
}

// Shared state
struct AppState {
    wallet: Arc<Mutex<Option<Wallet>>>,
    consensus: Arc<Mutex<Consensus>>,
    storage: Arc<Storage>,
    mempool: Arc<Mempool>,
    is_synced: Arc<AtomicBool>,
    is_running: Arc<AtomicBool>, // New flag for controlling the loop
    run_id: Arc<std::sync::atomic::AtomicU64>, // Generation counter
    chain_index: Arc<std::sync::atomic::AtomicU64>,
    mined_by_me_count: Arc<std::sync::atomic::AtomicU64>,
    peer_count: Arc<AtomicUsize>,
    tx_sender: Arc<Mutex<Option<tokio::sync::mpsc::Sender<Transaction>>>>,
    mining_enabled: Arc<AtomicBool>,
    node_type: Arc<Mutex<NodeType>>,
    vdf_ips: Arc<std::sync::atomic::AtomicU64>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn create_wallet(state: State<'_, AppState>) -> Result<wallet::WalletExport, String> {
    let mut wallet_guard = state.wallet.lock().unwrap();

    // Generate Mnemonic (12 words) using 16 bytes of entropy
    let mut entropy = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut entropy);
    let mnemonic = bip39::Mnemonic::from_entropy(&entropy).map_err(|e| e.to_string())?;
    let seed = mnemonic.to_seed("");

    // Derive keypair from seed (simplified for lab, using first 32 bytes)
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&seed[0..32]);

    let keypair = libp2p::identity::Keypair::ed25519_from_bytes(key_bytes).unwrap();
    let peer_id = keypair.public().to_peer_id();
    let address = peer_id.to_string();

    let keypair_bytes = keypair.to_protobuf_encoding().unwrap();
    let keys_json = serde_json::to_string(&keypair_bytes).unwrap();

    // Save to DB
    let _ = state.storage.save_wallet_keys(&keys_json);

    let export = wallet::WalletExport {
        address: address.clone(),
        private_key: hex::encode(&keypair_bytes),
        mnemonic: mnemonic.to_string(),
    };

    let new_wallet = Wallet {
        start_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        address: address.clone(),
        alias: None,
        keypair: keypair_bytes,
    };

    *wallet_guard = Some(new_wallet);

    // Update mined blocks counter for the new wallet
    let count = state.storage.count_blocks_by_author(&address).unwrap_or(0);
    state.mined_by_me_count.store(count, Ordering::Relaxed);

    Ok(export)
}

#[tauri::command]
fn import_wallet(state: State<'_, AppState>, private_key_hex: String) -> Result<String, String> {
    let mut wallet_guard = state.wallet.lock().unwrap();

    let keypair_bytes = if private_key_hex.split_whitespace().count() == 12 {
        // Handle Mnemonic
        let mnemonic = bip39::Mnemonic::parse(&private_key_hex)
            .map_err(|e| format!("Invalid mnemonic: {}", e))?;
        let seed = mnemonic.to_seed("");
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&seed[0..32]);
        let keypair = libp2p::identity::Keypair::ed25519_from_bytes(key_bytes).unwrap();
        keypair.to_protobuf_encoding().unwrap()
    } else {
        // Handle HEX
        hex::decode(private_key_hex).map_err(|e| format!("Invalid hex: {}", e))?
    };

    // Validate keypair
    let keypair = libp2p::identity::Keypair::from_protobuf_encoding(&keypair_bytes)
        .map_err(|e| format!("Invalid keypair data: {}", e))?;

    let address = keypair.public().to_peer_id().to_string();

    let new_wallet = Wallet {
        start_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        address: address.clone(),
        alias: None,
        keypair: keypair_bytes,
    };

    let keys_json = serde_json::to_string(&new_wallet.keypair).unwrap();
    let _ = state.storage.save_wallet_keys(&keys_json);

    *wallet_guard = Some(new_wallet);

    // Update mined blocks counter for the new wallet
    let count = state.storage.count_blocks_by_author(&address).unwrap_or(0);
    state.mined_by_me_count.store(count, Ordering::Relaxed);

    Ok(address)
}

#[derive(serde::Serialize, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub trust_score: f64,
    pub is_verified: bool,
    pub latency: u64,
    pub addresses: Vec<String>,
}

#[tauri::command]
fn get_network_info(state: State<'_, AppState>) -> Vec<PeerInfo> {
    let consensus = state.consensus.lock().unwrap();
    consensus
        .nodes
        .values()
        .map(|n| PeerInfo {
            peer_id: n.peer_id.clone(),
            trust_score: n.trust_score,
            is_verified: n.is_verified,
            latency: 0,
            addresses: n.addresses.clone(),
        })
        .collect()
}

#[derive(serde::Serialize)]
pub struct SelfNodeInfo {
    pub peer_id: String,
    pub addresses: Vec<String>,
}

#[tauri::command]
fn get_self_node_info(state: State<'_, AppState>) -> Option<SelfNodeInfo> {
    let consensus = state.consensus.lock().unwrap();
    consensus.local_peer_id.as_ref().map(|id| {
        let addresses = consensus
            .nodes
            .get(id)
            .map(|n| n.addresses.clone())
            .unwrap_or_default();
        SelfNodeInfo {
            peer_id: id.clone(),
            addresses,
        }
    })
}

#[tauri::command]
fn get_wallet_info(state: State<'_, AppState>) -> Option<wallet::WalletInfo> {
    let wallet_guard = state.wallet.lock().unwrap();
    if let Some(w) = wallet_guard.as_ref() {
        let total_balance = state.storage.calculate_balance(&w.address).unwrap_or(0);
        let pending_spend = state.mempool.get_total_pending_spend(&w.address);
        let available_balance = total_balance.saturating_sub(pending_spend);

        Some(wallet::WalletInfo {
            address: w.address.clone(),
            balance: available_balance,
            alias: w.alias.clone(),
        })
    } else {
        None
    }
}

#[tauri::command]
async fn logout_wallet(state: State<'_, AppState>) -> Result<(), String> {
    println!("Backend: logout_wallet called");

    // 1. Clear in-memory wallet
    {
        let mut wallet = state.wallet.lock().map_err(|e| e.to_string())?;
        *wallet = None;
    }

    // 2. Clear mined count (optional, but makes sense for UI)
    state.mined_by_me_count.store(0, Ordering::SeqCst);

    // 3. Delete from storage
    state
        .storage
        .delete_wallet_keys()
        .map_err(|e| e.to_string())?;

    println!("Backend: Wallet logged out successfully");
    Ok(())
}

#[tauri::command]
async fn start_node(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Check if wallet exists
    {
        let wallet_guard = state.wallet.lock().unwrap();
        if wallet_guard.is_none() {
            return Err(
                "Wallet required to start node. Please create or import a wallet first."
                    .to_string(),
            );
        }
    }

    // Check if already running
    if state.is_running.load(Ordering::Relaxed) {
        return Ok("Node is already running".to_string());
    }
    state.is_running.store(true, Ordering::Relaxed);

    // Increment run_id to invalidate previous loops
    let my_run_id = state.run_id.fetch_add(1, Ordering::Relaxed) + 1;

    let storage = state.storage.clone();
    let mempool = state.mempool.clone();
    let is_synced = state.is_synced.clone();
    let is_running = state.is_running.clone();
    let run_id_arc = state.run_id.clone();
    let peer_count = state.peer_count.clone();
    let node_type_arc = state.node_type.clone();
    let node_type_p2p = node_type_arc.clone();

    // Create channels for mining loop -> p2p and submit_tx -> p2p
    let (block_sender, block_receiver) = tokio::sync::mpsc::channel::<Box<chain::Block>>(100);
    let (tx_sender, tx_receiver) = tokio::sync::mpsc::channel::<chain::Transaction>(100);

    // Store tx_sender in state for command access
    {
        let mut sender_guard = state.tx_sender.lock().unwrap();
        *sender_guard = Some(tx_sender);
    }

    // Spawn P2P
    let app_handle_clone = app_handle.clone();
    let consensus_p2p = state.consensus.clone(); // Clone for P2P

    let chain_index = state.chain_index.clone();
    // Fetch settings
    let settings = match state.storage.get_setting("app_settings") {
        Ok(Some(json)) => serde_json::from_str::<AppSettings>(&json).unwrap_or_default(),
        _ => AppSettings::default(),
    };
    let relay_addr_str = settings.relay_address.clone();

    tauri::async_runtime::spawn(async move {
        if let Err(e) = p2p::start_p2p_node(
            app_handle_clone,
            storage,
            mempool,
            consensus_p2p,
            is_synced,
            is_running,
            run_id_arc,
            peer_count,
            chain_index,
            relay_addr_str,
            my_run_id,
            block_receiver,
            tx_receiver,
            node_type_p2p,
        )
        .await
        {
            log::error!("P2P Node error: {}", e);
        }
    });

    // Spawn Genesis Checker & Block Production Loop
    let storage_clone = state.storage.clone();
    let mempool_clone = state.mempool.clone();
    let is_synced_loop = state.is_synced.clone();
    let consensus_clone = state.consensus.clone();
    let is_running_loop = state.is_running.clone();
    let run_id_loop = state.run_id.clone();
    let peer_count_loop = state.peer_count.clone();
    let wallet_addr = {
        let w = state.wallet.lock().unwrap();
        w.as_ref().unwrap().address.clone()
    };
    let app_handle_loop = app_handle.clone();

    // Initial load from storage to sync memory counter
    if let Ok(count) = storage_clone.count_blocks_by_author(&wallet_addr) {
        state.mined_by_me_count.store(count, Ordering::Relaxed);
    }
    let block_sender_loop = block_sender.clone();
    let chain_index_loop = state.chain_index.clone();
    let mined_by_me_count_loop = state.mined_by_me_count.clone();
    let wallet_clone = state.wallet.clone(); // Clone ARC for loop
    let mining_enabled_arc = state.mining_enabled.clone();

    // Initialize metrics from storage
    let current_height = state.storage.get_latest_index().unwrap_or(0);
    state.chain_index.store(current_height, Ordering::Relaxed);

    if let Some(w) = state.wallet.lock().unwrap().as_ref() {
        let count = state
            .storage
            .count_blocks_by_author(&w.address)
            .unwrap_or(0);
        state.mined_by_me_count.store(count, Ordering::Relaxed);
    }

    // Spawn VDF Heartbeat Loop
    let app_handle_vdf = app_handle.clone();
    let is_running_vdf = state.is_running.clone();
    let run_id_vdf = state.run_id.clone();
    let vdf_ips_arc = state.vdf_ips.clone();

    tauri::async_runtime::spawn(async move {
        println!("VDF Heartbeat: Thread started for run_id: {}", my_run_id);
        let mut last_emit = std::time::Instant::now();
        let difficulty = 200_000; // Standard difficulty for display

        loop {
            if !is_running_vdf.load(Ordering::Relaxed)
                || run_id_vdf.load(Ordering::Relaxed) != my_run_id
            {
                break;
            }

            // Benchmark VDF performance
            let start = std::time::Instant::now();
            let vdf = AntigravityVDF::new(50_000); // Small batch for responsiveness
            vdf.solve(b"heartbeat_challenge");
            let elapsed = start.elapsed();

            let ips = (50_000.0 / elapsed.as_secs_f64()) as u64;
            vdf_ips_arc.store(ips, Ordering::Relaxed);

            if last_emit.elapsed() >= Duration::from_secs(1) {
                let _ = app_handle_vdf.emit(
                    "vdf-status",
                    VdfStatus {
                        iterations_per_second: ips,
                        difficulty,
                        is_active: true,
                    },
                );
                last_emit = std::time::Instant::now();
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        println!("VDF Heartbeat: Terminating run_id {}", my_run_id);
    });

    tauri::async_runtime::spawn(async move {
        println!("Mining Loop: Thread started for run_id: {}", my_run_id);
        // Phase 1: Connect to Relay
        let _ = app_handle_loop.emit("node-status", "Connecting to Relay...");

        let mut relay_connected = false;
        for i in 0..10 {
            if !is_running_loop.load(Ordering::Relaxed) {
                println!("Mining Loop: Aborted during relay wait");
                break;
            }

            let current_count = peer_count_loop.load(Ordering::Relaxed);
            if current_count > 0 {
                println!("Mining Loop: Relay/Peer detected! Count: {}", current_count);
                relay_connected = true;
                break;
            }
            println!("Mining Loop: Waiting for relay... (attempt {})", i + 1);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        if !relay_connected {
            println!("Mining Loop: Relay connection failed/timed out. Continuing as standalone...");
            let _ = app_handle_loop.emit("node-status", "Relay Timeout (Standalone)");
        }

        // Phase 2: Discovery (Wait for peers)
        let _ = app_handle_loop.emit("node-status", "Discovering Peers...");
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Phase 3: Decision
        let peers = peer_count_loop.load(Ordering::Relaxed);
        let local_height_opt = storage_clone.get_block(0).unwrap_or(None);
        let local_exists = local_height_opt.is_some();

        println!(
            "Mining Loop: Decision Phase - Peers: {}, LocalChainExists: {}",
            peers, local_exists
        );

        if !local_exists {
            // We move Genesis creation to the mining loop to ensure it gets assigned to the active miner
            // when the chain is empty (especially after a reset).
            println!("Mining Loop: First node! Creating Genesis...");
            let _ = app_handle_loop.emit("node-status", "Creating Genesis...");

            let genesis_tx = Transaction {
                id: "genesis".to_string(),
                sender: "SYSTEM".to_string(),
                receiver: wallet_addr.clone(),
                amount: crate::chain::GENESIS_SUPPLY,
                timestamp: 0,
                signature: "genesis".to_string(),
            };
            let mut genesis_block = chain::Block::new(
                0,
                wallet_addr.clone(),
                vec![genesis_tx],
                "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                100,
                200_000,
                0,                            // Fees for genesis
                crate::chain::GENESIS_SUPPLY, // Genesis supply as reward
            );

            // VDF
            let vdf = AntigravityVDF::new(genesis_block.vdf_difficulty);
            let challenge = genesis_block.calculate_hash();
            genesis_block.vdf_proof = vdf.solve(challenge.as_bytes());
            genesis_block.hash = genesis_block.calculate_hash();
            genesis_block.size = genesis_block.calculate_size();

            let _ = storage_clone.save_block(&genesis_block);
            chain_index_loop.store(0, Ordering::Relaxed);
            mined_by_me_count_loop.fetch_add(1, Ordering::Relaxed);
            let _ = app_handle_loop.emit("new-block", genesis_block);
        }

        is_synced_loop.store(true, Ordering::Relaxed);
        let _ = app_handle_loop.emit("node-status", "Active");
        println!("Mining Loop: Node is now ACTIVE");

        // Phase 4: Main Loop
        let mut last_production_time = std::time::Instant::now();

        loop {
            if !is_running_loop.load(Ordering::Relaxed)
                || run_id_loop.load(Ordering::Relaxed) != my_run_id
            {
                println!("Mining Loop: Terminating run_id {}", my_run_id);
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;

            if is_synced_loop.load(Ordering::Relaxed) {
                let (is_leader, leader_id) = {
                    let consensus = consensus_clone.lock().unwrap();
                    let leader = consensus.select_leader();
                    let me = consensus.local_peer_id.clone();
                    println!(
                        "Mining Loop: Leader Election - Leader: {:?}, Me: {:?}",
                        leader, me
                    );
                    (leader.is_some() && leader == me, leader)
                };

                let pending = mempool_clone.get_pending_transactions();
                let mining_enabled = mining_enabled_arc.load(Ordering::Relaxed);

                let elapsed = last_production_time.elapsed().as_secs();
                if mining_enabled && is_leader {
                    if elapsed >= 10 || pending.len() >= 10 {
                        let current_idx = chain_index_loop.load(Ordering::Relaxed);
                        let is_empty = storage_clone.get_total_blocks().unwrap_or(0) == 0;
                        let target_idx = if is_empty { 0 } else { current_idx + 1 };

                        println!(
                            "Mining Loop: Producing block {}... (elapsed: {}s, txs: {})",
                            target_idx,
                            elapsed,
                            pending.len()
                        );

                        let current_wallet_addr = wallet_clone
                            .lock()
                            .unwrap()
                            .as_ref()
                            .map(|w| w.address.clone())
                            .unwrap_or_else(|| wallet_addr.clone());

                        last_production_time = std::time::Instant::now();

                        let block_reward = if target_idx == 0 {
                            crate::chain::GENESIS_SUPPLY
                        } else {
                            crate::chain::calculate_mining_reward(target_idx)
                        };

                        let total_fees = pending
                            .iter()
                            .map(|tx| crate::chain::calculate_fee(tx.amount))
                            .sum::<u64>();

                        let coinbase_tx = if target_idx == 0 {
                            chain::Transaction {
                                id: "genesis".to_string(),
                                sender: "SYSTEM".to_string(),
                                receiver: current_wallet_addr.clone(),
                                amount: block_reward,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                                signature: "genesis".to_string(),
                            }
                        } else {
                            chain::Transaction {
                                id: uuid::Uuid::new_v4().to_string(),
                                sender: "SYSTEM".to_string(),
                                receiver: current_wallet_addr.clone(),
                                amount: block_reward + total_fees,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                                signature: "reward".to_string(),
                            }
                        };

                        let mut block_txs = vec![coinbase_tx];
                        block_txs.extend(pending.clone());

                        let prev_hash = if target_idx == 0 {
                            "0000000000000000000000000000000000000000000000000000000000000000"
                                .to_string()
                        } else {
                            storage_clone
                                .get_block(current_idx)
                                .unwrap_or(None)
                                .map(|b| b.hash)
                                .unwrap_or_else(|| {
                                    "0000000000000000000000000000000000000000000000000000000000000000"
                                        .to_string()
                                })
                        };

                        let mut new_block = chain::Block::new(
                            target_idx,
                            current_wallet_addr.clone(),
                            block_txs,
                            prev_hash,
                            100,
                            200_000,
                            total_fees,
                            block_reward,
                        );

                        let vdf = AntigravityVDF::new(new_block.vdf_difficulty);
                        let challenge = new_block.calculate_hash();
                        new_block.vdf_proof = vdf.solve(challenge.as_bytes());
                        new_block.hash = new_block.calculate_hash();
                        new_block.size = new_block.calculate_size();

                        let _ = storage_clone.save_block(&new_block);

                        // Pruning logic for local miner
                        let nt = {
                            let guard = node_type_arc.lock().unwrap();
                            guard.clone()
                        };
                        if nt == NodeType::Pruned {
                            let _ = storage_clone.prune_history(2000);
                        }

                        chain_index_loop.store(new_block.index, Ordering::Relaxed);
                        mined_by_me_count_loop.fetch_add(1, Ordering::Relaxed);
                        let _ = app_handle_loop.emit("new-block", new_block.clone());

                        if let Err(e) = block_sender_loop.send(Box::new(new_block)).await {
                            log::error!("Broadcast Error: {}", e);
                        }

                        let tx_ids: Vec<String> = pending.iter().map(|tx| tx.id.clone()).collect();
                        mempool_clone.remove_transactions(&tx_ids);
                    }
                } else if !mining_enabled {
                    // Periodic log to show thread is alive
                    if elapsed % 10 == 0 {
                        println!("Mining Loop: Waiting (Mining disabled)");
                    }
                } else {
                    if elapsed % 10 == 0 {
                        println!(
                            "Mining Loop: Waiting (Not leader, leader is {:?})",
                            leader_id
                        );
                    }
                }
            }
        }
    });

    Ok("Node started".to_string())
}

#[tauri::command]
fn stop_node(state: State<'_, AppState>) -> Result<String, String> {
    state.is_running.store(false, Ordering::Relaxed);
    // Note: We don't necessarily need to increment run_id here since is_running=false is checked.
    // But incrementing ensures double safety.
    state.run_id.fetch_add(1, Ordering::Relaxed);
    Ok("Node stopped".to_string())
}

#[tauri::command]
fn get_block(state: State<'_, AppState>, index: u64) -> Result<Option<chain::Block>, String> {
    state.storage.get_block(index).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_block_by_hash(
    state: State<'_, AppState>,
    hash: String,
) -> Result<Option<chain::Block>, String> {
    state
        .storage
        .get_block_by_hash(&hash)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_transaction(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<(crate::chain::Transaction, chain::Block)>, String> {
    state
        .storage
        .get_transaction_by_id(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_latest_block(state: State<'_, AppState>) -> Result<Option<chain::Block>, String> {
    let latest_index = state
        .storage
        .get_latest_index()
        .map_err(|e| e.to_string())?;
    state
        .storage
        .get_block(latest_index)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_blocks_paginated(
    state: State<'_, AppState>,
    page: usize,
    limit: usize,
) -> Result<Vec<chain::Block>, String> {
    println!(
        "Backend: get_blocks_paginated called (page: {}, limit: {})",
        page, limit
    );
    state
        .storage
        .get_blocks_paginated(page, limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_recent_blocks(
    state: State<'_, AppState>,
    limit: usize,
) -> Result<Vec<chain::Block>, String> {
    state
        .storage
        .get_recent_blocks(limit)
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct ChainStats {
    pub total_blocks: u64,
    pub height: u64,
}

#[tauri::command]
fn get_chain_stats(state: State<'_, AppState>) -> Result<ChainStats, String> {
    let total = state
        .storage
        .get_total_blocks()
        .map_err(|e| e.to_string())?;
    let height = state.chain_index.load(Ordering::Relaxed);
    println!(
        "Backend: get_chain_stats called (total: {}, height: {})",
        total, height
    );
    Ok(ChainStats {
        total_blocks: total,
        height,
    })
}

#[tauri::command]
fn get_mined_blocks_count(state: State<'_, AppState>) -> u64 {
    let count = state.mined_by_me_count.load(Ordering::Relaxed);
    let wallet_addr = state
        .wallet
        .lock()
        .unwrap()
        .as_ref()
        .map(|w| w.address.clone())
        .unwrap_or_else(|| "No Wallet".to_string());
    println!(
        "Backend: get_mined_blocks_count called (count: {}, wallet: {})",
        count, wallet_addr
    );
    count
}

#[tauri::command]
fn submit_transaction(
    state: State<'_, AppState>,
    receiver: String,
    amount: u64,
) -> Result<String, String> {
    let wallet_guard = state.wallet.lock().unwrap();

    // Check Peer Count
    if state.peer_count.load(Ordering::Relaxed) == 0 {
        return Err("Not connected to network (0 peers). Try restarting or wait.".to_string());
    }

    // Validate Address
    if let Err(_) = receiver.parse::<libp2p::PeerId>() {
        return Err("Invalid receiver address. Address must be a valid Network Identity (e.g., starts with 12D3...)".to_string());
    }

    if let Some(wallet) = wallet_guard.as_ref() {
        if receiver == wallet.address {
            return Err("You cannot send coins to your own address.".to_string());
        }

        // Fee Logic
        let dynamic_fee = crate::chain::calculate_fee(amount);
        let balance = state
            .storage
            .calculate_balance(&wallet.address)
            .unwrap_or(0);

        // Check Mempool Spend (Effective Balance)
        let pending_spend = state.mempool.get_total_pending_spend(&wallet.address);
        let effective_balance = balance.saturating_sub(pending_spend);

        // Check Balance
        let total_required = amount.saturating_add(dynamic_fee);
        if total_required > effective_balance {
            let divisor = crate::chain::ONE_AGT as f64;
            return Err(format!(
                "Insufficient funds. Balance: {:.6} AGT (Pending spent: {:.6}), Required: {:.6} AGT",
                balance as f64 / divisor,
                pending_spend as f64 / divisor,
                total_required as f64 / divisor
            ));
        }

        // Create Transaction
        // In real app: Sign with Keypair
        let tx = Transaction {
            id: uuid::Uuid::new_v4().to_string(), // Need uuid crate or simple random
            sender: wallet.address.clone(),
            receiver,
            amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            signature: "sig".to_string(),
        };

        state.mempool.add_transaction(tx.clone()).map_err(|e| e)?;

        // Broadcast to P2P
        let sender_guard = state.tx_sender.lock().unwrap();
        if let Some(sender) = sender_guard.as_ref() {
            if let Err(e) = sender.try_send(tx.clone()) {
                log::error!("Broadcast Channel Error: {}", e);
            }
        }

        Ok(tx.id)
    } else {
        Err("No wallet".to_string())
    }
}

#[tauri::command]
fn get_mempool_transactions(state: State<'_, AppState>) -> Vec<Transaction> {
    state.mempool.get_pending_transactions()
}

#[tauri::command]
fn get_app_settings(state: State<'_, AppState>) -> AppSettings {
    match state.storage.get_setting("app_settings") {
        Ok(Some(json)) => serde_json::from_str(&json).unwrap_or_default(),
        _ => AppSettings::default(),
    }
}

#[tauri::command]
fn save_app_settings(state: State<'_, AppState>, settings: AppSettings) -> Result<(), String> {
    // Update reactive flags
    state
        .mining_enabled
        .store(settings.mining_enabled, Ordering::Relaxed);

    {
        let mut nt = state.node_type.lock().unwrap();
        *nt = settings.node_type.clone();
    }

    let json = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    state
        .storage
        .save_setting("app_settings", &json)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn reset_chain_data(state: State<'_, AppState>) -> Result<(), String> {
    state.storage.reset_blocks().map_err(|e| e.to_string())?;
    state.chain_index.store(0, Ordering::Relaxed);
    // Also reset mined_by_me if we want a full reset
    state.mined_by_me_count.store(0, Ordering::Relaxed);
    Ok(())
}

#[derive(serde::Serialize)]
pub struct TokenomicsInfo {
    pub total_supply: u64,
    pub max_supply: u64,
    pub circulating_supply: u64,
    pub remaining_supply: u64,
    pub next_halving_at: u64,
    pub blocks_until_halving: u64,
    pub current_reward: u64,
    pub halving_interval: u64,
}

#[tauri::command]
fn get_tokenomics_info(state: State<'_, AppState>) -> TokenomicsInfo {
    let height = state.chain_index.load(Ordering::Relaxed);
    let next_halving = if height <= 500_000 {
        ((height / crate::chain::HALVING_INTERVAL_EARLY) + 1) * crate::chain::HALVING_INTERVAL_EARLY
    } else {
        500_000
            + (((height - 500_000) / crate::chain::HALVING_INTERVAL_STABLE) + 1)
                * crate::chain::HALVING_INTERVAL_STABLE
    };

    let halving_interval = if height < 500_000 {
        crate::chain::HALVING_INTERVAL_EARLY
    } else {
        crate::chain::HALVING_INTERVAL_STABLE
    };

    let circulating = crate::chain::calculate_circulating_supply(height);

    TokenomicsInfo {
        total_supply: crate::chain::TOTAL_SUPPLY,
        max_supply: crate::chain::TOTAL_SUPPLY,
        circulating_supply: circulating,
        remaining_supply: crate::chain::TOTAL_SUPPLY.saturating_sub(circulating),
        next_halving_at: next_halving,
        blocks_until_halving: next_halving.saturating_sub(height),
        current_reward: crate::chain::calculate_mining_reward(height),
        halving_interval,
    }
}

#[tauri::command]
fn exit_app() {
    std::process::exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize DB
    let mut db_path = std::env::temp_dir();
    db_path.push("antigravity.db");
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
            tx_sender: Arc::new(Mutex::new(None)),
            mining_enabled: Arc::new(AtomicBool::new(initial_mining)),
            node_type: Arc::new(Mutex::new(initial_node_type)),
            vdf_ips: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            create_wallet,
            import_wallet,
            get_wallet_info,
            start_node,
            stop_node,
            get_block,
            get_block_by_hash,
            get_transaction,
            get_latest_block,
            get_recent_blocks,
            get_blocks_paginated,
            get_chain_stats,
            get_mined_blocks_count,
            submit_transaction,
            get_mempool_transactions,
            exit_app,
            get_network_info,
            get_self_node_info,
            get_app_settings,
            save_app_settings,
            reset_chain_data,
            logout_wallet,
            get_tokenomics_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
