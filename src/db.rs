use dotenvy::dotenv;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::env;
use std::str::FromStr;

use crate::error::AppResult;
use crate::providers::twitch::auth::StoredToken;

pub async fn create_db() -> AppResult<SqliteDb> {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:db/db.sqlite".to_string());

    tracing::info!("Connecting to database at {}", db_url);

    let connection_options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);

    let pool = SqlitePool::connect_with(connection_options).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(SqliteDb::new(pool))
}

pub async fn init_db(db: &dyn Db) -> AppResult<()> {
    let existing_king = db.get_king().await?;
    if existing_king.is_none() {
        db.insert_king("Star").await?;
    }

    let existing_last = db.get_last_day_donaters().await?;
    if existing_last.is_empty() {
        db.insert_last_day_donater("Echo").await?;
        db.insert_last_day_donater("Night Wolf").await?;
        db.insert_last_day_donater("Shadow Hunter").await?;
    }

    let existing_month = db.get_month_donaters().await?;
    if existing_month.is_empty() {
        db.insert_month_donater("Star").await?;
        db.insert_month_donater("Echo").await?;
        db.insert_month_donater("Vortex").await?;
        db.insert_month_donater("Night Wolf").await?;
        db.insert_month_donater("Shadow Hunter").await?;
    }

    Ok(())
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PushSubscription {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub user_id: Option<String>,
}

#[async_trait::async_trait]
pub trait Db: Send + Sync {
    async fn get_king(&self) -> AppResult<Option<String>>;
    async fn get_last_day_donaters(&self) -> AppResult<Vec<String>>;
    async fn get_month_donaters(&self) -> AppResult<Vec<String>>;
    async fn insert_king(&self, name: &str) -> AppResult<()>;
    async fn insert_last_day_donater(&self, name: &str) -> AppResult<()>;
    async fn insert_month_donater(&self, name: &str) -> AppResult<()>;

    async fn insert_subscription(
        &self,
        endpoint: &str,
        p256dh: &str,
        auth: &str,
        user_id: Option<&str>,
    ) -> AppResult<()>;
    async fn get_all_subscriptions(&self) -> AppResult<Vec<PushSubscription>>;
    async fn delete_subscription(&self, endpoint: &str) -> AppResult<()>;

    async fn get_twitch_token(&self) -> AppResult<Option<StoredToken>>;
    async fn save_twitch_token(&self, token: &StoredToken) -> AppResult<()>;
}

pub struct SqliteDb {
    pool: SqlitePool,
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

#[async_trait::async_trait]
impl Db for SqliteDb {
    async fn get_king(&self) -> AppResult<Option<String>> {
        let result: Option<(String,)> =
            sqlx::query_as("SELECT name FROM king ORDER BY id DESC LIMIT 1")
                .fetch_optional(&self.pool)
                .await?;
        Ok(result.map(|(name,)| name))
    }

    async fn get_last_day_donaters(&self) -> AppResult<Vec<String>> {
        let results: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM last_day_donaters ORDER BY id DESC LIMIT 10")
                .fetch_all(&self.pool)
                .await?;
        Ok(results.into_iter().map(|(name,)| name).collect())
    }

    async fn get_month_donaters(&self) -> AppResult<Vec<String>> {
        let results: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM month_donaters ORDER BY id DESC LIMIT 10")
                .fetch_all(&self.pool)
                .await?;
        Ok(results.into_iter().map(|(name,)| name).collect())
    }

    async fn insert_king(&self, name: &str) -> AppResult<()> {
        sqlx::query("INSERT INTO king (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn insert_last_day_donater(&self, name: &str) -> AppResult<()> {
        sqlx::query("INSERT INTO last_day_donaters (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn insert_month_donater(&self, name: &str) -> AppResult<()> {
        sqlx::query("INSERT INTO month_donaters (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn insert_subscription(
        &self,
        endpoint: &str,
        p256dh: &str,
        auth: &str,
        user_id: Option<&str>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO push_subscriptions (endpoint, p256dh, auth, user_id) VALUES (?, ?, ?, ?)",
        )
        .bind(endpoint)
        .bind(p256dh)
        .bind(auth)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_all_subscriptions(&self) -> AppResult<Vec<PushSubscription>> {
        let results = sqlx::query_as::<_, PushSubscription>(
            "SELECT id, endpoint, p256dh, auth, user_id, created_at FROM push_subscriptions",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(results)
    }

    async fn delete_subscription(&self, endpoint: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM push_subscriptions WHERE endpoint = ?")
            .bind(endpoint)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_twitch_token(&self) -> AppResult<Option<StoredToken>> {
        let result: Option<(String, String, String)> = sqlx::query_as(
            "SELECT access_token, refresh_token, created_at FROM twitch_tokens WHERE id = 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(
            result.map(|(access_token, refresh_token, created_at)| StoredToken {
                access_token,
                refresh_token,
                created_at,
            }),
        )
    }

    async fn save_twitch_token(&self, token: &StoredToken) -> AppResult<()> {
        sqlx::query(
            "UPDATE twitch_tokens SET access_token = ?, refresh_token = ?, updated_at = datetime('now') WHERE id = 1",
        )
        .bind(&token.access_token)
        .bind(&token.refresh_token)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
