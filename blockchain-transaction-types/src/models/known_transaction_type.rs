use crate::models::coin_type::CoinType;
use crate::models::error::Error;
use crate::models::transaction::{SignableTransactionRequest, TransactionRequest};

pub enum KnownTransactionRequestType {
    Ethereum(web3::types::TransactionRequest),
    Solana(solana_sdk::transaction::Transaction),
}

impl KnownTransactionRequestType {
    pub fn transaction_request(&self) -> &dyn TransactionRequest {
        match self {
            Self::Ethereum(tx) => tx,
            Self::Solana(_tx) => unimplemented!("Cant handle Solana yet"),
        }
    }

    // pub fn sender(&self) -> &dyn Account {
    //     match self {
    //         Self::Ethereum(tx) => &tx.from,
    //         Self::Solana(_tx) => unimplemented!("Cant handle Solana yet")
    //     }
    // }

    #[cfg(feature = "signing")]
    pub fn signable_transaction_request(self) -> Box<dyn SignableTransactionRequest> {
        match self {
            Self::Ethereum(tx) => Box::new(tx),
            Self::Solana(_tx) => unimplemented!("Can't sign Solana yet"),
        }
    }
}

impl KnownTransactionRequestType {
    pub fn from_json(
        value: serde_json::Value,
        coin_type: CoinType,
        _chain_id: Option<u64>,
    ) -> Result<KnownTransactionRequestType, Error> {
        match coin_type {
            CoinType::Ethereum => {
                let transaction = serde_json::from_value(value)?;
                Ok(KnownTransactionRequestType::Ethereum(transaction))
            }
            CoinType::Solana => {
                let transaction = serde_json::from_value(value)?;
                Ok(KnownTransactionRequestType::Solana(transaction))
            }
            _ => Err(Error::InvalidCoinType),
        }
    }
}
