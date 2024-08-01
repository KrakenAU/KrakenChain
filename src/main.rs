use KrakenChain::blockchain::{Blockchain, Transaction};
use chrono::Duration;
use ring::signature::KeyPair;

fn create_keypair() -> (ring::signature::Ed25519KeyPair, String) {
    let rng = ring::rand::SystemRandom::new();
    let pkcs8_bytes = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
    let key_pair = ring::signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();
    let public_key = key_pair.public_key();
    let address = hex::encode(public_key.as_ref());
    (key_pair, address)
}

fn main() {
    // Create a new blockchain
    let mut blockchain = Blockchain::new(4, 10.0, Duration::seconds(10));

    // Create some keypairs for testing
    let (alice_key, alice_address) = create_keypair();
    let (bob_key, bob_address) = create_keypair();
    let (charlie_key, charlie_address) = create_keypair();

    // Add some initial balance to Alice and Bob
    blockchain.add_balance(&alice_address, 100.0);
    blockchain.add_balance(&bob_address, 50.0);

    println!("Initial balances:");
    println!("Alice: {}", blockchain.get_balance(&alice_address));
    println!("Bob: {}", blockchain.get_balance(&bob_address));
    println!("Charlie: {}", blockchain.get_balance(&charlie_address));

    // Create and add transactions to mempool
    let mut tx1 = Transaction::new(alice_address.clone(), bob_address.clone(), 30.0, 0.1);
    tx1.sign(&alice_key);
    blockchain.add_to_mempool(tx1).unwrap();

    let mut tx2 = Transaction::new(bob_address.clone(), charlie_address.clone(), 15.0, 0.1);
    tx2.sign(&bob_key);
    blockchain.add_to_mempool(tx2).unwrap();

    let mut tx3 = Transaction::new(alice_address.clone(), charlie_address.clone(), 20.0, 0.1);
    tx3.sign(&alice_key);
    blockchain.add_to_mempool(tx3).unwrap();

    println!("\nTransactions added to mempool. Mining first block...");

    // Mine pending transactions
    blockchain.mine_pending_transactions(&bob_address).unwrap();

    println!("\nFirst block mined. Current balances:");
    println!("Alice: {}", blockchain.get_balance(&alice_address));
    println!("Bob: {}", blockchain.get_balance(&bob_address));
    println!("Charlie: {}", blockchain.get_balance(&charlie_address));

    // Validate the blockchain
    println!("\nIs blockchain valid? {}", blockchain.validate_chain());

    // Add more transactions
    let mut tx4 = Transaction::new(charlie_address.clone(), alice_address.clone(), 5.0, 0.1);
    tx4.sign(&charlie_key);
    blockchain.add_to_mempool(tx4).unwrap();

    let mut tx5 = Transaction::new(bob_address.clone(), alice_address.clone(), 10.0, 0.1);
    tx5.sign(&bob_key);
    blockchain.add_to_mempool(tx5).unwrap();

    println!("\nMore transactions added to mempool. Mining second block...");

    // Mine pending transactions
    blockchain.mine_pending_transactions(&charlie_address).unwrap();

    println!("\nSecond block mined. Final balances:");
    println!("Alice: {}", blockchain.get_balance(&alice_address));
    println!("Bob: {}", blockchain.get_balance(&bob_address));
    println!("Charlie: {}", blockchain.get_balance(&charlie_address));

    // Display blockchain information
    println!("\nBlockchain length: {}", blockchain.chain.len());

    // Validate the blockchain again
    println!("Is blockchain valid? {}", blockchain.validate_chain());
}