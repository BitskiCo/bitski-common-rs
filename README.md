# Bitski Common Library for Rust

## About

This is a collection of Rust crates that implement shared Bitski backend libraries.

## Getting Started

### Prerequisites

Install Rust and Cargo. This library requires Rust 1.60 or higher.

### Installing

In `Cargo.toml` add the relevant dependencies:

```
[dependencies]
bitski-common = { git = "https://github.com/BitskiCo/bitski-common-rs" }
blockchain-transaction-types = { git = "https://github.com/BitskiCo/bitski-common-rs" }
```

## Testing

Run `cargo test --all-features` in the repository directory to test all the code. The sub-crates will be tested as part of the Cargo workspace.
