//! Utilities for Diesel.

use anyhow::{Context as _, Result};
use diesel::r2d2::{Pool, PooledConnection};
use diesel::{pg::PgConnection, r2d2::ConnectionManager};

use crate::env::{parse_env, parse_env_or};

const DEFAULT_DATABASE_URL: &str = "postgres://root@localhost:5432/defaultdb";
const DEFAULT_DATABASE_POOL_MAX_SIZE: u32 = 4;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

/// Initializes a Diesel PostgreSQL connection pool.
pub fn init_database_pool() -> Result<PgPool> {
    let database_url: String = parse_env_or("DATABASE_URL", DEFAULT_DATABASE_URL)?;
    let min_size: Option<u32> = parse_env("DATABASE_POOL_MIN_SIZE")?;
    let max_size: u32 = parse_env_or("DATABASE_POOL_MAX_SIZE", DEFAULT_DATABASE_POOL_MAX_SIZE)?;

    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder()
        .min_idle(min_size)
        .max_size(max_size)
        .build(manager)
        .context("error building database pool")
}
