pub mod models;
#[cfg(test)]
pub mod tests;

mod prelude;

pub use web3::types as web3_types;

use crate::models::coin_type::CoinType;
use crate::prelude::*;

pub fn known_transaction_request_type_from_json(
    json: serde_json::Value,
    coin_type: CoinType,
    chain_id: Option<u64>,
) -> Result<models::known_transaction_type::KnownTransactionRequestType> {
    models::known_transaction_type::KnownTransactionRequestType::from_json(
        json, coin_type, chain_id,
    )
}
