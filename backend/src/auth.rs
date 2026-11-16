use crate::models::{ApiError, ErrorCode, Role, User};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use sqlx::SqlitePool;
use tower_sessions::Session;

pub fn session_error(_: tower_sessions::session::Error) -> ApiError {
    ApiError(ErrorCode::InternalError)
}
pub fn token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}
pub async fn csrf(session: &Session) -> Result<String, ApiError> {
    if let Some(token) = session.get::<String>("csrf").await.map_err(session_error)? {
        return Ok(token);
    }
    let token = token();
    session
        .insert("csrf", &token)
        .await
        .map_err(session_error)?;
    Ok(token)
}
pub async fn user_by_id(pool: &SqlitePool, id: &str) -> Result<Option<User>, ApiError> {
    let row: Option<(String, String, String)> =
        sqlx::query_as("SELECT id,username,role FROM users WHERE id=? AND active=1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(id, username, role)| User {
        id,
        username,
        role: if role == "admin" {
            Role::Admin
        } else {
            Role::Programmer
        },
    }))
}
pub async fn current_user(pool: &SqlitePool, session: &Session) -> Result<Option<User>, ApiError> {
    match session
        .get::<String>("user_id")
        .await
        .map_err(session_error)?
    {
        Some(id) => user_by_id(pool, &id).await,
        None => Ok(None),
    }
}
pub async fn require_user(pool: &SqlitePool, session: &Session) -> Result<User, ApiError> {
    current_user(pool, session)
        .await?
        .ok_or(ApiError(ErrorCode::Unauthenticated))
}
pub fn require_admin(user: &User) -> Result<(), ApiError> {
    if user.role != Role::Admin {
        return Err(ApiError(ErrorCode::Forbidden));
    }
    Ok(())
}
pub async fn authenticate(
    pool: &SqlitePool,
    username: &str,
    password: String,
) -> Result<User, ApiError> {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT id,password_hash FROM users WHERE username=? AND active=1")
            .bind(username)
            .fetch_optional(pool)
            .await?;
    let (id, hash) = match row {
        Some(row) => row,
        None => (
            String::new(),
            hash_password("invalid-account-placeholder".into()).await?,
        ),
    };
    let valid = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash).is_ok_and(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
    })
    .await
    .map_err(|_| ApiError(ErrorCode::InternalError))?;
    if !valid || id.is_empty() {
        return Err(ApiError(ErrorCode::Unauthenticated));
    }
    user_by_id(pool, &id)
        .await?
        .ok_or(ApiError(ErrorCode::Unauthenticated))
}
pub async fn hash_password(password: String) -> Result<String, ApiError> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(|_| ApiError(ErrorCode::InternalError))
    })
    .await
    .map_err(|_| ApiError(ErrorCode::InternalError))?
}
pub async fn create_user(
    pool: &SqlitePool,
    username: &str,
    password: String,
    admin: bool,
) -> Result<User, ApiError> {
    if username.trim().is_empty() || password.is_empty() {
        return Err(ApiError(ErrorCode::InvalidInput));
    }
    let hash = hash_password(password).await?;
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    if admin {
        let existing: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE role='admin'")
            .fetch_one(&mut *tx)
            .await?;
        if existing > 0 {
            return Err(ApiError(ErrorCode::Forbidden));
        }
    }
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username=?)")
        .bind(username)
        .fetch_one(&mut *tx)
        .await?;
    if exists {
        return Err(ApiError(ErrorCode::InvalidInput));
    }
    let user = User {
        id: uuid::Uuid::new_v4().to_string(),
        username: username.into(),
        role: if admin { Role::Admin } else { Role::Programmer },
    };
    sqlx::query("INSERT INTO users(id,username,password_hash,role) VALUES(?,?,?,?)")
        .bind(&user.id)
        .bind(username)
        .bind(hash)
        .bind(if admin { "admin" } else { "programmer" })
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(user)
}
