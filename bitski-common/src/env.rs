//! Utilities for parsing env variables.

use std::env;
use std::fmt::Debug;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::str::FromStr;

use crate::{Error, Result};

const DEFAULT_ADDR: &str = "127.0.0.1:8080";

/// Initializes env variables from .env files.
pub fn init_env() {
    match dotenv::dotenv() {
        Ok(path) => tracing::info!("Loaded .env from {}", path.to_string_lossy()),
        Err(dotenv::Error::Io(err)) if err.kind() == ErrorKind::NotFound => (),
        Err(err) => tracing::warn!("Error loading .env: {err}"),
    }
}

/// Parses the server listen from the `ADDR` env variable.
pub fn parse_env_addr() -> Result<SocketAddr> {
    parse_env_or_else("ADDR", || DEFAULT_ADDR.parse::<SocketAddr>().unwrap())
}

/// Parses a value from an env variable.
///
/// # Examples
///
/// ```rust
/// use bitski_common::env::parse_env;
///
/// let cargo_pkg_name: Option<String> = parse_env("CARGO_PKG_NAME").unwrap();
/// assert_eq!(cargo_pkg_name, Some("bitski-common".into()));
///
/// let foobar: Option<u32> = parse_env("__FOOBAR__").unwrap();
/// assert_eq!(foobar, None);
/// ```
pub fn parse_env<T>(name: &'static str) -> Result<Option<T>>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + Debug + Send + Sync + std::error::Error,
{
    match env::var(name) {
        Ok(s) => Ok(Some(s.parse().map_err(|err| {
            Error::internal().with_message(format!(
                "error parsing env {name} as {}: {err}",
                std::any::type_name::<T>()
            ))
        })?)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(Error::internal().with_message(format!("error parsing env {name}: {err}"))),
    }
}

/// Parses a value from an env variable or a default value.
///
/// # Examples
///
/// ```rust
/// use bitski_common::env::parse_env_or;
///
/// let foobar: String = parse_env_or("__FOOBAR__", "default").unwrap();
/// assert_eq!(foobar, "default");
///
/// let intval: u32 = parse_env_or("__INTVAL__", 10).unwrap();
/// assert_eq!(intval, 10);
/// ```
pub fn parse_env_or<T, D>(name: &'static str, default: D) -> Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + Debug + Send + Sync + std::error::Error,
    D: TryInto<T>,
    <D as TryInto<T>>::Error: 'static + std::fmt::Debug + Send + Sync + std::error::Error,
{
    match env::var(name) {
        Ok(s) => s.parse().map_err(|err| {
            Error::internal().with_message(format!(
                "error parsing env {name} as {}: {err}",
                std::any::type_name::<T>()
            ))
        }),
        Err(env::VarError::NotPresent) => Ok(default.try_into().map_err(|err| {
            Error::internal()
                .with_message(format!("error parsing default value for env {name}: {err}"))
        })?),
        Err(err) => Err(Error::internal().with_message(format!("error parsing env {name}: {err}"))),
    }
}

/// Parses a value from an env variable or a default value.
///
/// # Examples
///
/// ```rust
/// use bitski_common::env::parse_env_or_else;
///
/// let foobar: String = parse_env_or_else("__FOOBAR__", || "default").unwrap();
/// assert_eq!(foobar, "default");
///
/// let int_from_str: u32 = parse_env_or_else("__FOOBAR__", || 10).unwrap();
/// assert_eq!(int_from_str, 10);
/// ```
pub fn parse_env_or_else<T, D, F>(name: &'static str, default: F) -> Result<T>
where
    T: FromStr + TryFrom<D>,
    <T as FromStr>::Err: 'static + Debug + Send + Sync + std::error::Error,
    D: TryInto<T>,
    <D as TryInto<T>>::Error: 'static + std::fmt::Debug + Send + Sync + std::error::Error,
    F: FnOnce() -> D,
{
    match env::var(name) {
        Ok(s) => s.parse().map_err(|err| {
            Error::internal().with_message(format!(
                "error parsing env {name} as {}: {err}",
                std::any::type_name::<T>()
            ))
        }),
        Err(env::VarError::NotPresent) => Ok(default().try_into().map_err(|err| {
            Error::internal()
                .with_message(format!("error parsing default value for env {name}: {err}"))
        })?),
        Err(err) => Err(Error::internal().with_message(format!("error parsing env {name}: {err}"))),
    }
}
