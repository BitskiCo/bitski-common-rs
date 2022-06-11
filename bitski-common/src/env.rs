//! Utilities for parsing env variables.
//!
//! # Eager parsing
//!
//! Eager parsing is useful for exiting a program early when env variables are
//! invalid or missing. To implement eager parsing for env variables, read env
//! variables in object constructors and eagerly create dependencies on program
//! startup:
//!
//! ```rust
//! use bitski_common::Result;
//! use bitski_common::env::require_env;
//!
//! struct Candy {
//!     name: String
//! }
//!
//! impl Candy {
//!     fn new() -> Result<Self> {
//!         let name = require_env("CANDY_NAME")?;
//!         Ok(Self { name })
//!     }
//! }
//!
//! fn main() {
//!     init_env();
//!     let _candy = Candy::new().unwrap(); // Crash OK
//!
//!     // ...
//! }
//! ```

use std::fmt::Debug;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
#[cfg(feature = "humantime")]
use std::time::Duration;
use std::{env, net::ToSocketAddrs};

use crate::{Error, Result};

/// Initializes env variables from .env files.
pub fn init_env() {
    match dotenv::dotenv() {
        Ok(path) => tracing::info!("Loaded .env from {}", path.to_string_lossy()),
        Err(dotenv::Error::Io(err)) if err.kind() == ErrorKind::NotFound => (),
        Err(err) => tracing::warn!("Error loading .env: {err}"),
    }
}

/// Initializes env variables for tests using [`std::sync::Once`].
#[cfg(feature = "test")]
#[cfg_attr(docsrs, doc(cfg(feature = "test")))]
pub fn init_env_for_test() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        init_env();
    });
}

/// Parses the server listen from the `ADDR` env variable or a default value.
pub fn parse_env_addr_or<T>(default: T) -> Result<SocketAddr>
where
    T: ToSocketAddrs,
{
    let addr = if let Some(addr) = parse_env::<String>("ADDR")? {
        addr.to_socket_addrs()
            .map_err(|err| {
                Error::invalid_argument().with_message(format!("Error parsing env ADDR: {err}"))
            })?
            .next()
    } else {
        default
            .to_socket_addrs()
            .map_err(|err| {
                Error::invalid_argument()
                    .with_message(format!("Error parsing default value for env ADDR: {err}"))
            })?
            .next()
    };
    addr.ok_or_else(|| {
        Error::invalid_argument().with_message("Error parsing env ADDR: no address specified")
    })
}

/// Parses the server listen from the `ADDR` env variable or returns `127.0.0.1:8000`.
pub fn parse_env_addr_or_default() -> Result<SocketAddr> {
    parse_env_or_else("ADDR", || {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000)
    })
}

/// Parses a value from an env variable.
///
/// # Examples
///
/// ```rust
/// # use anyhow::Result;
/// # use bitski_common::env::parse_env;
/// #
/// # fn main() -> Result<()> {
/// let cargo_pkg_name: Option<String> = parse_env("CARGO_PKG_NAME")?;
/// assert_eq!(cargo_pkg_name, Some("bitski-common".into()));
///
/// let foobar: Option<u32> = parse_env("FOOBAR")?;
/// assert_eq!(foobar, None);
/// # Ok(())
/// # }
/// ```
pub fn parse_env<T>(name: &'static str) -> Result<Option<T>>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + Debug + Send + Sync + std::error::Error,
{
    match env::var(name) {
        Ok(s) => Ok(Some(s.parse().map_err(|err| {
            Error::invalid_argument().with_message(format!(
                "Error parsing env {name} as {}: {err}",
                std::any::type_name::<T>()
            ))
        })?)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => {
            Err(Error::invalid_argument().with_message(format!("Error parsing env {name}: {err}")))
        }
    }
}

/// Parses a required value from an env variable.
///
/// # Examples
///
/// ```rust
/// # use anyhow::Result;
/// # use bitski_common::env::require_env;
/// #
/// # fn main() -> Result<()> {
/// let cargo_pkg_name: String = require_env("CARGO_PKG_NAME")?;
/// assert_eq!(cargo_pkg_name, "bitski-common");
///
/// let foobar = require_env::<u32>("FOOBAR");
/// assert!(foobar.is_err());
/// # Ok(())
/// # }
/// ```
pub fn require_env<T>(name: &'static str) -> Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + Debug + Send + Sync + std::error::Error,
{
    match parse_env(name) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(Error::not_found().with_message(format!("Missing required env {name}"))),
        Err(err) => Err(err),
    }
}

/// Parses a value from an env variable or a default value.
///
/// # Examples
///
/// ```rust
/// # use anyhow::Result;
/// # use bitski_common::env::parse_env_or;
/// #
/// # fn main() -> Result<()> {
/// let foobar: String = parse_env_or("FOOBAR", "default")?;
/// assert_eq!(foobar, "default");
///
/// let val: u32 = parse_env_or("BARBAZ", 10)?;
/// assert_eq!(val, 10);
/// # Ok(())
/// # }
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
            Error::invalid_argument().with_message(format!(
                "Error parsing env {name} as {}: {err}",
                std::any::type_name::<T>()
            ))
        }),
        Err(env::VarError::NotPresent) => Ok(default.try_into().map_err(|err| {
            Error::invalid_argument()
                .with_message(format!("Error parsing default value for env {name}: {err}"))
        })?),
        Err(err) => {
            Err(Error::invalid_argument().with_message(format!("Error parsing env {name}: {err}")))
        }
    }
}

/// Parses a value from an env variable or a default value.
///
/// # Examples
///
/// ```rust
/// # use anyhow::Result;
/// # use bitski_common::env::parse_env_or_else;
/// #
/// # fn main() -> Result<()> {
/// let foobar: String = parse_env_or_else("FOOBAR", || "default")?;
/// assert_eq!(foobar, "default");
///
/// let val: u32 = parse_env_or_else("BARBAZ", || 10)?;
/// assert_eq!(val, 10);
/// # Ok(())
/// # }
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
            Error::invalid_argument().with_message(format!(
                "Error parsing env {name} as {}: {err}",
                std::any::type_name::<T>()
            ))
        }),
        Err(env::VarError::NotPresent) => Ok(default().try_into().map_err(|err| {
            Error::invalid_argument()
                .with_message(format!("Error parsing default value for env {name}: {err}"))
        })?),
        Err(err) => {
            Err(Error::invalid_argument().with_message(format!("Error parsing env {name}: {err}")))
        }
    }
}

/// Parses a value from an env variable or returns the default value.
///
/// # Examples
///
/// ```rust
/// # use anyhow::Result;
/// # use bitski_common::env::parse_env_or_default;
/// #
/// # fn main() -> Result<()> {
/// let foobar: String = parse_env_or_default("FOOBAR")?;
/// assert_eq!(foobar, "");
///
/// let val: u32 = parse_env_or_default("BARBAZ")?;
/// assert_eq!(val, 0);
/// # Ok(())
/// # }
/// ```
pub fn parse_env_or_default<T>(name: &'static str) -> Result<T>
where
    T: Default + FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    match env::var(name) {
        Ok(s) => s.parse().map_err(|err| {
            Error::invalid_argument().with_message(format!(
                "Error parsing env {name} as {}: {err}",
                std::any::type_name::<T>()
            ))
        }),
        Err(env::VarError::NotPresent) => Ok(Default::default()),
        Err(err) => {
            Err(Error::invalid_argument().with_message(format!("Error parsing env {name}: {err}")))
        }
    }
}

/// Parses a comma separated list of values from an env variable.
///
/// # Examples
///
/// ```rust
/// # use anyhow::Result;
/// # use bitski_common::env::parse_env_list;
/// #
/// # fn main() -> Result<()> {
/// std::env::set_var("FOOBAR", "foo,bar");
/// let foobar: Option<Vec<String>> = parse_env_list("FOOBAR")?;
/// assert_eq!(foobar, Some(vec!["foo".to_string(), "bar".to_string()]));
/// # Ok(())
/// # }
/// ```
pub fn parse_env_list<T>(name: &'static str) -> Result<Option<Vec<T>>>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + Debug + Send + Sync + std::error::Error,
{
    match env::var(name) {
        Ok(s) => {
            let mut list: Vec<T> = vec![];
            for ss in s.split_terminator(',') {
                let item = ss.trim().parse().map_err(|err| {
                    Error::invalid_argument().with_message(format!(
                        "Error parsing env {name} as {}: {err}",
                        std::any::type_name::<T>()
                    ))
                })?;
                list.push(item);
            }
            Ok(Some(list))
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => {
            Err(Error::invalid_argument().with_message(format!("Error parsing env {name}: {err}")))
        }
    }
}

/// Parses a comma separated list of required values from an env variable.
///
/// # Examples
///
/// ```rust
/// # use anyhow::Result;
/// # use bitski_common::env::require_env_list;
/// #
/// # fn main() -> Result<()> {
/// std::env::set_var("FOOBAR", "foo,bar");
/// let foobar: Vec<String> = require_env_list("FOOBAR")?;
/// assert_eq!(foobar, vec!["foo".to_string(), "bar".to_string()]);
///
/// let barbaz = require_env_list::<u32>("BARBAZ");
/// assert!(barbaz.is_err());
/// # Ok(())
/// # }
/// ```
pub fn require_env_list<T>(name: &'static str) -> Result<Vec<T>>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + Debug + Send + Sync + std::error::Error,
{
    match parse_env_list(name) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(Error::not_found().with_message(format!("Missing required env {name}"))),
        Err(err) => Err(err),
    }
}

/// Parses a value from an env variable or a default value.
///
/// # Examples
///
/// ```rust
/// # use anyhow::Result;
/// # use bitski_common::env::parse_env_list_or;
/// #
/// # fn main() -> Result<()> {
/// let foobar: Vec<String> = parse_env_list_or("FOOBAR", ["bar", "baz"])?;
/// assert_eq!(foobar, ["bar".to_string(), "baz".to_string()]);
///
/// let list: Vec<u32> = parse_env_list_or("BARBAZ", [10, 17])?;
/// assert_eq!(list, [10, 17]);
/// # Ok(())
/// # }
/// ```
pub fn parse_env_list_or<T, L, D>(name: &'static str, default: L) -> Result<Vec<T>>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + Debug + Send + Sync + std::error::Error,
    L: IntoIterator<Item = D>,
    D: TryInto<T>,
    <D as TryInto<T>>::Error: 'static + std::fmt::Debug + Send + Sync + std::error::Error,
{
    match parse_env_list::<T>(name) {
        Ok(Some(list)) => Ok(list),
        Ok(None) => {
            let mut list: Vec<T> = vec![];
            for item in default.into_iter() {
                let item = item.try_into().map_err(|err| {
                    Error::invalid_argument()
                        .with_message(format!("Error parsing default value for env {name}: {err}"))
                })?;
                list.push(item);
            }
            Ok(list)
        }
        Err(err) => Err(err),
    }
}

/// Parses a value from an env variable or a default value.
///
/// # Examples
///
/// ```rust
/// # use anyhow::Result;
/// # use bitski_common::env::parse_env_list_or_else;
/// #
/// # fn main() -> Result<()> {
/// let foobar: Vec<String> = parse_env_list_or_else("FOOBAR", || ["bar", "baz"])?;
/// assert_eq!(foobar, ["bar".to_string(), "baz".to_string()]);
///
/// let list: Vec<u32> = parse_env_list_or_else("BARBAZ", || [10, 17])?;
/// assert_eq!(list, [10, 17]);
/// # Ok(())
/// # }
/// ```
pub fn parse_env_list_or_else<T, L, D, F>(name: &'static str, default: F) -> Result<Vec<T>>
where
    T: FromStr + TryFrom<D>,
    <T as FromStr>::Err: 'static + Debug + Send + Sync + std::error::Error,
    L: IntoIterator<Item = D>,
    D: TryInto<T>,
    <D as TryInto<T>>::Error: 'static + std::fmt::Debug + Send + Sync + std::error::Error,
    F: FnOnce() -> L,
{
    match parse_env_list::<T>(name) {
        Ok(Some(list)) => Ok(list),
        Ok(None) => {
            let mut list: Vec<T> = vec![];
            for item in default().into_iter() {
                let item = item.try_into().map_err(|err| {
                    Error::invalid_argument()
                        .with_message(format!("Error parsing default value for env {name}: {err}"))
                })?;
                list.push(item);
            }
            Ok(list)
        }
        Err(err) => Err(err),
    }
}

/// Parses a [`Duration`] from an env variable or returns an empty list.
///
/// # Examples
///
/// ```rust
/// # use anyhow::Result;
/// # use bitski_common::env::parse_env_list_or_default;
/// #
/// # fn main() -> Result<()> {
/// let foobar: Vec<String> = parse_env_list_or_default("FOOBAR")?;
/// assert!(foobar.is_empty());
///
/// let list: Vec<u32> = parse_env_list_or_default("BARBAZ")?;
/// assert!(list.is_empty());
/// # Ok(())
/// # }
/// ```
pub fn parse_env_list_or_default<T>(name: &'static str) -> Result<Vec<T>>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + Debug + Send + Sync + std::error::Error,
{
    match parse_env_list::<T>(name) {
        Ok(Some(list)) => Ok(list),
        Ok(None) => Ok(Default::default()),
        Err(err) => Err(err),
    }
}

/// Parses a [`Duration`] from an env variable.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
///
/// # use anyhow::Result;
/// # use bitski_common::env::parse_env_duration;
/// #
/// # fn main() -> Result<()> {
/// let duration = parse_env_duration("DURATION")?;
/// assert_eq!(duration, None);
///
/// std::env::set_var("DURATION", "1s");
/// let duration = parse_env_duration("DURATION")?;
/// assert_eq!(duration, Some(Duration::from_secs(1)));
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "humantime")]
#[cfg_attr(docsrs, doc(cfg(feature = "humantime")))]
pub fn parse_env_duration(name: &'static str) -> Result<Option<Duration>> {
    if let Some(s) = parse_env::<String>(name)? {
        Ok(Some(humantime::parse_duration(&s)?))
    } else {
        Ok(None)
    }
}

/// Parses a required [`Duration`] from an env variable.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
///
/// # use anyhow::Result;
/// # use bitski_common::env::require_env_duration;
/// #
/// # fn main() -> Result<()> {
/// let duration = require_env_duration("DURATION");
/// assert!(duration.is_err());
///
/// std::env::set_var("DURATION", "1s");
/// let duration = require_env_duration("DURATION")?;
/// assert_eq!(duration, Duration::from_secs(1));
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "humantime")]
#[cfg_attr(docsrs, doc(cfg(feature = "humantime")))]
pub fn require_env_duration(name: &'static str) -> Result<Duration> {
    let s = require_env::<String>(name)?;
    Ok(humantime::parse_duration(&s)?)
}

/// Parses a [`Duration`] from an env variable or a default [`Duration`].
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
///
/// # use anyhow::Result;
/// # use bitski_common::env::parse_env_duration_or;
/// #
/// # fn main() -> Result<()> {
/// let duration = parse_env_duration_or("DURATION", Duration::from_secs(4))?;
/// assert_eq!(duration, Duration::from_secs(4));
///
/// std::env::set_var("DURATION", "1s");
/// let duration = parse_env_duration_or("DURATION", Duration::from_secs(4))?;
/// assert_eq!(duration, Duration::from_secs(1));
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "humantime")]
#[cfg_attr(docsrs, doc(cfg(feature = "humantime")))]
pub fn parse_env_duration_or(name: &'static str, default: Duration) -> Result<Duration> {
    Ok(parse_env_duration(name)?.unwrap_or(default))
}

/// Parses a [`Duration`] from an env variable or a default [`Duration`].
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
///
/// # use anyhow::Result;
/// # use bitski_common::env::parse_env_duration_or_else;
/// #
/// # fn main() -> Result<()> {
/// let duration = parse_env_duration_or_else("DURATION", || Duration::from_secs(4))?;
/// assert_eq!(duration, Duration::from_secs(4));
///
/// std::env::set_var("DURATION", "1s");
/// let duration = parse_env_duration_or_else("DURATION", || Duration::from_secs(4))?;
/// assert_eq!(duration, Duration::from_secs(1));
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "humantime")]
#[cfg_attr(docsrs, doc(cfg(feature = "humantime")))]
pub fn parse_env_duration_or_else<F>(name: &'static str, default: F) -> Result<Duration>
where
    F: FnOnce() -> Duration,
{
    Ok(parse_env_duration(name)?.unwrap_or_else(default))
}

/// Parses a [`Duration`] from an env variable or returns `Duration::ZERO`.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
///
/// # use anyhow::Result;
/// # use bitski_common::env::parse_env_duration_or_default;
/// #
/// # fn main() -> Result<()> {
/// let duration = parse_env_duration_or_default("DURATION")?;
/// assert_eq!(duration, Duration::ZERO);
///
/// std::env::set_var("DURATION", "1s");
/// let duration = parse_env_duration_or_default("DURATION")?;
/// assert_eq!(duration, Duration::from_secs(1));
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "humantime")]
#[cfg_attr(docsrs, doc(cfg(feature = "humantime")))]
pub fn parse_env_duration_or_default(name: &'static str) -> Result<Duration> {
    Ok(parse_env_duration(name)?.unwrap_or_default())
}
