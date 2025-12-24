use futures::StreamExt;
use libp2p::{
    gossipsub, identity, kad, mdns, noise,
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
pub struct AntigravityBehaviour {
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

use tauri::{AppHandle, Emitter};

use crate::chain::{Block, SyncRequest, SyncResponse, Transaction};
use crate::consensus::Consensus;
use crate::mempool::Mempool;
use crate::storage::Storage;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

pub async fn start_p2p_node(
    app_handle: AppHandle,
    storage: Arc<Storage>,
    mempool: Arc<Mempool>,
    consensus: Arc<Mutex<Consensus>>, // Added Consensus
    is_synced: Arc<AtomicBool>,
    is_running: Arc<AtomicBool>,
    run_id: Arc<std::sync::atomic::AtomicU64>,
    peer_count: Arc<AtomicUsize>,
    chain_index: Arc<std::sync::atomic::AtomicU64>, // Renamed for clarity
    relay_addr: String,                             // Added for dynamic config
    my_run_id: u64,
    mut block_receiver: tokio::sync::mpsc::Receiver<Box<crate::chain::Block>>,
    mut tx_receiver: tokio::sync::mpsc::Receiver<crate::chain::Transaction>,
    node_type: Arc<Mutex<crate::NodeType>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let local_key = identity::Keypair::generate_ed25519();
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
                .set_protocol_names(vec![libp2p::StreamProtocol::new("/antigravity/kad/1.0.0")]);
            let store = kad::store::MemoryStore::new(key.public().to_peer_id());
            let kad = kad::Behaviour::with_config(key.public().to_peer_id(), store, kad_config);

            // MDNS
            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), PeerId::from(key.public()))?;

            // DCUtR (Direct Connection Upgrade through Relay)
            let dcutr = libp2p::dcutr::Behaviour::new(key.public().to_peer_id());

            // Identify
            let identify = libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
                "/antigravity/1.0.0".to_string(),
                key.public(),
            ));

            // Ping
            let ping = libp2p::ping::Behaviour::new(
                libp2p::ping::Config::new().with_interval(Duration::from_secs(5)),
            );

            // Request-Response (Sync)
            let sync = libp2p::request_response::cbor::Behaviour::new(
                [(
                    libp2p::StreamProtocol::new("/antigravity/sync/1"),
                    libp2p::request_response::ProtocolSupport::Full,
                )],
                libp2p::request_response::Config::default(),
            );

            Ok(AntigravityBehaviour {
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
    let topic_blocks = gossipsub::IdentTopic::new("antigravity-blocks");
    let topic_transactions = gossipsub::IdentTopic::new("antigravity-transactions");

    swarm.behaviour_mut().gossipsub.subscribe(&topic_blocks)?;
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topic_transactions)?;

    // Listen on all interfaces
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Attempt to dial the hardcoded relay
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
        Ok(_) => log::info!("Dialing relay..."),
        Err(e) => {
            log::error!("Failed to dial relay: {}", e);
            let _ = app_handle.emit("relay-status", "disconnected");
        }
    }

    let _ = app_handle.emit("relay-status", "Connecting...");
    let _ = app_handle.emit("node-status", "Connecting");

    // Event Loop
    let mut check_interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        if !is_running.load(Ordering::Relaxed) || run_id.load(Ordering::Relaxed) != my_run_id {
            log::info!("P2P Node stopping...");
            let _ = app_handle.emit("relay-status", "Disconnected");
            break;
        }

        tokio::select! {
            _ = check_interval.tick() => {
                 let count = swarm.network_info().num_peers();
                 peer_count.store(count, Ordering::Relaxed);
                 let _ = app_handle.emit("peer-count", count);
            }
            Some(block) = block_receiver.recv() => {
                 log::info!("Broadcasting mined block index: {}", block.index);
                 let json = serde_json::to_vec(&*block).unwrap();
                 if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic_blocks.clone(), json) {
                     log::error!("Gossip block publish error: {:?}", e);
                 }
            }
            Some(tx) = tx_receiver.recv() => {
                log::info!("Broadcasting local transaction: {}", tx.id);
                let json = serde_json::to_vec(&tx).unwrap();
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic_transactions.clone(), json) {
                    log::error!("Gossip tx publish error: {:?}", e);
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    log::info!("Local node is listening on {:?}", address);
                    let mut c = consensus.lock().unwrap();
                    if let Some(id) = c.local_peer_id.clone() {
                        if let Some(node) = c.nodes.get_mut(&id) {
                            let addr_str = address.to_string();
                            if !node.addresses.contains(&addr_str) {
                                node.addresses.push(addr_str);
                            }
                        }
                    }
                }
                SwarmEvent::Behaviour(AntigravityBehaviourEvent::Identify(libp2p::identify::Event::Received {
                    peer_id,
                    info,
                    ..
                })) => {
                    log::info!("Identified peer {:?} with addresses {:?}", peer_id, info.listen_addrs);
                    let mut c = consensus.lock().unwrap();
                    let pid_str = peer_id.to_string();
                    let node = c.nodes.entry(pid_str.clone()).or_insert_with(|| crate::consensus::NodeState::new(pid_str));
                    for addr in info.listen_addrs {
                        let addr_str = addr.to_string();
                        if !node.addresses.contains(&addr_str) {
                            node.addresses.push(addr_str);
                        }
                    }
                }
                SwarmEvent::Behaviour(AntigravityBehaviourEvent::Gossipsub(
                    gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message,
                        ..
                    },
                )) => {
                   let topic_hash = message.topic.clone();
                   let payload = message.data;
                   if topic_hash == topic_blocks.hash() {
                       log::info!("Received BLOCK from {:?}", peer_id);
                       if let Ok(block) = serde_json::from_slice::<Block>(&payload) {
                           if !block.is_vdf_valid() {
                               log::error!("VDF VALIDATION FAILED on block #{} from {:?}", block.index, peer_id);
                           } else {
                               log::info!("Block index: {}, txs: {}", block.index, block.transactions.len());
                               if storage.get_block(block.index).unwrap_or(None).is_none() {
                                   if let Err(e) = storage.save_block(&block) {
                                       log::error!("Failed to save block: {}", e);
                                   } else {
                                        // Pruning logic
                                        let nt = {
                                            let guard = node_type.lock().unwrap();
                                            guard.clone()
                                        };
                                        if nt == crate::NodeType::Pruned {
                                            let _ = storage.prune_history(2000);
                                        }

                                        let cur = chain_index.load(Ordering::Relaxed);
                                        if block.index > cur {
                                            chain_index.store(block.index, Ordering::Relaxed);
                                        }

                                        let tx_ids: Vec<String> = block.transactions.iter().map(|tx| tx.id.clone()).collect();
                                        mempool.remove_transactions(&tx_ids);
                                        let _ = app_handle.emit("new-block", block);
                                   }
                               }
                           }
                       }
                   } else if topic_hash == topic_transactions.hash() {
                       log::info!("Received TX from {:?}", peer_id);
                       if let Ok(tx) = serde_json::from_slice::<Transaction>(&payload) {
                           let sender_balance = storage.calculate_balance(&tx.sender).unwrap_or(0);
                           let required = tx.amount.saturating_add(crate::chain::calculate_fee(tx.amount));

                           if sender_balance < required {
                               log::warn!("Rejected tx {}: Insufficient funds", tx.id);
                           } else if let Err(e) = mempool.add_transaction(tx.clone()) {
                               log::warn!("Rejected tx: {}", e);
                           } else {
                                let _ = app_handle.emit("new-transaction", tx);
                           }
                       }
                   }
                }
                SwarmEvent::Behaviour(AntigravityBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        let _ = swarm.behaviour_mut().sync.send_request(&peer_id, SyncRequest::GetHeight);
                    }
                    peer_count.store(swarm.network_info().num_peers(), Ordering::Relaxed);
                }
                SwarmEvent::Behaviour(AntigravityBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                    peer_count.store(swarm.network_info().num_peers(), Ordering::Relaxed);
                }
                SwarmEvent::Behaviour(AntigravityBehaviourEvent::Sync(
                    libp2p::request_response::Event::Message { peer, message, .. },
                )) => match message {
                    libp2p::request_response::Message::Request { request, channel, .. } => {
                        match request {
                            SyncRequest::GetHeight => {
                                let height = storage.get_latest_index().unwrap_or(0);
                                let _ = swarm.behaviour_mut().sync.send_response(channel, SyncResponse::Height(height));
                            }
                            SyncRequest::GetBlock(index) => {
                                let block_opt = storage.get_block(index).unwrap_or(None);
                                let _ = swarm.behaviour_mut().sync.send_response(channel, SyncResponse::Block(block_opt));
                            }
                            SyncRequest::GetMempool => {
                                let txs = mempool.get_pending_transactions();
                                let _ = swarm.behaviour_mut().sync.send_response(channel, SyncResponse::Mempool(txs));
                            }
                        }
                    }
                    libp2p::request_response::Message::Response { response, .. } => {
                        match response {
                            SyncResponse::Height(h) => {
                                if h > 0 {
                                    let _ = app_handle.emit("node-status", "Syncing");
                                } else {
                                    is_synced.store(true, Ordering::Relaxed);
                                    let _ = app_handle.emit("node-status", "Active");
                                }
                                let _ = swarm.behaviour_mut().sync.send_request(&peer, SyncRequest::GetMempool);
                            }
                            SyncResponse::Block(b) => {
                                if let Some(block) = b {
                                     if storage.get_block(block.index).unwrap_or(None).is_none() {
                                         if let Ok(_) = storage.save_block(&block) {
                                             let nt = {
                                                 let guard = node_type.lock().unwrap();
                                                 guard.clone()
                                             };
                                             if nt == crate::NodeType::Pruned {
                                                 let _ = storage.prune_history(2000);
                                             }
                                             let _ = app_handle.emit("new-block", block);
                                         }
                                     }
                                }
                            }
                            SyncResponse::Mempool(txs) => {
                                for tx in txs {
                                    let _ = mempool.add_transaction(tx);
                                }
                            }
                        }
                    }
                },
                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                    let remote_addr = endpoint.get_remote_address().to_string();
                    if endpoint.is_dialer() && remote_addr.contains(&relay_addr) {
                        log::info!("Relay connection established with {}", peer_id);
                        let _ = app_handle.emit("relay-status", "connected");
                        let _ = app_handle.emit("relay-info", peer_id.to_string());
                        // Explicitly remove relay from consensus nodes if it was added via Identify/Mdns
                        consensus.lock().unwrap().nodes.remove(&peer_id.to_string());
                    } else {
                        // Only register non-relay nodes in consensus
                        consensus.lock().unwrap().register_node(peer_id.to_string());
                    }
                    peer_count.store(swarm.network_info().num_peers(), Ordering::Relaxed);
                }
                SwarmEvent::ConnectionClosed { peer_id, endpoint, .. } => {
                    let remote_addr = endpoint.get_remote_address().to_string();
                    if remote_addr.contains(&relay_addr) {
                        log::warn!("Relay connection closed: {}", peer_id);
                        let _ = app_handle.emit("relay-status", "disconnected");
                    }
                    peer_count.store(swarm.network_info().num_peers(), Ordering::Relaxed);
                }
                SwarmEvent::Behaviour(AntigravityBehaviourEvent::Kad(kad::Event::RoutingUpdated { .. })) => {
                     peer_count.store(swarm.network_info().num_peers(), Ordering::Relaxed);
                }
                _ => {}
            },
        }
    }
    Ok(())
}
