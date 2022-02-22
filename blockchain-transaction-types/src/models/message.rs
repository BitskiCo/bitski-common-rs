use crate::models::error::Error;
use crate::models::transaction_info::TransactionInfo;
use std::future::Future;
use thiserror::Error as ThisError;

pub trait Message {
    fn from_json(json: serde_json::Value) -> Result<Self, Error>
    where
        Self: Sized;

    fn from_raw(bytes: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;

    fn message_info(&self) -> MessageInfo;

    fn meta_transaction_info(&self) -> Option<TransactionInfo>;
}

pub enum MessageInfo {
    String(String),
    Json(serde_json::Value),
}

pub trait SignableMessage: Message {
    fn message_hash(&self, chain_id: u64) -> Result<Vec<u8>, Error>;
}

#[derive(Debug, ThisError)]
pub enum SignError<E> {
    Hash(Error),
    Sign(E),
}

impl dyn SignableMessage {
    pub async fn sign_message<
        E,
        O: Future<Output = Result<(Vec<u8>, u64), E>>,
        F: FnOnce(Vec<u8>) -> O,
    >(
        &self,
        chain_id: u64,
        provider: F,
    ) -> Result<(Vec<u8>, u64), SignError<E>> {
        let hash = self
            .message_hash(chain_id)
            .map_err(|error| SignError::<E>::Hash(error))?;
        let (signature, recovery) = provider(hash)
            .await
            .map_err(|error| SignError::<E>::Sign(error))?;
        Ok((signature, recovery))
    }
}
