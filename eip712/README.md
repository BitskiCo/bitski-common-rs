# EIP-712

## About

[EIP-712][eip-712] hasher for signing.

## Getting Started

### Prerequisites

Install Rust and Cargo.

### Installing

In `Cargo.toml` add the dependency:

```
[dependencies]
bitski-eip712 = {git = "https://github.com/BitskiCo/blockchain-transaction-types"}
```

End with an example of getting some data out of the system or using it for a little demo.

## Usage

```rs
use bitski_eip712::{Hasher, TypedData};
use serde_json::json;

fn example() {
    let typed_data = serde_json::from_value::<TypedData>(json!({
        "types": {
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "version", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"}
            ],
            "Person": [
                {"name": "name", "type": "string"},
                {"name": "wallet", "type": "address"}
            ],
            "Mail": [
                {"name": "from", "type": "Person"},
                {"name": "to", "type": "Person"},
                {"name": "contents", "type": "string"}
            ]
        },
        "primaryType": "Mail",
        "domain": {
            "name": "Ether Mail",
            "version": "1",
            "chainId": 1,
            "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
        },
        "message": {
            "from": {
                "name": "Cow",
                "wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
            },
            "to": {
                "name": "Bob",
                "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
            },
            "contents": "Hello, Bob!"
        }
    })).unwrap();
    let hasher = Hasher::try_from(&typed_data).unwrap();
    let result = hasher.hash(&typed_data).unwrap();

    assert_eq!(
        format!("{}", result.encode_hex::<String>()),
        "be609aee343fb3c4b28e1df9e632fca64fcfaede20f02e86244efddf30957bd2"
    );
}
```

## Updating Tests

The EIP-712 hashes are calculated using the Solidity examples under `solidity_examples/`. The Solidity contracts may be run using the [Ethereum Remix IDE][remix].

[eip-712]: https://eips.ethereum.org/EIPS/eip-712
[remix]: https://remix.ethereum.org
