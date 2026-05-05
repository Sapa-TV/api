use crate::{
    error::AppResult,
    infrastructure::db_sqlite::{SqliteDb, SqliteDbBuilder},
};

pub async fn create_db() -> AppResult<SqliteDb> {
    SqliteDbBuilder::create().await
}
