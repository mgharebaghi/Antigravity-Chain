use libp2p::{
    futures::StreamExt,
    identity, noise, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, SwarmBuilder,
};
use std::error::Error;
use std::net::SocketAddr;

#[derive(NetworkBehaviour)]
struct RelayServerBehaviour {
    relay: relay::Behaviour,
    identify: libp2p::identify::Behaviour,
    ping: libp2p::ping::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // 1. Generate keys
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Relay Node ID: {:?}", local_peer_id);

    // 2. Configure Relay
    let relay_config = relay::Config::default();

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
            let relay_behaviour = relay::Behaviour::new(key.public().to_peer_id(), relay_config);

            let identify = libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
                "/antigravity/relay/1.0.0".to_string(),
                key.public(),
            ));

            let ping = libp2p::ping::Behaviour::new(
                libp2p::ping::Config::new().with_interval(std::time::Duration::from_secs(5)),
            );

            Ok(RelayServerBehaviour {
                relay: relay_behaviour,
                identify,
                ping,
            })
        })?
        .with_swarm_config(|cfg| {
            cfg.with_idle_connection_timeout(std::time::Duration::from_secs(u64::MAX))
        })
        .build();

    // 4. Listen on a specific port (e.g., 9090)
    let listen_addr: SocketAddr = "0.0.0.0:9090".parse()?;
    swarm.listen_on(format!("/ip4/{}/tcp/{}", listen_addr.ip(), listen_addr.port()).parse()?)?;

    println!(
        "Relay server listening on /ip4/{}/tcp/{}",
        listen_addr.ip(),
        listen_addr.port()
    );

    // 5. Event Loop
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {:?}", address);
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                println!("✅ Node connected: {:?} from {:?}", peer_id, endpoint);
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                println!("❌ Node disconnected: {:?} (cause: {:?})", peer_id, cause);
            }
            SwarmEvent::Behaviour(RelayServerBehaviourEvent::Relay(event)) => {
                println!("🔄 Relay event: {:?}", event);
            }
            SwarmEvent::Behaviour(RelayServerBehaviourEvent::Identify(event)) => {
                println!("🔍 Identify event: {:?}", event);
            }
            _ => {}
        }
    }
}
