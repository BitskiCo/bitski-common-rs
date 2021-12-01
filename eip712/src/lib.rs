//! [EIP-712] Ethereum typed structured data.
//! [EIP-712]: https://eips.ethereum.org/EIPS/eip-712

extern crate anyhow;
extern crate hex;
extern crate num;
extern crate regex;
extern crate web3;

mod hasher;
mod types;

use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;
use web3::types::Address;
use web3::types::{H256, U256};

use crate::hasher::Hasher;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct TypedData {
    pub types: HashMap<String, Vec<MemberType>>,
    pub primary_type: String,
    pub domain: serde_json::Value,
    pub message: serde_json::Value,
}

impl TypedData {
    pub fn hash(&self) -> Result<H256> {
        Hasher::try_from(self)?.hash(self)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct MemberType {
    pub name: String,
    pub r#type: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Domain {
    pub name: Option<String>,
    pub version: Option<String>,
    pub chain_id: Option<U256>,
    pub verifying_contract: Option<Address>,
    pub salt: Option<H256>,
}

#[cfg(test)]
mod tests {
    use hex::ToHex as _;
    use serde_json::json;

    use super::*;

    #[derive(Default)]
    struct BufHasher(Vec<u8>);

    impl std::hash::Hasher for BufHasher {
        fn write(&mut self, bytes: &[u8]) {
            self.0.extend(bytes);
        }
        fn finish(&self) -> u64 {
            panic!("unexpected call");
        }
    }

    #[test]
    fn hasher_try_from_typed_data_ok() {
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
        }))
        .unwrap();

        assert_eq!(
            format!("{}", typed_data.hash().unwrap().encode_hex::<String>()),
            "be609aee343fb3c4b28e1df9e632fca64fcfaede20f02e86244efddf30957bd2"
        );
    }
}
