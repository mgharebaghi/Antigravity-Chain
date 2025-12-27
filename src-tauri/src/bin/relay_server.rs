use libp2p::{
    futures::StreamExt,
    identity, kad, noise, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, SwarmBuilder,
};
use std::error::Error;
use std::net::SocketAddr;
use std::time::Duration;

#[derive(NetworkBehaviour)]
struct RelayServerBehaviour {
    relay: relay::Behaviour,
    kad: kad::Behaviour<kad::store::MemoryStore>,
    identify: libp2p::identify::Behaviour,
    ping: libp2p::ping::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    flexi_logger::Logger::try_with_str("info")?.start()?;
    log::info!("Starting Centichain Relay Server (Bootstrap Node)...");

    // 1. Generate keys (In production, load these from file!)
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    log::info!("Relay Node ID: {:?}", local_peer_id);

    // 2. Configure Relay (Circuit Relay v2)
    let relay_config = relay::Config {
        max_reservations: 1024,
        max_circuits: 1024,
        reservation_duration: Duration::from_secs(60 * 60), // 1 Hour
        ..Default::default()
    };

    // 3. Build Swarm
    let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_dns()?
        .with_behaviour(|key| {
            // A. Relay Behaviour
            let relay_behaviour = relay::Behaviour::new(key.public().to_peer_id(), relay_config);

            // B. Kademlia (Server Mode) - Crucial for storing peer records
            let mut kad_config = kad::Config::default();
            kad_config
                .set_protocol_names(vec![libp2p::StreamProtocol::new("/centichain/kad/1.0.0")]);

            let store = kad::store::MemoryStore::new(key.public().to_peer_id());
            let mut kad_behaviour =
                kad::Behaviour::with_config(key.public().to_peer_id(), store, kad_config);

            // Force Server Mode so this node actively handles DHT queries and stores records
            kad_behaviour.set_mode(Some(kad::Mode::Server));

            // C. Identify - Must match exactly with RPC/GUI nodes (/antigravity/1.0.0)
            let identify = libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
                "/centichain/1.0.0".to_string(),
                key.public(),
            ));

            // D. Ping
            let ping = libp2p::ping::Behaviour::new(
                libp2p::ping::Config::new().with_interval(Duration::from_secs(5)),
            );

            Ok(RelayServerBehaviour {
                relay: relay_behaviour,
                kad: kad_behaviour,
                identify,
                ping,
            })
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    // 4. Listen on a specific port (e.g., 9090)
    let listen_addr: SocketAddr = "0.0.0.0:9090".parse()?;
    swarm.listen_on(format!("/ip4/{}/tcp/{}", listen_addr.ip(), listen_addr.port()).parse()?)?;

    log::info!(
        "Relay server listening on /ip4/{}/tcp/{}",
        listen_addr.ip(),
        listen_addr.port()
    );

    // 5. Event Loop
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                log::info!("Listening on {:?}", address);
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                log::info!("✅ Node connected: {:?} from {:?}", peer_id, endpoint);
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                log::info!("❌ Node disconnected: {:?} (cause: {:?})", peer_id, cause);
            }
            SwarmEvent::Behaviour(RelayServerBehaviourEvent::Relay(
                relay::Event::ReservationReqAccepted { src_peer_id, .. },
            )) => {
                log::info!("Relay Reservation Accepted for: {}", src_peer_id);
            }
            SwarmEvent::Behaviour(RelayServerBehaviourEvent::Identify(
                libp2p::identify::Event::Received { peer_id, info, .. },
            )) => {
                log::info!("Identify: Received info from {}, adding to DHT", peer_id);
                for addr in info.listen_addrs {
                    swarm.behaviour_mut().kad.add_address(&peer_id, addr);
                }
            }
            SwarmEvent::Behaviour(RelayServerBehaviourEvent::Kad(kad::Event::RoutingUpdated {
                peer,
                ..
            })) => {
                log::info!("DHT: Routing Table updated with peer {}", peer);
            }
            _ => {}
        }
    }
}
