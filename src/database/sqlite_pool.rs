use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::env;
use std::str::FromStr;

use crate::error::AppResult;

pub async fn create_db_pool() -> AppResult<SqlitePool> {
    let db_url = env::var("DATABASE_URL")?;
    tracing::info!("Connecting to database at {}", db_url);

    let connection_options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(connection_options).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
