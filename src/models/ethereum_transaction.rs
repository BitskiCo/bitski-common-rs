use crate::models::error::Error;
use crate::models::transaction::{IdentifyableTransction, SignableTransactionRequest, Transaction, TransactionRequest};
use crate::models::transaction_info::TransactionInfo;
use rlp::RlpStream;
use serde_json::Value;
use web3::types::{TransactionRequest as Web3TransactionRequest, TransactionParameters as Web3TransactionParameters, Transaction as Web3Transaction , U256};


const METHOD_LENGTH: usize = 12;

impl Transaction for Web3Transaction {
    fn from_json(json: Value) -> Result<Self, Error> {
        let transaction = serde_json::from_value(json)?;
        Ok(transaction)
    }

    fn from_raw(bytes: &[u8]) -> Result<Self, Error> {
        let transaction = serde_json::from_slice(bytes)?;
        Ok(transaction)
    }

    fn hash(&self) -> Vec<u8> {
        self.hash.0.to_vec()
    }
}

impl IdentifyableTransction for Web3Transaction {
    fn transaction_info(&self) -> TransactionInfo {
        let value = Some(serde_json::json!(self.value).as_str().unwrap().to_owned());
        let input = serde_json::json!(self.input).as_str().unwrap().to_owned();
        match (input.split_at(12), input.len()) {
            _ => TransactionInfo::Unknown { value },
        }
    }
}


fn rlp_append_unsigned(request: &Web3TransactionRequest, rlp: &mut RlpStream, chain_id: u64) {
        rlp.begin_list(9);
        rlp.append(&request.nonce);
        rlp.append(&request.gas_price);
        rlp.append(&request.gas);
        if let Some(to) = request.to {
            rlp.append(&to);
        } else {
            rlp.append(&"");
        }
        rlp.append(&request.value);
        rlp.append(&request.data.as_ref().map(|data| data.0.clone()));
        rlp.append(&chain_id);
        rlp.append(&0u8);
        rlp.append(&0u8);
}

impl TransactionRequest for Web3TransactionRequest {
    fn from_json(json: Value) -> Result<Self, Error> {
        let request = serde_json::from_value(json)?;
        Ok(request)
    }

    fn from_raw(bytes: &[u8]) -> Result<Self, Error> {
        let request = serde_json::from_slice(bytes)?;
        Ok(request)
    }

    fn transaction_info(&self) -> TransactionInfo {
        if self.value.clone().unwrap_or_default() > U256::zero()
            && self.to.is_some()
            && self.data.clone().unwrap_or_default().0.len() == 0
        {
            return TransactionInfo::TokenTransfer {
                from: serde_json::json!(self.from)
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                to: serde_json::json!(self.to)
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                amount: serde_json::json!(self.value)
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                token_id: None,
                token_info: None,
            };
        }

        let value = Some(
            serde_json::json!(self.value)
                .as_str()
                .unwrap_or_default()
                .to_owned(),
        );
        let input = serde_json::json!(self.data)
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
impl SignableTransactionRequest for Web3TransactionRequest {
    fn message_hash(&self, chain_id: u64) -> Vec<u8> {
        use web3::signing::keccak256;

        let mut rlp = RlpStream::new();
        rlp_append_unsigned(&self, &mut rlp, chain_id);

        let hash = keccak256(rlp.as_raw());
        Vec::from(hash)
    }
}

fn parameters_from_request(request: &Web3TransactionRequest, chain_id: Option<u64>) -> Result<Web3TransactionParameters, Error> {
    let gas = request.gas.clone().ok_or(Error::InvalidData)?;
    let value = request.value.clone().ok_or(Error::InvalidData)?;
    let data = request.data.clone().ok_or(Error::InvalidData)?;
    Ok(Web3TransactionParameters {
        nonce: request.nonce.clone(),
        gas_price: request.gas_price.clone(),
        gas,
        to: request.to.clone(),
        value,
        data,
        chain_id: chain_id,
        transaction_type: request.transaction_type.clone(),
        access_list: request.access_list.clone(),
    })
}

impl TransactionRequest for Web3TransactionParameters {
    fn from_json(json: Value) -> Result<Self, Error> {
        let chain_id = json["chainId"].as_u64();
        let request: Web3TransactionRequest = serde_json::from_value(json)?;
        let parameters = parameters_from_request(&request, chain_id)?;
        Ok(parameters)
    }

    fn from_raw(bytes: &[u8]) -> Result<Self, Error> {
        let request: Web3TransactionRequest = serde_json::from_slice(bytes)?;
        let parameters = parameters_from_request(&request, None)?;
        Ok(parameters)
    }

    fn transaction_info(&self) -> TransactionInfo {
        todo!()
    }
}

#[cfg(feature = "signing")]
impl SignableTransactionRequest for Web3TransactionParameters {
    fn message_hash(&self, _chain_id: u64) -> Vec<u8> {
        todo!()
    }
}