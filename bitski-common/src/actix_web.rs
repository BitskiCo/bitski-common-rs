//! # Utilities for Actix Web.

/// Configures an Actix Web app with common middleware.
///
/// Example:
///
/// ```rust,no_run
/// use actix_web::{web, App, HttpServer};
/// use anyhow::Result;
/// use bitski_common::{
///     actix_web_app,
///     env::{init_env, parse_env_addr},
///     telemetry::{init_instruments, shutdown_instruments},
/// };
///
/// async fn index(data: web::Path<(String, String)>) -> &'static str {
///     "Hello World!"
/// }
///
/// #[actix_web::main]
/// async fn main() -> Result<()> {
///     init_env();
///     init_instruments()?;
///
///     let addr = parse_env_addr()?;
///     tracing::info!("Listening on {}", addr);
///
///     HttpServer::new(move || {
///         actix_web_app!().route("/", web::get().to(index))
///     })
///     .bind(addr)?
///     .run()
///     .await?;
///
///     shutdown_instruments()?;
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
        $app.wrap(actix_web::middleware::Compress::default())
            .wrap(actix_web_opentelemetry::RequestTracing::new())
            .wrap(
                actix_web_opentelemetry::RequestMetricsBuilder::new()
                    .build(opentelemetry::global::meter("actix_web")),
            )
            .wrap(actix_web::middleware::Logger::default())
    };
}
