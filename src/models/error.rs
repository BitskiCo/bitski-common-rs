use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Could not decode JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Unknown coin type")]
    InvalidCoinType,
    #[error("Invalid data")]
    InvalidData,
    #[error("Invalid key")]
    Key(secp256k1::Error),
}
