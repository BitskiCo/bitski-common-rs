use crate::models::error::Error;
use crate::models::transaction::{SignableTransactionRequest, Transaction, TransactionRequest};
use crate::models::transaction_info::TransactionInfo;
use rlp::RlpStream;
use serde_json::Value;
use web3::types::{TransactionRequest as Web3TransactionRequest, U256};

pub struct EthereumTransaction(web3::types::Transaction);

const METHOD_LENGTH: usize = 12;

impl Transaction for EthereumTransaction {
    fn from_json(json: Value) -> Result<Self, Error> {
        let transaction = serde_json::from_value(json)?;
        Ok(EthereumTransaction(transaction))
    }

    fn from_raw(bytes: &[u8]) -> Result<Self, Error> {
        let transaction = serde_json::from_slice(bytes)?;
        Ok(EthereumTransaction(transaction))
    }

    fn transaction_info(&self) -> TransactionInfo {
        let value = Some(serde_json::json!(self.0.value).as_str().unwrap().to_owned());
        let input = serde_json::json!(self.0.input).as_str().unwrap().to_owned();
        match (input.split_at(12), input.len()) {
            _ => TransactionInfo::Unknown { value },
        }
    }

    fn hash(&self) -> Vec<u8> {
        self.0.hash.0.to_vec()
    }
}

#[derive(Clone)]
pub struct EthereumTransactionRequest(Web3TransactionRequest);

impl EthereumTransactionRequest {
    fn rlp_append_unsigned(&self, rlp: &mut RlpStream, chain_id: u64) {
        rlp.begin_list(9);
        rlp.append(&self.0.nonce);
        rlp.append(&self.0.gas_price);
        rlp.append(&self.0.gas);
        if let Some(to) = self.0.to {
            rlp.append(&to);
        } else {
            rlp.append(&"");
        }
        rlp.append(&self.0.value);
        rlp.append(&self.0.data.as_ref().map(|data| data.0.clone()));
        rlp.append(&chain_id);
        rlp.append(&0u8);
        rlp.append(&0u8);
    }
}

impl TransactionRequest for EthereumTransactionRequest {
    fn from_json(json: Value) -> Result<Self, Error> {
        let request = serde_json::from_value(json)?;
        Ok(Self(request))
    }

    fn from_raw(bytes: &[u8]) -> Result<Self, Error> {
        let request = serde_json::from_slice(bytes)?;
        Ok(Self(request))
    }

    fn transaction_info(&self) -> TransactionInfo {
        if self.0.value.clone().unwrap_or_default() > U256::zero()
            && self.0.to.is_some()
            && self.0.data.clone().unwrap_or_default().0.len() == 0
        {
            return TransactionInfo::TokenTransfer {
                from: serde_json::json!(self.0.from)
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                to: serde_json::json!(self.0.to)
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                amount: serde_json::json!(self.0.value)
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                token_id: None,
                token_info: None,
            };
        }

        let value = Some(
            serde_json::json!(self.0.value)
                .as_str()
                .unwrap_or_default()
                .to_owned(),
        );
        let input = serde_json::json!(self.0.data)
            .as_str()
            .unwrap_or_default()
            .to_owned();
        let method = if input.len() > METHOD_LENGTH {
            input.clone()[0..METHOD_LENGTH].to_string()
        } else {
            String::new()
        };

        match (method.as_str(), input.len()) {
            _ => TransactionInfo::Unknown { value },
        }
    }
}

#[cfg(feature = "signing")]
impl SignableTransactionRequest for EthereumTransactionRequest {
    fn message_hash(&self, chain_id: u64) -> Vec<u8> {
        use web3::signing::keccak256;

        let mut rlp = RlpStream::new();
        self.rlp_append_unsigned(&mut rlp, chain_id);

        let hash = keccak256(rlp.as_raw());
        Vec::from(hash)
    }
}
