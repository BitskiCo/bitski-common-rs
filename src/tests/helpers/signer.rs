use secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::ops::Deref;
use web3::signing::SigningError;

pub struct TestSigner {
    key: SecretKey,
}

impl Deref for TestSigner {
    type Target = SecretKey;

    fn deref(&self) -> &Self::Target {
        &self.key
    }
}

impl TestSigner {
    pub fn new() -> Self {
        let key = SecretKey::new(&mut rand::thread_rng());
        TestSigner { key }
    }

    pub fn public_key(&self) -> secp256k1::PublicKey {
        let context = Secp256k1::signing_only();
        PublicKey::from_secret_key(&context, &self.key)
    }

    pub fn ethereum_address(&self) -> web3::types::Address {
        web3::signing::Key::address(self)
    }

    pub fn sign_recoverable(
        &self,
        hash: &[u8],
        chain_id: Option<u64>,
    ) -> Result<(Vec<u8>, u64), SigningError> {
        let signature = web3::signing::Key::sign(self, hash, chain_id)?;
        let mut bytes = Vec::new();
        bytes.append(&mut signature.r.as_bytes().to_vec());
        bytes.append(&mut signature.s.as_bytes().to_vec());

        let v = if let Some(chain_id) = chain_id {
            signature.v - (35 + chain_id * 2)
        } else {
            signature.v - 27
        };

        Ok((bytes, v))
    }
}
