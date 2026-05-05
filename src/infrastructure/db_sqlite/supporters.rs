use crate::{
    error::AppResult, infrastructure::db_sqlite::SqliteDb, supporters::SupporterRepository,
};

#[async_trait::async_trait]
impl SupporterRepository for SqliteDb {
    async fn get_king(&self) -> AppResult<Option<String>> {
        let result: Option<(String,)> =
            sqlx::query_as("SELECT name FROM king ORDER BY id DESC LIMIT 1")
                .fetch_optional(&self.pool)
                .await?;
        Ok(result.map(|(name,)| name))
    }

    async fn insert_king(&self, name: &str) -> AppResult<()> {
        sqlx::query("INSERT INTO king (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_day_supporters(&self) -> AppResult<Vec<String>> {
        let results: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM last_day_donaters ORDER BY id DESC LIMIT 10")
                .fetch_all(&self.pool)
                .await?;
        Ok(results.into_iter().map(|(name,)| name).collect())
    }

    async fn insert_day_supporter(&self, name: &str) -> AppResult<()> {
        sqlx::query("INSERT INTO last_day_donaters (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_month_supporters(&self) -> AppResult<Vec<String>> {
        let results: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM month_donaters ORDER BY id DESC LIMIT 10")
                .fetch_all(&self.pool)
                .await?;
        Ok(results.into_iter().map(|(name,)| name).collect())
    }

    async fn insert_month_supporter(&self, name: &str) -> AppResult<()> {
        sqlx::query("INSERT INTO month_donaters (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

const KING_DEFAULT: &str = "Star";
const DAY_LIST_DEFAULT: &[&str] = &["Echo", "Night Wolf", "Shadow Hunter"];
const MONTH_LIST_DEFAULT: &[&str] = &["Star", "Echo", "Vortex", "Night Wolf", "Shadow Hunter"];

pub async fn init_supporters<DB: SupporterRepository>(db: &DB) -> AppResult<()> {
    let existing_king = db.get_king().await?;
    if existing_king.is_none() {
        db.insert_king(KING_DEFAULT).await?;
    }

    let existing_last = db.get_day_supporters().await?;
    if existing_last.is_empty() {
        for supporter in DAY_LIST_DEFAULT {
            db.insert_last_day_supporters(*supporter).await?;
        }
    }

    let existing_month = db.get_month_supporters().await?;
    if existing_month.is_empty() {
        for supporter in MONTH_LIST_DEFAULT {
            db.insert_month_supporter(*supporter).await?;
        }
    }

    Ok(())
}
