use libp2p::{
    futures::StreamExt,
    noise, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, SwarmBuilder,
};
use std::error::Error;
use std::time::Duration;

#[derive(NetworkBehaviour)]
struct RelayBehaviour {
    relay: relay::Behaviour,
    identify: libp2p::identify::Behaviour,
    ping: libp2p::ping::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    flexi_logger::Logger::try_with_str("info")?.start()?;

    // Create a static keypair for the relay so its PeerId doesn't change on restart
    // In a real usage, load this from a file
    let local_key = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Relay PeerId: {}", local_peer_id);

    let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_dns()?
        .with_behaviour(|key| {
            let relay_config = relay::Config {
                max_reservations: 1024,
                max_circuits: 1024,
                max_circuit_duration: Duration::from_secs(2 * 60),
                max_circuit_bytes: 2 * 1024 * 1024 * 1024, // 2GB
                ..Default::default()
            };

            RelayBehaviour {
                relay: relay::Behaviour::new(local_peer_id, relay_config),
                identify: libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
                    "/antigravity/relay/1.0.0".to_string(),
                    key.public(),
                )),
                ping: libp2p::ping::Behaviour::new(libp2p::ping::Config::new()),
            }
        })?
        .build();

    // Bind to 0.0.0.0:9090
    let addr: Multiaddr = "/ip4/0.0.0.0/tcp/9090".parse()?;
    swarm.listen_on(addr.clone())?;

    println!("Relay listening on {}", addr);

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {:?}", address);
            }
            SwarmEvent::IncomingConnection { .. } => {
                // println!("Incoming connection...");
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connected to {:?}", peer_id);
            }
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                eprintln!("Outgoing connection error: {:?}", error);
            }
            _ => {}
        }
    }
}
