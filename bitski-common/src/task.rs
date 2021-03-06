//! Utilities for spawning tasks.

use std::future::Future;

use opentelemetry::trace::FutureExt as _;

/// Spawns a new asynchronous task with Tokio.
///
/// Propagates the current OpenTelemetry context. Unless the task needs to run
/// concurrently, prefer [`spawn_local`].
///
/// See [`tokio::spawn`].
pub fn spawn<T>(future: T) -> tokio::task::JoinHandle<T::Output>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    tokio::spawn(future.with_current_context())
}

/// Spawns a new asynchronous task with Tokio, on the current thread.
///
/// Propagates the current OpenTelemetry context.
///
/// See [`tokio::task::spawn_local`].
pub fn spawn_local<T>(future: T) -> tokio::task::JoinHandle<T::Output>
where
    T: Future + 'static,
    T::Output: 'static,
{
    tokio::task::spawn_local(future.with_current_context())
}

/// Spawns a blocking task with Tokio.
///
/// Propagates the current OpenTelemetry context.
///
/// See [`tokio::task::spawn_blocking`].
pub fn spawn_blocking<F, R>(f: F) -> tokio::task::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let ctx = opentelemetry::Context::current();
    tokio::task::spawn_blocking(move || {
        let _guard = ctx.attach();
        f()
    })
}
