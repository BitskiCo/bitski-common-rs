use thiserror::Error as ThisError;

pub type Result<T, E = Error>
where
    E: std::error::Error,
= std::result::Result<T, E>;

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
