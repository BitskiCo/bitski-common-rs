#[cfg(feature = "tower")]
mod tower;

use anyhow::Result;
use tracing_subscriber::prelude::*;

use crate::env::parse_env_or_else;

#[cfg(feature = "tower")]
pub use self::tower::*;

/// Initializes OpenTelemetry for tracing.
pub fn init_instruments() -> Result<()> {
    let jaeger_service_name: String = parse_env_or_else("JAEGER_SERVICE_NAME", || {
        option_env!("CARGO_BIN_NAME").unwrap_or(env!("CARGO_PKG_NAME"))
    })?;

    opentelemetry::global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());

    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(jaeger_service_name)
        .install_batch(opentelemetry::runtime::TokioCurrentThread)?;

    tracing_subscriber::Registry::default()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_ansi(false))
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    Ok(())
}

/// Shuts down OpenTelemetry trace providers.
pub fn shutdown_instruments() -> Result<()> {
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
