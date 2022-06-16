//! # Utilities for telemetry.
//!
//! See [`with_instruments`][`bitski_common_macros::with_instruments`].

use opentelemetry::{
    sdk::{metrics::PushController, trace, Resource},
    util::tokio_interval_stream,
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_semantic_conventions::resource::{
    SERVICE_INSTANCE_ID, SERVICE_NAME, SERVICE_NAMESPACE, SERVICE_VERSION,
};
use tracing_subscriber::prelude::*;
use uuid::Uuid;

use crate::env::{parse_env_or, parse_env_or_else};
use crate::Result;

const DEFAULT_SERVICE_NAMESPACE: &str = "?";

#[doc(hidden)]
#[macro_export]
macro_rules! init_instruments {
    () => {
        $crate::telemetry::init_instruments_with_defaults(
            option_env!("CARGO_BIN_NAME").unwrap_or(env!("CARGO_PKG_NAME")),
            env!("CARGO_PKG_VERSION"),
        )
    };
}

#[cfg(feature = "test")]
#[cfg_attr(docsrs, doc(cfg(feature = "test")))]
#[macro_export]
macro_rules! init_instruments_for_test {
    () => {
        $crate::telemetry::init_instruments_with_defaults_for_test(
            option_env!("CARGO_BIN_NAME").unwrap_or(env!("CARGO_PKG_NAME")),
            env!("CARGO_PKG_VERSION"),
        )
    };
}

#[doc(hidden)]
pub fn init_instruments_with_defaults(
    default_service_name: &str,
    default_service_version: &str,
) -> Result<PushController> {
    tracing::debug!("Initializing instruments");
    let resources = tracing_resources(default_service_name, default_service_version)?;

    let metrics = init_metrics(&resources)?;
    init_tracing(&resources)?;

    tracing::info!("Configured instruments with {:?}", resources);

    Ok(metrics)
}

#[cfg(feature = "test")]
#[cfg_attr(docsrs, doc(cfg(feature = "test")))]
pub fn init_instruments_with_defaults_for_test(
    default_service_name: &str,
    default_service_version: &str,
) {
    tracing::debug!("Initializing instruments");
    let resources = tracing_resources(default_service_name, default_service_version).unwrap();

    init_metrics(&resources).unwrap();
    init_tracing_for_test();

    tracing::info!("Configured instruments with {:?}", resources);
}

/// Shuts down OpenTelemetry providers.
#[doc(hidden)]
pub fn shutdown_instruments(metrics: PushController) {
    tracing::debug!("Shutting down instruments");
    opentelemetry::global::shutdown_tracer_provider();
    drop(metrics);
}

fn init_metrics(resources: &[KeyValue]) -> Result<PushController> {
    let meter = opentelemetry_otlp::new_pipeline()
        .metrics(tokio::spawn, tokio_interval_stream)
        .with_resource(resources.to_owned())
        .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_env())
        .build()?;

    opentelemetry::global::set_meter_provider(meter.provider());

    Ok(meter)
}

fn init_tracing(resources: &[KeyValue]) -> Result<()> {
    opentelemetry::global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(trace::config().with_resource(Resource::new(resources.to_owned())))
        .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_env())
        .install_batch(opentelemetry::runtime::TokioCurrentThread)?;

    tracing_subscriber::Registry::default()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_ansi(false))
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();
    Ok(())
}

#[cfg(feature = "test")]
#[cfg_attr(docsrs, doc(cfg(feature = "test")))]
fn init_tracing_for_test() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        opentelemetry::global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());

        let tracer = {
            use opentelemetry::trace::TracerProvider;
            opentelemetry::sdk::trace::TracerProvider::default().tracer("test")
        };

        tracing_subscriber::Registry::default()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer().with_ansi(true))
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .init();
    });
}

fn tracing_resources(
    default_service_name: &str,
    default_service_version: &str,
) -> Result<Vec<KeyValue>> {
    let service_namespace: String = parse_env_or("SERVICE_NAMESPACE", DEFAULT_SERVICE_NAMESPACE)?;
    let service_name: String = parse_env_or("SERVICE_NAME", default_service_name)?;
    let service_instance_id: String =
        parse_env_or_else("SERVICE_INSTANCE_ID", || Uuid::new_v4().to_string())?;
    let service_version: String = parse_env_or("SERVICE_VERSION", default_service_version)?;

    let resources = vec![
        KeyValue::new(SERVICE_NAMESPACE, service_namespace),
        KeyValue::new(SERVICE_NAME, service_name),
        KeyValue::new(SERVICE_INSTANCE_ID, service_instance_id),
        KeyValue::new(SERVICE_VERSION, service_version),
    ];

    Ok(resources)
}
