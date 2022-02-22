use crate::models::error::Error;
use crate::models::known_meta_transaction_types::known_typed_data_meta_transaction;
use crate::models::message::{MessageInfo, SignableMessage};
use crate::models::transaction_info::TransactionInfo;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum Message {
    String(String),
    TypedData(bitski_eip_712::TypedData),
}

impl crate::models::message::Message for Message {
    fn from_json(json: Value) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(serde_json::from_value(json)?)
    }

    fn from_raw(_bytes: &[u8]) -> Result<Self, Error>
    where
        Self: Sized,
    {
        todo!()
    }

    fn message_info(&self) -> MessageInfo {
        todo!()
    }

    fn meta_transaction_info(&self) -> Option<TransactionInfo> {
        match self {
            Message::String(_) => None,
            Message::TypedData(data) => known_typed_data_meta_transaction(data),
        }
    }
}

impl SignableMessage for Message {
    fn message_hash(&self, _chain_id: u64) -> Result<Vec<u8>, Error> {
        match self {
            Message::String(s) => Ok({
                let mut s = s.as_bytes().to_vec();
                let mut vec = format!("\x19Ethereum Signed Message:\n{}", s.len()).into_bytes();
                vec.append(&mut s);
                vec
            }),
            Message::TypedData(s) => s
                .hash()
                .map(|h| h.as_bytes().to_vec())
                .or(Err(Error::InvalidData)),
        }
    }
}
