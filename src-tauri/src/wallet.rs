use libp2p::identity::Keypair;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    pub start_timestamp: u64, // When this wallet was created (for Patience calculation)
    pub address: String,
    pub alias: Option<String>,
    #[serde(skip)] // Don't serialize private key easily
    pub keypair: Vec<u8>, // Stored as bytes for simplicity in this demo
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WalletInfo {
    pub address: String,
    pub balance: u64,
    pub alias: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WalletExport {
    pub address: String,
    pub private_key: String,
    pub mnemonic: String, // Placeholder for now
}

impl Wallet {
    pub fn new() -> Self {
        let keypair = Keypair::generate_ed25519();
        let peer_id = keypair.public().to_peer_id().to_string();

        Wallet {
            start_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            address: peer_id,
            alias: None,
            keypair: keypair.to_protobuf_encoding().unwrap(),
        }
    }

    pub fn get_keypair(&self) -> Keypair {
        Keypair::from_protobuf_encoding(&self.keypair).expect("Invalid keypair")
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let keypair = self.get_keypair();
        keypair.sign(message).expect("Signing failed")
    }

    pub fn set_alias(&mut self, alias: String) {
        // In a real app, this would need to claim the alias on chain
        self.alias = Some(alias);
    }
}
