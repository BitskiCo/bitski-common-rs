//! # Utilities for telemetry.

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

/// Guard for telemetry instrument resources.
///
/// If dropped, telemetry may not work properly.
pub struct InstrumentGuard {
    _metrics: PushController,
}

impl Drop for InstrumentGuard {
    fn drop(&mut self) {
        shutdown_instruments();
    }
}

/// Initializes OpenTelemetry for tracing.
///
/// The returned guard object must be kept alive until instruments are no longer
/// needed, e.g. on server shutdown.
///
/// The OTLP exporter configurable with the following env variables:
///
/// * `OTEL_EXPORTER_OTLP_ENDPOINT=https://localhost:4317` Sets the target to
///   which the exporter is going to send spans or metrics.
///
/// * `OTEL_EXPORTER_OTLP_TIMEOUT=10` Sets the max waiting time for the backend
///   to process each spans or metrics batch in seconds.
///
/// * `RUST_LOG=error` Sets the logging level for logs and spans. See
///   [`tracing_subscriber::EnvFilter`].
///
/// * `SERVICE_NAMESPACE=?` Sets the Otel `service.namespace` resource value.
///
/// * `SERVICE_NAME=${CARGO_BIN_NAME:-$CARGO_PKG_NAME}` Sets the Otel
///   `service.name` resource value. Defaults to the value of `CARGO_BIN_NAME`
///   or `CARGO_PKG_NAME` at build time.
///
/// * `SERVICE_INSTANCE_ID=$(uuidgen)` Sets the Otel `service.instance.id`
///   resource value. Defaults to a random [Uuid].
///
/// * `SERVICE_VERSION=${CARGO_PKG_VERSION}` Sets the Otel `service.version`
///   resource value. Defaults to the value of `CARGO_PKG_VERSION` at build
///   time.
///
/// To override the configuration in Kubernetes:
///
/// ```yaml
/// apiVersion: v1
/// kind: Pod
/// metadata:
///   name: test-pod
/// spec:
///   containers:
///     - name: test-container
///       image: busybox
///       command: ["/bin/sh", "-c", "env"]
///       env:
///         - name: SERVICE_NAMESPACE
///           valueFrom:
///             fieldRef:
///               fieldPath: metadata.namespace
///         - name: SERVICE_NAME
///           valueFrom:
///             fieldRef:
///               fieldPath: metadata.labels.app
///         - name: SERVICE_INSTANCE_ID
///           valueFrom:
///             fieldRef:
///               fieldPath: metadata.name
///         - name: SERVICE_VERSION
///           valueFrom:
///             fieldRef:
///               fieldPath: metadata.labels.version
/// ```
#[macro_export]
macro_rules! init_instruments {
    () => {
        $crate::telemetry::init_instruments_with_defaults(
            option_env!("CARGO_BIN_NAME").unwrap_or(env!("CARGO_PKG_NAME")),
            env!("CARGO_PKG_VERSION"),
        )
    };
}

#[doc(hidden)]
pub fn init_instruments_with_defaults(
    default_service_name: &str,
    default_service_version: &str,
) -> Result<InstrumentGuard> {
    let resources = tracing_resources(default_service_name, default_service_version)?;

    let metrics = init_metrics(&resources)?;
    init_tracing(&resources)?;

    tracing::info!("Configured instruments with {:?}", resources);

    Ok(InstrumentGuard { _metrics: metrics })
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

/// Shuts down OpenTelemetry trace providers.
fn shutdown_instruments() {
    opentelemetry::global::shutdown_tracer_provider();
}
