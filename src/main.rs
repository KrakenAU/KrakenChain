use KrakenChain::blockchain::{Blockchain, Transaction};
use chrono::Duration;
use ring::signature::KeyPair; // Added this line

fn create_keypair() -> (ring::signature::Ed25519KeyPair, String) {
    let rng = ring::rand::SystemRandom::new();
    let pkcs8_bytes = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
    let key_pair = ring::signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();
    let public_key = key_pair.public_key();
    let address = hex::encode(public_key.as_ref());
    (key_pair, address)
}

#[allow(unused_variables)]
fn main() {
    // Create a new blockchain
    let mut blockchain = Blockchain::new(4, 10.0, Duration::seconds(10));

    // Create some keypairs for testing
    let (alice_key, alice_address) = create_keypair();
    let (bob_key, bob_address) = create_keypair();
    let (charlie_key, charlie_address) = create_keypair();

    // Add some initial balance to Alice
    blockchain.add_balance(&alice_address, 100.0);

    // Create and add transactions to mempool
    let mut tx1 = Transaction::new(alice_address.clone(), bob_address.clone(), 30.0, 0.1);
    tx1.sign(&alice_key);
    blockchain.add_to_mempool(tx1).unwrap();

    let mut tx2 = Transaction::new(alice_address.clone(), charlie_address.clone(), 20.0, 0.1);
    tx2.sign(&alice_key);
    blockchain.add_to_mempool(tx2).unwrap();

    // Mine pending transactions
    blockchain.mine_pending_transactions(&bob_address).unwrap();

    // Display blockchain information
    println!("Blockchain length: {}", blockchain.chain.len());
    println!("Alice's balance: {}", blockchain.get_balance(&alice_address));
    println!("Bob's balance: {}", blockchain.get_balance(&bob_address));
    println!("Charlie's balance: {}", blockchain.get_balance(&charlie_address));

    // Validate the blockchain
    println!("Is blockchain valid? {}", blockchain.validate_chain());
}