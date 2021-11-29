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

use serde::Deserialize;
use web3::types::Address;
use web3::types::{H256, U256};

pub use crate::hasher::Hasher;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct TypedData {
    pub types: HashMap<String, Vec<MemberType>>,
    pub primary_type: String,
    pub domain: serde_json::Value,
    pub message: serde_json::Value,
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
