//! # P2P Node Module
//!
//! Main P2P networking implementation for Centichain.
//! Handles network events, gossip messaging, and blockchain synchronization.

use futures::StreamExt;
use libp2p::multiaddr::Protocol;
use libp2p::{
    gossipsub, identity, kad, mdns, noise, relay, swarm::SwarmEvent, tcp, yamux, PeerId,
    SwarmBuilder,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io;

use tauri::{AppHandle, Emitter};

use crate::chain::{ingest_block, Block, BlockAcceptResult, SyncRequest, SyncResponse, Transaction};
use crate::consensus::mempool::Mempool;
use crate::consensus::Consensus;
use crate::storage::Storage;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

use super::behaviour::{
    message_id_fn, CentichainBehaviour, CentichainBehaviourEvent, SYNC_PROTOCOL,
};
use super::commands::{P2PCommand, TopologyUpdate};
use super::startup::{NodeStartupState, StartupConfig};

// =============================================================================
// Main P2P Node Function
// =============================================================================

/// Starts the P2P network node
///
/// This function sets up libp2p networking and runs the main event loop.
/// It handles:
/// - Gossipsub messaging for blocks, transactions, and status updates
/// - Kademlia DHT for peer discovery
/// - mDNS for local network discovery
/// - Relay connections for NAT traversal
/// - Request-response protocol for blockchain sync
pub async fn start_p2p_node(
    app_handle: AppHandle,
    storage: Arc<Storage>,
    mempool: Arc<Mempool>,
    consensus: Arc<Mutex<Consensus>>,
    is_synced: Arc<AtomicBool>,
    is_running: Arc<AtomicBool>,
    run_id: Arc<AtomicU64>,
    peer_count: Arc<AtomicUsize>,
    validator_count: Arc<AtomicUsize>,
    chain_index: Arc<AtomicU64>,
    relay_addrs: Vec<String>,
    my_run_id: u64,
    mut block_receiver: tokio::sync::mpsc::Receiver<Box<crate::chain::Block>>,
    mut tx_receiver: tokio::sync::mpsc::Receiver<crate::chain::Transaction>,
    mut receipt_receiver: tokio::sync::mpsc::Receiver<crate::chain::Receipt>,
    mut vdf_receiver: tokio::sync::mpsc::Receiver<crate::chain::VdfProofMessage>,
    node_type: Arc<Mutex<crate::NodeType>>,
    relay_connected: Arc<AtomicBool>,
    wallet_keypair: Option<identity::Keypair>,
    mut cmd_rx: tokio::sync::mpsc::Receiver<P2PCommand>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize keypair and peer ID
    let local_key = wallet_keypair.unwrap_or_else(identity::Keypair::generate_ed25519);
    let local_peer_id = PeerId::from(local_key.public());
    log::info!("Local peer id: {:?}", local_peer_id);

    // Register self in consensus
    consensus
        .lock()
        .unwrap()
        .set_local_peer_id(local_peer_id.to_string());

    // Build the swarm
    let mut swarm = build_swarm(local_key.clone())?;

    // Setup gossipsub topics
    let topics = setup_topics(&mut swarm, &consensus, &local_peer_id)?;

    // Listen on all interfaces
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Connect to relays
    let relay_peer_id_opt = connect_to_relays(
        &mut swarm,
        &relay_addrs,
        &local_peer_id,
        &consensus,
        &relay_connected,
        &app_handle,
    );

    // Bootstrap DHT
    if let Err(e) = swarm.behaviour_mut().kad.bootstrap() {
        log::warn!(
            "Kademlia bootstrap failed (non-fatal if first node): {:?}",
            e
        );
    }

    let _ = app_handle.emit("node-status", "Connecting");

    // Network graph state for topology visualization
    let mut network_graph: HashMap<String, Vec<String>> = HashMap::new();

    // Startup state machine
    let startup_config = StartupConfig::default();
    let mut startup_state = NodeStartupState::new_connecting();

    // Event loop timers
    let mut check_interval = tokio::time::interval(Duration::from_secs(1));
    let mut discovery_interval = tokio::time::interval(Duration::from_secs(15));
    let mut topology_gossip_interval = tokio::time::interval(Duration::from_secs(30));

    // Clone relay_peer_id for use in loop
    let mut relay_peer_id_opt = relay_peer_id_opt;

    // Main event loop
    loop {
        // Check if we should stop
        if !is_running.load(Ordering::Relaxed) || run_id.load(Ordering::Relaxed) != my_run_id {
            log::info!("P2P Node stopping...");
            let _ = app_handle.emit("relay-status", "Disconnected");
            relay_connected.store(false, Ordering::Relaxed);
            break;
        }

        // Handle startup state transitions
        handle_startup_state(
            &mut startup_state,
            &startup_config,
            &relay_connected,
            &mut swarm,
            relay_peer_id_opt,
            &app_handle,
        );

        tokio::select! {
            // Handle P2P commands
            Some(cmd) = cmd_rx.recv() => {
                handle_command(
                    cmd,
                    &mut swarm,
                    &consensus,
                    &local_peer_id,
                    relay_peer_id_opt,
                    &topics,
                );
            }

            // Periodic sync/discovery check
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                handle_periodic_sync(
                    &mut swarm,
                    &app_handle,
                    &startup_state,
                    &is_synced,
                    &local_peer_id,
                    relay_peer_id_opt,
                );
            }

            // Periodic random walk discovery
            _ = discovery_interval.tick() => {
                let connected = swarm.connected_peers().count();
                if connected < 5 {
                    log::info!("P2P Loop: Performing Random Walk for Discovery...");
                    let random_peer_id = PeerId::random();
                    swarm.behaviour_mut().kad.get_closest_peers(random_peer_id);
                }
            }

            // Topology gossip broadcast
            _ = topology_gossip_interval.tick() => {
                broadcast_topology(
                    &mut swarm,
                    &local_peer_id,
                    &mut network_graph,
                    &topics,
                    &app_handle,
                );
            }

            // Peer count check
            _ = check_interval.tick() => {
                update_peer_counts(
                    &swarm,
                    &peer_count,
                    &validator_count,
                    &consensus,
                    relay_peer_id_opt,
                    &startup_state,
                    &app_handle,
                );
            }

            // Block broadcast from mining
            Some(block) = block_receiver.recv() => {
                log::info!("Broadcasting mined block index: {}", block.index);
                let json = serde_json::to_vec(&*block).unwrap();
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topics.shard_blocks.clone(), json) {
                    log::error!("Gossip block publish error: {:?}", e);
                }
            }

            // Transaction broadcast
            Some(tx) = tx_receiver.recv() => {
                log::info!("Broadcasting local transaction: {}", tx.id);
                let json = serde_json::to_vec(&tx).unwrap();
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topics.shard_txs.clone(), json) {
                    log::error!("Gossip tx publish error: {:?}", e);
                }
            }

            // Receipt broadcast
            Some(receipt) = receipt_receiver.recv() => {
                log::info!("Broadcasting Cross-Shard Receipt: {}", receipt.original_tx_id);
                let json = serde_json::to_vec(&receipt).unwrap();
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topics.receipts.clone(), json) {
                    log::error!("Gossip receipt publish error: {:?}", e);
                }
            }

            // VDF proof broadcast
            Some(vdf_msg) = vdf_receiver.recv() => {
                log::info!("Broadcasting VDF Proof for {}", vdf_msg.peer_id);
                let json = serde_json::to_vec(&vdf_msg).unwrap();
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topics.vdf_proofs.clone(), json) {
                    log::error!("Gossip VDF publish error: {:?}", e);
                }
            }

            // Swarm events
            event = swarm.select_next_some() => {
                handle_swarm_event(
                    event,
                    &mut swarm,
                    &app_handle,
                    &storage,
                    &mempool,
                    &consensus,
                    &chain_index,
                    &is_synced,
                    &peer_count,
                    &relay_addrs,
                    &mut relay_peer_id_opt,
                    &relay_connected,
                    &node_type,
                    &topics,
                    &mut network_graph,
                );
            }
        }
    }
    Ok(())
}

// =============================================================================
// Helper Structs
// =============================================================================

/// Gossipsub topics used by the network
pub struct GossipTopics {
    pub shard_blocks: gossipsub::IdentTopic,
    pub shard_txs: gossipsub::IdentTopic,
    pub receipts: gossipsub::IdentTopic,
    pub vdf_proofs: gossipsub::IdentTopic,
    pub topology: gossipsub::IdentTopic,
    pub node_status: gossipsub::IdentTopic,
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Builds the libp2p swarm with all required behaviours
fn build_swarm(
    local_key: identity::Keypair,
) -> Result<libp2p::Swarm<CentichainBehaviour>, Box<dyn std::error::Error>> {
    let swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_dns()?
        .with_relay_client(noise::Config::new, yamux::Config::default)?
        .with_behaviour(|key, relay_client| {
            // Gossipsub
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .message_id_fn(message_id_fn)
                .build()
                .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?;

            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            // Kademlia DHT
            let mut kad_config = kad::Config::default();
            kad_config
                .set_protocol_names(vec![libp2p::StreamProtocol::new("/centichain/kad/1.0.0")]);
            let store = kad::store::MemoryStore::new(key.public().to_peer_id());
            let kad = kad::Behaviour::with_config(key.public().to_peer_id(), store, kad_config);

            // MDNS
            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), PeerId::from(key.public()))?;

            // DCUtR
            let dcutr = libp2p::dcutr::Behaviour::new(key.public().to_peer_id());

            // Identify
            let identify = libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
                "/centichain/1.0.0".to_string(),
                key.public(),
            ));

            // Ping
            let ping = libp2p::ping::Behaviour::new(
                libp2p::ping::Config::new().with_interval(Duration::from_secs(5)),
            );

            // Request-Response (Sync)
            let sync = libp2p::request_response::cbor::Behaviour::new(
                [(
                    libp2p::StreamProtocol::new(SYNC_PROTOCOL),
                    libp2p::request_response::ProtocolSupport::Full,
                )],
                libp2p::request_response::Config::default(),
            );

            Ok(CentichainBehaviour {
                gossipsub,
                kad,
                mdns,
                relay_client,
                dcutr,
                identify,
                ping,
                sync,
            })
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    Ok(swarm)
}

/// Sets up gossipsub topics and subscriptions
fn setup_topics(
    swarm: &mut libp2p::Swarm<CentichainBehaviour>,
    consensus: &Arc<Mutex<Consensus>>,
    local_peer_id: &PeerId,
) -> Result<GossipTopics, Box<dyn std::error::Error>> {
    let shard_id = {
        let c = consensus.lock().unwrap();
        c.get_assigned_shard(&local_peer_id.to_string(), 0)
    };
    log::info!("P2P: Subscribing to Shard #{} topics", shard_id);

    let topics = GossipTopics {
        shard_blocks: gossipsub::IdentTopic::new(format!("centichain-shard-{}-blocks", shard_id)),
        shard_txs: gossipsub::IdentTopic::new(format!("centichain-shard-{}-txs", shard_id)),
        receipts: gossipsub::IdentTopic::new("centichain-receipts"),
        vdf_proofs: gossipsub::IdentTopic::new("centichain-vdf-proofs"),
        topology: gossipsub::IdentTopic::new("centichain-topology"),
        node_status: gossipsub::IdentTopic::new("centichain-node-status"),
    };

    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topics.shard_blocks)?;
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topics.shard_txs)?;
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topics.receipts)?;
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topics.vdf_proofs)?;
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topics.topology)?;
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topics.node_status)?;

    Ok(topics)
}

/// Connects to relay servers
fn connect_to_relays(
    swarm: &mut libp2p::Swarm<CentichainBehaviour>,
    relay_addrs: &[String],
    local_peer_id: &PeerId,
    consensus: &Arc<Mutex<Consensus>>,
    relay_connected: &Arc<AtomicBool>,
    app_handle: &AppHandle,
) -> Option<PeerId> {
    let mut relay_connected_count = 0;
    let mut relay_peer_id_opt = None;

    for relay_str in relay_addrs {
        if let Ok(relay_addr_parsed) = relay_str.parse::<libp2p::Multiaddr>() {
            match swarm.dial(relay_addr_parsed.clone()) {
                Ok(_) => {
                    log::info!("Dialing relay: {}", relay_addr_parsed);
                    if let Err(e) =
                        swarm.listen_on(relay_addr_parsed.clone().with(Protocol::P2pCircuit))
                    {
                        log::error!("Failed to listen on relay circuit {}: {}", relay_str, e);
                    } else {
                        log::info!(
                            "Listening on relay circuit {} for incoming P2P connections.",
                            relay_str
                        );
                        let external_addr = relay_addr_parsed
                            .clone()
                            .with(Protocol::P2pCircuit)
                            .with(Protocol::P2p(*local_peer_id));
                        log::info!("Announcing external address: {}", external_addr);
                        swarm.add_external_address(external_addr);
                        relay_connected_count += 1;

                        if relay_peer_id_opt.is_none() {
                            relay_peer_id_opt = relay_addr_parsed.iter().find_map(|p| match p {
                                Protocol::P2p(id) => Some(id),
                                _ => None,
                            });

                            if let Some(rid) = relay_peer_id_opt {
                                log::info!("P2P Init: Relay PeerID identified: {}", rid);
                                consensus.lock().unwrap().nodes.remove(&rid.to_string());
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to dial relay {}: {}", relay_str, e);
                }
            }
        } else {
            log::error!("Invalid relay address: {}", relay_str);
        }
    }

    if relay_connected_count == 0 {
        let _ = app_handle.emit("relay-status", "disconnected");
        log::warn!("Warning: Failed to connect to any relay. Operating in restricted mode.");
    } else {
        let _ = app_handle.emit(
            "relay-status",
            format!("Connected ({} relays)", relay_connected_count),
        );
        relay_connected.store(true, Ordering::Relaxed);
        if let Some(rid) = relay_peer_id_opt {
            consensus.lock().unwrap().nodes.remove(&rid.to_string());
        }
    }

    relay_peer_id_opt
}

/// Handles startup state transitions
fn handle_startup_state(
    startup_state: &mut NodeStartupState,
    config: &StartupConfig,
    relay_connected: &Arc<AtomicBool>,
    swarm: &mut libp2p::Swarm<CentichainBehaviour>,
    relay_peer_id_opt: Option<PeerId>,
    app_handle: &AppHandle,
) {
    match startup_state {
        NodeStartupState::ConnectingToRelay { start_time } => {
            if relay_connected.load(Ordering::Relaxed) {
                log::info!("Startup: Relay Connected. Switching to Discovery Phase.");
                let _ = app_handle.emit("node-status", "Searching for Network...");
                *startup_state = NodeStartupState::new_discovering();

                let _ = swarm.behaviour_mut().kad.bootstrap();
                if let Some(rid) = relay_peer_id_opt {
                    let _ = swarm.behaviour_mut().kad.get_closest_peers(rid);
                }
            } else if start_time.elapsed() >= config.relay_timeout {
                log::error!("Startup: Relay Connection Timed Out!");
                let _ = app_handle.emit("node-status", "Error: Relay Not Found");
                let _ = app_handle.emit("relay-status", "Connection Failed");
                *startup_state = NodeStartupState::RelayConnectionFailed;
            } else {
                let _ = app_handle.emit("node-status", "Connecting to Relay...");
            }
        }
        NodeStartupState::RelayConnectionFailed => {
            let _ = app_handle.emit("node-status", "Error: Relay Not Found");
            let _ = app_handle.emit("relay-status", "Connection Failed");
        }
        NodeStartupState::DiscoveringPeers { start_time } => {
            let _ = app_handle.emit("node-status", "Searching for Network...");
            if start_time.elapsed() >= config.discovery_duration {
                log::info!("Startup: Discovery Phase Complete. Entering Normal Operation.");
                *startup_state = NodeStartupState::Running;
            }
        }
        NodeStartupState::Running => {
            // Normal operation
        }
    }
}

/// Handles P2P commands
fn handle_command(
    cmd: P2PCommand,
    swarm: &mut libp2p::Swarm<CentichainBehaviour>,
    consensus: &Arc<Mutex<Consensus>>,
    local_peer_id: &PeerId,
    relay_peer_id_opt: Option<PeerId>,
    topics: &GossipTopics,
) {
    match cmd {
        P2PCommand::SyncWithNetwork => {
            log::info!("P2P: Received Sync Command. Contacting peers...");
            let connected_peers: Vec<PeerId> = swarm.connected_peers().cloned().collect();

            if let Some(relay_id) = relay_peer_id_opt {
                let _ = swarm.behaviour_mut().kad.get_closest_peers(relay_id);
            }

            if connected_peers.is_empty() {
                log::warn!("P2P Sync: No direct peers. Querying DHT...");
            } else {
                for peer in connected_peers {
                    if Some(peer) == relay_peer_id_opt {
                        continue;
                    }
                    log::info!("P2P Sync: Sending GetHeight request to peer {}", peer);
                    swarm
                        .behaviour_mut()
                        .sync
                        .send_request(&peer, SyncRequest::GetHeight);
                }
            }
        }
        P2PCommand::BroadcastMiningStatus { mining_active } => {
            log::info!("P2P: Broadcasting mining status change: {}", mining_active);

            {
                let mut c = consensus.lock().unwrap();
                c.set_peer_mining_status(&local_peer_id.to_string(), mining_active);
            }

            let status_update =
                crate::chain::NodeStatusUpdate::new(local_peer_id.to_string(), mining_active);

            if let Ok(data) = serde_json::to_vec(&status_update) {
                let _ = swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(topics.node_status.clone(), data);
                log::info!("P2P: Mining status broadcast complete");
            }
        }
    }
}

/// Handles periodic sync and discovery
fn handle_periodic_sync(
    swarm: &mut libp2p::Swarm<CentichainBehaviour>,
    app_handle: &AppHandle,
    startup_state: &NodeStartupState,
    is_synced: &Arc<AtomicBool>,
    local_peer_id: &PeerId,
    relay_peer_id_opt: Option<PeerId>,
) {
    // Emit DHT info for UI
    let mut dht_peers = Vec::new();
    for bucket in swarm.behaviour_mut().kad.kbuckets() {
        for entry in bucket.iter() {
            dht_peers.push(entry.node.key.preimage().to_string());
        }
    }
    let _ = app_handle.emit("dht-peers-update", dht_peers);

    if startup_state.is_running() && !is_synced.load(Ordering::Relaxed) {
        log::info!("P2P Loop: Expanding discovery. Searching DHT for neighbors...");
        swarm.behaviour_mut().kad.get_closest_peers(*local_peer_id);

        if let Some(rid) = relay_peer_id_opt {
            swarm.behaviour_mut().kad.get_closest_peers(rid);
        }

        let peers: Vec<PeerId> = swarm.connected_peers().cloned().collect();
        for peer in peers {
            if Some(peer) == relay_peer_id_opt {
                continue;
            }
            log::info!("P2P Loop: Timed Sync Request to {}", peer);
            let _ = app_handle.emit("sync-status", format!("Requesting height from {}", peer));
            swarm
                .behaviour_mut()
                .sync
                .send_request(&peer, SyncRequest::GetHeight);
        }
    }
}

/// Broadcasts topology update
fn broadcast_topology(
    swarm: &mut libp2p::Swarm<CentichainBehaviour>,
    local_peer_id: &PeerId,
    network_graph: &mut HashMap<String, Vec<String>>,
    topics: &GossipTopics,
    app_handle: &AppHandle,
) {
    let connected_peers: Vec<String> = swarm.connected_peers().map(|p| p.to_string()).collect();

    let update = TopologyUpdate::new(local_peer_id.to_string(), connected_peers.clone());

    network_graph.insert(local_peer_id.to_string(), connected_peers);
    let _ = app_handle.emit("network-topology-update", network_graph.clone());

    match serde_json::to_vec(&update) {
        Ok(json) => {
            if let Err(e) = swarm
                .behaviour_mut()
                .gossipsub
                .publish(topics.topology.clone(), json)
            {
                log::error!("Gossip topology publish error: {:?}", e);
            }
        }
        Err(e) => log::error!("Failed to serialize topology update: {:?}", e),
    }
}

/// Updates peer and validator counts
fn update_peer_counts(
    swarm: &libp2p::Swarm<CentichainBehaviour>,
    peer_count: &Arc<AtomicUsize>,
    validator_count: &Arc<AtomicUsize>,
    consensus: &Arc<Mutex<Consensus>>,
    relay_peer_id_opt: Option<PeerId>,
    startup_state: &NodeStartupState,
    app_handle: &AppHandle,
) {
    let total_peers = swarm.network_info().num_peers();
    let relay_is_conn = relay_peer_id_opt
        .map(|rid| swarm.is_connected(&rid))
        .unwrap_or(false);
    let valid_peers = if relay_is_conn {
        total_peers.saturating_sub(1)
    } else {
        total_peers
    };

    peer_count.store(valid_peers, Ordering::Relaxed);
    let _ = app_handle.emit("peer-count", valid_peers);

    match startup_state {
        NodeStartupState::ConnectingToRelay { .. } => {
            let _ = app_handle.emit("node-status", "Connecting to Relay...");
        }
        NodeStartupState::RelayConnectionFailed => {
            let _ = app_handle.emit("node-status", "Error: Relay Not Found");
        }
        NodeStartupState::DiscoveringPeers { .. } => {
            let _ = app_handle.emit("node-status", "Searching for Network...");
        }
        NodeStartupState::Running => {}
    }

    let v_count = {
        let c = consensus.lock().unwrap();
        c.nodes.len()
    };
    validator_count.store(v_count, Ordering::Relaxed);
    let _ = app_handle.emit("validator-count", v_count);
}

/// Handles swarm events
fn handle_swarm_event<THandlerErr: std::error::Error>(
    event: SwarmEvent<CentichainBehaviourEvent, THandlerErr>,
    swarm: &mut libp2p::Swarm<CentichainBehaviour>,
    app_handle: &AppHandle,
    storage: &Arc<Storage>,
    mempool: &Arc<Mempool>,
    consensus: &Arc<Mutex<Consensus>>,
    chain_index: &Arc<AtomicU64>,
    is_synced: &Arc<AtomicBool>,
    peer_count: &Arc<AtomicUsize>,
    relay_addrs: &[String],
    relay_peer_id_opt: &mut Option<PeerId>,
    relay_connected: &Arc<AtomicBool>,
    node_type: &Arc<Mutex<crate::NodeType>>,
    topics: &GossipTopics,
    network_graph: &mut HashMap<String, Vec<String>>,
) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            log::info!("Local node is listening on {:?}", address);
        }

        SwarmEvent::Behaviour(CentichainBehaviourEvent::Identify(
            libp2p::identify::Event::Received { peer_id, info },
        )) => {
            log::info!(
                "Identify: Connected to {} ({:?})",
                peer_id,
                info.agent_version
            );

            for addr in &info.listen_addrs {
                let is_relay_addr = relay_addrs.iter().any(|r| addr.to_string().contains(r));
                if !is_relay_addr {
                    swarm
                        .behaviour_mut()
                        .kad
                        .add_address(&peer_id, addr.clone());
                }
            }

            if relay_peer_id_opt.is_none() {
                if info
                    .listen_addrs
                    .iter()
                    .any(|a| relay_addrs.iter().any(|r| a.to_string().contains(r)))
                {
                    log::info!("Relay Identified via address match: {}", peer_id);
                    *relay_peer_id_opt = Some(peer_id);
                    let _ = app_handle.emit("relay-info", peer_id.to_string());
                }
            }
        }

        SwarmEvent::Behaviour(CentichainBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
            for (peer_id, _multiaddr) in list {
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                let _ = swarm
                    .behaviour_mut()
                    .sync
                    .send_request(&peer_id, SyncRequest::GetHeight);
            }
            peer_count.store(swarm.network_info().num_peers(), Ordering::Relaxed);
        }

        SwarmEvent::Behaviour(CentichainBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
            for (peer_id, _multiaddr) in list {
                swarm
                    .behaviour_mut()
                    .gossipsub
                    .remove_explicit_peer(&peer_id);
            }
            peer_count.store(swarm.network_info().num_peers(), Ordering::Relaxed);
        }

        SwarmEvent::Behaviour(CentichainBehaviourEvent::Gossipsub(gossipsub::Event::Message {
            propagation_source: peer_id,
            message,
            ..
        })) => {
            handle_gossip_message(
                &message,
                peer_id,
                storage,
                mempool,
                consensus,
                chain_index,
                topics,
                network_graph,
                app_handle,
            );
        }

        SwarmEvent::Behaviour(CentichainBehaviourEvent::Sync(
            libp2p::request_response::Event::Message { peer, message, .. },
        )) => {
            handle_sync_message(
                message,
                peer,
                swarm,
                storage,
                mempool,
                consensus,
                chain_index,
                is_synced,
                node_type,
                *relay_peer_id_opt,
                app_handle,
            );
        }

        SwarmEvent::ConnectionEstablished {
            peer_id, endpoint, ..
        } => {
            if endpoint.is_dialer() {
                let remote_addr = endpoint.get_remote_address().to_string();
                if relay_addrs.iter().any(|r| remote_addr.contains(r)) {
                    log::info!("Connection established with Relay: {}", peer_id);
                    *relay_peer_id_opt = Some(peer_id);
                    let _ = app_handle.emit("relay-status", "connected");
                    let _ = app_handle.emit("relay-info", peer_id.to_string());
                    relay_connected.store(true, Ordering::Relaxed);
                    consensus.lock().unwrap().nodes.remove(&peer_id.to_string());
                } else {
                    log::info!("Connection established with Peer: {}", peer_id);
                    consensus.lock().unwrap().register_node(peer_id.to_string());
                }
            } else {
                consensus.lock().unwrap().register_node(peer_id.to_string());
            }

            let total_peers = swarm.network_info().num_peers();
            let relay_is_conn = relay_peer_id_opt
                .map(|rid| swarm.is_connected(&rid))
                .unwrap_or(false);
            let valid_peers = if relay_is_conn {
                total_peers.saturating_sub(1)
            } else {
                total_peers
            };
            peer_count.store(valid_peers, Ordering::Relaxed);
        }

        SwarmEvent::ConnectionClosed {
            peer_id, endpoint, ..
        } => {
            let remote_addr = endpoint.get_remote_address().to_string();
            if relay_addrs.iter().any(|r| remote_addr.contains(r)) {
                log::warn!("Relay connection closed: {}", peer_id);
                let _ = app_handle.emit("relay-status", "disconnected");
                relay_connected.store(false, Ordering::Relaxed);
            }

            let total_peers = swarm.network_info().num_peers();
            let relay_is_conn = relay_peer_id_opt
                .map(|rid| swarm.is_connected(&rid))
                .unwrap_or(false);
            let valid_peers = if relay_is_conn {
                total_peers.saturating_sub(1)
            } else {
                total_peers
            };
            peer_count.store(valid_peers, Ordering::Relaxed);
        }

        SwarmEvent::Behaviour(CentichainBehaviourEvent::RelayClient(
            relay::client::Event::ReservationReqAccepted { .. },
        )) => {
            log::info!("Relay reservation accepted! Visible to network.");
            let _ = app_handle.emit("relay-status", "active");
        }

        SwarmEvent::Behaviour(CentichainBehaviourEvent::RelayClient(
            relay::client::Event::ReservationReqFailed { error, .. },
        )) => {
            log::error!("Relay reservation failed: {:?}", error);
            let _ = app_handle.emit("relay-status", "reservation-failed");
        }

        SwarmEvent::Behaviour(CentichainBehaviourEvent::Kad(
            kad::Event::OutboundQueryProgressed { result, .. },
        )) => {
            if let kad::QueryResult::GetClosestPeers(Ok(ok)) = result {
                for peer in ok.peers {
                    if Some(peer) != *relay_peer_id_opt && !swarm.is_connected(&peer) {
                        let _ = swarm.dial(peer);
                    }
                }
            }
        }

        SwarmEvent::Behaviour(CentichainBehaviourEvent::Kad(kad::Event::RoutingUpdated {
            peer,
            ..
        })) => {
            if Some(peer) != *relay_peer_id_opt && !swarm.is_connected(&peer) {
                let dial_opts = libp2p::swarm::dial_opts::DialOpts::peer_id(peer)
                    .condition(libp2p::swarm::dial_opts::PeerCondition::Disconnected)
                    .build();
                let _ = swarm.dial(dial_opts);
            }

            let total_peers = swarm.network_info().num_peers();
            let relay_is_conn = relay_peer_id_opt
                .map(|rid| swarm.is_connected(&rid))
                .unwrap_or(false);
            let valid_peers = if relay_is_conn {
                total_peers.saturating_sub(1)
            } else {
                total_peers
            };
            peer_count.store(valid_peers, Ordering::Relaxed);
        }

        _ => {}
    }
}

/// Handles gossipsub messages
fn handle_gossip_message(
    message: &gossipsub::Message,
    peer_id: PeerId,
    storage: &Arc<Storage>,
    mempool: &Arc<Mempool>,
    consensus: &Arc<Mutex<Consensus>>,
    chain_index: &Arc<AtomicU64>,
    topics: &GossipTopics,
    network_graph: &mut HashMap<String, Vec<String>>,
    app_handle: &AppHandle,
) {
    if message.topic.as_str() == topics.shard_blocks.hash().as_str() {
        if let Ok(block) = serde_json::from_slice::<Block>(&message.data) {
            log::info!("Received Gossip Block #{} from {}", block.index, peer_id);
            match ingest_block(storage, mempool, consensus, &block, false) {
                BlockAcceptResult::Accepted => {
                    chain_index.store(block.index, Ordering::Relaxed);
                    let _ = app_handle.emit("new-block", block);
                }
                BlockAcceptResult::Duplicate => {}
                BlockAcceptResult::NeedsSync { missing_from } => {
                    log::info!(
                        "Block #{} needs sync from height {}",
                        block.index, missing_from
                    );
                }
                BlockAcceptResult::Rejected(reason) => {
                    log::warn!("Rejected gossip block #{}: {}", block.index, reason);
                }
            }
        }
    } else if message.topic.as_str() == topics.shard_txs.hash().as_str() {
        if let Ok(tx) = serde_json::from_slice::<Transaction>(&message.data) {
            if let Err(e) = mempool.add_transaction(tx.clone()) {
                log::debug!("Rejected gossip tx {}: {}", tx.id, e);
            } else {
                let _ = app_handle.emit("new-transaction", tx);
            }
        }
    } else if message.topic.as_str() == topics.vdf_proofs.hash().as_str() {
        if let Ok(msg) = serde_json::from_slice::<crate::chain::VdfProofMessage>(&message.data) {
            log::info!("Received VDF Proof from {}", msg.peer_id);
            let mut c = consensus.lock().unwrap();
            if c.verify_peer(msg.peer_id.clone(), msg.proof) {
                log::info!(
                    "Verified peer {} via VDF! Trust Score set to 1.0",
                    msg.peer_id
                );
                c.persist_to_storage(storage);
                let _ = app_handle.emit("peer-update", msg.peer_id);
            } else {
                log::warn!("Invalid VDF Proof from {}", msg.peer_id);
            }
        }
    } else if message.topic.as_str() == topics.topology.hash().as_str() {
        if let Ok(msg) = serde_json::from_slice::<TopologyUpdate>(&message.data) {
            network_graph.insert(msg.source, msg.connections);
            let _ = app_handle.emit("network-topology-update", network_graph.clone());
        }
    } else if message.topic.as_str() == topics.node_status.hash().as_str() {
        if let Ok(status_update) =
            serde_json::from_slice::<crate::chain::NodeStatusUpdate>(&message.data)
        {
            log::info!(
                "P2P: Received mining status update from {}: mining_active={}",
                status_update.peer_id,
                status_update.mining_active
            );

            let mut c = consensus.lock().unwrap();
            c.set_peer_mining_status(&status_update.peer_id, status_update.mining_active);

            let _ = app_handle.emit("peer-mining-status", &status_update);
        }
    }
}

/// Handles sync protocol messages
fn handle_sync_message(
    message: libp2p::request_response::Message<SyncRequest, SyncResponse>,
    peer: PeerId,
    swarm: &mut libp2p::Swarm<CentichainBehaviour>,
    storage: &Arc<Storage>,
    mempool: &Arc<Mempool>,
    consensus: &Arc<Mutex<Consensus>>,
    chain_index: &Arc<AtomicU64>,
    is_synced: &Arc<AtomicBool>,
    node_type: &Arc<Mutex<crate::NodeType>>,
    _relay_peer_id_opt: Option<PeerId>,
    app_handle: &AppHandle,
) {
    match message {
        libp2p::request_response::Message::Request {
            request, channel, ..
        } => match request {
            SyncRequest::GetHeight => {
                let height = storage.get_latest_index().unwrap_or(0);
                log::info!(
                    "P2P Sync: Responding to GetHeight from {} with {}",
                    peer,
                    height
                );
                let _ = swarm
                    .behaviour_mut()
                    .sync
                    .send_response(channel, SyncResponse::Height(height));
            }
            SyncRequest::GetBlock(index) => {
                let block_opt = storage.get_block(index).unwrap_or(None);
                let _ = swarm
                    .behaviour_mut()
                    .sync
                    .send_response(channel, SyncResponse::Block(block_opt));
            }
            SyncRequest::GetBlocksRange(start, end) => {
                let mut blocks = Vec::new();
                for i in start..=end {
                    if let Ok(Some(b)) = storage.get_block(i) {
                        blocks.push(b);
                    } else {
                        break;
                    }
                }
                let _ = swarm
                    .behaviour_mut()
                    .sync
                    .send_response(channel, SyncResponse::BlocksBatch(blocks));
            }
            SyncRequest::GetMempool => {
                let txs = mempool.get_pending_transactions();
                let _ = swarm
                    .behaviour_mut()
                    .sync
                    .send_response(channel, SyncResponse::Mempool(txs));
            }
            SyncRequest::GetHeaders(start, end) => {
                let mut headers = Vec::new();
                for i in start..=end {
                    if let Ok(Some(b)) = storage.get_block(i) {
                        headers.push(crate::chain::Header::from_block(&b));
                    } else {
                        break;
                    }
                }
                let _ = swarm
                    .behaviour_mut()
                    .sync
                    .send_response(channel, SyncResponse::HeadersBatch(headers));
            }
        },
        libp2p::request_response::Message::Response { response, .. } => match response {
            SyncResponse::Height(remote_height) => {
                let local_height = chain_index.load(Ordering::Relaxed);
                let total_blocks = storage.get_total_blocks().unwrap_or(0);
                log::info!(
                    "P2P Sync: Remote Height {}, Local Height {}, Total Blocks {}",
                    remote_height,
                    local_height,
                    total_blocks
                );

                let start = if total_blocks == 0 {
                    0
                } else {
                    local_height + 1
                };

                if remote_height >= start {
                    let end = (start + 100).min(remote_height);
                    let msg = format!("Batch Syncing {}..{}", start, end);
                    log::info!("P2P Sync: {}", msg);
                    let _ = app_handle.emit("node-status", msg);
                    let _ = app_handle.emit(
                        "sync-status",
                        serde_json::json!({
                            "state": "syncing",
                            "current": start,
                            "target": end,
                            "peer": peer.to_string()
                        })
                        .to_string(),
                    );
                    swarm
                        .behaviour_mut()
                        .sync
                        .send_request(&peer, SyncRequest::GetBlocksRange(start, end));
                } else if !is_synced.load(Ordering::Relaxed) {
                    if total_blocks > 0 {
                        log::info!(
                            "P2P Sync: Local chain detected (Height {}). Remote is {}. Marked as Synced.",
                            local_height,
                            remote_height
                        );
                        is_synced.store(true, Ordering::Relaxed);
                        let _ = app_handle.emit("node-status", "Active");
                    } else if remote_height > 0 {
                        log::info!(
                            "P2P Sync: Local is empty, Remote is at {}. requesting genesis...",
                            remote_height
                        );
                        swarm
                            .behaviour_mut()
                            .sync
                            .send_request(&peer, SyncRequest::GetBlocksRange(0, 50));
                    } else {
                        log::warn!(
                            "P2P Sync: Both Local and Remote are empty (Genesis pending). Waiting..."
                        );
                    }
                }
            }
            SyncResponse::BlocksBatch(blocks) => {
                let mut last_idx = 0;
                for block in blocks {
                    last_idx = block.index;
                    log::info!("P2P Sync: Batch Received Block #{}", block.index);
                    match ingest_block(storage, mempool, consensus, &block, false) {
                        BlockAcceptResult::Accepted => {
                            chain_index.store(block.index, Ordering::Relaxed);
                            let _ = app_handle.emit("new-block", block);
                        }
                        BlockAcceptResult::Duplicate => {}
                        BlockAcceptResult::NeedsSync { .. } => {
                            log::warn!("Sync batch out of order at block #{}", block.index);
                        }
                        BlockAcceptResult::Rejected(reason) => {
                            log::warn!("Sync rejected block #{}: {}", block.index, reason);
                        }
                    }
                }
                log::info!(
                    "P2P Sync: Batch processed up to {}. Checking height...",
                    last_idx
                );
                swarm
                    .behaviour_mut()
                    .sync
                    .send_request(&peer, SyncRequest::GetHeight);
            }
            SyncResponse::Block(Some(block)) => {
                log::info!("P2P Sync: Received Block #{}", block.index);
                match ingest_block(storage, mempool, consensus, &block, false) {
                    BlockAcceptResult::Accepted => {
                        chain_index.store(block.index, Ordering::Relaxed);
                        let _ = app_handle.emit("new-block", block.clone());
                        let _ = app_handle
                            .emit("node-status", format!("Synced Block #{}", block.index));

                        let nt = {
                            let guard = node_type.lock().unwrap();
                            guard.clone()
                        };
                        if nt == crate::NodeType::Pruned {
                            let _ = storage.prune_history(2000);
                        }

                        if block.index % 50 == 0 {
                            swarm
                                .behaviour_mut()
                                .sync
                                .send_request(&peer, SyncRequest::GetHeight);
                        }
                    }
                    BlockAcceptResult::Duplicate => {}
                    BlockAcceptResult::NeedsSync { missing_from } => {
                        log::info!("Sync needs blocks from {}", missing_from);
                    }
                    BlockAcceptResult::Rejected(reason) => {
                        log::warn!("Sync rejected block #{}: {}", block.index, reason);
                    }
                }
            }
            SyncResponse::Mempool(txs) => {
                for tx in txs {
                    let _ = mempool.add_transaction(tx);
                }
            }
            _ => {}
        },
    }
}
