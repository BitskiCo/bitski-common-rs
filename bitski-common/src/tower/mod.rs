//! # Utilities for Tower servers.

mod span;

use std::time::Duration;

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
use crate::Result;

const DEFAULT_SERVER_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Bitski middleware layer.
///
/// # Examples
///
/// ```rust,no_run
/// use anyhow::Result;
/// use bitski_common::{
///     env::{init_env, parse_env_addr_or_default},
///     tower::{BitskiLayer, BitskiLayerExt as _},
///     with_instruments,
/// };
/// use hyper::header;
/// use tonic::transport::Server;
///
/// #[with_instruments]
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     init_env();
///     let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
///
///     // Empty string is convention for server health
///     health_reporter
///         .set_service_status("", tonic_health::ServingStatus::Serving)
///         .await;
///
///     let addr = parse_env_addr_or_default()?;
///     tracing::info!("Listening on {}", addr);
///
///     Server::builder()
///         .layer(BitskiLayer::from_env()?)
///         .add_service(health_service)
///         .serve(addr)
///         .await?;
///
///     Ok(())
/// }
/// ```
pub type BitskiLayer = Stack<
    CompressionLayer,
    Stack<
        TraceLayer<SharedClassifier<GrpcErrorsAsFailures>, PropagatingSpan>,
        Stack<SetSensitiveHeadersLayer, Stack<TimeoutLayer, Identity>>,
    >,
>;

/// An extension trait for [`BitskiLayer`] that provides a variety of convenient adapters.
pub trait BitskiLayerExt {
    fn from_env() -> Result<Self>
    where
        Self: Sized;
}

impl BitskiLayerExt for BitskiLayer {
    /// Creates a middleware stack from env variables.
    ///
    /// The [`BitskiLayer`] is configurable with the following env variables:
    ///
    /// * `SERVER_REQUEST_TIMEOUT_MS=10000` Server request timeout for the Otel `service.namespace` resource.
    fn from_env() -> Result<Self> {
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
}
