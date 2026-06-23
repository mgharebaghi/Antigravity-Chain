//! # Transaction Types
//!
//! Transaction structure, Ed25519 signing, and verification.

use libp2p::identity::{Keypair, PublicKey};
use serde::{Deserialize, Serialize};

/// Sentinel signatures for protocol-level (coinbase / genesis) transactions.
pub const SYSTEM_SIG_GENESIS: &str = "SYSTEM:genesis";
pub const SYSTEM_SIG_REWARD: &str = "SYSTEM:reward";

/// A blockchain transaction
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Transaction {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub shard_id: u16,
    pub timestamp: u64,
    /// Hex-encoded Ed25519 signature over [`signing_payload`](Transaction::signing_payload).
    pub signature: String,
    /// Hex-encoded protobuf public key — required to verify user transactions on the network.
    #[serde(default)]
    pub sender_pubkey: String,
}

impl Transaction {
    /// Canonical byte payload that must be signed (prevents tampering after signing).
    pub fn signing_payload(&self) -> Vec<u8> {
        format!(
            "{}|{}|{}|{}|{}|{}",
            self.sender, self.receiver, self.amount, self.shard_id, self.timestamp, self.id
        )
        .into_bytes()
    }

    /// Signs this transaction in-place using the wallet keypair.
    pub fn sign_with_keypair(&mut self, keypair: &Keypair) -> Result<(), String> {
        let pubkey_bytes = keypair.public().encode_protobuf();
        self.sender_pubkey = hex::encode(pubkey_bytes);

        let sig = keypair
            .sign(&self.signing_payload())
            .map_err(|e| format!("Signing failed: {e}"))?;
        self.signature = hex::encode(sig);
        Ok(())
    }

    pub fn is_system(&self) -> bool {
        self.sender == "SYSTEM"
    }

    /// Validates structure, signature, and addresses.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Transaction id is empty".into());
        }
        if self.receiver.is_empty() {
            return Err("Receiver is empty".into());
        }
        if self.amount == 0 && !self.is_system() {
            return Err("Amount must be greater than zero".into());
        }

        if self.is_system() {
            return self.validate_system_signature();
        }

        self.receiver
            .parse::<libp2p::PeerId>()
            .map_err(|_| "Invalid receiver PeerId".to_string())?;
        self.sender
            .parse::<libp2p::PeerId>()
            .map_err(|_| "Invalid sender PeerId".to_string())?;

        if self.sender_pubkey.is_empty() {
            return Err("Missing sender_pubkey".into());
        }
        if self.signature.is_empty() {
            return Err("Missing signature".into());
        }

        let pubkey_bytes = hex::decode(&self.sender_pubkey)
            .map_err(|_| "Invalid sender_pubkey hex".to_string())?;
        let public_key = PublicKey::try_decode_protobuf(&pubkey_bytes)
            .map_err(|_| "Invalid sender_pubkey protobuf".to_string())?;

        if public_key.to_peer_id().to_string() != self.sender {
            return Err("sender_pubkey does not match sender PeerId".into());
        }

        let sig_bytes =
            hex::decode(&self.signature).map_err(|_| "Invalid signature hex".to_string())?;

        if !public_key.verify(&self.signing_payload(), &sig_bytes) {
            return Err("Invalid transaction signature".into());
        }

        Ok(())
    }

    fn validate_system_signature(&self) -> Result<(), String> {
        match self.signature.as_str() {
            SYSTEM_SIG_GENESIS | SYSTEM_SIG_REWARD => Ok(()),
            other if other == "genesis" || other == "reward" => {
                // Legacy blocks created before Phase 1
                Ok(())
            }
            _ => Err(format!("Invalid SYSTEM transaction signature: {}", self.signature)),
        }
    }

    /// Checks if this transaction is independent of another (no shared accounts).
    pub fn is_independent(&self, other: &Self) -> bool {
        self.sender != other.sender
            && self.sender != other.receiver
            && self.receiver != other.sender
            && self.receiver != other.receiver
    }
}

/// Calculates transaction fee (0.01%, minimum 0.001 AGT)
pub fn calculate_fee(amount: u64) -> u64 {
    let fee = (amount as f64 * 0.0001).ceil() as u64;
    fee.max(1_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_roundtrip() {
        let keypair = Keypair::generate_ed25519();
        let sender = keypair.public().to_peer_id().to_string();
        let receiver = Keypair::generate_ed25519().public().to_peer_id().to_string();

        let mut tx = Transaction {
            id: uuid::Uuid::new_v4().to_string(),
            sender,
            receiver,
            amount: 1_000_000,
            shard_id: 0,
            timestamp: 1_700_000_000,
            signature: String::new(),
            sender_pubkey: String::new(),
        };

        tx.sign_with_keypair(&keypair).unwrap();
        assert!(tx.validate().is_ok());

        tx.amount += 1;
        assert!(tx.validate().is_err());
    }
}
