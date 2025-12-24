use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{Method, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use libp2p::{
    futures::StreamExt,
    gossipsub, identity, kad, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, SwarmBuilder,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tauri_appantigravity_chain_lib::{
    chain::{Block, SyncRequest, SyncResponse, Transaction},
    consensus::Consensus,
    mempool::Mempool,
    p2p::message_id_fn,
    storage::Storage,
};
use tokio::io;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

// --- Shared State for API ---
struct AppState {
    storage: Arc<Storage>,
    mempool: Arc<Mempool>,
    consensus: Arc<Mutex<Consensus>>,
    chain_index: Arc<AtomicU64>,
    peer_count: Arc<std::sync::atomic::AtomicUsize>,
    tx_sender: tokio::sync::mpsc::Sender<Transaction>, // To submit tx to P2P
    evt_sender: broadcast::Sender<Event>,              // Broadcast events to WebSockets
}

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type", content = "data")]
enum Event {
    NewBlock(Block),
    NewTransaction(Transaction),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Logger
    flexi_logger::Logger::try_with_str("info")?.start()?;
    log::info!("Starting Antigravity RPC Node (Advanced)...");

    // Initialize Components
    let storage = Arc::new(Storage::new("antigravity.db")?); // Same DB as main app if running in same dir
    let mempool = Arc::new(Mempool::new(storage.clone()));
    let consensus = Arc::new(Mutex::new(Consensus::new()));

    // Determine latest index
    let current_height = storage.get_latest_index().unwrap_or(0);
    let chain_index = Arc::new(AtomicU64::new(current_height));
    let peer_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    // Channels
    let (tx_submit_sender, mut tx_submit_receiver) = tokio::sync::mpsc::channel::<Transaction>(100);
    // Broadcast channel for Real-time events (Capacity 100)
    let (evt_sender, _) = broadcast::channel::<Event>(100);

    // --- P2P Setup (Headless) ---
    // We duplicate some P2P setup logic here because p2p.rs is tightly coupled with Tauri AppHandle
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    log::info!("RPC Node Peer ID: {}", local_peer_id);

    consensus
        .lock()
        .unwrap()
        .set_local_peer_id(local_peer_id.to_string());

    let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_dns()?
        .with_relay_client(noise::Config::new, yamux::Config::default)?
        .with_behaviour(|key, relay_client| {
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .message_id_fn(message_id_fn)
                    .build()
                    .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?,
            )?;

            let mut kad_config = kad::Config::default();
            kad_config
                .set_protocol_names(vec![libp2p::StreamProtocol::new("/antigravity/kad/1.0.0")]);
            let kad = kad::Behaviour::with_config(
                key.public().to_peer_id(),
                kad::store::MemoryStore::new(key.public().to_peer_id()),
                kad_config,
            );

            Ok(HeaderlessBehaviour {
                gossipsub,
                kad,
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    PeerId::from(key.public()),
                )?,
                relay_client,
                dcutr: libp2p::dcutr::Behaviour::new(key.public().to_peer_id()),
                identify: libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
                    "/antigravity/1.0.0".to_string(),
                    key.public(),
                )),
                ping: libp2p::ping::Behaviour::new(libp2p::ping::Config::new()),
                sync: libp2p::request_response::cbor::Behaviour::new(
                    [(
                        libp2p::StreamProtocol::new("/antigravity/sync/1"),
                        libp2p::request_response::ProtocolSupport::Full,
                    )],
                    libp2p::request_response::Config::default(),
                ),
            })
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    // Subscribe topics
    let topic_blocks = gossipsub::IdentTopic::new("antigravity-blocks");
    let topic_transactions = gossipsub::IdentTopic::new("antigravity-transactions");
    swarm.behaviour_mut().gossipsub.subscribe(&topic_blocks)?;
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topic_transactions)?;

    // Listen
    swarm.listen_on("/ip4/0.0.0.0/tcp/9091".parse()?)?; // Use 9091 to allow running alongside Relay (9090)

    // Connect to local relay if valid (assuming default)
    let relay_addr = "/ip4/127.0.0.1/tcp/9090";
    if let Ok(addr) = relay_addr.parse::<Multiaddr>() {
        let _ = swarm.dial(addr);
    }

    // Shared refs for P2P loop
    let p2p_storage = storage.clone();
    let p2p_mempool = mempool.clone();
    let _p2p_consensus = consensus.clone();
    let p2p_chain_index = chain_index.clone();
    let p2p_peer_count = peer_count.clone();
    let p2p_evt_sender = evt_sender.clone(); // Clone for loop

    // Spin up P2P Task
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(tx) = tx_submit_receiver.recv() => {
                    log::info!("API Broadcasting TX: {}", tx.id);
                    let json = serde_json::to_vec(&tx).unwrap();
                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic_transactions.clone(), json) {
                        log::error!("Gossip publish error: {:?}", e);
                    } else {
                         // Also notify WS clients that WE sent a tx
                         let _ = p2p_evt_sender.send(Event::NewTransaction(tx));
                    }
                }
                event = swarm.select_next_some() => match event {
                    SwarmEvent::NewListenAddr { address, .. } => log::info!("P2P listening on {:?}", address),
                    SwarmEvent::Behaviour(HeaderlessBehaviourEvent::Gossipsub(
                        gossipsub::Event::Message { message, .. }
                    )) => {
                        let topic = message.topic.clone();
                        if topic == topic_blocks.hash() {
                            if let Ok(block) = serde_json::from_slice::<Block>(&message.data) {
                                if block.is_vdf_valid() {
                                    if p2p_storage.get_block(block.index).unwrap_or(None).is_none() {
                                        let _ = p2p_storage.save_block(&block);
                                        p2p_chain_index.store(block.index, Ordering::Relaxed);

                                        // Clean mempool
                                        let tx_ids: Vec<String> = block.transactions.iter().map(|t| t.id.clone()).collect();
                                        p2p_mempool.remove_transactions(&tx_ids);

                                        // Notify WS
                                        let _ = p2p_evt_sender.send(Event::NewBlock(block));
                                    }
                                }
                            }
                        } else if topic == topic_transactions.hash() {
                            if let Ok(tx) = serde_json::from_slice::<Transaction>(&message.data) {
                                if let Ok(_) = p2p_mempool.add_transaction(tx.clone()) {
                                    // Notify WS
                                    let _ = p2p_evt_sender.send(Event::NewTransaction(tx));
                                }
                            }
                        }
                    }
                    SwarmEvent::Behaviour(HeaderlessBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                         for (peer_id, _) in list {
                             swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                             let _ = swarm.behaviour_mut().sync.send_request(&peer_id, SyncRequest::GetHeight);
                         }
                         p2p_peer_count.store(swarm.network_info().num_peers(), Ordering::Relaxed);
                    }
                    SwarmEvent::Behaviour(HeaderlessBehaviourEvent::Sync(
                         libp2p::request_response::Event::Message { peer: _, message }
                    )) => {
                        match message {
                            libp2p::request_response::Message::Request { request, channel, .. } => {
                                match request {
                                    SyncRequest::GetHeight => {
                                        let h = p2p_storage.get_latest_index().unwrap_or(0);
                                        let _ = swarm.behaviour_mut().sync.send_response(channel, SyncResponse::Height(h));
                                    },
                                    SyncRequest::GetBlock(idx) => {
                                        let b = p2p_storage.get_block(idx).unwrap_or(None);
                                        let _ = swarm.behaviour_mut().sync.send_response(channel, SyncResponse::Block(b));
                                    },
                                    SyncRequest::GetMempool => {
                                        let txs = p2p_mempool.get_pending_transactions();
                                        let _ = swarm.behaviour_mut().sync.send_response(channel, SyncResponse::Mempool(txs));
                                    }
                                }
                            },
                             libp2p::request_response::Message::Response { response, .. } => {
                                match response {
                                    SyncResponse::Height(_h) => {},
                                    SyncResponse::Block(Some(block)) => {
                                        if block.is_vdf_valid() {
                                            if let Ok(_) = p2p_storage.save_block(&block) {
                                                 p2p_evt_sender.send(Event::NewBlock(block)).ok();
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                    SwarmEvent::ConnectionEstablished { .. } | SwarmEvent::ConnectionClosed { .. } => {
                         p2p_peer_count.store(swarm.network_info().num_peers(), Ordering::Relaxed);
                    }
                    _ => {}
                }
            }
        }
    });

    // --- API Server ---
    let app_state = Arc::new(AppState {
        storage,
        mempool,
        consensus,
        chain_index,
        peer_count,
        tx_sender: tx_submit_sender,
        evt_sender,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/v1/status", get(get_status))
        .route("/api/v1/blocks", get(get_blocks_paginated)) // New
        .route("/api/v1/blocks/index/:index", get(get_block_by_index))
        .route("/api/v1/blocks/hash/:hash", get(get_block_by_hash))
        .route("/api/v1/transactions/:id", get(get_transaction))
        .route("/api/v1/balance/:address", get(get_balance))
        .route("/api/v1/broadcast", post(broadcast_tx))
        .route("/api/v1/network/stats", get(get_network_stats)) // New
        .route("/ws", get(websocket_handler)) // New
        .layer(cors)
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    log::info!("RPC API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// --- API Handlers ---

#[derive(Serialize)]
struct StatusResponse {
    node_type: String,
    chain_height: u64,
    peer_count: usize,
    network: String,
}

// WS Handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

async fn websocket_connection(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.evt_sender.subscribe();
    if let Err(e) = socket
        .send(Message::Text(
            "Connected to Antigravity Real-time Feed".to_string(),
        ))
        .await
    {
        log::error!("WS send error: {}", e);
        return;
    }

    loop {
        match rx.recv().await {
            Ok(event) => {
                if let Ok(json) = serde_json::to_string(&event) {
                    if socket.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {}
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

async fn get_status(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
    let height = state.chain_index.load(Ordering::Relaxed);
    let peers = state.peer_count.load(Ordering::Relaxed);

    Json(StatusResponse {
        node_type: "RPC".to_string(),
        chain_height: height,
        peer_count: peers,
        network: "Antigravity Mainnet".to_string(),
    })
}

#[derive(Deserialize)]
struct Pagination {
    page: Option<usize>,
    limit: Option<usize>,
}

async fn get_blocks_paginated(
    State(state): State<Arc<AppState>>,
    Query(params): Query<Pagination>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(0);
    let limit = params.limit.unwrap_or(20);

    match state.storage.get_blocks_paginated(page, limit) {
        Ok(blocks) => Json(blocks).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Storage error").into_response(),
    }
}

#[derive(Serialize)]
struct NetworkStats {
    supply: u64,
    max_supply: u64,
    circulating: u64,
    halving_block: u64,
    current_reward: u64,
    mining_difficulty: u64,
}

async fn get_network_stats(State(state): State<Arc<AppState>>) -> Json<NetworkStats> {
    let height = state.chain_index.load(Ordering::Relaxed);
    let supply = tauri_appantigravity_chain_lib::chain::calculate_circulating_supply(height);
    let reward = tauri_appantigravity_chain_lib::chain::calculate_mining_reward(height + 1);

    // Calculate simple halving info
    let (_halving_interval, _phase) = if height <= 500_000 {
        (100_000, "Early")
    } else {
        (400_000, "Stable")
    };

    // This is rough estimation for "next halving", logic mirrors chain::calculate_mining_reward
    let next_halving = if height <= 500_000 {
        ((height / 100_000) + 1) * 100_000
    } else {
        500_000 + ((height - 500_000) / 400_000 + 1) * 400_000
    };

    Json(NetworkStats {
        supply,
        max_supply: tauri_appantigravity_chain_lib::chain::TOTAL_SUPPLY,
        circulating: supply, // Simplifying for now
        halving_block: next_halving,
        current_reward: reward,
        mining_difficulty: 200_000, // Hardcoded for VDF PoC
    })
}

async fn get_block_by_index(
    State(state): State<Arc<AppState>>,
    Path(index): Path<u64>,
) -> impl IntoResponse {
    match state.storage.get_block(index) {
        Ok(Some(block)) => Json(block).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Block not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Storage error").into_response(),
    }
}

async fn get_block_by_hash(
    State(state): State<Arc<AppState>>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    match state.storage.get_block_by_hash(&hash) {
        Ok(Some(block)) => Json(block).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Block not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Storage error").into_response(),
    }
}

async fn get_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.storage.get_transaction_by_id(&id) {
        Ok(Some((tx, block))) => Json(serde_json::json!({
            "transaction": tx,
            "block_index": block.index,
            "block_hash": block.hash,
            "status": "confirmed"
        }))
        .into_response(),
        Ok(None) => {
            // Check mempool
            let pending = state.mempool.get_pending_transactions();
            if let Some(tx) = pending.iter().find(|t| t.id == id) {
                Json(serde_json::json!({
                    "transaction": tx,
                    "status": "pending"
                }))
                .into_response()
            } else {
                (StatusCode::NOT_FOUND, "Transaction not found").into_response()
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Storage error").into_response(),
    }
}

async fn get_balance(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    match state.storage.calculate_balance(&address) {
        Ok(balance) => Json(serde_json::json!({
            "address": address,
            "balance": balance,
            "currency": "AGT"
        }))
        .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Storage error").into_response(),
    }
}

#[derive(Deserialize)]
struct BroadcastRequest {
    transaction: Transaction,
}

async fn broadcast_tx(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BroadcastRequest>,
) -> impl IntoResponse {
    // Basic validation
    // Verify signature logic would ideally be here or in mempool

    if let Err(e) = state.mempool.add_transaction(payload.transaction.clone()) {
        return (
            StatusCode::BAD_REQUEST,
            format!("Invalid transaction: {}", e),
        )
            .into_response();
    }

    // Send to P2P loop to broadcast
    if let Err(_) = state.tx_sender.send(payload.transaction.clone()).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to broadcast").into_response();
    }

    Json(serde_json::json!({
        "status": "accepted",
        "tx_id": payload.transaction.id
    }))
    .into_response()
}

// --- Network Behaviour ---
#[derive(NetworkBehaviour)]
pub struct HeaderlessBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub kad: kad::Behaviour<kad::store::MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
    pub relay_client: libp2p::relay::client::Behaviour,
    pub dcutr: libp2p::dcutr::Behaviour,
    pub identify: libp2p::identify::Behaviour,
    pub ping: libp2p::ping::Behaviour,
    pub sync: libp2p::request_response::cbor::Behaviour<SyncRequest, SyncResponse>,
}
