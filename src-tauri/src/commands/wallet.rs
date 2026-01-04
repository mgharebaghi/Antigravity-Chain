use crate::state::AppState;
use crate::wallet::{self, Wallet};
use rand::RngCore;
use std::sync::atomic::Ordering;
use tauri::State;

#[tauri::command]
pub fn create_wallet(state: State<'_, AppState>) -> Result<wallet::WalletExport, String> {
    let mut wallet_guard = state.wallet.lock().unwrap();

    // Generate Mnemonic (12 words) using 16 bytes of entropy
    let mut entropy = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut entropy);
    let mnemonic = bip39::Mnemonic::from_entropy(&entropy).map_err(|e| e.to_string())?;
    let seed = mnemonic.to_seed("");

    // Derive keypair from seed (simplified for lab, using first 32 bytes)
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&seed[0..32]);

    let keypair = libp2p::identity::Keypair::ed25519_from_bytes(key_bytes).unwrap();
    let peer_id = keypair.public().to_peer_id();
    let address = peer_id.to_string();

    let keypair_bytes = keypair.to_protobuf_encoding().unwrap();
    let keys_json = serde_json::to_string(&keypair_bytes).unwrap();

    // Save to DB
    let _ = state.storage.save_wallet_keys(&keys_json);

    let export = wallet::WalletExport {
        address: address.clone(),
        private_key: hex::encode(&keypair_bytes),
        mnemonic: mnemonic.to_string(),
    };

    let new_wallet = Wallet {
        start_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        address: address.clone(),
        alias: None,
        keypair: keypair_bytes,
    };

    *wallet_guard = Some(new_wallet);

    // Update mined blocks counter for the new wallet
    let count = state.storage.count_blocks_by_author(&address).unwrap_or(0);
    state.mined_by_me_count.store(count, Ordering::Relaxed);

    Ok(export)
}

#[tauri::command]
pub fn import_wallet(
    state: State<'_, AppState>,
    private_key_hex: String,
) -> Result<String, String> {
    let mut wallet_guard = state.wallet.lock().unwrap();

    let keypair_bytes = if private_key_hex.split_whitespace().count() == 12 {
        // Handle Mnemonic
        let mnemonic = bip39::Mnemonic::parse(&private_key_hex)
            .map_err(|e| format!("Invalid mnemonic: {}", e))?;
        let seed = mnemonic.to_seed("");
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&seed[0..32]);
        let keypair = libp2p::identity::Keypair::ed25519_from_bytes(key_bytes).unwrap();
        keypair.to_protobuf_encoding().unwrap()
    } else {
        // Handle HEX
        hex::decode(private_key_hex).map_err(|e| format!("Invalid hex: {}", e))?
    };

    // Validate keypair
    let keypair = libp2p::identity::Keypair::from_protobuf_encoding(&keypair_bytes)
        .map_err(|e| format!("Invalid keypair data: {}", e))?;

    let address = keypair.public().to_peer_id().to_string();

    let new_wallet = Wallet {
        start_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        address: address.clone(),
        alias: None,
        keypair: keypair_bytes,
    };

    let keys_json = serde_json::to_string(&new_wallet.keypair).unwrap();
    let _ = state.storage.save_wallet_keys(&keys_json);

    *wallet_guard = Some(new_wallet);

    // Update mined blocks counter for the new wallet
    let count = state.storage.count_blocks_by_author(&address).unwrap_or(0);
    state.mined_by_me_count.store(count, Ordering::Relaxed);

    Ok(address)
}

#[tauri::command]
pub fn get_wallet_info(state: State<'_, AppState>) -> Option<wallet::WalletInfo> {
    let wallet_guard = state.wallet.lock().unwrap();
    if let Some(w) = wallet_guard.as_ref() {
        let total_balance = state.storage.calculate_balance(&w.address).unwrap_or(0);
        let pending_spend = state.mempool.get_total_pending_spend(&w.address);
        let available_balance = total_balance.saturating_sub(pending_spend);

        Some(wallet::WalletInfo {
            address: w.address.clone(),
            balance: available_balance,
            alias: w.alias.clone(),
            private_key: Some(hex::encode(&w.keypair)),
        })
    } else {
        None
    }
}

#[tauri::command]
pub async fn logout_wallet(state: State<'_, AppState>) -> Result<(), String> {
    println!("Backend: logout_wallet called");

    // 1. Clear in-memory wallet
    {
        let mut wallet = state.wallet.lock().map_err(|e| e.to_string())?;
        *wallet = None;
    }

    // 2. Clear mined count (optional, but makes sense for UI)
    state.mined_by_me_count.store(0, Ordering::SeqCst);

    // 3. Delete from storage
    state
        .storage
        .delete_wallet_keys()
        .map_err(|e| e.to_string())?;

    println!("Backend: Wallet logged out successfully");
    Ok(())
}
