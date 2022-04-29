//! # bitski-common
//!
//! Example usage:
//!
//! ```rust,no_run
//! use std::time::Duration;
//!
//! use anyhow::Result;
//! use hyper::header;
//! use tonic::transport::Server;
//! use tower::ServiceBuilder;
//! use tower_http::{
//!     classify::{GrpcCode, GrpcErrorsAsFailures, SharedClassifier},
//!     compression::CompressionLayer,
//!     sensitive_headers::SetSensitiveHeadersLayer,
//!     trace::TraceLayer,
//! };
//!
//! // Use feature gate for tests: https://github.com/rust-lang/rust/issues/93083
//! #[cfg(feature = "tower")]
//! use bitski_common::{
//!     env::{init_env, parse_env_addr},
//!     instrumentation::{init_instruments, shutdown_instruments, tower::PropagatingSpan},
//! };
//!
//! #[cfg(feature = "tower")]
//! async fn serve<T>() -> Result<()> {
//!     init_env();
//!     init_instruments()?;
//!
//!     let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
//!
//!     // Empty string is convention for server health
//!     health_reporter
//!         .set_service_status("", tonic_health::ServingStatus::Serving)
//!         .await;
//!
//!     let classifier = GrpcErrorsAsFailures::new()
//!         .with_success(GrpcCode::InvalidArgument)
//!         .with_success(GrpcCode::NotFound);
//!
//!     let middleware = ServiceBuilder::new()
//!         .timeout(Duration::from_secs(10))
//!         .layer(CompressionLayer::new())
//!         .layer(SetSensitiveHeadersLayer::new(vec![header::AUTHORIZATION]))
//!         .layer(
//!             TraceLayer::new(SharedClassifier::new(classifier))
//!                 .make_span_with(PropagatingSpan::new()),
//!         )
//!         .into_inner();
//!
//!     let addr = parse_env_addr()?;
//!     tracing::info!("Listening on {}", addr);
//!
//!     Server::builder()
//!         .layer(middleware)
//!         .add_service(health_service)
//!         .serve(addr)
//!         .await?;
//!
//!     shutdown_instruments()?;
//!
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "diesel")]
#[cfg_attr(docsrs, doc(cfg(feature = "diesel")))]
pub mod diesel;
pub mod env;
pub mod instrumentation;
pub mod task;
