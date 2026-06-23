use crate::chain::{validate_transaction, Transaction};
use crate::storage::Storage;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Mempool {
    pub pending_txs: Arc<Mutex<HashMap<String, Transaction>>>,
    pub storage: Arc<Storage>,
}

impl Mempool {
    pub fn new(storage: Arc<Storage>) -> Self {
        Mempool {
            pending_txs: Arc::new(Mutex::new(HashMap::new())),
            storage,
        }
    }

    pub fn load_from_db(&self) -> Result<(), String> {
        match self.storage.get_all_pending_txs() {
            Ok(txs) => {
                let mut pool = self.pending_txs.lock().unwrap();
                for tx in txs {
                    pool.insert(tx.id.clone(), tx);
                }
                Ok(())
            }
            Err(e) => Err(format!("Failed to load mempool from DB: {}", e)),
        }
    }

    pub fn reconcile_with_chain(&self) -> Result<usize, String> {
        let pending_txs = {
            let pool = self.pending_txs.lock().unwrap();
            pool.values().cloned().collect::<Vec<_>>()
        };

        let mut removed_count = 0;
        for tx in pending_txs {
            // 1. Check if tx exists in any block
            if self.is_tx_mined(&tx.id).unwrap_or(false) {
                self.remove_transactions(&[tx.id.clone()]);
                removed_count += 1;
                continue;
            }

            // 2. Check if still valid (Sender has enough balance)
            if tx.sender != "SYSTEM" {
                let balance = self.storage.calculate_balance(&tx.sender).unwrap_or(0);
                let required = tx
                    .amount
                    .saturating_add(crate::chain::calculate_fee(tx.amount));
                if balance < required {
                    log::info!("Dropping invalid mempool tx {}: Insufficient funds (Balance: {}, Required: {})", tx.id, balance, required);
                    self.remove_transactions(&[tx.id]);
                    removed_count += 1;
                }
            }
        }

        Ok(removed_count)
    }

    fn is_tx_mined(&self, tx_id: &str) -> Result<bool, anyhow::Error> {
        self.storage.is_tx_mined(tx_id)
    }

    pub fn add_transaction(&self, tx: Transaction) -> Result<(), String> {
        if tx.is_system() {
            return Err("SYSTEM transactions cannot enter the mempool".into());
        }

        tx.validate()?;

        let pending_spend = self.get_total_pending_spend(&tx.sender);
        validate_transaction(&tx, &self.storage, pending_spend)?;

        let mut pool = self.pending_txs.lock().unwrap();
        if pool.contains_key(&tx.id) {
            return Err("Transaction already in mempool".to_string());
        }

        // Save to Persistence
        if let Err(e) = self.storage.save_pending_tx(&tx) {
            log::error!("Failed to persist mempool transaction {}: {}", tx.id, e);
        }

        pool.insert(tx.id.clone(), tx);
        Ok(())
    }

    pub fn get_pending_transactions(&self) -> Vec<Transaction> {
        let pool = self.pending_txs.lock().unwrap();
        pool.values().cloned().collect()
    }

    pub fn get_total_pending_spend(&self, address: &str) -> u64 {
        let pool = self.pending_txs.lock().unwrap();
        pool.values()
            .filter(|tx| tx.sender == address && tx.sender != "SYSTEM")
            .map(|tx| {
                tx.amount
                    .saturating_add(crate::chain::calculate_fee(tx.amount))
            })
            .sum()
    }

    pub fn remove_transactions(&self, tx_ids: &[String]) {
        let mut pool = self.pending_txs.lock().unwrap();
        for id in tx_ids {
            pool.remove(id);
            // Remove from Persistence
            if let Err(e) = self.storage.remove_pending_tx(id) {
                log::warn!(
                    "Failed to remove persisted mempool transaction {}: {}",
                    id,
                    e
                );
            }
        }
    }

    pub fn len(&self) -> usize {
        let pool = self.pending_txs.lock().unwrap();
        pool.len()
    }

    pub fn clear(&self) {
        let mut pool = self.pending_txs.lock().unwrap();
        pool.clear();
        if let Err(e) = self.storage.remove_all_pending_txs() {
            log::warn!("Failed to clear pending txs from storage: {}", e);
        }
    }
}
