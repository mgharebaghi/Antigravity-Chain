//! # Network Behaviour Module
//!
//! Defines the libp2p network behaviour for Centichain nodes.
//! Combines multiple protocols: gossipsub, kademlia, mdns, relay, etc.

use libp2p::{gossipsub, kad, mdns, swarm::NetworkBehaviour};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Sync protocol identifier
pub const SYNC_PROTOCOL: &str = "/centichain/sync/1.0.0";

/// Helper to create a unique message id for gossipsub deduplication
pub fn message_id_fn(message: &gossipsub::Message) -> gossipsub::MessageId {
    let mut s = DefaultHasher::new();
    message.data.hash(&mut s);
    gossipsub::MessageId::from(s.finish().to_string())
}

/// Combined network behaviour for Centichain
///
/// This struct combines all the libp2p protocols we use:
/// - gossipsub: Pub/sub for blocks, transactions, and status updates
/// - kad: Kademlia DHT for peer discovery
/// - mdns: Local network peer discovery
/// - relay_client: NAT traversal via relay servers
/// - dcutr: Direct connection upgrade through relay
/// - identify: Protocol identification
/// - ping: Connection keepalive
/// - sync: Request-response for blockchain sync
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
