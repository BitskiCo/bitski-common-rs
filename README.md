# Bitski Common Library for Rust

## About

This is a collection of Rust crates that implement shared Bitski backend libraries.

## Getting Started

### Prerequisites

Install Rust and Cargo.

### Installing

In `Cargo.toml` add the relevant dependencies:

```
[dependencies]
bitski-eip712 = {git = "https://github.com/BitskiCo/blockchain-transaction-types"}
blockchain-transaction-types = {git = "https://github.com/BitskiCo/blockchain-transaction-types"}
```

## Testing

Run `cargo test --all-features` in the repository directory to test all the code. The sub-crates will be tested as part of the Cargo workspace.
