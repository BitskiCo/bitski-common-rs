use crate::models::error::Error;
use crate::models::transaction_info::TransactionInfo;

pub trait Transaction {
    fn from_json(json: serde_json::Value) -> Result<Self, Error>
    where
        Self: Sized;

    fn from_raw(bytes: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;

    fn hash(&self) -> Vec<u8>;
}

pub trait IdentifyableTransction: Transaction {
    fn transaction_info(&self) -> TransactionInfo;
}

pub trait TransactionRequest {
    fn from_json(json: serde_json::Value) -> Result<Self, Error>
    where
        Self: Sized;
    fn from_raw(bytes: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;

    fn transaction_info(&self) -> TransactionInfo;
}

pub trait GasPricedTransactionRequest: TransactionRequest {
    fn gas_price(&self) -> String;
}

pub trait SignableTransactionRequest: TransactionRequest {
    fn message_hash(&self, chain_id: u64) -> Vec<u8>;
}

impl dyn SignableTransactionRequest {
    pub fn sign_transaction<E, F: Fn(&[u8]) -> Result<(Vec<u8>, u64), E>>(
        &self,
        chain_id: u64,
        provider: F,
    ) -> Result<(Vec<u8>, u64), E> {
        let hash = self.message_hash(chain_id);
        let (signature, recovery) = provider(&hash)?;
        Ok((signature, recovery))
    }
}
