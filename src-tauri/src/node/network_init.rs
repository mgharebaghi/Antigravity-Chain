//! # Network Initialization Module
//!
//! Handles Phase 2 of the mining loop: network discovery and synchronization.

use crate::chain::{self, Transaction};
use crate::consensus::vdf::CentichainVDF;
use crate::consensus::Consensus;
use crate::storage::Storage;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Time to search for peers before assuming first node (seconds)
pub const PEER_DISCOVERY_TIMEOUT: u64 = 60;

/// Maximum time to wait for sync (seconds)
pub const SYNC_TIMEOUT: u64 = 300;

/// Initializes network state: discovers peers, syncs, or becomes first node
///
/// This function handles the second phase of node startup:
/// - Checks for existing peers
/// - Syncs with the network if peers are found
/// - Creates genesis block if this is the first node
#[allow(clippy::too_many_arguments)]
pub async fn initialize_network_state(
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
    cmd_tx: &tokio::sync::mpsc::Sender<crate::network::P2PCommand>,
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
            // CRITICAL FIX: Persistence of Eligibility
            // Since NodeState is in-memory, we lose 'activated_at' on restart.
            // We must infer our status from the blockchain history.
            if local_chain_exists {
                let should_activate =
                    check_eligibility_from_history(storage, consensus, wallet_addr);

                if should_activate {
                    let mut c = consensus.lock().unwrap();
                    c.force_activate_local();
                    log::info!("Mining Loop: Eligibility restored from chain history.");
                } else {
                    log::info!(
                        "Mining Loop: No history of active mining found - entering Patience mode."
                    );
                }
            }

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

/// Checks if we should be activated based on blockchain history
fn check_eligibility_from_history(
    storage: &Arc<Storage>,
    consensus: &Arc<Mutex<Consensus>>,
    wallet_addr: &str,
) -> bool {
    let mut inferred_active = false;
    let c = consensus.lock().unwrap();
    let local_peer_id = c.local_peer_id.clone().unwrap_or_default();
    drop(c); // Unlock to avoid locking issues

    // Helper to check authorship
    let is_author = |author: &String| -> bool {
        author == wallet_addr || (!local_peer_id.is_empty() && author == &local_peer_id)
    };

    // 1. Check Genesis (Are we the creator?)
    if let Ok(Some(genesis)) = storage.get_block(0) {
        if is_author(&genesis.author) {
            log::info!(
                "Mining Loop: Identified as Genesis Creator (Author={}) - Restoring Active Status",
                genesis.author
            );
            inferred_active = true;
        }
    }

    // 2. Check recent history (Did we mine recently?)
    // If we aren't Genesis, check if we were active recently
    if !inferred_active {
        let height = storage.get_latest_index().unwrap_or(0);
        let scan_depth = 500;
        let start = height.saturating_sub(scan_depth);
        for i in (start..=height).rev() {
            if let Ok(Some(block)) = storage.get_block(i) {
                if is_author(&block.author) {
                    log::info!("Mining Loop: Found recently mined block #{} by {} - Restoring Active Status", i, block.author);
                    inferred_active = true;
                    break;
                }
            }
        }
    }

    inferred_active
}

/// Waits for peers to be discovered
pub async fn wait_for_peers(
    app_handle: &AppHandle,
    is_running: &Arc<AtomicBool>,
    run_id: &Arc<AtomicU64>,
    my_run_id: u64,
    validator_count: &Arc<AtomicUsize>,
    cmd_tx: &tokio::sync::mpsc::Sender<crate::network::P2PCommand>,
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
        let _ = cmd_tx.try_send(crate::network::P2PCommand::SyncWithNetwork);

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    false
}

/// Syncs with the network
pub async fn sync_with_network(
    app_handle: &AppHandle,
    is_running: &Arc<AtomicBool>,
    run_id: &Arc<AtomicU64>,
    my_run_id: u64,
    storage: &Arc<Storage>,
    is_synced: &Arc<AtomicBool>,
    cmd_tx: &tokio::sync::mpsc::Sender<crate::network::P2PCommand>,
    peer_count: &Arc<AtomicUsize>,
) -> bool {
    log::info!("Mining Loop: Starting sync with network");
    let _ = app_handle.emit("node-status", "Synchronizing...");
    let _ = cmd_tx
        .send(crate::network::P2PCommand::SyncWithNetwork)
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
            let _ = cmd_tx.try_send(crate::network::P2PCommand::SyncWithNetwork);
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
pub async fn create_genesis_block(
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
