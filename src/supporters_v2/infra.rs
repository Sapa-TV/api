use std::sync::Arc;

use crate::error::AppResult;
use crate::shared_infra_v2::sqlite_db::SqliteDb;
use crate::supporters_v2::domain::{SupporterRepository, SupporterRepositoryData};

#[derive(Clone)]
pub struct SqliteSupporterRepository {
    db: Arc<SqliteDb>,
}

impl SqliteSupporterRepository {
    pub fn new(db: Arc<SqliteDb>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl SupporterRepository for SqliteSupporterRepository {
    async fn init_supporters(&self, data: SupporterRepositoryData) -> AppResult<()> {
        let existing_king = self.get_king().await?;
        if existing_king.is_none() {
            self.insert_king(&data.king).await?;
        }

        let existing_last = self.get_day_supporters().await?;
        if existing_last.is_empty() {
            for supporter in data.day_supporters {
                self.insert_day_supporter(&supporter).await?;
            }
        }

        let existing_month = self.get_month_supporters().await?;
        if existing_month.is_empty() {
            for supporter in data.month_supporters {
                self.insert_month_supporter(&supporter).await?;
            }
        }

        Ok(())
    }

    async fn get_king(&self) -> AppResult<Option<String>> {
        let result: Option<(String,)> =
            sqlx::query_as("SELECT name FROM king ORDER BY id DESC LIMIT 1")
                .fetch_optional(self.db.pool())
                .await?;
        Ok(result.map(|(name,)| name))
    }

    async fn insert_king(&self, name: &str) -> AppResult<()> {
        sqlx::query("INSERT INTO king (name) VALUES (?)")
            .bind(name)
            .execute(self.db.pool())
            .await?;
        Ok(())
    }

    async fn get_day_supporters(&self) -> AppResult<Vec<String>> {
        let results: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM last_day_donaters ORDER BY id DESC LIMIT 10")
                .fetch_all(self.db.pool())
                .await?;
        Ok(results.into_iter().map(|(name,)| name).collect())
    }

    async fn insert_day_supporter(&self, name: &str) -> AppResult<()> {
        sqlx::query("INSERT INTO last_day_donaters (name) VALUES (?)")
            .bind(name)
            .execute(self.db.pool())
            .await?;
        Ok(())
    }

    async fn get_month_supporters(&self) -> AppResult<Vec<String>> {
        let results: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM month_donaters ORDER BY id DESC LIMIT 10")
                .fetch_all(self.db.pool())
                .await?;
        Ok(results.into_iter().map(|(name,)| name).collect())
    }

    async fn insert_month_supporter(&self, name: &str) -> AppResult<()> {
        sqlx::query("INSERT INTO month_donaters (name) VALUES (?)")
            .bind(name)
            .execute(self.db.pool())
            .await?;
        Ok(())
    }
}
