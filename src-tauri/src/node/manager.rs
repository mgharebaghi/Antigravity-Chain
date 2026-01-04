use crate::chain;
use crate::network;
use crate::node::{mining, vdf};
use crate::state::{AppSettings, AppState};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, State};

pub async fn start_node_service(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Check if wallet exists
    {
        let wallet_guard = state.wallet.lock().unwrap();
        if wallet_guard.is_none() {
            return Err(
                "Wallet required to start node. Please create or import a wallet first."
                    .to_string(),
            );
        }
    }

    // Check if already running
    if state.is_running.load(Ordering::Relaxed) {
        return Ok("Node is already running".to_string());
    }
    state.is_running.store(true, Ordering::Relaxed);

    // Increment run_id to invalidate previous loops
    let my_run_id = state.run_id.fetch_add(1, Ordering::Relaxed) + 1;

    let node_type_arc = state.node_type.clone();

    // Create Channels
    let (block_sender, block_receiver) = tokio::sync::mpsc::channel::<Box<chain::Block>>(100);
    let (tx_sender, tx_receiver) = tokio::sync::mpsc::channel::<chain::Transaction>(1000);

    let (receipt_sender, receipt_receiver) = tokio::sync::mpsc::channel::<chain::Receipt>(1000);
    // New Channel for broadcasting VDF Proofs
    let (vdf_sender, vdf_receiver) = tokio::sync::mpsc::channel::<chain::VdfProofMessage>(100);

    {
        let mut ts = state.tx_sender.lock().unwrap();
        *ts = Some(tx_sender.clone());
    }
    {
        let mut rs = state.receipt_sender.lock().unwrap();
        *rs = Some(receipt_sender.clone());
    }

    // Spawn P2P
    // Extract wallet keypair to ensure Node ID matches Wallet ID
    let wallet_keypair = {
        let w_guard = state.wallet.lock().unwrap();
        if let Some(w) = w_guard.as_ref() {
            libp2p::identity::Keypair::from_protobuf_encoding(&w.keypair).ok()
        } else {
            None
        }
    };

    // Fetch settings
    let settings = match state.storage.get_setting("app_settings") {
        Ok(Some(json)) => serde_json::from_str::<AppSettings>(&json).unwrap_or_default(),
        _ => AppSettings::default(),
    };
    let relay_addresses = settings.relay_addresses.clone();
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(100);
    let validator_count_p2p = state.validator_count.clone();
    let peer_count_p2p = state.peer_count.clone();
    let is_synced_p2p = state.is_synced.clone();
    let is_running_p2p = state.is_running.clone();
    let consensus_p2p = state.consensus.clone();
    let storage_p2p = state.storage.clone();
    let mempool_p2p = state.mempool.clone();
    let run_id_p2p = state.run_id.clone();
    let chain_index_p2p = state.chain_index.clone();
    let node_type_p2p = state.node_type.clone();
    let relay_connected_p2p = state.relay_connected.clone();
    let app_handle_p2p = app_handle.clone();

    // --- P2P START ---
    tokio::spawn(async move {
        if let Err(e) = network::p2p::start_p2p_node(
            app_handle_p2p,
            storage_p2p,
            mempool_p2p,
            consensus_p2p,
            is_synced_p2p,
            is_running_p2p,
            run_id_p2p,
            peer_count_p2p,
            validator_count_p2p,
            chain_index_p2p,
            relay_addresses, // Vec<String>
            my_run_id,
            block_receiver,
            tx_receiver,
            receipt_receiver,
            vdf_receiver,
            node_type_p2p,
            relay_connected_p2p,
            wallet_keypair,
            cmd_rx,
        )
        .await
        {
            log::error!("P2P Node Error: {:?}", e);
        }
    });

    // Spawn Genesis Checker & Block Production Loop
    let storage_clone = state.storage.clone();
    let mempool_clone = state.mempool.clone();
    let is_synced_loop = state.is_synced.clone();
    let consensus_clone = state.consensus.clone();
    let is_running_loop = state.is_running.clone();
    let run_id_loop = state.run_id.clone();
    let peer_count_loop = state.peer_count.clone();
    let wallet_addr = {
        let w = state.wallet.lock().unwrap();
        w.as_ref().unwrap().address.clone()
    };
    let app_handle_loop = app_handle.clone();

    // Initial load from storage to sync memory counter
    if let Ok(count) = storage_clone.count_blocks_by_author(&wallet_addr) {
        state.mined_by_me_count.store(count, Ordering::Relaxed);
    }
    let block_sender_loop = block_sender.clone();
    let chain_index_loop = state.chain_index.clone();
    let mined_by_me_count_loop = state.mined_by_me_count.clone();
    let wallet_clone = state.wallet.clone(); // Clone ARC for loop
    let mining_enabled_arc = state.mining_enabled.clone();
    let receipt_sender_loop = state.receipt_sender.clone();
    let validator_count_loop = state.validator_count.clone();

    // Initialize metrics from storage
    let current_height = state.storage.get_latest_index().unwrap_or(0);
    state.chain_index.store(current_height, Ordering::Relaxed);

    if let Some(w) = state.wallet.lock().unwrap().as_ref() {
        let count = state
            .storage
            .count_blocks_by_author(&w.address)
            .unwrap_or(0);
        state.mined_by_me_count.store(count, Ordering::Relaxed);
    }

    // Spawn VDF Heartbeat Loop
    let app_handle_vdf = app_handle.clone();
    let is_running_vdf = state.is_running.clone();
    let run_id_vdf = state.run_id.clone();
    let vdf_ips_arc = state.vdf_ips.clone();

    vdf::spawn_vdf_heartbeat(
        app_handle_vdf,
        is_running_vdf,
        run_id_vdf,
        vdf_ips_arc,
        my_run_id,
    );

    // Spawn Mining Loop
    let cmd_tx_loop = cmd_tx.clone();
    let relay_connected_loop = state.relay_connected.clone();

    mining::spawn_mining_loop(
        app_handle_loop,
        is_running_loop,
        run_id_loop,
        peer_count_loop,
        validator_count_loop,
        storage_clone,
        mempool_clone,
        consensus_clone,
        is_synced_loop,
        chain_index_loop,
        mined_by_me_count_loop,
        wallet_clone,
        mining_enabled_arc,
        receipt_sender_loop,
        node_type_arc,
        cmd_tx_loop,
        block_sender_loop,
        my_run_id,
        wallet_addr,
        relay_connected_loop,
    );

    // Spawn VDF Solver
    let consensus_clone_vdf = state.consensus.clone();
    let app_handle_vdf2 = app_handle.clone();
    let is_running_vdf2 = state.is_running.clone();
    let is_synced_vdf = state.is_synced.clone();
    let vdf_broadcaster = vdf_sender.clone();

    vdf::spawn_vdf_solver(
        app_handle_vdf2,
        is_running_vdf2,
        is_synced_vdf,
        consensus_clone_vdf,
        vdf_broadcaster,
    );

    Ok("Node started".to_string())
}
