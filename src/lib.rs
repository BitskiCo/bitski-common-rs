use crate::models::error::Error;

pub use web3::types as web3_types;

pub mod models;
#[cfg(test)]
pub mod tests;

pub fn known_transaction_request_type_from_json(
    json: serde_json::Value,
    coin_type: u64,
    chain_id: Option<u64>,
) -> Result<models::known_transaction_type::KnownTransactionRequestType, Error> {
    models::known_transaction_type::KnownTransactionRequestType::from_json(
        json, coin_type, chain_id,
    )
}
