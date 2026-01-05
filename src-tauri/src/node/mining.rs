//! # Mining Module
//!
//! This module handles block production and the main mining loop.
//!
//! Key responsibilities:
//! - Network discovery and initialization
//! - Genesis block creation (for first node)
//! - Slot-based leader election and block production
//! - Transaction processing and cross-shard receipts
//!
//! IMPORTANT: This loop is designed to be non-blocking and yield-friendly
//! to allow P2P and VDF operations to run concurrently.

use crate::chain::{self, Transaction};
use crate::consensus::mempool::Mempool;
use crate::consensus::vdf::CentichainVDF;
use crate::consensus::Consensus;
use crate::state::NodeType;
use crate::storage::Storage;
use crate::wallet::Wallet;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

// =============================================================================
// Constants
// =============================================================================

/// Time to wait for relay connection (seconds)
const RELAY_CONNECTION_TIMEOUT: u64 = 10;

/// Time to search for peers before assuming first node (seconds)
const PEER_DISCOVERY_TIMEOUT: u64 = 60;

/// Maximum time to wait for sync (seconds)
const SYNC_TIMEOUT: u64 = 300;

/// Minimum slot progress (seconds) before producing a block
/// This allows network gossip to propagate first
const MIN_SLOT_PROGRESS_FOR_PRODUCTION: u64 = 1;

// =============================================================================
// Mining Loop Entry Point
// =============================================================================

/// Spawns the main mining loop as an async task.
///
/// The loop has three phases:
/// 1. **Relay Connection**: Connect to relay and verify connectivity
/// 2. **Discovery/Sync**: Find peers or establish as first node
/// 3. **Production**: Continuous block production when leader
#[allow(clippy::too_many_arguments)]
pub fn spawn_mining_loop(
    app_handle: AppHandle,
    is_running: Arc<AtomicBool>,
    run_id: Arc<AtomicU64>,
    peer_count: Arc<AtomicUsize>,
    validator_count: Arc<AtomicUsize>,
    storage: Arc<Storage>,
    mempool: Arc<Mempool>,
    consensus: Arc<Mutex<Consensus>>,
    is_synced: Arc<AtomicBool>,
    chain_index: Arc<AtomicU64>,
    mined_by_me_count: Arc<AtomicU64>,
    wallet_store: Arc<Mutex<Option<Wallet>>>,
    mining_enabled: Arc<AtomicBool>,
    receipt_sender: Arc<Mutex<Option<tokio::sync::mpsc::Sender<crate::chain::Receipt>>>>,
    node_type: Arc<Mutex<NodeType>>,
    cmd_tx: tokio::sync::mpsc::Sender<crate::network::p2p::P2PCommand>,
    block_sender: tokio::sync::mpsc::Sender<Box<crate::chain::Block>>,
    my_run_id: u64,
    wallet_addr: String,
    relay_connected: Arc<AtomicBool>,
) {
    tauri::async_runtime::spawn(async move {
        log::info!("Mining Loop: Started for run_id: {}", my_run_id);

        // =====================================================================
        // Phase 1: Relay Connection
        // =====================================================================
        let _ = app_handle.emit("node-status", "Connecting to Relay...");

        let relay_ok = wait_for_relay(
            &is_running,
            &run_id,
            my_run_id,
            &relay_connected,
            RELAY_CONNECTION_TIMEOUT,
        )
        .await;

        if !relay_ok {
            log::error!("Mining Loop: Relay connection failed");
            emit_relay_error(&app_handle, &is_running, &run_id, my_run_id).await;
            return;
        }

        // =====================================================================
        // Phase 2: Network Discovery & Sync
        // =====================================================================
        let init_result = initialize_network_state(
            &app_handle,
            &is_running,
            &run_id,
            my_run_id,
            &validator_count,
            &storage,
            &is_synced,
            &consensus,
            &chain_index,
            &mined_by_me_count,
            &cmd_tx,
            &wallet_addr,
            &peer_count,
        )
        .await;

        if !init_result {
            log::error!("Mining Loop: Network initialization failed");
            return;
        }

        // =====================================================================
        // Phase 3: Block Production Loop
        // =====================================================================
        log::info!("Mining Loop: Entering production phase");

        block_production_loop(
            app_handle,
            is_running,
            run_id,
            my_run_id,
            validator_count,
            storage,
            mempool,
            consensus,
            is_synced,
            chain_index,
            mined_by_me_count,
            wallet_store,
            mining_enabled,
            receipt_sender,
            node_type,
            block_sender,
            wallet_addr,
        )
        .await;
    });
}

// =============================================================================
// Phase 1: Relay Connection
// =============================================================================

/// Waits for relay connection or timeout
async fn wait_for_relay(
    is_running: &Arc<AtomicBool>,
    run_id: &Arc<AtomicU64>,
    my_run_id: u64,
    relay_connected: &Arc<AtomicBool>,
    timeout_secs: u64,
) -> bool {
    for i in 0..timeout_secs {
        if !is_running.load(Ordering::Relaxed) || run_id.load(Ordering::Relaxed) != my_run_id {
            return false;
        }

        if relay_connected.load(Ordering::Relaxed) {
            log::info!("Mining Loop: Relay connected after {}s", i);
            return true;
        }

        log::debug!("Mining Loop: Waiting for relay... ({}s)", i);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    false
}

/// Emits relay error and waits for node stop
async fn emit_relay_error(
    app_handle: &AppHandle,
    is_running: &Arc<AtomicBool>,
    run_id: &Arc<AtomicU64>,
    my_run_id: u64,
) {
    let _ = app_handle.emit("node-status", "Error: Relay Unreachable");

    while is_running.load(Ordering::Relaxed) && run_id.load(Ordering::Relaxed) == my_run_id {
        let _ = app_handle.emit(
            "node-status",
            "Error: Relay Unreachable. Please check config/network.",
        );
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

// =============================================================================
// Phase 2: Network Initialization
// =============================================================================

/// Initializes network state: discovers peers, syncs, or becomes first node
#[allow(clippy::too_many_arguments)]
async fn initialize_network_state(
    app_handle: &AppHandle,
    is_running: &Arc<AtomicBool>,
    run_id: &Arc<AtomicU64>,
    my_run_id: u64,
    validator_count: &Arc<AtomicUsize>,
    storage: &Arc<Storage>,
    is_synced: &Arc<AtomicBool>,
    consensus: &Arc<Mutex<Consensus>>,
    chain_index: &Arc<AtomicU64>,
    mined_by_me_count: &Arc<AtomicU64>,
    cmd_tx: &tokio::sync::mpsc::Sender<crate::network::p2p::P2PCommand>,
    wallet_addr: &str,
    peer_count: &Arc<AtomicUsize>,
) -> bool {
    loop {
        if !is_running.load(Ordering::Relaxed) || run_id.load(Ordering::Relaxed) != my_run_id {
            return false;
        }

        let peers = validator_count.load(Ordering::Relaxed);
        let local_chain_exists = storage.get_block(0).unwrap_or(None).is_some();

        log::info!(
            "Mining Loop: Discovery - Peers: {}, LocalChain: {}",
            peers,
            local_chain_exists
        );

        if peers > 0 {
            // Peers found - sync with network
            return sync_with_network(
                app_handle, is_running, run_id, my_run_id, storage, is_synced, cmd_tx, peer_count,
            )
            .await;
        }

        // No peers - either wait for discovery or become first node
        if !local_chain_exists {
            // Wait for peer discovery
            let found_peers = wait_for_peers(
                app_handle,
                is_running,
                run_id,
                my_run_id,
                validator_count,
                cmd_tx,
                PEER_DISCOVERY_TIMEOUT,
            )
            .await;

            if found_peers {
                continue; // Restart loop to sync with found peers
            }

            // No peers found - become first node
            log::info!("Mining Loop: No peers found. Creating Genesis...");
            create_genesis_block(
                app_handle,
                storage,
                consensus,
                chain_index,
                mined_by_me_count,
                is_synced,
                wallet_addr,
            )
            .await;
            return true;
        }

        // Local chain exists, no peers - continue solo
        log::info!("Mining Loop: Resuming solo mining with existing chain");
        {
            let mut c = consensus.lock().unwrap();
            c.force_activate_local();
        }
        is_synced.store(true, Ordering::Relaxed);
        let _ = app_handle.emit("node-status", "Active (Solo)");
        return true;
    }
}

/// Waits for peers to be discovered
async fn wait_for_peers(
    app_handle: &AppHandle,
    is_running: &Arc<AtomicBool>,
    run_id: &Arc<AtomicU64>,
    my_run_id: u64,
    validator_count: &Arc<AtomicUsize>,
    cmd_tx: &tokio::sync::mpsc::Sender<crate::network::p2p::P2PCommand>,
    timeout_secs: u64,
) -> bool {
    for i in 0..timeout_secs {
        if !is_running.load(Ordering::Relaxed) || run_id.load(Ordering::Relaxed) != my_run_id {
            return false;
        }

        if validator_count.load(Ordering::Relaxed) > 0 {
            return true;
        }

        let _ = app_handle.emit("node-status", format!("Discovering Network... ({}s)", i));
        let _ = cmd_tx.try_send(crate::network::p2p::P2PCommand::SyncWithNetwork);

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    false
}

/// Syncs with the network
async fn sync_with_network(
    app_handle: &AppHandle,
    is_running: &Arc<AtomicBool>,
    run_id: &Arc<AtomicU64>,
    my_run_id: u64,
    storage: &Arc<Storage>,
    is_synced: &Arc<AtomicBool>,
    cmd_tx: &tokio::sync::mpsc::Sender<crate::network::p2p::P2PCommand>,
    peer_count: &Arc<AtomicUsize>,
) -> bool {
    log::info!("Mining Loop: Starting sync with network");
    let _ = app_handle.emit("node-status", "Synchronizing...");
    let _ = cmd_tx
        .send(crate::network::p2p::P2PCommand::SyncWithNetwork)
        .await;

    for i in 0..SYNC_TIMEOUT {
        if !is_running.load(Ordering::Relaxed) || run_id.load(Ordering::Relaxed) != my_run_id {
            return false;
        }

        let height = storage.get_latest_index().unwrap_or(0);
        let peers = peer_count.load(Ordering::Relaxed);

        // Check if synced (either flag set or have local blocks after timeout)
        if is_synced.load(Ordering::Relaxed) {
            log::info!("Mining Loop: Sync complete at height {}", height);
            return true;
        }

        if storage.get_block(0).unwrap_or(None).is_some() && i > 10 {
            log::info!("Mining Loop: Local chain detected, marking synced");
            is_synced.store(true, Ordering::Relaxed);
            return true;
        }

        if i % 5 == 0 {
            let _ = app_handle.emit(
                "node-status",
                format!("Synchronizing... ({} peers, {}s)", peers, i),
            );
            let _ = cmd_tx.try_send(crate::network::p2p::P2PCommand::SyncWithNetwork);
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // Timeout - check if we have any data
    if storage.get_block(0).unwrap_or(None).is_some() {
        is_synced.store(true, Ordering::Relaxed);
        return true;
    }

    false
}

/// Creates the genesis block
async fn create_genesis_block(
    app_handle: &AppHandle,
    storage: &Arc<Storage>,
    consensus: &Arc<Mutex<Consensus>>,
    chain_index: &Arc<AtomicU64>,
    mined_by_me_count: &Arc<AtomicU64>,
    is_synced: &Arc<AtomicBool>,
    wallet_addr: &str,
) {
    let _ = app_handle.emit("node-status", "Creating Genesis Block...");

    let genesis_tx = Transaction {
        id: "genesis".to_string(),
        sender: "SYSTEM".to_string(),
        receiver: wallet_addr.to_string(),
        amount: crate::utils::constants::GENESIS_SUPPLY,
        shard_id: 0,
        timestamp: 0,
        signature: "genesis".to_string(),
    };

    let mut genesis_block = chain::Block::new(
        0,
        wallet_addr.to_string(),
        vec![genesis_tx],
        "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        100,
        100, // Low difficulty for genesis
        0,
        0,
        crate::utils::constants::GENESIS_SUPPLY,
    );

    // Solve VDF for genesis (quick, low difficulty)
    let vdf = CentichainVDF::new(genesis_block.vdf_difficulty);
    let challenge = genesis_block.calculate_hash();
    genesis_block.vdf_proof = vdf.solve(challenge.as_bytes());
    genesis_block.hash = genesis_block.calculate_hash();
    genesis_block.size = genesis_block.calculate_size();

    // Save genesis
    if let Err(e) = storage.save_block(&genesis_block) {
        log::error!("Failed to save genesis block: {}", e);
        return;
    }

    chain_index.store(0, Ordering::Relaxed);
    mined_by_me_count.fetch_add(1, Ordering::Relaxed);
    let _ = app_handle.emit("new-block", genesis_block);

    // Activate local node as genesis creator
    {
        let mut c = consensus.lock().unwrap();
        c.force_activate_local();
    }

    is_synced.store(true, Ordering::Relaxed);
    let _ = app_handle.emit("node-status", "Active (Genesis)");
    log::info!("Mining Loop: Genesis block created successfully");
}

// =============================================================================
// Phase 3: Block Production
// =============================================================================

/// Main block production loop
#[allow(clippy::too_many_arguments)]
async fn block_production_loop(
    app_handle: AppHandle,
    is_running: Arc<AtomicBool>,
    run_id: Arc<AtomicU64>,
    my_run_id: u64,
    validator_count: Arc<AtomicUsize>,
    storage: Arc<Storage>,
    mempool: Arc<Mempool>,
    consensus: Arc<Mutex<Consensus>>,
    is_synced: Arc<AtomicBool>,
    chain_index: Arc<AtomicU64>,
    mined_by_me_count: Arc<AtomicU64>,
    wallet_store: Arc<Mutex<Option<Wallet>>>,
    mining_enabled: Arc<AtomicBool>,
    receipt_sender: Arc<Mutex<Option<tokio::sync::mpsc::Sender<crate::chain::Receipt>>>>,
    node_type: Arc<Mutex<NodeType>>,
    block_sender: tokio::sync::mpsc::Sender<Box<crate::chain::Block>>,
    wallet_addr: String,
) {
    let mut last_production_time = std::time::Instant::now();
    let mut last_log_time = std::time::Instant::now();

    loop {
        // Check if we should stop
        if !is_running.load(Ordering::Relaxed) || run_id.load(Ordering::Relaxed) != my_run_id {
            log::info!("Mining Loop: Stopping");
            break;
        }

        // Brief sleep to prevent busy-waiting + yield to other async tasks
        tokio::time::sleep(Duration::from_millis(500)).await;
        tokio::task::yield_now().await; // Allow P2P and VDF to run

        // Update consensus state
        {
            let mut c = consensus.lock().unwrap();
            c.update_active_status();
        }

        // Auto-pruning check
        run_auto_pruning(&storage);

        // Skip if not synced
        if !is_synced.load(Ordering::Relaxed) {
            continue;
        }

        // Check leadership
        let (is_leader, leader_id, current_slot, my_shard) = {
            let c = consensus.lock().unwrap();
            let slot = c.current_slot();
            let epoch = c.current_epoch();
            let me = c.local_peer_id.clone();

            let shard = me
                .as_ref()
                .map(|pid| c.get_assigned_shard(pid, epoch))
                .unwrap_or(0);
            let leader = c.get_shard_leader(shard, slot);

            (leader.is_some() && leader == me, leader, slot, shard)
        };

        let enabled = mining_enabled.load(Ordering::Relaxed);
        let elapsed = last_production_time.elapsed().as_secs();

        // Log status periodically
        if last_log_time.elapsed() >= Duration::from_secs(10) {
            log::info!(
                "Mining Loop: Slot {} | Shard {} | Leader: {:?} | IsLeader: {}",
                current_slot,
                my_shard,
                leader_id,
                is_leader
            );
            last_log_time = std::time::Instant::now();
        }

        if !enabled {
            continue;
        }

        if !is_leader {
            continue;
        }

        // === SLOT TIMING CHECK ===
        // Wait at least 1 second into slot before producing
        // This allows network gossip to propagate first
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let slot_progress = now_secs % crate::consensus::Consensus::SLOT_DURATION;

        if slot_progress < MIN_SLOT_PROGRESS_FOR_PRODUCTION {
            tokio::time::sleep(Duration::from_millis(100)).await;
            continue;
        }

        // === DUPLICATE BLOCK CHECK ===
        let current_idx = chain_index.load(Ordering::Relaxed);
        if let Ok(Some(latest_block)) = storage.get_block(current_idx) {
            let latest_slot = latest_block.timestamp / crate::consensus::Consensus::SLOT_DURATION;

            if latest_slot >= current_slot {
                // Block for this slot already exists
                continue;
            }
        }

        // === BLOCK PRODUCTION ===
        let pending_txs = mempool.get_pending_transactions();

        // Only produce if enough time passed or enough transactions
        if elapsed < crate::utils::constants::TARGET_BLOCK_TIME && pending_txs.len() < 100 {
            continue;
        }

        let target_idx = if storage.get_total_blocks().unwrap_or(0) == 0 {
            0
        } else {
            current_idx + 1
        };

        log::info!(
            "Mining Loop: Producing block {} (elapsed: {}s, txs: {})",
            target_idx,
            elapsed,
            pending_txs.len()
        );

        // Get current wallet address
        let current_wallet_addr = wallet_store
            .lock()
            .unwrap()
            .as_ref()
            .map(|w| w.address.clone())
            .unwrap_or_else(|| wallet_addr.clone());

        last_production_time = std::time::Instant::now();

        // Calculate rewards
        let block_reward = if target_idx == 0 {
            crate::utils::constants::GENESIS_SUPPLY
        } else {
            crate::chain::calculate_mining_reward(target_idx)
        };

        let total_fees: u64 = pending_txs
            .iter()
            .map(|tx| crate::chain::calculate_fee(tx.amount))
            .sum();

        // Create coinbase transaction
        let coinbase_tx =
            create_coinbase_tx(&current_wallet_addr, target_idx, block_reward, total_fees);

        // Filter and collect transactions for this shard
        let (block_txs, generated_receipts) = collect_shard_transactions(
            coinbase_tx,
            &pending_txs,
            my_shard,
            &consensus,
            &receipt_sender,
        );

        // Broadcast generated receipts
        for receipt in generated_receipts {
            if let Some(sender) = receipt_sender.lock().unwrap().as_ref() {
                let _ = sender.try_send(receipt);
            }
        }

        // Get previous block hash
        let prev_hash = if target_idx == 0 {
            "0000000000000000000000000000000000000000000000000000000000000000".to_string()
        } else {
            storage
                .get_block(current_idx)
                .unwrap_or(None)
                .map(|b| b.hash)
                .unwrap_or_else(|| {
                    "0000000000000000000000000000000000000000000000000000000000000000".to_string()
                })
        };

        // Calculate adaptive VDF difficulty
        let current_validators = validator_count.load(Ordering::Relaxed);
        let adaptive_difficulty = if current_validators <= 1 {
            100 // Fast solo production
        } else {
            100 + (current_validators as u64 * 100)
        };

        // Create block
        let mut new_block = chain::Block::new(
            target_idx,
            current_wallet_addr.clone(),
            block_txs,
            prev_hash,
            100,
            adaptive_difficulty,
            my_shard as u32,
            total_fees,
            block_reward,
        );

        // Solve VDF (quick for block production)
        let _ = app_handle.emit("node-status", "Active (Mining)");
        let vdf = CentichainVDF::new(new_block.vdf_difficulty);
        let challenge = new_block.calculate_hash();
        new_block.vdf_proof = vdf.solve(challenge.as_bytes());
        new_block.hash = new_block.calculate_hash();
        new_block.size = new_block.calculate_size();

        // Slash missed slots
        slash_missed_slots(&storage, &consensus, target_idx, &new_block, my_shard);

        // Save block
        if let Err(e) = storage.save_block(&new_block) {
            log::error!("Failed to save block: {}", e);
            continue;
        }

        // Pruning
        if *node_type.lock().unwrap() == NodeType::Pruned {
            let _ = storage.prune_history(2000);
        }

        // Update state
        chain_index.store(new_block.index, Ordering::Relaxed);
        mined_by_me_count.fetch_add(1, Ordering::Relaxed);
        let _ = app_handle.emit("new-block", new_block.clone());

        // Broadcast to network
        if let Err(e) = block_sender.send(Box::new(new_block)).await {
            log::error!("Failed to broadcast block: {}", e);
        }

        // Remove mined transactions from mempool
        let tx_ids: Vec<String> = pending_txs.iter().map(|tx| tx.id.clone()).collect();
        mempool.remove_transactions(&tx_ids);

        log::info!("Mining Loop: Block {} produced and broadcast", target_idx);
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Runs auto-pruning if needed
fn run_auto_pruning(storage: &Arc<Storage>) {
    let height = storage.get_latest_index().unwrap_or(0);
    if height > 1000 && height % 300 == 0 {
        if let Err(e) = storage.prune_history(1000) {
            log::error!("Auto-pruning failed: {}", e);
        } else {
            log::info!("Auto-pruning triggered at height {}", height);
        }
    }
}

/// Creates a coinbase transaction for block reward
fn create_coinbase_tx(
    receiver: &str,
    block_index: u64,
    block_reward: u64,
    total_fees: u64,
) -> chain::Transaction {
    if block_index == 0 {
        chain::Transaction {
            id: "genesis".to_string(),
            sender: "SYSTEM".to_string(),
            receiver: receiver.to_string(),
            amount: block_reward,
            shard_id: 0,
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
            receiver: receiver.to_string(),
            amount: block_reward + total_fees,
            shard_id: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            signature: "reward".to_string(),
        }
    }
}

/// Collects transactions for this shard and generates cross-shard receipts
fn collect_shard_transactions(
    coinbase_tx: chain::Transaction,
    pending_txs: &[chain::Transaction],
    my_shard_id: u16,
    consensus: &Arc<Mutex<Consensus>>,
    _receipt_sender: &Arc<Mutex<Option<tokio::sync::mpsc::Sender<crate::chain::Receipt>>>>,
) -> (Vec<chain::Transaction>, Vec<crate::chain::Receipt>) {
    let mut block_txs = vec![coinbase_tx];
    let mut receipts = Vec::new();
    let mut current_size = 300; // Approx coinbase size

    for tx in pending_txs.iter() {
        // Check shard routing
        if tx.shard_id != my_shard_id {
            continue;
        }

        // Check TPS limit
        if block_txs.len() >= crate::utils::constants::MAX_TXS_PER_BLOCK as usize {
            break;
        }

        // Check block size limit
        if current_size + 300 > crate::utils::constants::MAX_BLOCK_SIZE {
            break;
        }

        // Generate cross-shard receipt if needed
        let target_shard = {
            let c = consensus.lock().unwrap();
            c.get_assigned_shard(&tx.receiver, 0)
        };

        if target_shard != my_shard_id {
            let receipt = crate::chain::Receipt {
                original_tx_id: tx.id.clone(),
                source_shard: my_shard_id,
                target_shard,
                amount: tx.amount,
                receiver: tx.receiver.clone(),
                block_hash: "pending".to_string(),
                merkle_proof: vec![],
                status: crate::chain::ReceiptStatus::Pending,
            };
            receipts.push(receipt);
            log::info!(
                "Generated cross-shard receipt: {} -> Shard {}",
                tx.id,
                target_shard
            );
        }

        block_txs.push(tx.clone());
        current_size += 300;
    }

    (block_txs, receipts)
}

/// Slashes validators who missed their slots
fn slash_missed_slots(
    storage: &Arc<Storage>,
    consensus: &Arc<Mutex<Consensus>>,
    target_idx: u64,
    new_block: &chain::Block,
    my_shard_id: u16,
) {
    if target_idx == 0 {
        return;
    }

    let prev_block_timestamp = storage
        .get_block(target_idx - 1)
        .unwrap_or(None)
        .map(|b| b.timestamp)
        .unwrap_or(0);

    let prev_slot = prev_block_timestamp / crate::consensus::Consensus::SLOT_DURATION;
    let new_block_slot = new_block.timestamp / crate::consensus::Consensus::SLOT_DURATION;

    if new_block_slot > prev_slot + 1 {
        let mut c = consensus.lock().unwrap();
        let slashed = c.slash_missed_slots(prev_slot + 1, new_block_slot - 1, my_shard_id);
        if !slashed.is_empty() {
            log::warn!("Slashed nodes for missing slots: {:?}", slashed);
        }
    }
}
