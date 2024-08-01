use super::block::Block;
use super::transaction::Transaction;
use crate::blockchain::merkle_tree::MerkleTree;
use std::collections::HashMap;
use crate::utils::Logger;
use serde_json;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

const MIN_FEE_RATE: f64 = 0.00001; // Satoshis per byte

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
    pub max_mempool_size_bytes: usize,
    pub mempool_size_bytes: usize,
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
            max_mempool_size_bytes: 5_000_000, // 5 MB limit
            mempool_size_bytes: 0,
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

        let transactions = self.get_transactions_from_mempool(1000);
        let transactions = if transactions.is_empty() {
            self.pending_transactions.drain(..).collect()
        } else {
            transactions
        };

        let reward_transaction = Transaction::new(
            String::from("Blockchain"),
            miner_address.to_string(),
            self.mining_reward,
            0.0,
        );

        let mut all_transactions = transactions;
        all_transactions.push(reward_transaction);

        let new_block = Block::new(
            self.chain.len() as u64,
            all_transactions,
            self.get_latest_block().hash.clone(),
            self.difficulty,
        );

        let mineable_block = Arc::new(Mutex::new(new_block));
        let found = Arc::new(Mutex::new(false));
        let num_threads = num_cpus::get();

        let threads: Vec<_> = (0..num_threads)
            .map(|_| {
                let block = Arc::clone(&mineable_block);
                let found = Arc::clone(&found);
                let difficulty = self.difficulty;

                thread::spawn(move || {
                    let mut local_block = block.lock().unwrap().clone();
                    while !*found.lock().unwrap() {
                        if local_block.mine_block(difficulty) {
                            let mut found_lock = found.lock().unwrap();
                            if !*found_lock {
                                *found_lock = true;
                                let mut block_lock = block.lock().unwrap();
                                *block_lock = local_block;
                            }
                            break;
                        }
                    }
                })
            })
            .collect();

        for thread in threads {
            thread.join().unwrap();
        }

        let mined_block = mineable_block.lock().unwrap().clone();

        if self.is_valid_new_block(&mined_block, self.get_latest_block()) {
            self.chain.push(mined_block);
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

        // Calculate the average block time for the last difficulty adjustment interval
        let avg_block_time = actual_time / self.difficulty_adjustment_interval as i32;

        // Calculate the ratio of actual time to expected time
        let time_ratio = actual_time.num_seconds() as f64 / expected_time.num_seconds() as f64;

        // Adjust difficulty based on the time ratio, but limit the change to 25% in either direction
        let adjustment_factor = time_ratio.max(0.75).min(1.25);
        let new_difficulty = (self.difficulty as f64 / adjustment_factor).max(1.0);

        // Smooth out difficulty changes by averaging with the previous difficulty
        self.difficulty = ((self.difficulty as f64 + new_difficulty) / 2.0).round() as u32;

        // Update the block time window
        self.block_time_window.push(avg_block_time);
        if self.block_time_window.len() > 10 {
            self.block_time_window.remove(0);
        }

        Logger::info(&format!("Difficulty adjusted to: {}", self.difficulty));
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

        // Calculate transaction size (simplified, you may want to implement a more accurate size calculation)
        let tx_size = self.calculate_transaction_size(&transaction);
        let fee_rate = transaction.fee / tx_size as f64;

        if fee_rate < MIN_FEE_RATE {
            return Err("Transaction fee rate is too low".to_string());
        }

        // Check if adding this transaction would exceed the mempool size limit
        if self.mempool_size_bytes + tx_size > self.max_mempool_size_bytes {
            self.evict_transactions(tx_size);
        }

        // Add transaction to mempool
        self.mempool.push(transaction.clone());
        self.mempool_size_bytes += tx_size;

        // Sort mempool by fee rate (fee per byte)
        self.sort_mempool();

        Logger::info(&format!("Transaction added to mempool. Mempool size: {} bytes", self.mempool_size_bytes));
        Ok(())
    }

    fn evict_transactions(&mut self, required_space: usize) {
        while self.mempool_size_bytes + required_space > self.max_mempool_size_bytes {
            if let Some(tx) = self.mempool.pop() {
                self.mempool_size_bytes -= self.calculate_transaction_size(&tx);
                Logger::info(&format!("Evicted transaction {} from mempool", tx.id));
            } else {
                break;
            }
        }
    }

    pub fn get_transactions_from_mempool(&mut self, max_transactions: usize) -> Vec<Transaction> {
        let current_time = chrono::Utc::now().timestamp();
        self.mempool.retain(|tx| tx.expiration > current_time);

        let transactions: Vec<Transaction> = self.mempool.drain(..std::cmp::min(max_transactions, self.mempool.len())).collect();
        Logger::info(&format!("Retrieved {} transactions from mempool. Remaining mempool size: {}", transactions.len(), self.mempool.len()));
        transactions
    }

    pub fn replace_transaction(&mut self, new_transaction: Transaction) -> Result<(), String> {
        if !new_transaction.is_valid() {
            return Err("Invalid transaction".to_string());
        }

        let sender_balance = self.get_balance(&new_transaction.from);
        if sender_balance < new_transaction.amount + new_transaction.fee {
            return Err("Insufficient balance".to_string());
        }

        let old_tx_index = self.mempool.iter().position(|tx| tx.id == new_transaction.id);

        if let Some(index) = old_tx_index {
            let old_tx = &self.mempool[index];
            if new_transaction.fee <= old_tx.fee {
                return Err("New transaction must have a higher fee for RBF".to_string());
            }

            // Remove old transaction and update mempool size
            let old_tx_size = self.calculate_transaction_size(old_tx);
            self.mempool.remove(index);
            self.mempool_size_bytes -= old_tx_size;

            // Add new transaction
            let new_tx_size = self.calculate_transaction_size(&new_transaction);
            self.mempool.push(new_transaction);
            self.mempool_size_bytes += new_tx_size;

            // Re-sort mempool
            self.sort_mempool();

            Logger::info(&format!("Transaction replaced in mempool. New mempool size: {} bytes", self.mempool_size_bytes));
            Ok(())
        } else {
            Err("Original transaction not found in mempool".to_string())
        }
    }

    pub fn save_mempool(&self, file_path: &str) -> std::io::Result<()> {
        let serialized = serde_json::to_string(&self.mempool)?;
        let mut file = File::create(file_path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn load_mempool(&mut self, file_path: &str) -> std::io::Result<()> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        self.mempool = serde_json::from_str(&contents)?;
        self.mempool_size_bytes = self.mempool.iter().map(|tx| self.calculate_transaction_size(tx)).sum();
        Ok(())
    }

    fn calculate_transaction_size(&self, transaction: &Transaction) -> usize {
        // This is a simplified calculation and should be adjusted based on your actual transaction structure
        let base_size = std::mem::size_of::<Transaction>();
        let variable_size = transaction.from.len() + transaction.to.len() + transaction.signature.as_ref().map_or(0, |s| s.len());
        base_size + variable_size
    }

    pub fn clean_expired_transactions(&mut self) {
        let current_time = chrono::Utc::now().timestamp();
        let expired_transactions: Vec<_> = self.mempool
            .iter()
            .filter(|tx| tx.expiration < current_time)
            .cloned()
            .collect();

        for tx in expired_transactions {
            let tx_size = self.calculate_transaction_size(&tx);
            self.mempool.retain(|t| t.id != tx.id);
            self.mempool_size_bytes -= tx_size;
            Logger::info(&format!("Removed expired transaction {} from mempool", tx.id));
        }

        self.sort_mempool();
    }

    fn sort_mempool(&mut self) {
        let tx_sizes: Vec<_> = self.mempool.iter()
            .map(|tx| self.calculate_transaction_size(tx))
            .collect();
        
        let mut indices: Vec<usize> = (0..self.mempool.len()).collect();
        
        indices.sort_by(|&a, &b| {
            let a_fee_rate = self.mempool[a].fee / tx_sizes[a] as f64;
            let b_fee_rate = self.mempool[b].fee / tx_sizes[b] as f64;
            b_fee_rate.partial_cmp(&a_fee_rate).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Reorder the mempool based on the sorted indices
        let sorted_mempool: Vec<_> = indices.into_iter().map(|i| self.mempool[i].clone()).collect();
        self.mempool = sorted_mempool;
    }
}