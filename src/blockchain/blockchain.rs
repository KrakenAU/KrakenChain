use super::block::Block;
use super::transaction::Transaction;
use crate::blockchain::merkle_tree::MerkleTree;
use std::collections::HashMap;
use crate::utils::Logger;

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: u32,
    pub pending_transactions: Vec<Transaction>,
    pub mining_reward: f64,
    balances: HashMap<String, f64>,
    pub target_block_time: chrono::Duration,
    pub mempool: Vec<Transaction>,
    pub block_time_window: Vec<chrono::Duration>,
    pub difficulty_adjustment_interval: u64,
    pub max_mempool_size: usize,
}

impl Blockchain {
    pub fn new(difficulty: u32, mining_reward: f64, target_block_time: chrono::Duration) -> Self {
        Logger::info(&format!("Creating new blockchain with difficulty: {}, mining reward: {}, target block time: {:?}", difficulty, mining_reward, target_block_time));
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            difficulty,
            pending_transactions: Vec::new(),
            mining_reward,
            balances: HashMap::new(),
            target_block_time,
            mempool: Vec::new(),
            block_time_window: Vec::new(),
            difficulty_adjustment_interval: 10, // Adjust this value as needed
            max_mempool_size: 1000, // Adjust this value as needed
        };
        blockchain.create_genesis_block();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let genesis_block = Block::new(0, Vec::new(), String::from("0"), self.difficulty);
        self.chain.push(genesis_block);
    }

    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().expect("Blockchain is empty")
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        Logger::transaction(&format!("Adding new transaction: {:?}", transaction));
        if !transaction.is_valid() {
            return Err("Invalid transaction".to_string());
        }

        let sender_balance = self.get_balance(&transaction.from);
        if sender_balance < transaction.amount {
            return Err("Insufficient balance".to_string());
        }

        self.pending_transactions.push(transaction);
        Ok(())
    }

    pub fn add_balance(&mut self, address: &str, amount: f64) {
        *self.balances.entry(address.to_string()).or_insert(0.0) += amount;
    }

    pub fn mine_pending_transactions(&mut self, miner_address: &str) -> Result<(), String> {
        Logger::mining(&format!("Mining pending transactions for miner: {}", miner_address));
    
        // Get transactions from mempool
        let mut transactions = self.get_transactions_from_mempool(1000);
    
        // If mempool is empty, use pending transactions
        if transactions.is_empty() {
            transactions = self.pending_transactions.drain(..).collect();
        }
    
        let reward_transaction = Transaction::new(
            String::from("Blockchain"),
            miner_address.to_string(),
            self.mining_reward,
            0.0, // No fee for reward transactions
        );
        transactions.push(reward_transaction);
    
        let new_block = Block::new(
            self.chain.len() as u64,
            transactions,
            self.get_latest_block().hash.clone(),
            self.difficulty,
        );
    
        let mut mineable_block = new_block;
        mineable_block.mine_block(self.difficulty);
    
        if self.is_valid_new_block(&mineable_block, self.get_latest_block()) {
            self.chain.push(mineable_block);
            self.update_balances();
            self.adjust_difficulty();
            Logger::mining("Successfully mined and added new block");
            Ok(())
        } else {
            Logger::error("Failed to mine block: Invalid block");
            Err("Invalid block".to_string())
        }
    }

    fn is_valid_new_block(&self, new_block: &Block, previous_block: &Block) -> bool {
        Logger::validation(&format!("Validating new block: {:?}", new_block));
        if new_block.index != previous_block.index + 1 {
            return false;
        }
        if new_block.previous_hash != previous_block.hash {
            return false;
        }
        if new_block.calculate_hash() != new_block.hash {
            return false;
        }
        if !new_block.has_valid_transactions() {
            return false;
        }
        let merkle_tree = MerkleTree::new(&new_block.transactions);
        if new_block.merkle_root != merkle_tree.root {
            return false;
        }
        if new_block.timestamp <= previous_block.timestamp {
            return false;
        }
        if new_block.transactions.len() > 1000 {  // Arbitrary limit, adjust as needed
            return false;
        }
        let total_value: f64 = new_block.transactions.iter().map(|tx| tx.amount).sum();
        if total_value > 1_000_000.0 {  // Arbitrary limit, adjust as needed
            return false;
        }
        // Check if the hash meets the difficulty requirement
        let target = (1u128 << (128 - self.difficulty)) - 1;
        let hash_value = u128::from_str_radix(&new_block.hash[..32], 16).unwrap_or(u128::MAX);
        hash_value <= target
    }

    pub fn is_chain_valid(&self) -> bool {
        Logger::validation("Validating entire blockchain");
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if !self.is_valid_new_block(current_block, previous_block) {
                return false;
            }
        }
        true
    }

    fn update_balances(&mut self) {
        Logger::info("Updating balances");
        for block in &self.chain {
            for transaction in &block.transactions {
                *self.balances.entry(transaction.from.clone()).or_insert(0.0) -= transaction.amount;
                *self.balances.entry(transaction.to.clone()).or_insert(0.0) += transaction.amount;
            }
        }
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        *self.balances.get(address).unwrap_or(&0.0)
    }

    fn adjust_difficulty(&mut self) {
        Logger::info(&format!("Adjusting difficulty. Current difficulty: {}", self.difficulty));
        if self.chain.len() < self.difficulty_adjustment_interval as usize {
            return;
        }

        let last_adjusted_block = &self.chain[self.chain.len() - self.difficulty_adjustment_interval as usize];
        let expected_time = self.target_block_time * self.difficulty_adjustment_interval.try_into().unwrap();
        let actual_time = self.get_latest_block().timestamp - last_adjusted_block.timestamp;

        if actual_time < expected_time / 2 {
            self.difficulty += 1;
        } else if actual_time > expected_time * 2 {
            self.difficulty = self.difficulty.saturating_sub(1);
        }

        self.block_time_window.push(actual_time);
        if self.block_time_window.len() > 10 {
            self.block_time_window.remove(0);
        }
    }

    pub fn validate_chain(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            Logger::validation(&format!("Validating block {} of {}", i, self.chain.len() - 1));

            if !self.is_valid_new_block(current_block, previous_block) {
                Logger::error(&format!("Invalid block found at index {}", i));
                return false;
            }

            // Validate all transactions in the block
            for (j, transaction) in current_block.transactions.iter().enumerate() {
                if !transaction.is_valid() {
                    Logger::error(&format!("Invalid transaction found in block {} at index {}", i, j));
                    return false;
                }
            }
        }
        Logger::validation("Blockchain is valid");
        true
    }

    pub fn recalculate_balances(&mut self) {
        self.balances.clear();
        for block in &self.chain {
            for transaction in &block.transactions {
                *self.balances.entry(transaction.from.clone()).or_insert(0.0) -= transaction.amount;
                *self.balances.entry(transaction.to.clone()).or_insert(0.0) += transaction.amount;
            }
        }
    }

    pub fn get_transactions_for_address(&self, address: &str) -> Vec<&Transaction> {
        self.chain
            .iter()
            .flat_map(|block| &block.transactions)
            .filter(|tx| tx.from == address || tx.to == address)
            .collect()
    }

    pub fn add_to_mempool(&mut self, transaction: Transaction) -> Result<(), String> {
        if !transaction.is_valid() {
            return Err("Invalid transaction".to_string());
        }

        let sender_balance = self.get_balance(&transaction.from);
        if sender_balance < transaction.amount + transaction.fee {
            return Err("Insufficient balance".to_string());
        }

        // Check for double-spend
        if self.mempool.iter().any(|tx| tx.from == transaction.from && tx.amount + tx.fee > sender_balance - (transaction.amount + transaction.fee)) {
            return Err("Potential double-spend detected".to_string());
        }

        // Check if the transaction is already in the mempool
        if self.mempool.iter().any(|tx| tx.id == transaction.id) {
            return Err("Transaction already in mempool".to_string());
        }

        // Check expiration
        let current_time = chrono::Utc::now().timestamp();
        if transaction.expiration < current_time {
            return Err("Transaction has expired".to_string());
        }

        // Add transaction to mempool
        self.mempool.push(transaction);

        // Sort mempool by fee (highest fee first)
        self.mempool.sort_by(|a, b| b.fee.partial_cmp(&a.fee).unwrap_or(std::cmp::Ordering::Equal));

        // Remove lowest fee transactions if mempool is too large
        while self.mempool.len() > self.max_mempool_size {
            self.mempool.pop();
        }

        Logger::info(&format!("Transaction added to mempool. Mempool size: {}", self.mempool.len()));
        Ok(())
    }

    pub fn get_transactions_from_mempool(&mut self, max_transactions: usize) -> Vec<Transaction> {
        let current_time = chrono::Utc::now().timestamp();
        self.mempool.retain(|tx| tx.expiration > current_time);

        let transactions: Vec<Transaction> = self.mempool.drain(..std::cmp::min(max_transactions, self.mempool.len())).collect();
        Logger::info(&format!("Retrieved {} transactions from mempool. Remaining mempool size: {}", transactions.len(), self.mempool.len()));
        transactions
    }
}