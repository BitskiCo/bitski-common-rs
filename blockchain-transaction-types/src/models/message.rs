use crate::models::error::Error;

pub trait Message {
    fn from_json(json: serde_json::Value) -> Result<Self, Error>
    where
        Self: Sized;

    fn from_raw(bytes: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;

    fn message_info(&self) -> MessageInfo;
}

pub enum MessageInfo {
    String(String),
    Json(serde_json::Value),
}
