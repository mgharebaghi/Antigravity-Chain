use crate::chain::{Block, Transaction};
use crate::state::AppState;
use std::sync::atomic::Ordering;
use tauri::State;

#[derive(serde::Serialize)]
pub struct ChainStats {
    pub total_blocks: u64,
    pub height: u64,
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
pub fn get_block(state: State<'_, AppState>, index: u64) -> Result<Option<Block>, String> {
    state.storage.get_block(index).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_block_by_hash(
    state: State<'_, AppState>,
    hash: String,
) -> Result<Option<Block>, String> {
    state
        .storage
        .get_block_by_hash(&hash)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_transaction(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<(Transaction, Block)>, String> {
    state
        .storage
        .get_transaction_by_id(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_latest_block(state: State<'_, AppState>) -> Result<Option<Block>, String> {
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
pub async fn get_blocks_paginated(
    state: State<'_, AppState>,
    page: usize,
    limit: usize,
) -> Result<Vec<Block>, String> {
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
pub fn get_recent_blocks(state: State<'_, AppState>, limit: usize) -> Result<Vec<Block>, String> {
    state
        .storage
        .get_recent_blocks(limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_chain_stats(state: State<'_, AppState>) -> Result<ChainStats, String> {
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
pub fn get_mined_blocks_count(state: State<'_, AppState>) -> u64 {
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
pub fn submit_transaction(
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
            let divisor = crate::utils::constants::ONE_AGT as f64;
            return Err(format!(
                "Insufficient funds. Balance: {:.6} AGT (Pending spent: {:.6}), Required: {:.6} AGT",
                balance as f64 / divisor,
                pending_spend as f64 / divisor,
                total_required as f64 / divisor
            ));
        }

        // Calculate Shard ID for the user transaction
        let shard_id = {
            let consensus = state.consensus.lock().unwrap();
            consensus.get_assigned_shard(&wallet.address, 0)
        };

        // Create Transaction
        // In real app: Sign with Keypair
        let tx = Transaction {
            id: uuid::Uuid::new_v4().to_string(), // Need uuid crate or simple random
            sender: wallet.address.clone(),
            receiver,
            amount,
            shard_id,
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
pub fn get_mempool_transactions(state: State<'_, AppState>) -> Vec<Transaction> {
    state.mempool.get_pending_transactions()
}

#[tauri::command]
pub fn reset_chain_data(state: State<'_, AppState>) -> Result<(), String> {
    state.storage.reset_blocks().map_err(|e| e.to_string())?;
    state.chain_index.store(0, Ordering::Relaxed);
    // Also reset mined_by_me if we want a full reset
    state.mined_by_me_count.store(0, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub fn get_tokenomics_info(state: State<'_, AppState>) -> TokenomicsInfo {
    let height = state.chain_index.load(Ordering::Relaxed);
    // Standard Halving Logic
    let current_interval = height / crate::utils::constants::HALVING_INTERVAL;
    let next_halving = (current_interval + 1) * crate::utils::constants::HALVING_INTERVAL;

    let halving_interval = crate::utils::constants::HALVING_INTERVAL;

    let circulating = crate::chain::calculate_circulating_supply(height);

    TokenomicsInfo {
        total_supply: crate::utils::constants::TOTAL_SUPPLY,
        max_supply: crate::utils::constants::TOTAL_SUPPLY,
        circulating_supply: circulating,
        remaining_supply: crate::utils::constants::TOTAL_SUPPLY.saturating_sub(circulating),
        next_halving_at: next_halving,
        blocks_until_halving: next_halving.saturating_sub(height),
        current_reward: crate::chain::calculate_mining_reward(height),
        halving_interval,
    }
}

#[tauri::command]
pub fn get_consensus_status(state: State<'_, AppState>) -> crate::consensus::NodeConsensusStatus {
    let wallet_guard = state.wallet.lock().unwrap();
    let consensus_guard = state.consensus.lock().unwrap();

    let peer_id = match wallet_guard.as_ref() {
        Some(w) => match libp2p::identity::Keypair::from_protobuf_encoding(&w.keypair) {
            Ok(kp) => kp.public().to_peer_id().to_string(),
            Err(_) => {
                return crate::consensus::NodeConsensusStatus {
                    state: "Error".to_string(),
                    queue_position: 0,
                    estimated_blocks: 0,
                    patience_progress: 0.0,
                    remaining_seconds: 0,
                    shard_id: 0,
                    is_slot_leader: false,
                }
            }
        },
        None => {
            return crate::consensus::NodeConsensusStatus {
                state: "Wallet Locked".to_string(),
                queue_position: 0,
                estimated_blocks: 0,
                patience_progress: 0.0,
                remaining_seconds: 0,
                shard_id: 0,
                is_slot_leader: false,
            }
        }
    };

    consensus_guard.get_node_status(&peer_id)
}
