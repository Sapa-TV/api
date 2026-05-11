use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::supporters::repository::SupporterRepository;

#[derive(Clone)]
pub struct SqliteSupporterRepository {
    db: SqlitePool,
}

impl SqliteSupporterRepository {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl SupporterRepository for SqliteSupporterRepository {
    async fn get_king(&self) -> AppResult<Option<String>> {
        let result: Option<(String,)> =
            sqlx::query_as("SELECT name FROM king ORDER BY id DESC LIMIT 1")
                .fetch_optional(&self.db)
                .await?;
        Ok(result.map(|(name,)| name))
    }

    async fn set_king(&self, name: &str) -> AppResult<()> {
        sqlx::query("INSERT INTO king (name) VALUES (?)")
            .bind(name)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    async fn get_day_supporters(&self) -> AppResult<Vec<String>> {
        let results: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM last_day_donaters ORDER BY id DESC LIMIT 10")
                .fetch_all(&self.db)
                .await?;
        Ok(results.into_iter().map(|(name,)| name).collect())
    }

    async fn insert_day_supporter(&self, name: &str) -> AppResult<()> {
        sqlx::query("INSERT INTO last_day_donaters (name) VALUES (?)")
            .bind(name)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    async fn get_month_supporters(&self) -> AppResult<Vec<String>> {
        let results: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM month_donaters ORDER BY id DESC LIMIT 10")
                .fetch_all(&self.db)
                .await?;
        Ok(results.into_iter().map(|(name,)| name).collect())
    }

    async fn insert_month_supporter(&self, name: &str) -> AppResult<()> {
        sqlx::query("INSERT INTO month_donaters (name) VALUES (?)")
            .bind(name)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}
