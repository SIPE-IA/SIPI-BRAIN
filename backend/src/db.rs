use crate::models::{ApiError, ErrorCode};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{path::Path, time::Duration};

pub async fn open(path: &Path) -> Result<SqlitePool, ApiError> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_millis(150));
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;
    migrate(&pool).await?;
    Ok(pool)
}
pub async fn migrate(pool: &SqlitePool) -> Result<(), ApiError> {
    sqlx::migrate!()
        .run(pool)
        .await
        .map_err(|_| ApiError(ErrorCode::InternalError))
}
pub fn now() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .expect("UTC timestamp is representable")
}
impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        let code = match &error {
            sqlx::Error::Database(e)
                if matches!(e.code().as_deref(), Some("5" | "6" | "261" | "517")) =>
            {
                ErrorCode::StorageBusy
            }
            sqlx::Error::PoolTimedOut => ErrorCode::StorageBusy,
            _ => ErrorCode::InternalError,
        };
        Self(code)
    }
}
