//! # Block & Chain Validation (Phase 1)
//!
//! Central rules for accepting blocks and transactions before they touch storage.

use crate::chain::{
    calculate_fee, calculate_merkle_root, calculate_mining_reward, Block, Transaction,
    SYSTEM_SIG_GENESIS, SYSTEM_SIG_REWARD,
};
use crate::consensus::Consensus;
use crate::storage::Storage;
use crate::utils::constants::{MAX_TXS_PER_BLOCK, TOTAL_SUPPLY};

/// Result of attempting to append a block to the local chain.
#[derive(Debug, PartialEq, Eq)]
pub enum BlockAcceptResult {
    /// Block stored and chain extended.
    Accepted,
    /// Same index + hash already present.
    Duplicate,
    /// Parent block not available locally — sync required.
    NeedsSync { missing_from: u64 },
    /// Invalid or conflicting block.
    Rejected(String),
}

/// Context for validating a new block against the current tip.
pub struct BlockContext<'a> {
    pub tip: Option<&'a Block>,
    pub consensus: Option<&'a Consensus>,
    /// Skip leader/VDF checks for locally trusted genesis creation.
    pub is_local_genesis: bool,
}

/// Validates a transaction (signature + economics) against current chain state.
pub fn validate_transaction(
    tx: &Transaction,
    storage: &Storage,
    pending_spend_from_mempool: u64,
) -> Result<(), String> {
    tx.validate()?;

    if tx.is_system() {
        return Ok(());
    }

    let balance = storage
        .calculate_balance(&tx.sender)
        .map_err(|e| e.to_string())?;
    let fee = calculate_fee(tx.amount);
    let required = tx.amount.saturating_add(fee);
    let effective = balance.saturating_sub(pending_spend_from_mempool);

    if required > effective {
        return Err(format!(
            "Insufficient funds for {}: need {}, effective {}",
            tx.sender, required, effective
        ));
    }

    Ok(())
}

/// Validates block structure, linkage, consensus rules, and all transactions.
pub fn validate_block(block: &Block, ctx: &BlockContext<'_>) -> Result<(), String> {
    if block.transactions.is_empty() {
        return Err("Block has no transactions".into());
    }
    if block.transactions.len() > MAX_TXS_PER_BLOCK as usize {
        return Err("Block exceeds max transaction count".into());
    }

    // Hash integrity
    let expected_hash = block.calculate_hash();
    if block.hash != expected_hash {
        return Err("Block hash mismatch".into());
    }

    let expected_merkle = calculate_merkle_root(&block.transactions);
    if block.merkle_root != expected_merkle {
        return Err("Merkle root mismatch".into());
    }

    if !ctx.is_local_genesis && !block.is_vdf_valid() {
        return Err("Invalid VDF proof".into());
    }

    // Chain linkage
    match (&ctx.tip, block.index) {
        (None, 0) => {
            if block.previous_hash
                != "0000000000000000000000000000000000000000000000000000000000000000"
            {
                return Err("Genesis previous_hash must be zero".into());
            }
        }
        (Some(tip), idx) if idx == tip.index + 1 => {
            if block.previous_hash != tip.hash {
                return Err("previous_hash does not match chain tip".into());
            }
        }
        (Some(tip), idx) if idx == tip.index => {
            return Err("Duplicate block index at tip".into());
        }
        (Some(tip), idx) if idx < tip.index => {
            return Err(format!("Stale block index {} (tip {})", idx, tip.index));
        }
        (Some(tip), idx) if idx > tip.index + 1 => {
            return Err(format!(
                "Non-contiguous block index {} (expected {})",
                idx,
                tip.index + 1
            ));
        }
        (None, idx) if idx > 0 => {
            return Err("Cannot append block > 0 on empty chain".into());
        }
        _ => {}
    }

    // Leader check (skip genesis bootstrap)
    if let Some(consensus) = ctx.consensus {
        if block.index > 0 {
            let slot = block.timestamp / Consensus::SLOT_DURATION;
            let shard = block.shard_id as u16;
            let expected_leader = consensus
                .get_shard_leader(shard, slot)
                .ok_or_else(|| "No eligible leader for slot".to_string())?;
            if expected_leader != block.author {
                return Err(format!(
                    "Wrong block author: expected {}, got {}",
                    expected_leader, block.author
                ));
            }
        }
    }

    validate_block_transactions(block, ctx.tip)?;

    Ok(())
}

fn validate_block_transactions(block: &Block, tip: Option<&Block>) -> Result<(), String> {
    let expected_reward = calculate_mining_reward(block.index);
    let mut user_tx_count = 0usize;
    let mut coinbase_count = 0usize;
    let mut computed_fees = 0u64;

    for tx in &block.transactions {
        if tx.is_system() {
            coinbase_count += 1;
            validate_system_tx(tx, block, expected_reward)?;
        } else {
            user_tx_count += 1;
            tx.validate()?;
            computed_fees = computed_fees.saturating_add(calculate_fee(tx.amount));
        }
    }

    if coinbase_count != 1 {
        return Err(format!(
            "Block must contain exactly one SYSTEM reward tx, found {}",
            coinbase_count
        ));
    }

    if block.index > 0 && user_tx_count == 0 && computed_fees != block.total_fees {
        // Allow fee drift only when there are no user txs
    } else if computed_fees != block.total_fees {
        return Err(format!(
            "total_fees mismatch: header {}, computed {}",
            block.total_fees, computed_fees
        ));
    }

    if block.block_reward != expected_reward {
        return Err(format!(
            "block_reward mismatch: header {}, expected {}",
            block.block_reward, expected_reward
        ));
    }

    // Supply cap check on this block's mint
    let coinbase = block
        .transactions
        .iter()
        .find(|t| t.is_system())
        .expect("checked above");
    if coinbase.amount > TOTAL_SUPPLY {
        return Err("Coinbase exceeds total supply".into());
    }

    // Replay protection: tx ids must be unique within block
    let mut seen_ids = std::collections::HashSet::new();
    for tx in &block.transactions {
        if !seen_ids.insert(tx.id.clone()) {
            return Err(format!("Duplicate transaction id in block: {}", tx.id));
        }
    }

    // Optional: reject re-used tx ids from parent chain (simple check on tip only)
    if let Some(tip) = tip {
        for tx in &block.transactions {
            if tip.transactions.iter().any(|t| t.id == tx.id) {
                return Err(format!("Transaction {} already in parent block", tx.id));
            }
        }
    }

    Ok(())
}

fn validate_system_tx(
    tx: &Transaction,
    block: &Block,
    expected_reward: u64,
) -> Result<(), String> {
    if tx.sender != "SYSTEM" {
        return Err("SYSTEM tx must have sender SYSTEM".into());
    }
    if tx.receiver != block.author {
        return Err("SYSTEM reward must pay block author".into());
    }

    let expected_amount = if block.index == 0 {
        expected_reward
    } else {
        expected_reward.saturating_add(block.total_fees)
    };

    if tx.amount != expected_amount {
        return Err(format!(
            "SYSTEM payout mismatch: tx {}, expected {}",
            tx.amount, expected_amount
        ));
    }

    if block.index == 0 {
        if tx.signature != SYSTEM_SIG_GENESIS && tx.signature != "genesis" {
            return Err("Invalid genesis SYSTEM signature".into());
        }
    } else if tx.signature != SYSTEM_SIG_REWARD && tx.signature != "reward" {
        return Err("Invalid reward SYSTEM signature".into());
    }

    Ok(())
}

/// Fork-choice + validation + persistence entry point.
pub fn try_accept_block(
    storage: &Storage,
    block: &Block,
    consensus: Option<&Consensus>,
    is_local_genesis: bool,
) -> Result<BlockAcceptResult, String> {
    let tip_index = storage
        .get_latest_index()
        .map_err(|e| e.to_string())?;
    let tip_block = storage
        .get_block(tip_index)
        .map_err(|e| e.to_string())?;

    if let Some(existing) = storage
        .get_block(block.index)
        .map_err(|e| e.to_string())?
    {
        if existing.hash == block.hash {
            return Ok(BlockAcceptResult::Duplicate);
        }
        return Ok(BlockAcceptResult::Rejected(format!(
            "Fork at index {}: different hash",
            block.index
        )));
    }

    if block.index > tip_index + 1 {
        return Ok(BlockAcceptResult::NeedsSync {
            missing_from: tip_index + 1,
        });
    }

    let tip_ref = tip_block.as_ref();
    if block.index > 0 {
        if tip_ref.is_none() {
            return Ok(BlockAcceptResult::NeedsSync {
                missing_from: 0,
            });
        }
        if let Some(parent) = storage
            .get_block(block.index - 1)
            .map_err(|e| e.to_string())?
        {
            if block.previous_hash != parent.hash {
                return Ok(BlockAcceptResult::Rejected(
                    "previous_hash does not match stored parent".into(),
                ));
            }
        } else {
            return Ok(BlockAcceptResult::NeedsSync {
                missing_from: block.index - 1,
            });
        }
    }

    let ctx = BlockContext {
        tip: tip_ref,
        consensus,
        is_local_genesis,
    };

    validate_block(block, &ctx).map_err(|e| e.to_string())?;

    storage
        .save_block(block)
        .map_err(|e| format!("Storage error: {e}"))?;

    Ok(BlockAcceptResult::Accepted)
}

/// Validates and appends a block; updates consensus + mempool on success.
pub fn ingest_block(
    storage: &Storage,
    mempool: &crate::consensus::mempool::Mempool,
    consensus: &std::sync::Mutex<crate::consensus::Consensus>,
    block: &Block,
    is_local_genesis: bool,
) -> BlockAcceptResult {
    let result = {
        let c = consensus.lock().unwrap();
        match try_accept_block(storage, block, Some(&c), is_local_genesis) {
            Ok(r) => r,
            Err(e) => return BlockAcceptResult::Rejected(e),
        }
    };

    if result == BlockAcceptResult::Accepted {
        let mut c = consensus.lock().unwrap();
        c.register_block_author(block.author.clone());
        c.persist_to_storage(storage);

        let tx_ids: Vec<String> = block
            .transactions
            .iter()
            .filter(|t| !t.is_system())
            .map(|t| t.id.clone())
            .collect();
        if !tx_ids.is_empty() {
            mempool.remove_transactions(&tx_ids);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::Block;
    use crate::utils::constants::GENESIS_SUPPLY;
    use libp2p::identity::Keypair;

    fn genesis_block(author: &str) -> Block {
        let tx = Transaction {
            id: "genesis".into(),
            sender: "SYSTEM".into(),
            receiver: author.into(),
            amount: GENESIS_SUPPLY,
            shard_id: 0,
            timestamp: 0,
            signature: SYSTEM_SIG_GENESIS.into(),
            sender_pubkey: String::new(),
        };
        let mut b = Block::new(
            0,
            author.into(),
            vec![tx],
            "0000000000000000000000000000000000000000000000000000000000000000".into(),
            100,
            100,
            0,
            0,
            GENESIS_SUPPLY,
        );
        let vdf = crate::consensus::vdf::CentichainVDF::new(100);
        let challenge = b.calculate_hash();
        b.vdf_proof = vdf.solve(challenge.as_bytes());
        b.hash = b.calculate_hash();
        b
    }

    #[test]
    fn rejects_tampered_hash() {
        let author = Keypair::generate_ed25519()
            .public()
            .to_peer_id()
            .to_string();
        let mut block = genesis_block(&author);
        block.hash = "deadbeef".into();
        let ctx = BlockContext {
            tip: None,
            consensus: None,
            is_local_genesis: true,
        };
        assert!(validate_block(&block, &ctx).is_err());
    }
}
