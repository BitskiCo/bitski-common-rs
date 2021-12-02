use crate::models::account::Account;
use crate::models::error::Error;
use crate::models::transaction_info::TransactionInfo;
use std::future::Future;

pub trait Transaction {
    type Account: Account;
    fn from_json(json: serde_json::Value) -> Result<Self, Error>
    where
        Self: Sized;

    fn from_raw(bytes: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;

    fn hash(&self) -> Vec<u8>;

    fn sender(&self) -> Option<Self::Account>;
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
    pub async fn sign_transaction<
        E,
        O: Future<Output = Result<(Vec<u8>, u64), E>>,
        F: FnOnce(Vec<u8>) -> O,
    >(
        &self,
        chain_id: u64,
        provider: F,
    ) -> Result<(Vec<u8>, u64), E> {
        let hash = self.message_hash(chain_id);
        let (signature, recovery) = provider(hash).await?;
        Ok((signature, recovery))
    }
}
