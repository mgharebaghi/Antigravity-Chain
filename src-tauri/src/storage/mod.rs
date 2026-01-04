use crate::chain::Block;
use redb::{Database, Error, ReadableTable, TableDefinition};
use std::sync::Arc;

const BLOCKS_TABLE: TableDefinition<u64, &str> = TableDefinition::new("blocks");
const WALLET_TABLE: TableDefinition<&str, &str> = TableDefinition::new("wallet");
const SETTINGS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("settings");
const MEMPOOL_TABLE: TableDefinition<&str, &str> = TableDefinition::new("mempool");
const STATE_TABLE: TableDefinition<&str, u64> = TableDefinition::new("state");

pub struct Storage {
    db: Arc<Database>,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self, Error> {
        let db = Database::create(path)?;
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(BLOCKS_TABLE)?;
            let _ = write_txn.open_table(WALLET_TABLE)?;
            let _ = write_txn.open_table(SETTINGS_TABLE)?;
            let _ = write_txn.open_table(MEMPOOL_TABLE)?;
            let _ = write_txn.open_table(STATE_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Storage { db: Arc::new(db) })
    }

    pub fn save_block(&self, block: &Block) -> Result<(), anyhow::Error> {
        let write_txn = self.db.begin_write()?;
        {
            let mut blocks_table = write_txn.open_table(BLOCKS_TABLE)?;
            let mut state_table = write_txn.open_table(STATE_TABLE)?;

            let json = serde_json::to_string(block)?;
            blocks_table.insert(block.index, json.as_str())?;

            // Update state based on transactions
            for tx in &block.transactions {
                // Handle Sender (Deduct amount + fee)
                if tx.sender != "SYSTEM" {
                    let current_balance = state_table
                        .get(tx.sender.as_str())?
                        .map(|v| v.value())
                        .unwrap_or(0);
                    let fee = crate::chain::calculate_fee(tx.amount);
                    let deduction = tx.amount.saturating_add(fee);
                    state_table.insert(
                        tx.sender.as_str(),
                        current_balance.saturating_sub(deduction),
                    )?;
                }

                // Handle Receiver (Add amount)
                let current_recv_balance = state_table
                    .get(tx.receiver.as_str())?
                    .map(|v| v.value())
                    .unwrap_or(0);
                state_table.insert(
                    tx.receiver.as_str(),
                    current_recv_balance.saturating_add(tx.amount),
                )?;
            }

            // Note: Mining reward (COINBASE) is already a transaction from SYSTEM to Author in modern blocks.
            // If it's an old block or missing coinbase, we can add it here if needed,
            // but the mining loop in lib.rs already creates a SYSTEM transaction.
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_block(&self, index: u64) -> Result<Option<Block>, anyhow::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(BLOCKS_TABLE)?;
        let result = match table.get(index)? {
            Some(guard) => {
                let block = serde_json::from_str(guard.value())?;
                Some(block)
            }
            None => None,
        };
        Ok(result)
    }

    pub fn get_recent_blocks(&self, limit: usize) -> Result<Vec<Block>, anyhow::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(BLOCKS_TABLE)?;
        let mut blocks = Vec::new();

        let mut iter = table.iter()?;
        while let Some(res) = iter.next_back() {
            let (_, value) = res?;
            let block: Block = serde_json::from_str(value.value())?;
            blocks.push(block);
            if blocks.len() >= limit {
                break;
            }
        }

        Ok(blocks)
    }

    pub fn get_blocks_paginated(
        &self,
        page: usize,
        limit: usize,
    ) -> Result<Vec<Block>, anyhow::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(BLOCKS_TABLE)?;
        let mut blocks = Vec::new();

        let total = table.len()?;
        if total == 0 {
            return Ok(blocks);
        }

        let skip = (page.saturating_sub(1)) * limit;
        let mut iter = table.iter()?;

        // Skip logic
        for _ in 0..skip {
            if iter.next_back().is_none() {
                break;
            }
        }

        // Take from back
        while blocks.len() < limit {
            if let Some(res) = iter.next_back() {
                let (_, value) = res?;
                let block: Block = serde_json::from_str(value.value())?;
                blocks.push(block);
            } else {
                break;
            }
        }

        Ok(blocks)
    }

    pub fn get_latest_index(&self) -> Result<u64, anyhow::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(BLOCKS_TABLE)?;

        let mut last_idx = 0;
        let mut iter = table.iter()?;
        if let Some(res) = iter.next_back() {
            let (k, _) = res?;
            last_idx = k.value();
        }
        Ok(last_idx)
    }

    pub fn get_total_blocks(&self) -> Result<u64, anyhow::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(BLOCKS_TABLE)?;
        Ok(table.len()?)
    }

    // Save wallet keys securely (in real app, encrypt this!)
    pub fn save_wallet_keys(&self, keys_json: &str) -> Result<(), anyhow::Error> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(WALLET_TABLE)?;
            table.insert("main_key", keys_json)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_wallet_keys(&self) -> Result<Option<String>, anyhow::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(WALLET_TABLE)?;
        let result = match table.get("main_key")? {
            Some(guard) => Some(guard.value().to_string()),
            None => None,
        };
        Ok(result)
    }

    pub fn delete_wallet_keys(&self) -> Result<(), anyhow::Error> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(WALLET_TABLE)?;
            table.remove("main_key")?;
        }
        write_txn.commit()?;
        Ok(())
    }
    pub fn calculate_balance(&self, address: &str) -> Result<u64, anyhow::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(STATE_TABLE)?;

        let balance = match table.get(address)? {
            Some(v) => v.value(),
            None => 0,
        };

        Ok(balance)
    }
    pub fn count_blocks_by_author(&self, address: &str) -> Result<u64, anyhow::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(BLOCKS_TABLE)?;
        let mut count = 0;
        let iter = table.iter()?;
        for item in iter {
            let (_, value) = item?;
            let block: Block = serde_json::from_str(value.value())?;
            if block.author == address {
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn get_block_by_hash(&self, hash: &str) -> Result<Option<Block>, anyhow::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(BLOCKS_TABLE)?;
        let iter = table.iter()?;
        for item in iter {
            let (_, value) = item?;
            let block: Block = serde_json::from_str(value.value())?;
            if block.hash == hash {
                return Ok(Some(block));
            }
        }
        Ok(None)
    }

    pub fn get_transaction_by_id(
        &self,
        tx_id: &str,
    ) -> Result<Option<(crate::chain::Transaction, Block)>, anyhow::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(BLOCKS_TABLE)?;
        let iter = table.iter()?;
        for item in iter {
            let (_, value) = item?;
            let block: Block = serde_json::from_str(value.value())?;
            if let Some(tx) = block.transactions.iter().find(|t| t.id == tx_id) {
                return Ok(Some((tx.clone(), block)));
            }
        }
        Ok(None)
    }

    pub fn save_setting(&self, key: &str, value: &str) -> Result<(), anyhow::Error> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SETTINGS_TABLE)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, anyhow::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(SETTINGS_TABLE)?;
        let result = match table.get(key)? {
            Some(guard) => Some(guard.value().to_string()),
            None => None,
        };
        Ok(result)
    }

    pub fn save_pending_tx(&self, tx: &crate::chain::Transaction) -> Result<(), anyhow::Error> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(MEMPOOL_TABLE)?;
            let json = serde_json::to_string(tx)?;
            table.insert(tx.id.as_str(), json.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn remove_pending_tx(&self, id: &str) -> Result<(), anyhow::Error> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(MEMPOOL_TABLE)?;
            table.remove(id)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_all_pending_txs(&self) -> Result<Vec<crate::chain::Transaction>, anyhow::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(MEMPOOL_TABLE)?;
        let mut txs = Vec::new();
        for item in table.iter()? {
            let (_, value) = item?;
            let tx: crate::chain::Transaction = serde_json::from_str(value.value())?;
            txs.push(tx);
        }
        Ok(txs)
    }

    pub fn reset_blocks(&self) -> Result<(), anyhow::Error> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(BLOCKS_TABLE)?;
            let keys: Vec<u64> = table.iter()?.map(|i| i.unwrap().0.value()).collect();
            for k in keys {
                table.remove(k)?;
            }
            // Clear state table
            let mut state_table = write_txn.open_table(STATE_TABLE)?;
            let state_keys: Vec<String> = state_table
                .iter()?
                .map(|i| i.unwrap().0.value().to_string())
                .collect();
            for k in state_keys {
                state_table.remove(k.as_str())?;
            }

            // Also clear mempool on block reset if requested (usually for hard reset)
            let mut mempool_table = write_txn.open_table(MEMPOOL_TABLE)?;
            let mem_keys: Vec<String> = mempool_table
                .iter()?
                .map(|i| i.unwrap().0.value().to_string())
                .collect();
            for k in mem_keys {
                mempool_table.remove(k.as_str())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn prune_history(&self, keep_blocks: u64) -> Result<u64, anyhow::Error> {
        let latest = self.get_latest_index()?;
        if latest <= keep_blocks {
            return Ok(0);
        }

        let prune_up_to = latest.saturating_sub(keep_blocks);
        let mut to_prune = Vec::new();

        {
            let read_txn = self.db.begin_read()?;
            let table = read_txn.open_table(BLOCKS_TABLE)?;
            let mut iter = table.iter()?;

            while let Some(res) = iter.next() {
                let (index, value) = res?;
                let idx = index.value();

                if idx >= prune_up_to {
                    break;
                }

                let block_json = value.value();
                if !block_json.contains("\"transactions\":[]") {
                    to_prune.push((idx, block_json.to_string()));
                }
            }
        }

        if to_prune.is_empty() {
            return Ok(0);
        }

        let write_txn = self.db.begin_write()?;
        let mut count = 0;
        {
            let mut table = write_txn.open_table(BLOCKS_TABLE)?;
            for (idx, json) in to_prune {
                let mut block: Block = serde_json::from_str(&json)?;
                if !block.transactions.is_empty() {
                    block.transactions = Vec::new();
                    let pruned_json = serde_json::to_string(&block)?;
                    table.insert(idx, pruned_json.as_str())?;
                    count += 1;
                }
            }
        }
        write_txn.commit()?;
        Ok(count)
    }
    pub fn start_new_run(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    pub fn remove_all_pending_txs(&self) -> Result<(), anyhow::Error> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(MEMPOOL_TABLE)?;
            let keys: Vec<String> = table
                .iter()?
                .map(|i| i.unwrap().0.value().to_string())
                .collect();
            for k in keys {
                table.remove(k.as_str())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
