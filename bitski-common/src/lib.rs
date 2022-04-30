//! # bitski-common
//!
//! Bitski utilities for common tasks.

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "actix-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
pub mod actix_web;
#[cfg(feature = "diesel")]
#[cfg_attr(docsrs, doc(cfg(feature = "diesel")))]
pub mod diesel;
pub mod env;
pub mod error;
pub mod task;
pub mod telemetry;
#[cfg(feature = "tower")]
#[cfg_attr(docsrs, doc(cfg(feature = "tower")))]
pub mod tower;

pub use crate::error::Error;

/// [`Result`] with a default error type of [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;
