//! Utilities for Diesel.

use async_trait::async_trait;
use diesel::r2d2::ConnectionManager;
use diesel::Connection as _;
use r2d2::{Pool, PooledConnection};

use crate::env::parse_env_or;
use crate::task::spawn_blocking;
use crate::{Error, Result};

const DEFAULT_DATABASE_URL: &str = "postgres://root@localhost:5432/defaultdb";
const DEFAULT_DATABASE_POOL_MIN_IDLE: u32 = 1;
const DEFAULT_DATABASE_POOL_MAX_SIZE: u32 = 4;

/// PostgreSQL connection.
pub type PgConnection = diesel::pg::PgConnection;

/// PostgreSQL connection pool.
pub type PgPool = Pool<ConnectionManager<PgConnection>>;

/// PostgreSQL connection from a connection pool.
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

/// An extension trait for [`PgPool`] that provides a variety of convenient adapters.
#[async_trait]
pub trait PgPoolExt {
    /// Creates an instrumented Diesel PostgreSQL connection pool from env
    /// variables.
    ///
    ///
    /// * `DATABASE_URL=postgres://root@localhost:5432/defaultdb` Sets the
    ///   database URL.
    ///
    /// * `DATABASE_POOL_MIN_IDLE=1` Sets the minimum idle connection count
    ///   maintained by the pool.
    ///
    /// * `DATABASE_POOL_MAX_SIZE=4` Sets the maximum number of connections
    ///   managed by the pool.
    fn from_env() -> Result<Self>
    where
        Self: Sized;

    /// Executes the given function with a database connection.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use anyhow::Result;
    /// use bitski_common::diesel::{PgPool, PgPoolExt as _};
    /// use diesel::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = PgPool::from_env()?;
    ///
    /// let count = db.with_conn(|conn| {
    ///     conn.execute("SELECT 1")
    /// }).await?;
    ///
    /// assert_eq!(count, 1);
    /// # Ok(())
    /// # }
    /// ```
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

        let manager = ConnectionManager::<PgConnection>::new(database_url);

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
