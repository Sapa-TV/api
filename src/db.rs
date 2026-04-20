use sqlx::SqlitePool;

pub async fn create_db() -> Result<SqliteDb, sqlx::Error> {
    let pool = SqlitePool::connect("sqlite:db/db.sqlite").await?;
    Ok(SqliteDb::new(pool))
}

pub async fn init_db(db: &dyn Db) -> Result<(), sqlx::Error> {
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

#[async_trait::async_trait]
pub trait Db: Send + Sync {
    async fn get_king(&self) -> Result<Option<String>, sqlx::Error>;
    async fn get_last_day_donaters(&self) -> Result<Vec<String>, sqlx::Error>;
    async fn get_month_donaters(&self) -> Result<Vec<String>, sqlx::Error>;
    async fn insert_king(&self, name: &str) -> Result<(), sqlx::Error>;
    async fn insert_last_day_donater(&self, name: &str) -> Result<(), sqlx::Error>;
    async fn insert_month_donater(&self, name: &str) -> Result<(), sqlx::Error>;
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
    async fn get_king(&self) -> Result<Option<String>, sqlx::Error> {
        let result: Option<(String,)> =
            sqlx::query_as("SELECT name FROM king ORDER BY id DESC LIMIT 1")
                .fetch_optional(&self.pool)
                .await?;
        Ok(result.map(|(name,)| name))
    }

    async fn get_last_day_donaters(&self) -> Result<Vec<String>, sqlx::Error> {
        let results: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM last_day_donaters ORDER BY id DESC LIMIT 10")
                .fetch_all(&self.pool)
                .await?;
        Ok(results.into_iter().map(|(name,)| name).collect())
    }

    async fn get_month_donaters(&self) -> Result<Vec<String>, sqlx::Error> {
        let results: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM month_donaters ORDER BY id DESC LIMIT 10")
                .fetch_all(&self.pool)
                .await?;
        Ok(results.into_iter().map(|(name,)| name).collect())
    }

    async fn insert_king(&self, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO king (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn insert_last_day_donater(&self, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO last_day_donaters (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn insert_month_donater(&self, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO month_donaters (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
