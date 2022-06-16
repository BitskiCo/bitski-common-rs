// Based on https://github.com/tokio-rs/tokio/blob/master/tokio-macros/

#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned, ToTokens};

#[cfg(feature = "doc_cfg")]
use tracing_subscriber::EnvFilter;
#[cfg(feature = "doc_cfg")]
use uuid::Uuid;

/// Runs an async block with OpenTelemetry for tracing.
///
/// Examples:
///
/// `#[tokio::main]` must go last!
///
/// ```rust,no_run
/// # use bitski_common_macros::with_instruments;
/// #
/// #[with_instruments]
/// #[tokio::main]
/// async fn main() {
///     // ...
/// }
/// ```
///
/// Or wrap a separate `run()` function:
///
/// ```rust,no_run
/// # use bitski_common_macros::with_instruments;
/// #
/// #[tokio::main]
/// async fn main() {
///     run().await
/// }
///
/// #[with_instruments]
/// async fn run() {
///     // ...
/// }
/// ```
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
///   resource value. Defaults to a random [`uuid::Uuid`].
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
#[proc_macro_attribute]
pub fn with_instruments(_args: TokenStream, item: TokenStream) -> TokenStream {
    // If any of the steps for this macro fail, we still want to expand to an item that is as close
    // to the expected output as possible. This helps out IDEs such that completions and other
    // related features keep working.
    let mut input: syn::ItemFn = match syn::parse(item.clone()) {
        Ok(it) => it,
        Err(e) => return token_stream_with_error(item, e),
    };

    if input.sig.asyncness.is_none() {
        let msg = "the `async` keyword is missing from the function declaration or `#[tokio::main]` was not declared last";
        let err = syn::Error::new_spanned(input.sig.fn_token, msg);
        return token_stream_with_error(item, err);
    }

    // If type mismatch occurs, the current rustc points to the last statement.
    let (_last_stmt_start_span, last_stmt_end_span) = {
        let mut last_stmt = input
            .block
            .stmts
            .last()
            .map(ToTokens::into_token_stream)
            .unwrap_or_default()
            .into_iter();
        // `Span` on stable Rust has a limitation that only points to the first
        // token, not the whole tokens. We can work around this limitation by
        // using the first/last span of the tokens like
        // `syn::Error::new_spanned` does.
        let start = last_stmt.next().map_or_else(Span::call_site, |t| t.span());
        let end = last_stmt.last().map_or(start, |t| t.span());
        (start, end)
    };

    let body = &input.block;
    let brace_token = input.block.brace_token;
    input.block = syn::parse2(quote_spanned! {last_stmt_end_span=>
        {
            let body = async move { #body };
            let metrics = bitski_common::init_instruments!().expect("Instruments");
            let result = body.await;
            bitski_common::telemetry::shutdown_instruments(metrics);
            result
        }
    })
    .expect("Parsing failure");
    input.block.brace_token = brace_token;

    let result = quote! {
        #input
    };

    result.into()
}

fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(TokenStream::from(error.into_compile_error()));
    tokens
}
