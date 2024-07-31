use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ring::signature::Ed25519KeyPair;

use uuid::Uuid;
use crate::utils::Logger;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub fee: f64,
    pub timestamp: i64,
    pub expiration: i64,
    pub signature: Option<String>,
}
impl Transaction {
    pub fn new(from: String, to: String, amount: f64, fee: f64) -> Self {
        Logger::transaction(&format!("Creating new transaction: {} -> {}, amount: {}, fee: {}", from, to, amount, fee));
        Transaction {
            id: Uuid::new_v4().to_string(),
            from,
            to,
            amount,
            fee,
            timestamp: chrono::Utc::now().timestamp(),
            expiration: chrono::Utc::now().timestamp() + 3600, // Set expiration to 1 hour from now
            signature: None,
        }
    }

    pub fn calculate_hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(self.from.as_bytes());
        hasher.update(self.to.as_bytes());
        hasher.update(self.amount.to_string().as_bytes());
        hasher.update(self.timestamp.to_string().as_bytes());
        hasher.finalize().to_vec()
    }

    pub fn serialize_for_signing(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.id.as_bytes());
        data.extend_from_slice(self.from.as_bytes());
        data.extend_from_slice(self.to.as_bytes());
        data.extend_from_slice(self.amount.to_string().as_bytes());
        data.extend_from_slice(self.timestamp.to_string().as_bytes());
        data
    }

    pub fn is_valid(&self) -> bool {
        if self.from == "Blockchain" {
            // This is a mining reward transaction, no signature needed
            return true;
        }
    
        if self.amount <= 0.0 {
            return false;
        }
    
        if let Some(signature) = &self.signature {
            let message = self.calculate_hash();
            let public_key = hex::decode(&self.from).unwrap();
            let signature = hex::decode(signature).unwrap();
            ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, &public_key)
                .verify(&message, &signature)
                .is_ok()
        } else {
            false
        }
    }
    
    pub fn sign(&mut self, key_pair: &Ed25519KeyPair) {
        Logger::transaction(&format!("Signing transaction: {}", self.id));
        let message = self.calculate_hash();
        let signature = key_pair.sign(&message);
        self.signature = Some(hex::encode(signature.as_ref()));
    }
}