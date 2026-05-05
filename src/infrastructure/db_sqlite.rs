use crate::error::AppResult;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::env;

mod admin_session;
mod admin_whitelist;
mod models;
mod push_subscriptions;
mod supporters;
mod tokens;

pub struct SqliteDb {
    pool: SqlitePool,
}

pub async fn create_db() -> AppResult<SqliteDb> {
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:db/db.sqlite".to_string());

    tracing::info!("Connecting to database at {}", db_url);

    let connection_options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(connection_options).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(SqliteDb::new(pool))
}

impl Clone for SqliteDb {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl SqliteDb {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
