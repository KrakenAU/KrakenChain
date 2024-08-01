use sha2::{Digest, Sha256};
use super::transaction::Transaction;

pub struct MerkleTree {
    pub root: Vec<u8>,
    nodes: Vec<Vec<u8>>,
}

impl MerkleTree {
    pub fn new(transactions: &[Transaction]) -> Self {
        let mut nodes: Vec<Vec<u8>> = transactions.iter().map(|tx| tx.calculate_hash()).collect();
        
        // If there's an odd number of transactions, duplicate the last one
        if nodes.len() % 2 != 0 {
            nodes.push(nodes.last().unwrap().clone());
        }

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
            MerkleTree::hash_pair(left, right)
        }).collect()
    }

    fn hash_pair(left: &[u8], right: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().to_vec()
    }

    pub fn get_proof(&self, transaction: &Transaction) -> Option<Vec<Vec<u8>>> {
        let tx_hash = transaction.calculate_hash();
        let mut index = self.nodes.iter().position(|hash| hash == &tx_hash)?;
        let mut proof = Vec::new();
        let mut level_size = self.nodes.len() / 2;

        while level_size > 0 {
            let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            if sibling_index < self.nodes.len() {
                proof.push(self.nodes[sibling_index].clone());
            }
            index /= 2;
            level_size /= 2;
        }

        Some(proof)
    }

    pub fn verify_proof(root: &[u8], transaction: &Transaction, proof: &[Vec<u8>]) -> bool {
        let mut hash = transaction.calculate_hash();
        for sibling in proof {
            hash = if hash < *sibling {
                MerkleTree::hash_pair(&hash, sibling)
            } else {
                MerkleTree::hash_pair(sibling, &hash)
            };
        }
        hash == root
    }
}