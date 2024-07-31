# KrakenChain

KrakenChain is a lightweight blockchain implementation in Rust, designed for educational purposes and as a foundation for blockchain-based applications.

## Features

- Proof of Work consensus mechanism
- Transaction creation and validation
- Mempool for managing pending transactions
- Merkle tree for efficient transaction verification
- Dynamic difficulty adjustment
- Basic wallet functionality with Ed25519 key pairs

## Getting Started

### Prerequisites

- Rust 1.55 or higher
- Cargo (Rust's package manager)

### Installation

1. Clone the repository:
   ```sh
   git clone https://github.com/yourusername/krakenchain.git
   cd krakenchain
   ```

2. Build the project:
   ```sh
   cargo build --release
   ```

## Usage

Run the main example:
```sh
sh
cargo run --release
```

This will create a new blockchain, add some transactions, mine a block, and display the results.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
