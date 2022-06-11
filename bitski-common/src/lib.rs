#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "actix-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
pub mod actix_web;
#[cfg(all(feature = "diesel", feature = "postgres", feature = "r2d2"))]
#[cfg_attr(docsrs, doc(cfg(feature = "diesel")))]
pub mod diesel;
pub mod env;
pub mod error;
pub mod task;
pub mod telemetry;
#[cfg(feature = "tower")]
#[cfg_attr(docsrs, doc(cfg(feature = "tower")))]
pub mod tower;

#[cfg(feature = "actix-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
pub use actix_web_opentelemetry;
#[cfg(feature = "humantime")]
#[cfg_attr(docsrs, doc(cfg(feature = "humantime")))]
pub use humantime;
pub use opentelemetry;

pub use crate::error::Error;

/// [`Result`] with a default error type of [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;
