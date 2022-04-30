//! # Utilities for Tower servers.
//!
//! Example usage:
//!
//! ```rust,no_run
//! use anyhow::Result;
//! use bitski_common::{
//!     env::{init_env, parse_env_addr},
//!     telemetry::{init_instruments, shutdown_instruments},
//!     tower::middleware,
//! };
//! use hyper::header;
//! use tonic::transport::Server;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
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
//!     let addr = parse_env_addr()?;
//!     tracing::info!("Listening on {}", addr);
//!
//!     Server::builder()
//!         .layer(middleware()?)
//!         .add_service(health_service)
//!         .serve(addr)
//!         .await?;
//!
//!     shutdown_instruments()?;
//!
//!     Ok(())
//! }
//! ```

mod span;

use std::time::Duration;

use anyhow::Result;
use hyper::header;
use tower::{
    layer::util::{Identity, Stack},
    timeout::TimeoutLayer,
    ServiceBuilder,
};
use tower_http::{
    classify::{GrpcCode, GrpcErrorsAsFailures, SharedClassifier},
    compression::CompressionLayer,
    sensitive_headers::SetSensitiveHeadersLayer,
    trace::TraceLayer,
};

pub use self::span::*;
use crate::env::parse_env;

const DEFAULT_SERVER_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Initializes a middleware stack for use with Tonic.
#[allow(clippy::type_complexity)]
pub fn middleware() -> Result<
    Stack<
        CompressionLayer,
        Stack<
            TraceLayer<SharedClassifier<GrpcErrorsAsFailures>, PropagatingSpan>,
            Stack<SetSensitiveHeadersLayer, Stack<TimeoutLayer, Identity>>,
        >,
    >,
> {
    let server_request_timeout = parse_env("SERVER_REQUEST_TIMEOUT_MS")?
        .map(Duration::from_millis)
        .unwrap_or(DEFAULT_SERVER_REQUEST_TIMEOUT);

    let classifier = GrpcErrorsAsFailures::new()
        .with_success(GrpcCode::InvalidArgument)
        .with_success(GrpcCode::NotFound);

    let stack = ServiceBuilder::new()
        .timeout(server_request_timeout)
        .layer(SetSensitiveHeadersLayer::new(vec![header::AUTHORIZATION]))
        .layer(
            TraceLayer::new(SharedClassifier::new(classifier))
                .make_span_with(PropagatingSpan::new()),
        )
        .layer(CompressionLayer::new())
        .into_inner();

    Ok(stack)
}
