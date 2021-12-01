use crate::prelude::*;

pub trait Message {
    fn from_json(json: serde_json::Value) -> Result<Self>
    where
        Self: Sized;

    fn from_raw(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;

    fn message_info(&self) -> MessageInfo;
}

pub enum MessageInfo {
    String(String),
    Json(serde_json::Value),
}
