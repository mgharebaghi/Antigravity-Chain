//! # Block Production Helpers Module
//!
//! Helper functions for block production in the mining loop.

use crate::chain::{self, SYSTEM_SIG_GENESIS, SYSTEM_SIG_REWARD};
use crate::consensus::Consensus;
use crate::storage::Storage;
use std::sync::{Arc, Mutex};

// =============================================================================
// Helper Functions
// =============================================================================

/// Runs auto-pruning if needed
///
/// Prunes old blocks to save storage space. Triggered periodically
/// based on chain height.
pub fn run_auto_pruning(storage: &Arc<Storage>) {
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
///
/// The coinbase transaction is the first transaction in each block,
/// rewarding the block producer with newly minted coins plus fees.
pub fn create_coinbase_tx(
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
            signature: SYSTEM_SIG_GENESIS.to_string(),
            sender_pubkey: String::new(),
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
            signature: SYSTEM_SIG_REWARD.to_string(),
            sender_pubkey: String::new(),
        }
    }
}

/// Collects transactions for this shard and generates cross-shard receipts
///
/// Filters pending transactions to only include those belonging to this shard,
/// and generates receipts for any cross-shard transfers.
pub fn collect_shard_transactions(
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
///
/// Called during block production to penalize validators who
/// failed to produce blocks when they were the designated leader.
pub fn slash_missed_slots(
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
