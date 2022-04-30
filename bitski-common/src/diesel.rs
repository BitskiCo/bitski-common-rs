//! Utilities for Diesel.

use async_trait::async_trait;
use diesel::r2d2::ConnectionManager;
use diesel::Connection as _;
use diesel_tracing::pg::InstrumentedPgConnection;
use r2d2::{Pool, PooledConnection};

use crate::env::parse_env_or;
use crate::task::spawn_blocking;
use crate::{Error, Result};

const DEFAULT_DATABASE_URL: &str = "postgres://root@localhost:5432/defaultdb";
const DEFAULT_DATABASE_POOL_MIN_IDLE: u32 = 1;
const DEFAULT_DATABASE_POOL_MAX_SIZE: u32 = 4;

/// Instrumented PostgreSQL connection pool.
pub type PgPool = Pool<ConnectionManager<InstrumentedPgConnection>>;

/// Instrumented PostgreSQL connection from a connection pool.
pub type PgPooledConnection = PooledConnection<ConnectionManager<InstrumentedPgConnection>>;

/// An extension trait for [`PgPool`] that provides a variety of convenient adapters.
#[async_trait]
pub trait PgPoolExt {
    /// Creates an instrumented Diesel PostgreSQL connection pool from env variables.
    fn from_env() -> Result<Self>
    where
        Self: Sized;

    /// Executes the given function with a database connection.
    async fn with_conn<F, R, E>(&self, f: F) -> Result<R, Error>
    where
        R: Send + 'static,
        F: FnOnce(PgPooledConnection) -> Result<R, E> + Send + 'static,
        E: Into<Error>;
}

#[async_trait]
impl PgPoolExt for PgPool {
    fn from_env() -> Result<Self> {
        let database_url: String = parse_env_or("DATABASE_URL", DEFAULT_DATABASE_URL)?;
        let min_idle: u32 = parse_env_or("DATABASE_POOL_MIN_IDLE", DEFAULT_DATABASE_POOL_MIN_IDLE)?;
        let max_size: u32 = parse_env_or("DATABASE_POOL_MAX_SIZE", DEFAULT_DATABASE_POOL_MAX_SIZE)?;

        let manager = ConnectionManager::<InstrumentedPgConnection>::new(database_url);

        let builder = Pool::builder().min_idle(Some(min_idle)).max_size(max_size);
        #[cfg(feature = "test")]
        let builder = builder.min_idle(None).max_size(1);

        let pool = builder.build(manager)?;

        #[cfg(feature = "test")]
        pool.get()?.begin_test_transaction()?;

        Ok(pool)
    }

    async fn with_conn<F, R, E>(&self, f: F) -> Result<R, Error>
    where
        R: Send + 'static,
        F: FnOnce(PgPooledConnection) -> Result<R, E> + Send + 'static,
        E: Into<Error>,
    {
        let db = self.clone();
        spawn_blocking(move || {
            let conn = db.get()?;
            f(conn).map_err(Into::into)
        })
        .await?
    }
}
