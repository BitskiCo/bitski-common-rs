//! Utilities for parsing env variables.

use std::env;
use std::fmt::Debug;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Once;

use anyhow::{Context as _, Result};

const DEFAULT_ADDR: &str = "127.0.0.1:8080";

static INIT_ONCE: Once = Once::new();

/// Initializes env variables from .env files.
pub fn init_env() {
    INIT_ONCE.call_once(|| match dotenv::dotenv() {
        Ok(path) => tracing::info!("Loaded .env from {}", path.to_string_lossy()),
        Err(dotenv::Error::Io(err)) if err.kind() == ErrorKind::NotFound => (),
        Err(err) => tracing::warn!("Error loading .env: {err}"),
    });
}

/// Parses the server listen from the `ADDR` env variable.
pub fn parse_env_addr() -> Result<SocketAddr> {
    parse_env_or_else("ADDR", || DEFAULT_ADDR.parse::<SocketAddr>().unwrap())
}

/// Parses a value from an env variable.
///
/// Example:
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
        Ok(s) => Ok(Some(s.parse().context(format!(
            "error parsing env {name} as {}",
            std::any::type_name::<T>()
        ))?)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(err).context(format!("error parsing env {name}")),
    }
}

/// Parses a value from an env variable or a default value.
///
/// Example:
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
        Ok(s) => s.parse().context(format!(
            "error parsing env {name} as {}",
            std::any::type_name::<T>()
        )),
        Err(env::VarError::NotPresent) => Ok(default
            .try_into()
            .with_context(|| format!("error parsing default value for env {name}"))?),
        Err(err) => Err(err).context(format!("error parsing env {name}")),
    }
}

/// Parses a value from an env variable or a default value.
///
/// Example:
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
        Ok(s) => s.parse().context(format!(
            "error parsing env {name} as {}",
            std::any::type_name::<T>()
        )),
        Err(env::VarError::NotPresent) => Ok(default()
            .try_into()
            .with_context(|| format!("error parsing default value for env {name}"))?),
        Err(err) => Err(err).context(format!("error parsing env {name}")),
    }
}
