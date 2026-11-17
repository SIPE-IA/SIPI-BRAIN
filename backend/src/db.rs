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

type WriteFuture<'a, T> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, ApiError>> + Send + 'a>>;

pub async fn write<T, F>(pool: &SqlitePool, mut operation: F) -> Result<T, ApiError>
where
    F: for<'a> FnMut(&'a mut sqlx::SqliteConnection) -> WriteFuture<'a, T>,
{
    for attempt in 0..3 {
        let result = match pool.begin_with("BEGIN IMMEDIATE").await {
            Ok(mut tx) => match operation(&mut tx).await {
                Ok(value) => tx.commit().await.map(|_| value).map_err(ApiError::from),
                Err(error) => {
                    let _ = tx.rollback().await;
                    Err(error)
                }
            },
            Err(error) => Err(ApiError::from(error)),
        };
        match result {
            Err(ApiError(ErrorCode::StorageBusy)) if attempt < 2 => {
                tokio::time::sleep(Duration::from_millis(30)).await
            }
            other => return other,
        }
    }
    Err(ApiError(ErrorCode::StorageBusy))
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
