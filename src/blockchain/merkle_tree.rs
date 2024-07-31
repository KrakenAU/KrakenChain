use sha2::{Digest, Sha256};
use super::transaction::Transaction;

pub struct MerkleTree {
    pub root: Vec<u8>,
    nodes: Vec<Vec<u8>>,
}

impl MerkleTree {
    pub fn new(transactions: &[Transaction]) -> Self {
        let mut nodes: Vec<Vec<u8>> = transactions.iter().map(|tx| tx.calculate_hash()).collect();
        while nodes.len() > 1 {
            nodes = MerkleTree::pair_and_hash(nodes);
        }
        MerkleTree {
            root: nodes.first().cloned().unwrap_or_default(),
            nodes,
        }
    }

    fn pair_and_hash(nodes: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        nodes.chunks(2).map(|chunk| {
            let left = &chunk[0];
            let right = chunk.get(1).unwrap_or(left);
            let mut hasher = Sha256::new();
            hasher.update(left);
            hasher.update(right);
            hasher.finalize().to_vec()
        }).collect()
    }

    pub fn get_proof(&self, transaction: &Transaction) -> Vec<Vec<u8>> {
        let mut proof = Vec::new();
        let mut index = self.nodes.iter().position(|hash| hash == &transaction.calculate_hash()).unwrap_or(0);
        let mut level_size = self.nodes.len() / 2;

        while level_size > 0 {
            let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            if sibling_index < self.nodes.len() {
                proof.push(self.nodes[sibling_index].clone());
            }
            index /= 2;
            level_size /= 2;
        }

        proof
    }

    pub fn verify_proof(root: &[u8], transaction: &Transaction, proof: &[Vec<u8>]) -> bool {
        let mut hash = transaction.calculate_hash();
        for sibling in proof {
            let mut hasher = Sha256::new();
            if hash < *sibling {
                hasher.update(&hash);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(&hash);
            }
            hash = hasher.finalize().to_vec();
        }
        hash == root
    }
}