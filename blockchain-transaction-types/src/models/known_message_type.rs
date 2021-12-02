use crate::models::coin_type::CoinType;
use crate::models::error::Error;

pub enum KnownMessageType {
    Ethereum(crate::models::ethereum_message::Message),
}

impl KnownMessageType {
    pub fn message(&self) -> &dyn crate::models::message::Message {
        match self {
            KnownMessageType::Ethereum(message) => message,
        }
    }

    #[cfg(feature = "signing")]
    pub fn signable_message(&self) -> Box<dyn crate::models::message::SignableMessage> {
        match self {
            KnownMessageType::Ethereum(message) => Box::new(message.clone()),
        }
    }
}

impl KnownMessageType {
    pub fn from_json(
        value: serde_json::Value,
        coin_type: CoinType,
        _chain_id: Option<u64>,
    ) -> Result<KnownMessageType, Error> {
        match coin_type {
            CoinType::Ethereum => {
                let transaction: crate::models::ethereum_message::Message =
                    serde_json::from_value(value)?;
                Ok(KnownMessageType::Ethereum(transaction))
            }
            _ => Err(Error::InvalidCoinType),
        }
    }
}
