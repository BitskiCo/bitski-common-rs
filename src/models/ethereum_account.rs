use secp256k1::PublicKey;
use tiny_keccak::{Hasher, Keccak};

use crate::models::account::Account;
use crate::prelude::*;

impl Account for web3::types::Address {
    fn from_public_key(public_key_data: &[u8]) -> Result<Self> {
        let public_key = PublicKey::from_slice(public_key_data).map_err(Error::Key)?;
        let public_key = public_key.serialize_uncompressed();
        println!("Public key len: {}", public_key.len());
        debug_assert_eq!(public_key[0], 0x04);
        let hash = keccak256(&public_key[1..]);

        Ok(Self::from_slice(&hash[12..]))
    }

    fn address(&self) -> String {
        format!("{:#?}", self)
    }
}

fn keccak256(bytes: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    hasher.finalize(&mut output);
    output
}
