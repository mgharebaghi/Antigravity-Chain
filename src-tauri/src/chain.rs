use serde::{Deserialize, Serialize};
use std::time::SystemTime;

pub const AGT_DECIMALS: u32 = 6;
pub const ONE_AGT: u64 = 1_000_000;

// Tokenomics Constants
pub const TOTAL_SUPPLY: u64 = 21_000_000 * ONE_AGT;
pub const GENESIS_SUPPLY: u64 = 5_000_000 * ONE_AGT;
pub const INITIAL_REWARD: u64 = 126_839; // ~0.12 AGT (Targeting 21M supply over ~100 years at 2s block time)
pub const HALVING_INTERVAL: u64 = 63_072_000; // 4 Years in 2s blocks

pub const MAX_BLOCK_SIZE: u64 = 1_500_000; // 1.5 MB Cap per shard
pub const TARGET_BLOCK_TIME: u64 = 2; // 2 Seconds
pub const MAX_TXS_PER_BLOCK: u64 = 3_000; // 3000 txs / 2s = 1500 TPS

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Transaction {
    pub id: String,
    pub sender: String, // Public key or Alias
    pub receiver: String,
    pub amount: u64,
    pub shard_id: u16, // New scaling field
    pub timestamp: u64,
    pub signature: String,
}

impl Transaction {
    /// Checks if this transaction is independent of another (no shared state/accounts).
    /// Critical for Parallel Execution Engine.
    pub fn is_independent(&self, other: &Self) -> bool {
        self.sender != other.sender
            && self.sender != other.receiver
            && self.receiver != other.sender
            && self.receiver != other.receiver
    }
}

/// A Receipt proves that a transaction was executed on a Source Shard
/// and funds were burned/locked, allowing the Destination Shard to mint/unlock them.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Receipt {
    pub original_tx_id: String,
    pub source_shard: u16,
    pub target_shard: u16,
    pub amount: u64,
    pub receiver: String,
    pub block_hash: String,        // Block on source shard where burn happened
    pub merkle_proof: Vec<String>, // Proof of inclusion
}

/// Cross-Link is a summary of a Shard's block header, signed by the shard's committee,
/// sent to the Beacon Chain for finalization.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrossLink {
    pub shard_id: u16,
    pub block_height: u64,
    pub block_hash: String,
    pub state_root: String,
    pub signature: String, // Aggregated BLS signature (simplified to string here)
}

pub fn calculate_fee(amount: u64) -> u64 {
    // 0.01% fee, minimum 0.001 AGT (1,000 units). Upward rounding.
    let fee = (amount as f64 * 0.0001).ceil() as u64;
    fee.max(1_000)
}

pub fn calculate_mining_reward(index: u64) -> u64 {
    if index == 0 {
        return GENESIS_SUPPLY;
    }

    // Standard Halving Logic (Bitcoin-like)
    let halving_count = index / HALVING_INTERVAL;

    if halving_count >= 64 {
        0
    } else {
        INITIAL_REWARD >> halving_count
    }
}

pub fn calculate_circulating_supply(height: u64) -> u64 {
    // Rough calculation based on fixed heights for UI performance
    let mut supply = GENESIS_SUPPLY;

    let mut current_reward = INITIAL_REWARD;
    let mut blocks_processed = 0;

    // We iterate through halving epochs until we reach current height
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

pub fn calculate_merkle_root(transactions: &[Transaction]) -> String {
    use sha2::{Digest, Sha256};
    if transactions.is_empty() {
        return "0000000000000000000000000000000000000000000000000000000000000000".to_string();
    }

    let mut hashes: Vec<Vec<u8>> = transactions
        .iter()
        .map(|tx| {
            let mut hasher = Sha256::new();
            hasher.update(tx.id.as_bytes());
            hasher.finalize().to_vec()
        })
        .collect();

    while hashes.len() > 1 {
        if hashes.len() % 2 != 0 {
            let last = hashes.last().unwrap().clone();
            hashes.push(last);
        }

        let mut next_level = Vec::new();
        for chunk in hashes.chunks(2) {
            let mut hasher = Sha256::new();
            hasher.update(&chunk[0]);
            hasher.update(&chunk[1]);
            next_level.push(hasher.finalize().to_vec());
        }
        hashes = next_level;
    }

    hex::encode(&hashes[0])
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub author: String, // Node ID
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
    #[serde(default)] // Default for backward compatibility
    pub shard_id: u32,

    // Economic Metadata
    pub total_fees: u64,
    pub block_reward: u64,
    pub total_reward: u64,
}

impl Block {
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
            author: author.clone(),
            transactions,
            previous_hash,
            hash: String::new(),
            start_time_weight: weight,
            vdf_proof: String::new(),
            signature: String::new(),
            version: 1,
            merkle_root,
            state_root: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(), // Placeholder for state commitment
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

    pub fn calculate_size(&self) -> u64 {
        // Use bincode to get exact serialized size
        match bincode::serialize(self) {
            Ok(bytes) => bytes.len() as u64,
            Err(_) => 0,
        }
    }

    pub fn is_vdf_valid(&self) -> bool {
        use crate::vdf::AntigravityVDF;
        let vdf = AntigravityVDF::new(self.vdf_difficulty);
        let mut clone = self.clone();
        clone.vdf_proof = String::new();
        let challenge = clone.calculate_hash();
        vdf.verify(challenge.as_bytes(), &self.vdf_proof)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncRequest {
    GetBlock(u64),            // Request block by Index
    GetBlocksRange(u64, u64), // Request range of blocks (Start, End) inclusive
    GetHeight,                // Request current chain height
    GetMempool,               // Request current pending transactions
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncResponse {
    Block(Option<Block>),
    BlocksBatch(Vec<Block>),
    Height(u64),
    Mempool(Vec<Transaction>),
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root_empty() {
        let root = calculate_merkle_root(&[]);
        assert_eq!(
            root,
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_merkle_root_single_tx() {
        let tx = Transaction {
            id: "tx1".to_string(),
            sender: "a".to_string(),
            receiver: "b".to_string(),
            amount: 100,
            shard_id: 0,
            timestamp: 0,
            signature: "s".to_string(),
        };
        let root = calculate_merkle_root(&[tx]);
        // Hash of "tx1" should match
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update("tx1".as_bytes());
        let expected = hex::encode(hasher.finalize());
        assert_eq!(root, expected);
    }

    #[test]
    fn test_merkle_root_multiple_txs() {
        let tx1 = Transaction {
            id: "tx1".to_string(),
            sender: "a".to_string(),
            receiver: "b".to_string(),
            amount: 100,
            shard_id: 0,
            timestamp: 0,
            signature: "s".to_string(),
        };
        let tx2 = Transaction {
            id: "tx2".to_string(),
            sender: "a".to_string(),
            receiver: "b".to_string(),
            amount: 200,
            shard_id: 0,
            timestamp: 0,
            signature: "s".to_string(),
        };

        let root = calculate_merkle_root(&[tx1, tx2]);
        assert_ne!(
            root,
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }
}
