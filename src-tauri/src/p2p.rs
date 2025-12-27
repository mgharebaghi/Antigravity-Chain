use futures::StreamExt;
use libp2p::multiaddr::Protocol;
use libp2p::{
    gossipsub, identity, kad, mdns, noise, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, SwarmBuilder,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::io;

// Helper to create a message id for gossipsub
pub fn message_id_fn(message: &gossipsub::Message) -> gossipsub::MessageId {
    let mut s = DefaultHasher::new();
    message.data.hash(&mut s);
    gossipsub::MessageId::from(s.finish().to_string())
}

#[derive(NetworkBehaviour)]
pub struct CentichainBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub kad: kad::Behaviour<kad::store::MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
    pub relay_client: libp2p::relay::client::Behaviour,
    pub dcutr: libp2p::dcutr::Behaviour,
    pub identify: libp2p::identify::Behaviour,
    pub ping: libp2p::ping::Behaviour,
    pub sync: libp2p::request_response::cbor::Behaviour<
        crate::chain::SyncRequest,
        crate::chain::SyncResponse,
    >,
}

const SYNC_PROTOCOL: &str = "/centichain/sync/1.0.0";

use tauri::{AppHandle, Emitter};

use crate::chain::{Block, SyncRequest, SyncResponse, Transaction};
use crate::consensus::Consensus;
use crate::mempool::Mempool;
use crate::storage::Storage;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug)]
pub enum P2PCommand {
    SyncWithNetwork,
}

pub async fn start_p2p_node(
    app_handle: AppHandle,
    storage: Arc<Storage>,
    mempool: Arc<Mempool>,
    consensus: Arc<Mutex<Consensus>>,
    is_synced: Arc<AtomicBool>,
    is_running: Arc<AtomicBool>,
    run_id: Arc<std::sync::atomic::AtomicU64>,
    peer_count: Arc<AtomicUsize>,
    validator_count: Arc<AtomicUsize>,
    chain_index: Arc<std::sync::atomic::AtomicU64>,
    relay_addr: String,
    my_run_id: u64,
    mut block_receiver: tokio::sync::mpsc::Receiver<Box<crate::chain::Block>>,
    mut tx_receiver: tokio::sync::mpsc::Receiver<crate::chain::Transaction>,
    mut receipt_receiver: tokio::sync::mpsc::Receiver<crate::chain::Receipt>,
    mut vdf_receiver: tokio::sync::mpsc::Receiver<crate::chain::VdfProofMessage>, // New Arg
    node_type: Arc<Mutex<crate::NodeType>>,
    wallet_keypair: Option<identity::Keypair>,
    mut cmd_rx: tokio::sync::mpsc::Receiver<P2PCommand>,
) -> Result<(), Box<dyn std::error::Error>> {
    let local_key = wallet_keypair.unwrap_or_else(identity::Keypair::generate_ed25519);
    let local_peer_id = PeerId::from(local_key.public());
    log::info!("Local peer id: {:?}", local_peer_id);

    // Register self in consensus
    consensus
        .lock()
        .unwrap()
        .set_local_peer_id(local_peer_id.to_string());

    // Turn connection limits off for now
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

            // DCUtR (Direct Connection Upgrade through Relay)
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

    // Subscribe to topics
    // Dynamic Shard Subscription
    let shard_id = {
        let c = consensus.lock().unwrap();
        c.get_assigned_shard(&local_peer_id.to_string(), 0)
    };
    log::info!("P2P: Subscribing to Shard #{} topics", shard_id);

    let topic_shard_blocks =
        gossipsub::IdentTopic::new(format!("centichain-shard-{}-blocks", shard_id));
    let topic_shard_txs = gossipsub::IdentTopic::new(format!("centichain-shard-{}-txs", shard_id));
    let topic_receipts = gossipsub::IdentTopic::new("centichain-receipts");
    let topic_vdf_proofs = gossipsub::IdentTopic::new("centichain-vdf-proofs");
    // [NEW] Topology Gossip Topic
    let topic_topology = gossipsub::IdentTopic::new("centichain-topology");

    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topic_shard_blocks)?;
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topic_shard_txs)?;
    swarm.behaviour_mut().gossipsub.subscribe(&topic_receipts)?;
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topic_vdf_proofs)?;
    // [NEW] Subscribe
    swarm.behaviour_mut().gossipsub.subscribe(&topic_topology)?;

    // Listen on all interfaces
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Attempt to dial the configured relay
    let relay_addr_parsed: libp2p::Multiaddr = relay_addr.parse()?;

    // Bootnodes (Placeholder)
    let bootnodes: Vec<&str> = vec![];

    for peer in bootnodes {
        if let Ok(addr) = peer.parse::<libp2p::Multiaddr>() {
            if let Some(peer_id) = addr.iter().find_map(|p| match p {
                libp2p::multiaddr::Protocol::P2p(id) => Some(id),
                _ => None,
            }) {
                log::info!("Adding bootnode to DHT: {}", peer_id);
                swarm.behaviour_mut().kad.add_address(&peer_id, addr);
            }
        }
    }

    // Bootstrap DHT
    if let Err(e) = swarm.behaviour_mut().kad.bootstrap() {
        log::warn!(
            "Kademlia bootstrap failed (non-fatal if first node): {:?}",
            e
        );
    }

    match swarm.dial(relay_addr_parsed.clone()) {
        Ok(_) => {
            log::info!("Dialing relay: {}", relay_addr_parsed);
            // Reservation: This allows other nodes to reach us via the relay
            if let Err(e) = swarm.listen_on(relay_addr_parsed.clone().with(Protocol::P2pCircuit)) {
                log::error!("Failed to listen on relay circuit: {}", e);
            } else {
                log::info!("Listening on relay circuit for incoming P2P connections.");
                // CRITICAL: Announce our relayed address as an EXTERNAL address
                // so the Relay can tell others how to reach us.
                let external_addr = relay_addr_parsed
                    .clone()
                    .with(Protocol::P2pCircuit)
                    .with(Protocol::P2p(local_peer_id));
                log::info!("Announcing external address: {}", external_addr);
                swarm.add_external_address(external_addr);
            }

            // Relay is connected, but we do NOT add it to Kademlia DHT.
            // User Requirement: "Leader (Relay) should not be in DHT".
            // We rely on Identify/Mdns/Gossip for finding REAL peers.
        }
        Err(e) => {
            log::error!("Failed to dial relay: {}", e);
            let _ = app_handle.emit("relay-status", "disconnected");
        }
    }

    let _ = app_handle.emit("relay-status", "Connecting...");
    let _ = app_handle.emit("node-status", "Connecting");

    let mut relay_peer_id_opt = relay_addr_parsed.iter().find_map(|p| match p {
        Protocol::P2p(id) => Some(id),
        _ => None,
    });

    // [NEW] Network Graph State: PeerId -> Vec<PeerId> (Neighbors)
    let mut network_graph: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    // Event Loop
    let mut check_interval = tokio::time::interval(Duration::from_secs(1));
    let mut discovery_interval = tokio::time::interval(Duration::from_secs(15)); // New explicit discovery interval
    let mut topology_gossip_interval = tokio::time::interval(Duration::from_secs(30)); // [NEW] Broadcast topology

    loop {
        if !is_running.load(Ordering::Relaxed) || run_id.load(Ordering::Relaxed) != my_run_id {
            log::info!("P2P Node stopping...");
            let _ = app_handle.emit("relay-status", "Disconnected");
            break;
        }

        tokio::select! {
            // 1. Handle Commands from Lib (e.g. Start Sync)
            Some(cmd) = cmd_rx.recv() => {
                 match cmd {
                     P2PCommand::SyncWithNetwork => {
                         log::info!("P2P: Received Sync Command. Contacting peers...");
                         // Broadcast GetHeight to all connected peers AND closest DHT peers
                         let connected_peers: Vec<PeerId> = swarm.connected_peers().cloned().collect();
                         let targets = connected_peers.clone();

                         // Also try to reach out to peers in the DHT that we might not be fully connected to yet
                         if let Some(relay_id) = relay_peer_id_opt {
                             let _ = swarm.behaviour_mut().kad.get_closest_peers(relay_id);
                         }

                         if targets.is_empty() {
                             // Fallback: If no connected peers, try to start random sync with DHT peers (if any cached)
                             // This is a "Hail Mary" passed to the Kademlia event handler
                             log::warn!("P2P Sync: No direct peers. Querying DHT...");
                         } else {
                             for peer in targets {
                                 if Some(peer) == relay_peer_id_opt { continue; }
                                 log::info!("P2P Sync: Sending GetHeight request to peer {}", peer);
                                 swarm.behaviour_mut().sync.send_request(&peer, SyncRequest::GetHeight);
                             }
                         }
                     }
                 }
            }

            // Periodic Sync/Discovery Check
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                 // Emit DHT info for UI
                 let mut dht_peers = Vec::new();
                 for bucket in swarm.behaviour_mut().kad.kbuckets() {
                     for entry in bucket.iter() {
                         dht_peers.push(entry.node.key.preimage().to_string());
                     }
                 }
                 let _ = app_handle.emit("dht-peers-update", dht_peers);

                 if !is_synced.load(Ordering::Relaxed) {
                     // 1. Re-query DHT to find new peers - SEARCH FOR SELF to find nodes near us
                     log::info!("P2P Loop: Expanding discovery. Searching DHT for neighbors...");

                     // Query for self (standard Kademlia refresh)
                     swarm.behaviour_mut().kad.get_closest_peers(local_peer_id);

                     // Also query for the RELAY's neighbors (to find anyone else connected to it)
                     if let Some(rid) = relay_peer_id_opt {
                         swarm.behaviour_mut().kad.get_closest_peers(rid);
                     }

                     // 2. Ask existing peers for height
                     let peers: Vec<PeerId> = swarm.connected_peers().cloned().collect();
                     for peer in peers {
                         if Some(peer) == relay_peer_id_opt { continue; }
                         log::info!("P2P Loop: Timed Sync Request to {}", peer);
                         let _ = app_handle.emit("sync-status", format!("Requesting height from {}", peer));
                         swarm.behaviour_mut().sync.send_request(&peer, SyncRequest::GetHeight);
                     }
                 }
            }

            // Periodic "Random Walk" Discovery
            _ = discovery_interval.tick() => {
                 let connected = swarm.connected_peers().count();
                 if connected < 5 { // If we have few peers, aggressively look for more
                     log::info!("P2P Loop: Performing Random Walk for Discovery...");
                     let random_peer_id = PeerId::random();
                     swarm.behaviour_mut().kad.get_closest_peers(random_peer_id);
                 }
            }

            // [NEW] Topology Gossip Broadcast
            _ = topology_gossip_interval.tick() => {
                 let connected_peers: Vec<String> = swarm.connected_peers().map(|p| p.to_string()).collect();
                 // Create TopologyUpdate message
                 #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
                 struct TopologyUpdate {
                    source: String,
                    connections: Vec<String>,
                    timestamp: u64,
                 }

                 let update = TopologyUpdate {
                    source: local_peer_id.to_string(),
                    connections: connected_peers.clone(),
                    timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                 };

                 // Update Local Graph with Self
                 network_graph.insert(local_peer_id.to_string(), connected_peers);
                 // Emit local update immediately
                 let _ = app_handle.emit("network-topology-update", network_graph.clone());

                 match serde_json::to_vec(&update) {
                    Ok(json) => {
                         if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic_topology.clone(), json) {
                             log::error!("Gossip topology publish error: {:?}", e);
                         }
                    }
                    Err(e) => log::error!("Failed to serialize topology update: {:?}", e),
                 }
            }

            _ = check_interval.tick() => {
                 let total_peers = swarm.network_info().num_peers();
                 let relay_is_conn = if let Some(rid) = relay_peer_id_opt { swarm.is_connected(&rid) } else { false };
                 let valid_peers = if relay_is_conn { total_peers.saturating_sub(1) } else { total_peers };

                 peer_count.store(valid_peers, Ordering::Relaxed);
                 let _ = app_handle.emit("peer-count", valid_peers);

                 // CHECK: First Node Logic
                 // If we have 0 valid peers (active validators), we are the First Node.
                 // We should assume we are Synced and ready to Mine immediately.
                 if valid_peers == 0 && !is_synced.load(Ordering::Relaxed) {
                     log::info!("No peers detected. Assuming Role: FIRST NODE (Genesis). State -> Synced.");
                     is_synced.store(true, Ordering::Relaxed);
                     let _ = app_handle.emit("node-status", "Active (First Node)");
                 }

                 let v_count = {
                     let c = consensus.lock().unwrap();
                     c.nodes.len()
                 };
                 validator_count.store(v_count, Ordering::Relaxed);
                 let _ = app_handle.emit("validator-count", v_count);
            }

            Some(block) = block_receiver.recv() => {
                 log::info!("Broadcasting mined block index: {}", block.index);
                 let json = serde_json::to_vec(&*block).unwrap();
                 if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic_shard_blocks.clone(), json) {
                     log::error!("Gossip block publish error: {:?}", e);
                 }
            }

            Some(tx) = tx_receiver.recv() => {
                log::info!("Broadcasting local transaction: {}", tx.id);
                let json = serde_json::to_vec(&tx).unwrap();
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic_shard_txs.clone(), json) {
                    log::error!("Gossip tx publish error: {:?}", e);
                }
            }

            Some(receipt) = receipt_receiver.recv() => {
                log::info!("Broadcasting Cross-Shard Receipt: {}", receipt.original_tx_id);
                let json = serde_json::to_vec(&receipt).unwrap();
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic_receipts.clone(), json) {
                    log::error!("Gossip receipt publish error: {:?}", e);
                }
            }

            Some(vdf_msg) = vdf_receiver.recv() => {
                log::info!("Broadcasting VDF Proof for {}", vdf_msg.peer_id);
                let json = serde_json::to_vec(&vdf_msg).unwrap();
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic_vdf_proofs.clone(), json) {
                    log::error!("Gossip VDF publish error: {:?}", e);
                }
            }

            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    log::info!("Local node is listening on {:?}", address);
                }

                SwarmEvent::Behaviour(CentichainBehaviourEvent::Identify(libp2p::identify::Event::Received {
                    peer_id,
                    info,
                    ..
                })) => {
                    log::info!("Identified peer {:?} with version {:?}", peer_id, info.protocol_version);

                    // CRITICAL: Populate Kademlia DHT with the addresses we just learned.
                    // BUT ONLY IF IT IS NOT THE RELAY (Leader)
                    // Check if this peer matches our known relay peer ID
                    let is_relay = Some(peer_id) == relay_peer_id_opt
                        || info.listen_addrs.iter().any(|a| a.to_string().contains(&relay_addr));

                    if !is_relay {
                         for addr in &info.listen_addrs {
                             swarm.behaviour_mut().kad.add_address(&peer_id, addr.clone());
                         }
                    } else {
                        log::info!("Ignored Relay for DHT: {:?}", peer_id);
                    }

                    // Fallback Relay Detection
                    if relay_peer_id_opt.is_none() && info.listen_addrs.iter().any(|a| a.to_string().contains(&relay_addr)) {
                         log::info!("Relay Identified via listen address match: {}", peer_id);
                         relay_peer_id_opt = Some(peer_id);
                    }

                    // If this is the RELAY (detected now or before)
                    if Some(peer_id) == relay_peer_id_opt {
                             log::info!("P2P: Relay identified ({:?}). Bootstrapping DHT...", peer_id);
                             if let Err(e) = swarm.behaviour_mut().kad.bootstrap() {
                                 log::error!("Kademlia bootstrap error: {:?}", e);
                             }
                             // Safety: Ensure relay is removed from consensus
                             consensus.lock().unwrap().nodes.remove(&peer_id.to_string());
                    }

                    // PROACTIVE: If we aren't synced, ask this peer for their height immediately.
                    if !is_synced.load(Ordering::Relaxed) && Some(peer_id) != relay_peer_id_opt {
                         log::info!("P2P Sync: Identified NEW validator {}. Requesting height...", peer_id);
                         swarm.behaviour_mut().sync.send_request(&peer_id, SyncRequest::GetHeight);
                    }

                    // Only add to consensus if it is NOT the relay
                    if Some(peer_id) != relay_peer_id_opt {
                        let mut c = consensus.lock().unwrap();
                        let pid_str = peer_id.to_string();
                        let node = c.nodes.entry(pid_str.clone()).or_insert_with(|| crate::consensus::NodeState::new(pid_str));
                        node.addresses = info.listen_addrs.iter().map(|a| a.to_string()).collect();
                    }
                }

                SwarmEvent::Behaviour(CentichainBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        let _ = swarm.behaviour_mut().sync.send_request(&peer_id, SyncRequest::GetHeight);
                    }
                    peer_count.store(swarm.network_info().num_peers(), Ordering::Relaxed);
                }

                SwarmEvent::Behaviour(CentichainBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                    peer_count.store(swarm.network_info().num_peers(), Ordering::Relaxed);
                }

                SwarmEvent::Behaviour(CentichainBehaviourEvent::Gossipsub(
                    gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message,
                        ..
                    },
                )) => {
                    if message.topic.as_str() == topic_shard_blocks.hash().as_str() {
                         if let Ok(block) = serde_json::from_slice::<Block>(&message.data) {
                               log::info!("Received Gossip Block #{} from {}", block.index, peer_id);
                               if let Ok(_) = storage.save_block(&block) {
                                   chain_index.store(block.index, Ordering::Relaxed);
                                   let _ = app_handle.emit("new-block", block);
                               }
                         }
                    } else if message.topic.as_str() == topic_shard_txs.hash().as_str() {
                         if let Ok(tx) = serde_json::from_slice::<Transaction>(&message.data) {
                             let _ = mempool.add_transaction(tx.clone());
                             let _ = app_handle.emit("new-transaction", tx);
                         }
                    } else if message.topic.as_str() == topic_vdf_proofs.hash().as_str() {
                        if let Ok(msg) = serde_json::from_slice::<crate::chain::VdfProofMessage>(&message.data) {
                            log::info!("Received VDF Proof from {}", msg.peer_id);
                            let mut c = consensus.lock().unwrap();
                            if c.verify_peer(msg.peer_id.clone(), msg.proof) {
                                log::info!("Verified peer {} via VDF! Trust Score set to 1.0", msg.peer_id);
                                let _ = app_handle.emit("peer-update", msg.peer_id);
                            } else {
                                log::warn!("Invalid VDF Proof from {}", msg.peer_id);
                            }
                        }
                    } else if message.topic.as_str() == topic_topology.hash().as_str() {
                        // [NEW] Handle Topology Gossip
                         #[derive(serde::Serialize, serde::Deserialize, Debug)]
                         struct TopologyUpdate {
                            source: String,
                            connections: Vec<String>,
                            timestamp: u64,
                         }

                        if let Ok(msg) = serde_json::from_slice::<TopologyUpdate>(&message.data) {
                            // Update graph
                            network_graph.insert(msg.source, msg.connections);
                            // Emit raw graph to frontend
                            let _ = app_handle.emit("network-topology-update", network_graph.clone());
                        }
                    }
                }

                SwarmEvent::Behaviour(CentichainBehaviourEvent::Sync(
                    libp2p::request_response::Event::Message { peer, message, .. },
                )) => match message {
                    libp2p::request_response::Message::Request { request, channel, .. } => {
                        match request {
                            SyncRequest::GetHeight => {
                                let height = storage.get_latest_index().unwrap_or(0);
                                log::info!("P2P Sync: Responding to GetHeight from {} with {}", peer, height);
                                let _ = swarm.behaviour_mut().sync.send_response(channel, SyncResponse::Height(height));
                            }
                            SyncRequest::GetBlock(index) => {
                                let block_opt = storage.get_block(index).unwrap_or(None);
                                let _ = swarm.behaviour_mut().sync.send_response(channel, SyncResponse::Block(block_opt));
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
                                let _ = swarm.behaviour_mut().sync.send_response(channel, SyncResponse::BlocksBatch(blocks));
                            }
                            SyncRequest::GetMempool => {
                                let txs = mempool.get_pending_transactions();
                                let _ = swarm.behaviour_mut().sync.send_response(channel, SyncResponse::Mempool(txs));
                            }
                        }
                    }
                    libp2p::request_response::Message::Response { response, .. } => {
                        match response {
                            SyncResponse::Height(remote_height) => {
                                let local_height = chain_index.load(Ordering::Relaxed);
                                let total_blocks = storage.get_total_blocks().unwrap_or(0);
                                log::info!("P2P Sync: Remote Height {}, Local Height {}, Total Blocks {}", remote_height, local_height, total_blocks);

                                // Adjust start: If we have 0 blocks, we need block 0.
                                let start = if total_blocks == 0 { 0 } else { local_height + 1 };

                                if remote_height >= start {
                                    let end = (start + 100).min(remote_height);
                                    let msg = format!("Batch Syncing {}..{}", start, end);
                                    log::info!("P2P Sync: {}", msg);
                                    let _ = app_handle.emit("node-status", msg);
                                    let _ = app_handle.emit("sync-status", serde_json::json!({
                                        "state": "syncing",
                                        "current": start,
                                        "target": end,
                                        "peer": peer.to_string()
                                    }).to_string());
                                    swarm.behaviour_mut().sync.send_request(&peer, SyncRequest::GetBlocksRange(start, end));
                                } else {
                                    // If we are at the same height or ahead, AND we have at least one block, we are synced.
                                    if !is_synced.load(Ordering::Relaxed) {
                                        if total_blocks > 0 {
                                            log::info!("P2P Sync: Local chain detected (Height {}). Remote is {}. Marked as Synced.", local_height, remote_height);
                                            is_synced.store(true, Ordering::Relaxed);
                                            let _ = app_handle.emit("node-status", "Active");
                                        } else if remote_height > 0 {
                                            // Remote has blocks, we have 0. WE MUST SYNC.
                                             log::info!("P2P Sync: Local is empty, Remote is at {}. requesting genesis...", remote_height);
                                             swarm.behaviour_mut().sync.send_request(&peer, SyncRequest::GetBlocksRange(0, 50));
                                        } else {
                                           log::warn!("P2P Sync: Both Local and Remote are empty (Genesis pending). Waiting...");
                                        }
                                    }
                                }
                            }
                            SyncResponse::BlocksBatch(blocks) => {
                                let mut last_idx = 0;
                                for block in blocks {
                                    last_idx = block.index;
                                    log::info!("P2P Sync: Batch Received Block #{}", block.index);
                                    if storage.get_block(block.index).unwrap_or(None).is_none() {
                                        if let Ok(_) = storage.save_block(&block) {
                                            chain_index.store(block.index, Ordering::Relaxed);
                                            let _ = app_handle.emit("new-block", block);
                                        }
                                    }
                                }
                                // Request next batch or check height again
                                log::info!("P2P Sync: Batch processed up to {}. Checking height...", last_idx);
                                swarm.behaviour_mut().sync.send_request(&peer, SyncRequest::GetHeight);
                            }
                            SyncResponse::Block(Some(block)) => {
                                log::info!("P2P Sync: Received Block #{}", block.index);
                                 if storage.get_block(block.index).unwrap_or(None).is_none() {
                                    if let Ok(_) = storage.save_block(&block) {
                                        chain_index.store(block.index, Ordering::Relaxed);
                                        let _ = app_handle.emit("new-block", block.clone());
                                        let _ = app_handle.emit("node-status", format!("Synced Block #{}", block.index));

                                        // Pruning
                                        let nt = {
                                            let guard = node_type.lock().unwrap();
                                            guard.clone()
                                        };
                                        if nt == crate::NodeType::Pruned {
                                            let _ = storage.prune_history(2000);
                                        }

                                        if block.index % 50 == 0 {
                                             swarm.behaviour_mut().sync.send_request(&peer, SyncRequest::GetHeight);
                                        }
                                    }
                                }
                            }
                            SyncResponse::Mempool(txs) => {
                                for tx in txs {
                                    let _ = mempool.add_transaction(tx);
                                }
                            }
                            _ => {}
                        }
                    }
                },

                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                    let remote_addr = endpoint.get_remote_address().to_string();

                    // Allow broad match for relay (configured addr vs detected addr)
                    if endpoint.is_dialer() && remote_addr.contains(&relay_addr) {
                        log::info!("Relay connection established with {}. (Address Match)", peer_id);
                        relay_peer_id_opt = Some(peer_id); // Capture the Relay Peer ID

                        let _ = app_handle.emit("relay-status", "connected");
                        let _ = app_handle.emit("relay-info", peer_id.to_string());
                        consensus.lock().unwrap().nodes.remove(&peer_id.to_string());
                    } else {
                        consensus.lock().unwrap().register_node(peer_id.to_string());
                    }
                    // Update peer count (excluding relay)
                    let total_peers = swarm.network_info().num_peers();
                    let relay_is_conn = if let Some(rid) = relay_peer_id_opt { swarm.is_connected(&rid) } else { false };
                    let valid_peers = if relay_is_conn { total_peers.saturating_sub(1) } else { total_peers };
                    peer_count.store(valid_peers, Ordering::Relaxed);
                }

                SwarmEvent::ConnectionClosed { peer_id, endpoint, .. } => {
                    let remote_addr = endpoint.get_remote_address().to_string();
                    if remote_addr.contains(&relay_addr) {
                        log::warn!("Relay connection closed: {}", peer_id);
                        let _ = app_handle.emit("relay-status", "disconnected");
                    }
                    // Update peer count (excluding relay)
                    let total_peers = swarm.network_info().num_peers();
                    let relay_is_conn = if let Some(rid) = relay_peer_id_opt { swarm.is_connected(&rid) } else { false };
                    let valid_peers = if relay_is_conn { total_peers.saturating_sub(1) } else { total_peers };
                    peer_count.store(valid_peers, Ordering::Relaxed);
                }

                SwarmEvent::Behaviour(CentichainBehaviourEvent::RelayClient(relay::client::Event::ReservationReqAccepted { .. })) => {
                    log::info!("Relay reservation accepted! Visible to network.");
                    let _ = app_handle.emit("relay-status", "connected");
                }
                SwarmEvent::Behaviour(CentichainBehaviourEvent::RelayClient(relay::client::Event::ReservationReqFailed { error, .. })) => {
                    log::error!("Relay reservation failed: {:?}", error);
                    let _ = app_handle.emit("relay-status", "reservation-failed");
                }
                SwarmEvent::Behaviour(CentichainBehaviourEvent::Kad(kad::Event::OutboundQueryProgressed { result, .. })) => {
                    match result {
                        kad::QueryResult::GetClosestPeers(Ok(ok)) => {
                            for peer in ok.peers {
                                log::info!("Kademlia: Discovered neighbor {} via Query", peer);
                                if Some(peer) != relay_peer_id_opt && !swarm.is_connected(&peer) {
                                    log::info!("P2P: Actively dialing neighbor {}", peer);
                                    let _ = swarm.dial(peer);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                SwarmEvent::Behaviour(CentichainBehaviourEvent::Kad(kad::Event::RoutingUpdated { peer, .. })) => {
                     if Some(peer) != relay_peer_id_opt {
                         log::info!("Kademlia: Routing Table updated with peer {}", peer);
                         if !swarm.is_connected(&peer) {
                             log::info!("P2P: Dialing discovered peer {}", peer);
                             log::info!("P2P: Dialing discovered peer {}", peer);
                             let dial_opts = libp2p::swarm::dial_opts::DialOpts::peer_id(peer)
                                .condition(libp2p::swarm::dial_opts::PeerCondition::Disconnected)
                                .build();
                             let _ = swarm.dial(dial_opts);
                         }
                     }
                     // Update peer count (excluding relay)
                     let total_peers = swarm.network_info().num_peers();
                     let relay_is_conn = if let Some(rid) = relay_peer_id_opt { swarm.is_connected(&rid) } else { false };
                     let valid_peers = if relay_is_conn { total_peers.saturating_sub(1) } else { total_peers };
                     peer_count.store(valid_peers, Ordering::Relaxed);
                }
                _ => {}
            }
        }
    }
    Ok(())
}
