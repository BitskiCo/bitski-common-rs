# Bitski Common Library for Rust

## About

This is a collection of Rust crates that implement shared Bitski backend
libraries.

## Getting Started

### Prerequisites

Install Rust and Cargo. This library requires Rust 1.60 or higher.

### Installing

In `Cargo.toml` add the relevant dependencies:

```toml
[dependencies]
bitski-common = { git = "https://github.com/BitskiCo/bitski-common-rs" }
blockchain-transaction-types = { git = "https://github.com/BitskiCo/bitski-common-rs" }
```

### Documentation

Browse the docs by running:

```
RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features --open
```

This command [documents optional Rust features][optional-features] required for
each API.

[optional-features]: https://users.rust-lang.org/t/how-to-document-optional-features-in-api-docs/64577/3

## Testing

Run `cargo test --all-features` in the repository directory to test all the
code. The sub-crates will be tested as part of the Cargo workspace.
