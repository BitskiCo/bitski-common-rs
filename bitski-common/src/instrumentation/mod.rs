#[cfg(feature = "tower")]
#[cfg_attr(docsrs, doc(cfg(feature = "tower")))]
pub mod tower;

use anyhow::Result;
use opentelemetry::{
    sdk::{trace, Resource},
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

/// Initializes OpenTelemetry for tracing.
///
/// An OTLP exporter is configured using the following env variables:
///
/// - `SERVICE_NAMESPACE=?`
/// - `SERVICE_NAME` defaults to the value of `CARGO_BIN_NAME` or `CARGO_PKG_NAME` at build time
/// - `SERVICE_INSTANCE_ID` defaults to a random [Uuid]
/// - `SERVICE_VERSION` defaults to the value of `CARGO_PKG_VERSION` at build time
/// - `OTEL_EXPORTER_OTLP_ENDPOINT=https://localhost:4317`
/// - `OTEL_EXPORTER_OTLP_TIMEOUT=10` in seconds
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
pub fn init_instruments() -> Result<()> {
    let resources = tracing_resources()?;

    init_metrics(&resources)?;
    init_tracing(&resources)?;

    tracing::info!("Configured instruments with {:?}", resources);

    Ok(())
}

fn init_metrics(resources: &[KeyValue]) -> Result<()> {
    let meter = opentelemetry_otlp::new_pipeline()
        .metrics(tokio::spawn, tokio_interval_stream)
        .with_resource(resources.to_owned())
        .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_env())
        .build()?;

    opentelemetry::global::set_meter_provider(meter.provider());

    Ok(())
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

fn tracing_resources() -> Result<Vec<KeyValue>> {
    let service_namespace: String = parse_env_or("SERVICE_NAMESPACE", "?")?;

    let service_name: String = parse_env_or_else("SERVICE_NAME", || {
        option_env!("CARGO_BIN_NAME").unwrap_or(env!("CARGO_PKG_NAME"))
    })?;

    let service_instance_id: String =
        parse_env_or_else("SERVICE_INSTANCE_ID", || Uuid::new_v4().to_string())?;

    let service_version: String =
        parse_env_or_else("SERVICE_VERSION", || env!("CARGO_PKG_VERSION"))?;

    let resources = vec![
        KeyValue::new(SERVICE_NAMESPACE, service_namespace),
        KeyValue::new(SERVICE_NAME, service_name),
        KeyValue::new(SERVICE_INSTANCE_ID, service_instance_id),
        KeyValue::new(SERVICE_VERSION, service_version),
    ];

    Ok(resources)
}

/// Shuts down OpenTelemetry trace providers.
pub fn shutdown_instruments() -> Result<()> {
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
