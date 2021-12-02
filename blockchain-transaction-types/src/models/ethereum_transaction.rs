#[cfg(feature = "signing")]
use rlp::RlpStream;
use serde_json::Value;
use web3::types::{
    Address, Transaction as Web3Transaction, TransactionParameters as Web3TransactionParameters,
    TransactionRequest as Web3TransactionRequest, U256,
};

use crate::models::error::Error;
use crate::models::transaction::{
    IdentifyableTransction, SignableTransactionRequest, Transaction, TransactionRequest,
};
use crate::models::transaction_info::TransactionInfo;

#[cfg(feature = "signing")]
const EIP_1559_TRANSACTION_TYPE: u64 = 2;
#[cfg(feature = "signing")]
const EIP_2930_TRANSACTION_TYPE: u64 = 1;
const METHOD_LENGTH: usize = 10;

impl Transaction for Web3Transaction {
    type Account = Address;

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

    fn sender(&self) -> Option<Self::Account> {
        self.from
    }
}

fn safe_transfer_from_transaction_info(data: &str) -> TransactionInfo {
    let _method = data[2..10].to_string();
    let from = data[10..74]
        .trim_start_matches("000000000000000000000000")
        .to_string();
    let to = data[74..138]
        .trim_start_matches("000000000000000000000000")
        .to_string();
    let id = data[138..202].to_string();
    let value = data[202..266].to_string();
    let _data = data[266..].to_string();
    TransactionInfo::TokenTransfer {
        from: format!("0x{}", from),
        to: format!("0x{}", to),
        amount: format!("0x{}", value),
        token_id: Some(format!("0x{}", id)),
        token_info: None,
    }
}

const SAFE_TRANSFER_FROM: &'static str = "0xf242432a";

impl IdentifyableTransction for Web3Transaction {
    fn transaction_info(&self) -> TransactionInfo {
        let value = Some(serde_json::json!(self.value).as_str().unwrap().to_owned());
        let input = serde_json::json!(self.input).as_str().unwrap().to_owned();
        match input.split_at(10).0 {
            SAFE_TRANSFER_FROM => safe_transfer_from_transaction_info(&input),
            _ => TransactionInfo::Unknown { value },
        }
    }
}

/// RLP-encode an unsigned legacy transaction request.
///
/// The encoding is defined in [EIP-2712][eip-2718] as
/// `rlp([nonce, gasprice, startgas, to, value, data, chainid, 0, 0])`.
///
/// [eip-2718]: https://eips.ethereum.org/EIPS/eip-2718
#[cfg(feature = "signing")]
fn rlp_append_unsigned_legacy(
    request: &Web3TransactionRequest,
    rlp: &mut RlpStream,
    chain_id: u64,
) -> Result<(), Error> {
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

    Ok(())
}

/// RLP-encode an unsigned transaction request with optional access list.
///
/// The encoding is defined in [EIP-2930][eip-2930] as
///
/// `rlp([chainId, nonce, gasPrice, gasLimit, to, value, data, accessList])`
///
/// where `access_list` is
///
/// `[[accessed_addresses{20 bytes}, [accessed_storage_keys{32 bytes}...]]...]`
///
/// [eip-2930]: https://eips.ethereum.org/EIPS/eip-2930
#[cfg(feature = "signing")]
fn rlp_append_unsigned_eip_2930(
    request: &Web3TransactionRequest,
    rlp: &mut RlpStream,
    chain_id: u64,
) -> Result<(), Error> {
    rlp.begin_list(8);
    rlp.append(&chain_id);
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
    if let Some(access_list) = &request.access_list {
        for item in access_list.iter() {
            rlp.begin_list(2);
            rlp.append(&item.address);
            rlp.begin_list(item.storage_keys.len());
            for key in item.storage_keys.iter() {
                rlp.append(key);
            }
        }
    }

    Ok(())
}

/// RLP-encode an unsigned transaction request for EIP-1559.
///
/// The encoding is defined in [EIP-1559][eip-1559] as
///
/// `rlp([chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas_limit, destination, amount, data, access_list])`
///
/// where `access_list` is
///
/// `[[accessed_addresses{20 bytes}, [accessed_storage_keys{32 bytes}...]]...]`
///
/// [eip-1559]: https://eips.ethereum.org/EIPS/eip-1559
#[cfg(feature = "signing")]
fn rlp_append_unsigned_eip_1559(
    request: &Web3TransactionRequest,
    rlp: &mut RlpStream,
    chain_id: u64,
) -> Result<(), Error> {
    rlp.begin_list(9);
    rlp.append(&chain_id);
    rlp.append(&request.nonce);
    rlp.append(&request.max_priority_fee_per_gas);
    rlp.append(&request.max_fee_per_gas);
    rlp.append(&request.gas);
    if let Some(to) = request.to {
        rlp.append(&to);
    } else {
        rlp.append(&"");
    }
    rlp.append(&request.value);
    rlp.append(&request.data.as_ref().map(|data| data.0.clone()));
    if let Some(access_list) = &request.access_list {
        for item in access_list.iter() {
            rlp.begin_list(2);
            rlp.append(&item.address);
            rlp.begin_list(item.storage_keys.len());
            for key in item.storage_keys.iter() {
                rlp.append(key);
            }
        }
    }

    Ok(())
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

        match method.as_str() {
            SAFE_TRANSFER_FROM => safe_transfer_from_transaction_info(&input),
            _ => TransactionInfo::Unknown { value },
        }
    }
}

#[cfg(feature = "signing")]
impl SignableTransactionRequest for Web3TransactionRequest {
    fn message_hash(&self, chain_id: u64) -> Result<Vec<u8>, Error> {
        use web3::signing::keccak256;
        let mut rlp = RlpStream::new();

        match self.transaction_type.map(|t| t.as_u64()) {
            Some(EIP_1559_TRANSACTION_TYPE) => {
                // EIP-1559 transaction (Fee market change for ETH 1.0 chain)
                if self.gas_price.is_some() {
                    return Err(Error::InvalidData);
                }
                rlp_append_unsigned_eip_1559(self, &mut rlp, chain_id)?;
            }
            Some(EIP_2930_TRANSACTION_TYPE) => {
                // EIP-2930 transaction (Optional access lists)
                if self.max_fee_per_gas.is_some() || self.max_priority_fee_per_gas.is_some() {
                    return Err(Error::InvalidData);
                }
                rlp_append_unsigned_eip_2930(self, &mut rlp, chain_id)?;
            }
            Some(transaction_type)
                if transaction_type <= 0x7fu64 || transaction_type == 0xffu64 =>
            {
                return Err(Error::InvalidData);
            }
            _ => {
                // Legacy transaction
                if self.access_list.is_some()
                    || self.max_fee_per_gas.is_some()
                    || self.max_priority_fee_per_gas.is_some()
                {
                    return Err(Error::InvalidData);
                }
                rlp_append_unsigned_legacy(self, &mut rlp, chain_id)?;
            }
        }

        let hash = keccak256(rlp.as_raw());
        Ok(Vec::from(hash))
    }
}

fn parameters_from_request(
    request: &Web3TransactionRequest,
    chain_id: Option<u64>,
) -> Result<Web3TransactionParameters, Error> {
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
        max_fee_per_gas: request.max_fee_per_gas,
        max_priority_fee_per_gas: request.max_priority_fee_per_gas,
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
    fn message_hash(&self, _chain_id: u64) -> Result<Vec<u8>, Error> {
        todo!()
    }
}
