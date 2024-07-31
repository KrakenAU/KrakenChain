use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uint::construct_uint;
use crate::utils::Logger;

use super::transaction::Transaction;
use super::merkle_tree::MerkleTree;

construct_uint! {
    pub struct U256(4);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: DateTime<Utc>,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub difficulty: u32,
    pub merkle_root: Vec<u8>,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String, difficulty: u32) -> Self {
        Logger::block(&format!("Creating new block with index: {}, transactions: {}, difficulty: {}", index, transactions.len(), difficulty));
        let merkle_tree = MerkleTree::new(&transactions);
        let mut block = Block {
            index,
            timestamp: Utc::now(),
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            difficulty,
            merkle_root: merkle_tree.root,
        };
        block.hash = block.calculate_hash();
        Logger::block(&format!("New block created with hash: {}", block.hash));
        block
    }

    pub fn calculate_hash(&self) -> String {
        Logger::block(&format!("Calculating hash for block: {}", self.index));
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string());
        hasher.update(self.timestamp.to_string());
        hasher.update(&self.merkle_root);
        hasher.update(&self.previous_hash);
        hasher.update(self.nonce.to_string());
        hasher.update(self.difficulty.to_string());
        let hash = format!("{:x}", hasher.finalize());
        Logger::block(&format!("Calculated hash for block {}: {}", self.index, hash));
        hash
    }

    pub fn mine_block(&mut self, difficulty: u32) {
        Logger::mining(&format!("Mining block: {} with difficulty: {}", self.index, difficulty));
        let target = (1u128 << (128 - difficulty)) - 1;
        let mut attempts = 0;
        while u128::from_str_radix(&self.hash[..32], 16).unwrap_or(u128::MAX) > target {
            self.nonce += 1;
            self.hash = self.calculate_hash();
            attempts += 1;
            if attempts % 100000 == 0 {
                Logger::mining(&format!("Mining attempt {}: current hash {}", attempts, self.hash));
            }
        }
        Logger::mining(&format!("Block {} mined successfully after {} attempts. Final hash: {}", self.index, attempts, self.hash));
    }

    pub fn has_valid_transactions(&self) -> bool {
        Logger::validation(&format!("Validating transactions for block: {}", self.index));
        let valid = self.transactions.iter().all(|tx| tx.is_valid());
        Logger::validation(&format!("Checking transactions validity for block {}: {}", self.index, valid));
        valid
    }

    pub fn hash_to_u256(&self, hash: &str) -> U256 {
        let u256 = U256::from_big_endian(&hex::decode(hash).unwrap());
        Logger::info(&format!("Converted hash to U256 for block {}: {}", self.index, u256));
        u256
    }
}