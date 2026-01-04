use crate::chain::{self, Transaction};
use crate::consensus::mempool::Mempool;
use crate::consensus::vdf::CentichainVDF;
use crate::consensus::Consensus;
use crate::state::NodeType;
use crate::storage::Storage;
use crate::wallet::Wallet;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

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
        println!("Mining Loop: Thread started for run_id: {}", my_run_id);
        // Phase 1: Connect to Relay
        let _ = app_handle.emit("node-status", "Connecting to Relay...");

        let mut is_connected = false;
        for i in 0..10 {
            if !is_running.load(Ordering::Relaxed) {
                println!("Mining Loop: Aborted during relay wait");
                break;
            }

            // Check Atomic Bool directly
            if relay_connected.load(Ordering::Relaxed) {
                println!("Mining Loop: Relay connection verified via AtomicBool.");
                is_connected = true;
                break;
            }
            println!("Mining Loop: Waiting for relay... (attempt {})", i + 1);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        if !is_connected {
            println!("Mining Loop: Relay connection failed. MANDATORY RELAY NOT FOUND.");
            let _ = app_handle.emit("node-status", "Error: Relay Unreachable");
            // Wait for node to be stopped by user or keep erroring
            while is_running.load(Ordering::Relaxed) && run_id.load(Ordering::Relaxed) == my_run_id
            {
                let _ = app_handle.emit(
                    "node-status",
                    "Error: Relay Unreachable. Please check config/network.",
                );
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            return;
        }

        // Phase 2: Decision & Discovery Loop
        'discovery_outer: loop {
            if !is_running.load(Ordering::Relaxed) || run_id.load(Ordering::Relaxed) != my_run_id {
                break 'discovery_outer;
            }

            let peers = validator_count.load(Ordering::Relaxed);
            let local_height_opt = storage.get_block(0).unwrap_or(None);
            let local_exists = local_height_opt.is_some();

            println!(
                "Mining Loop: Decision Phase - Peers: {}, LocalChainExists: {}",
                peers, local_exists
            );

            if peers > 0 {
                println!(
                    "Mining Loop: Peers detected ({}). Checking network for sync (Local: {})...",
                    peers, local_exists
                );

                // Active Sync: Request from peers
                let _ = app_handle.emit("node-status", "Requesting Sync...");
                let _ = cmd_tx
                    .send(crate::network::p2p::P2PCommand::SyncWithNetwork)
                    .await;

                // Wait for sync with dynamic checking
                let max_sync_wait = 300; // 5 minutes
                for i in 0..max_sync_wait {
                    if !is_running.load(Ordering::Relaxed) {
                        println!("Mining Loop: Aborted during sync wait");
                        break 'discovery_outer;
                    }

                    let h = storage.get_latest_index().unwrap_or(0);
                    let peers = peer_count.load(Ordering::Relaxed);

                    // If we are marked as synced by P2P or have blocks
                    if is_synced.load(Ordering::Relaxed)
                        || (storage.get_block(0).unwrap_or(None).is_some() && i > 10)
                    {
                        println!("Mining Loop: Sync sufficient (Height {}).", h);
                        is_synced.store(true, Ordering::Relaxed);
                        break;
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
            } else {
                // No peers found, but relay it connected (checked in Phase 1)
                // Wait briefly for discovery if no local data exists
                if !local_exists {
                    println!("Mining Loop: Searching for peers (Relay connected)...");
                    for i in 0..15 {
                        if !is_running.load(Ordering::Relaxed) {
                            break 'discovery_outer;
                        }
                        if validator_count.load(Ordering::Relaxed) > 0 {
                            continue 'discovery_outer; // Found some!
                        }
                        let _ = app_handle
                            .emit("node-status", format!("Discovering Network... ({}s)", i));
                        let _ = cmd_tx.try_send(crate::network::p2p::P2PCommand::SyncWithNetwork);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }

                    // Still no peers after discovery phase
                    println!("Mining Loop: No peers found and no local data. I am the First Node.");
                    println!("Mining Loop: Creating Genesis Block...");
                    let _ = app_handle.emit("node-status", "Creating Genesis...");

                    let genesis_tx = Transaction {
                        id: "genesis".to_string(),
                        sender: "SYSTEM".to_string(),
                        receiver: wallet_addr.clone(),
                        amount: crate::utils::constants::GENESIS_SUPPLY,
                        shard_id: 0,
                        timestamp: 0,
                        signature: "genesis".to_string(),
                    };
                    let mut genesis_block = chain::Block::new(
                        0,
                        wallet_addr.clone(),
                        vec![genesis_tx],
                        "0000000000000000000000000000000000000000000000000000000000000000"
                            .to_string(),
                        100,
                        100,
                        0,
                        0,
                        crate::utils::constants::GENESIS_SUPPLY,
                    );

                    let vdf = CentichainVDF::new(genesis_block.vdf_difficulty);
                    let challenge = genesis_block.calculate_hash();
                    genesis_block.vdf_proof = vdf.solve(challenge.as_bytes());
                    genesis_block.hash = genesis_block.calculate_hash();
                    genesis_block.size = genesis_block.calculate_size();

                    let _ = storage.save_block(&genesis_block);
                    chain_index.store(0, Ordering::Relaxed);
                    mined_by_me_count.fetch_add(1, Ordering::Relaxed);
                    let _ = app_handle.emit("new-block", genesis_block);

                    println!("Mining Loop: Genesis Created.");
                    is_synced.store(true, Ordering::Relaxed);
                    let _ = app_handle.emit("node-status", "Active (Genesis)");
                } else {
                    // Local Chain exists and no peers found
                    println!(
                        "Mining Loop: Local Chain detected and NO peers. Resuming Solo Mining."
                    );
                    is_synced.store(true, Ordering::Relaxed);
                    let _ = app_handle.emit("node-status", "Active (Solo Continuation)");
                }
            }
            println!("Mining Loop: Node is now ACTIVE");
            break;
        }

        // Phase 3: Main Loop
        let mut last_production_time = std::time::Instant::now();

        loop {
            if !is_running.load(Ordering::Relaxed) || run_id.load(Ordering::Relaxed) != my_run_id {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;

            // Auto-Pruning Check (Every ~5 mins)
            let latest_height = storage.get_latest_index().unwrap_or(0);
            if latest_height > 1000 && latest_height % 300 == 0 {
                // Keep last 1000 blocks (~30 mins for demo), 10000 for prod
                if let Err(e) = storage.prune_history(1000) {
                    eprintln!("Pruning failed: {}", e);
                } else {
                    println!("Auto-Pruning triggered at height {}", latest_height);
                }
            }

            if is_synced.load(Ordering::Relaxed) {
                let (is_leader, leader_id) = {
                    let consensus = consensus.lock().unwrap();
                    let current_slot = consensus.current_slot();
                    let current_epoch = consensus.current_epoch();
                    let me = consensus.local_peer_id.clone();

                    let my_shard = if let Some(pid) = &me {
                        consensus.get_assigned_shard(pid, current_epoch)
                    } else {
                        0
                    };

                    let leader = consensus.get_shard_leader(my_shard, current_slot);

                    println!(
                        "Mining Loop: Slot {} (Epoch {}) - Shard {} Leader: {:?}, Me: {:?}",
                        current_slot, current_epoch, my_shard, leader, me
                    );
                    (leader.is_some() && leader == me, leader)
                };

                let pending = mempool.get_pending_transactions();
                let enabled = mining_enabled.load(Ordering::Relaxed);

                let elapsed = last_production_time.elapsed().as_secs();
                if enabled && is_leader {
                    // Strict Consensus Check: Did we already receive a block for this slot?
                    let current_idx = chain_index.load(Ordering::Relaxed);
                    if let Ok(Some(latest_b)) = storage.get_block(current_idx) {
                        let latest_slot =
                            latest_b.timestamp / crate::consensus::Consensus::SLOT_DURATION;
                        let current_slot = {
                            let c = consensus.lock().unwrap();
                            c.current_slot()
                        };

                        if latest_slot >= current_slot {
                            // We already have the block for this slot (synced from valid leader).
                            // Do NOT overwrite it with a fork.
                            if elapsed % 5 == 0 {
                                // Log periodically only
                                println!("Mining Loop: Skipping production. Block for slot {} already exists.", current_slot);
                            }
                            // We don't sleep here, just continue loop to next iteration
                            // But we need to ensure we don't spam.
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            continue;
                        }
                    }

                    if elapsed >= crate::utils::constants::TARGET_BLOCK_TIME || pending.len() >= 100
                    {
                        let is_empty = storage.get_total_blocks().unwrap_or(0) == 0;
                        let target_idx = if is_empty { 0 } else { current_idx + 1 };

                        println!(
                            "Mining Loop: Producing block {}... (elapsed: {}s, txs: {})",
                            target_idx,
                            elapsed,
                            pending.len()
                        );

                        let current_wallet_addr = wallet_store
                            .lock()
                            .unwrap()
                            .as_ref()
                            .map(|w| w.address.clone())
                            .unwrap_or_else(|| wallet_addr.clone());

                        last_production_time = std::time::Instant::now();

                        let block_reward = if target_idx == 0 {
                            crate::utils::constants::GENESIS_SUPPLY
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
                                shard_id: 0,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                                signature: "genesis".to_string(),
                            }
                        } else {
                            // Calculate shard_id for coinbase based on receiver?
                            // For now, coinbase is valid on ALL shards (or specific system shard 0)
                            // Let's assign it to Shard 0 for simplicity in this phase
                            chain::Transaction {
                                id: uuid::Uuid::new_v4().to_string(),
                                sender: "SYSTEM".to_string(),
                                receiver: current_wallet_addr.clone(),
                                amount: block_reward + total_fees,
                                shard_id: 0,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                                signature: "reward".to_string(),
                            }
                        };

                        // Phase 2: Shard Engine - Filter & Limit
                        let my_shard_id = {
                            let consensus = consensus.lock().unwrap();
                            if let Some(peer_id) = &consensus.local_peer_id {
                                consensus.get_assigned_shard(peer_id, 0)
                            } else {
                                0
                            }
                        };

                        let mut block_txs = vec![coinbase_tx];
                        let mut current_size = 300; // Approx size of coinbase

                        // Select transactions for THIS shard only
                        for tx in pending.iter() {
                            // 1. Check Shard Routing
                            if tx.shard_id != my_shard_id {
                                continue;
                            }

                            // Phase 3: Cross-Shard Receipt Generation
                            let target_shard = {
                                let consensus = consensus.lock().unwrap();
                                consensus.get_assigned_shard(&tx.receiver, 0)
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

                                // Send to P2P for broadcasting
                                if let Some(sender) = receipt_sender.lock().unwrap().as_ref() {
                                    if let Err(e) = sender.try_send(receipt) {
                                        log::warn!("Failed to send receipt: {}", e);
                                    } else {
                                        log::info!(
                                            "Generated Receipt for Tx {} -> Shard {}",
                                            tx.id,
                                            target_shard
                                        );
                                    }
                                }
                            }

                            // 2. Check TPS Limit (per block)
                            if block_txs.len()
                                >= crate::utils::constants::MAX_TXS_PER_BLOCK as usize
                            {
                                break;
                            }

                            // 3. Check Block Size Limit
                            // Approx 300 bytes per tx for now (serialization is heavier but this is a safety guard)
                            if current_size + 300 > crate::utils::constants::MAX_BLOCK_SIZE {
                                break;
                            }

                            block_txs.push(tx.clone());
                            current_size += 300;
                        }

                        let prev_hash = if target_idx == 0 {
                            "0000000000000000000000000000000000000000000000000000000000000000"
                                .to_string()
                        } else {
                            storage
                                .get_block(current_idx)
                                .unwrap_or(None)
                                .map(|b| b.hash)
                                .unwrap_or_else(|| {
                                    "0000000000000000000000000000000000000000000000000000000000000000"
                                        .to_string()
                                })
                        };

                        let current_validators = validator_count.load(Ordering::Relaxed);
                        let adaptive_difficulty = if current_validators <= 1 {
                            100 // Fast solo production
                        } else {
                            // Scale difficulty linearly with network size to manage collision
                            100 + (current_validators as u64 * 100)
                        };

                        let mut new_block = chain::Block::new(
                            target_idx,
                            current_wallet_addr.clone(),
                            block_txs,
                            prev_hash,
                            100, // weight
                            adaptive_difficulty,
                            my_shard_id as u32,
                            total_fees,
                            block_reward,
                        );

                        let vdf = CentichainVDF::new(new_block.vdf_difficulty);
                        let challenge = new_block.calculate_hash();
                        let _ = app_handle.emit("node-status", "Active (Mining)");
                        new_block.vdf_proof = vdf.solve(challenge.as_bytes());
                        new_block.hash = new_block.calculate_hash();
                        new_block.size = new_block.calculate_size();

                        // Slashing Check vs Previous Block
                        let prev_block_timestamp = if target_idx == 0 {
                            0
                        } else {
                            storage
                                .get_block(target_idx - 1)
                                .unwrap_or(None)
                                .map(|b| b.timestamp)
                                .unwrap_or(0)
                        };

                        let prev_slot =
                            prev_block_timestamp / crate::consensus::Consensus::SLOT_DURATION;
                        let new_block_slot =
                            new_block.timestamp / crate::consensus::Consensus::SLOT_DURATION;

                        if target_idx > 0 && new_block_slot > prev_slot + 1 {
                            let mut c = consensus.lock().unwrap();
                            let slashed = c.slash_missed_slots(
                                prev_slot + 1,
                                new_block_slot - 1,
                                my_shard_id,
                            );
                            if !slashed.is_empty() {
                                println!(
                                    "Mining Loop: SLASHED nodes for missing turns: {:?}",
                                    slashed
                                );
                            }
                        }

                        let _ = storage.save_block(&new_block);

                        // Pruning logic for local miner
                        let nt = {
                            let guard = node_type.lock().unwrap();
                            guard.clone()
                        };
                        if nt == NodeType::Pruned {
                            let _ = storage.prune_history(2000);
                        }

                        chain_index.store(new_block.index, Ordering::Relaxed);
                        mined_by_me_count.fetch_add(1, Ordering::Relaxed);
                        let _ = app_handle.emit("new-block", new_block.clone());

                        if let Err(e) = block_sender.send(Box::new(new_block)).await {
                            log::error!("Broadcast Error: {}", e);
                        }

                        let tx_ids: Vec<String> = pending.iter().map(|tx| tx.id.clone()).collect();
                        mempool.remove_transactions(&tx_ids);
                    }
                } else if !enabled {
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
}
