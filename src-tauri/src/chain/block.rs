//! # Block Structure
//!
//! Core blockchain block implementation.

use crate::chain::{calculate_merkle_root, Transaction};
use crate::consensus::vdf::CentichainVDF;
use crate::utils::constants::*;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A blockchain block
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub author: String,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub start_time_weight: u64,
    #[serde(default)]
    pub vdf_proof: String,
    #[serde(default)]
    pub signature: String,

    // Infrastructure Metadata
    pub version: u32,
    pub merkle_root: String,
    pub state_root: String,
    pub nonce: u64,
    pub vdf_difficulty: u64,
    pub size: u64,
    #[serde(default)]
    pub shard_id: u32,

    // Economic Metadata
    pub total_fees: u64,
    pub block_reward: u64,
    pub total_reward: u64,
}

impl Block {
    /// Create a new block
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        index: u64,
        author: String,
        transactions: Vec<Transaction>,
        previous_hash: String,
        weight: u64,
        vdf_difficulty: u64,
        shard_id: u32,
        total_fees: u64,
        block_reward: u64,
    ) -> Self {
        let merkle_root = calculate_merkle_root(&transactions);
        let mut block = Block {
            index,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            author,
            transactions,
            previous_hash,
            hash: String::new(),
            start_time_weight: weight,
            vdf_proof: String::new(),
            signature: String::new(),
            version: 1,
            merkle_root,
            state_root: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            nonce: rand::random::<u64>(),
            vdf_difficulty,
            size: 0,
            total_fees,
            block_reward,
            total_reward: total_fees + block_reward,
            shard_id,
        };
        block.size = block.calculate_size();
        block.hash = block.calculate_hash();
        block
    }

    /// Calculate block hash
    pub fn calculate_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_be_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(self.author.as_bytes());
        hasher.update(self.previous_hash.as_bytes());
        hasher.update(self.vdf_proof.as_bytes());
        hasher.update(self.merkle_root.as_bytes());
        hasher.update(self.state_root.as_bytes());
        hasher.update(self.nonce.to_be_bytes());
        hasher.update(self.vdf_difficulty.to_be_bytes());
        hasher.update(self.version.to_be_bytes());
        hasher.update(self.total_fees.to_be_bytes());
        hasher.update(self.block_reward.to_be_bytes());
        hasher.update(self.total_reward.to_be_bytes());
        hex::encode(hasher.finalize())
    }

    /// Calculate serialized block size
    pub fn calculate_size(&self) -> u64 {
        match bincode::serialize(self) {
            Ok(bytes) => bytes.len() as u64,
            Err(_) => 0,
        }
    }

    /// Verify VDF proof
    pub fn is_vdf_valid(&self) -> bool {
        let vdf = CentichainVDF::new(self.vdf_difficulty);
        let mut clone = self.clone();
        clone.vdf_proof = String::new();
        let challenge = clone.calculate_hash();
        vdf.verify(challenge.as_bytes(), &self.vdf_proof)
    }
}

/// Block header (lightweight version for sync)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub index: u64,
    pub author: String,
    pub previous_hash: String,
    pub hash: String,
    pub weight: u64,
    pub vdf_difficulty: u64,
    pub shard_id: u32,
    pub timestamp: u64,
}

impl Header {
    /// Create header from block
    pub fn from_block(block: &Block) -> Self {
        Header {
            index: block.index,
            author: block.author.clone(),
            previous_hash: block.previous_hash.clone(),
            hash: block.hash.clone(),
            weight: block.start_time_weight,
            vdf_difficulty: block.vdf_difficulty,
            shard_id: block.shard_id,
            timestamp: block.timestamp,
        }
    }
}

/// Sync protocol requests
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncRequest {
    GetBlock(u64),
    GetBlocksRange(u64, u64),
    GetHeaders(u64, u64),
    GetHeight,
    GetMempool,
}

/// Sync protocol responses
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncResponse {
    Block(Option<Block>),
    BlocksBatch(Vec<Block>),
    HeadersBatch(Vec<Header>),
    Height(u64),
    Mempool(Vec<Transaction>),
}

/// Calculate mining reward based on block index
pub fn calculate_mining_reward(index: u64) -> u64 {
    if index == 0 {
        return GENESIS_SUPPLY;
    }

    let halving_count = index / HALVING_INTERVAL;
    if halving_count >= 64 {
        0
    } else {
        INITIAL_REWARD >> halving_count
    }
}

/// Calculate circulating supply up to given height
pub fn calculate_circulating_supply(height: u64) -> u64 {
    let mut supply = GENESIS_SUPPLY;
    let mut current_reward = INITIAL_REWARD;
    let mut blocks_processed = 0;

    while blocks_processed < height {
        let remaining = height - blocks_processed;
        let blocks_in_epoch = if remaining > HALVING_INTERVAL {
            HALVING_INTERVAL
        } else {
            remaining
        };

        supply += blocks_in_epoch * current_reward;
        blocks_processed += blocks_in_epoch;
        current_reward /= 2;

        if current_reward == 0 {
            break;
        }
    }

    supply
}
