use crate::prelude::*;

pub trait Account: Sized {
    fn from_public_key(public_key: &[u8]) -> Result<Self>;
    fn address(&self) -> String;
}
