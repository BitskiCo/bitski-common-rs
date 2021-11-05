use crate::models::error::Error;
use crate::models::ethereum_transaction::EthereumTransactionRequest;
use crate::models::transaction::{SignableTransactionRequest, TransactionRequest};

pub enum KnownTransactionRequestType {
    Ethereum(EthereumTransactionRequest),
}

impl KnownTransactionRequestType {
    pub fn transaction_request(&self) -> &dyn TransactionRequest {
        match self {
            Self::Ethereum(tx) => tx,
        }
    }

    pub fn signable_transaction_request(self) -> Box<dyn SignableTransactionRequest> {
        match self {
            Self::Ethereum(tx) => Box::new(tx),
        }
    }
}

impl KnownTransactionRequestType {
    pub fn from_json(
        json: serde_json::Value,
        coin_type: u64,
        _chain_id: Option<u64>,
    ) -> Result<KnownTransactionRequestType, Error> {
        match coin_type {
            60 => {
                let transaction = EthereumTransactionRequest::from_json(json)?;
                Ok(KnownTransactionRequestType::Ethereum(transaction))
            }
            _ => Err(Error::InvalidCoinType),
        }
    }
}
