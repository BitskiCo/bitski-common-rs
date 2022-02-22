use crate::models::coin_type::CoinType;
use crate::models::error::Error;
use crate::models::message::Message;
use crate::models::transaction_info::{TokenInfo, TransactionInfo};

use crate::models::wyvern;
use bigdecimal::BigDecimal;
use serde::Deserialize;
use web3::types::{Address, Bytes, BytesArray, U256};

#[cfg(feature = "ethereum")]
pub fn known_typed_data_meta_transaction(
    info: &bitski_eip_712::TypedData,
) -> Option<TransactionInfo> {
    match (
        info.domain["chainId"].as_u64().unwrap_or_default(),
        info.domain["verifyingContract"]
            .as_str()
            .unwrap_or_default(),
    ) {
        (chain_id, WYVERN_2_3_EXCHANGE_CONTRACT_ADDRESS) => {
            wyvern::parse_wyvern_meta_transaction(chain_id, info)
        }
        (chain_id, address) => {
            println!(
                "Don't know how to decode chain id {} with address {}",
                chain_id, address
            );
            None
        }
    }
}
