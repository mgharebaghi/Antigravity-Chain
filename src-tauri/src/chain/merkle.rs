//! # Merkle Tree Utilities
//!
//! Functions for calculating and verifying Merkle roots.

use crate::chain::Transaction;
use sha2::{Digest, Sha256};

/// Calculate Merkle root from a list of transactions
pub fn calculate_merkle_root(transactions: &[Transaction]) -> String {
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
