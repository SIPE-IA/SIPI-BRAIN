use crate::models::{ApiError, ErrorCode, User};
use sqlx::{SqliteConnection, SqlitePool};

pub async fn require_assignment(
    pool: &SqlitePool,
    user: &User,
    wiki_id: &str,
) -> Result<(), ApiError> {
    let mut connection = pool.acquire().await?;
    assignment_on(&mut connection, &user.id, wiki_id).await
}
pub async fn assignment_on(
    connection: &mut SqliteConnection,
    user_id: &str,
    wiki_id: &str,
) -> Result<(), ApiError> {
    let assigned: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM wiki_assignments a JOIN users u ON u.id=a.user_id JOIN wikis w ON w.id=a.wiki_id JOIN agents g ON g.id=w.agent_id WHERE a.user_id=? AND a.wiki_id=? AND u.active=1 AND g.active=1)").bind(user_id).bind(wiki_id).fetch_one(connection).await?;
    if !assigned {
        return Err(ApiError(ErrorCode::NotFound));
    }
    Ok(())
}
