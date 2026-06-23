//! # Mining Module
//!
//! This module handles block production and the main mining loop.
//!
//! Key responsibilities:
//! - Spawning the mining loop
//! - Slot-based leader election and block production
//! - Transaction processing and cross-shard receipts
//!
//! IMPORTANT: This loop is designed to be non-blocking and yield-friendly
//! to allow P2P and VDF operations to run concurrently.

use crate::chain::{ingest_block, BlockAcceptResult};
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

use super::helpers::{
    collect_shard_transactions, create_coinbase_tx, run_auto_pruning, slash_missed_slots,
};
use super::network_init::initialize_network_state;
use super::relay::{emit_relay_error, wait_for_relay, RELAY_CONNECTION_TIMEOUT};

// =============================================================================
// Constants
// =============================================================================

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
    cmd_tx: tokio::sync::mpsc::Sender<crate::network::P2PCommand>,
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
        let mut new_block = crate::chain::Block::new(
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

        // Validate and save block (Phase 1 security)
        match ingest_block(&storage, &mempool, &consensus, &new_block, false) {
            BlockAcceptResult::Accepted => {}
            other => {
                log::error!("Mining Loop: Block {} not accepted: {:?}", target_idx, other);
                continue;
            }
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

        log::info!("Mining Loop: Block {} produced and broadcast", target_idx);
    }
}
