//! # Utilities for Actix Web.

pub use actix_web::*;

/// Configures an Actix Web app with common middleware.
///
/// Example:
///
/// ```rust,no_run
/// use actix_web::{web, App, HttpServer};
/// use anyhow::Result;
/// use bitski_common::{
///     actix_web_app,
///     env::{init_env, parse_env_addr_or_default},
///     with_instruments,
/// };
///
/// async fn index() -> &'static str {
///     "Hello World!"
/// }
///
/// #[actix_web::main]
/// #[with_instruments]
/// async fn main() -> Result<()> {
///     init_env();
///
///     // listens on `localhost:8000`
///     let addr = parse_env_addr_or_default()?;
///     tracing::info!("Listening on {}", addr);
///
///     HttpServer::new(move || actix_web_app!().route("/", web::get().to(index)))
///         .bind(addr)?
///         .run()
///         .await?;
///
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! actix_web_app {
    () => {
        actix_web_app!(App::new())
    };
    ($app:expr) => {
        $app.wrap($crate::actix_web::middleware::Compress::default())
            .wrap($crate::actix_web_opentelemetry::RequestTracing::new())
            .wrap(
                $crate::actix_web_opentelemetry::RequestMetricsBuilder::new()
                    .build($crate::opentelemetry::global::meter("actix_web")),
            )
            .wrap($crate::actix_web::middleware::Logger::default())
    };
}
