use crate::models::error::Error;
use crate::models::message::{MessageInfo, SignableMessage};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum Message {
    String(String),
}

impl crate::models::message::Message for Message {
    fn from_json(_json: Value) -> Result<Self, Error>
    where
        Self: Sized,
    {
        todo!()
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
        }
    }
}
