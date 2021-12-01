use crate::models::error::Error;

pub trait Account: Sized {
    fn from_public_key(public_key: &[u8]) -> Result<Self, Error>;
    fn address(&self) -> String;
}
