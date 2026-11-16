use crate::{auth, db, models::*};
use sqlx::SqlitePool;

pub async fn assignments(pool: &SqlitePool, user: &User) -> Result<Assignments, ApiError> {
    auth::require_admin(user)?;
    let users: Vec<(String, String, String)> =
        sqlx::query_as("SELECT id,username,role FROM users WHERE active=1 ORDER BY username,id")
            .fetch_all(pool)
            .await?;
    let wikis: Vec<(String,String,String)> = sqlx::query_as("SELECT w.id,w.agent_id,w.name FROM wikis w JOIN agents g ON g.id=w.agent_id WHERE g.active=1 ORDER BY w.name,w.id").fetch_all(pool).await?;
    let items: Vec<(String, String)> =
        sqlx::query_as("SELECT user_id,wiki_id FROM wiki_assignments ORDER BY user_id,wiki_id")
            .fetch_all(pool)
            .await?;
    Ok(Assignments {
        users: users
            .into_iter()
            .map(|(id, username, role)| User {
                id,
                username,
                role: if role == "admin" {
                    Role::Admin
                } else {
                    Role::Programmer
                },
            })
            .collect(),
        wikis: wikis
            .into_iter()
            .map(|(id, agent_id, name)| Wiki { id, agent_id, name })
            .collect(),
        items: items
            .into_iter()
            .map(|(user_id, wiki_id)| Assignment { user_id, wiki_id })
            .collect(),
    })
}
pub async fn set_assignment(
    pool: &SqlitePool,
    user: &User,
    input: AssignmentInput,
) -> Result<AssignmentResult, ApiError> {
    auth::require_admin(user)?;
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let admin: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id=? AND active=1 AND role='admin')",
    )
    .bind(&user.id)
    .fetch_one(&mut *tx)
    .await?;
    if !admin {
        return Err(ApiError(ErrorCode::Forbidden));
    }
    let valid:bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users u,wikis w JOIN agents g ON g.id=w.agent_id WHERE u.id=? AND u.active=1 AND w.id=? AND g.active=1)").bind(&input.user_id).bind(&input.wiki_id).fetch_one(&mut *tx).await?;
    if !valid {
        return Err(ApiError(ErrorCode::InvalidInput));
    }
    let assigned = matches!(input.action, AssignmentAction::Grant);
    if assigned {
        sqlx::query("INSERT INTO wiki_assignments(user_id,wiki_id,assigned_by,assigned_at) VALUES(?,?,?,?) ON CONFLICT(user_id,wiki_id) DO NOTHING").bind(&input.user_id).bind(&input.wiki_id).bind(&user.id).bind(db::now()).execute(&mut *tx).await?;
    } else {
        sqlx::query("DELETE FROM wiki_assignments WHERE user_id=? AND wiki_id=?")
            .bind(&input.user_id)
            .bind(&input.wiki_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(AssignmentResult {
        user_id: input.user_id,
        wiki_id: input.wiki_id,
        assigned,
    })
}
pub async fn wikis(pool: &SqlitePool, user: &User) -> Result<Items<Wiki>, ApiError> {
    let rows:Vec<(String,String,String)> = sqlx::query_as("SELECT w.id,w.agent_id,w.name FROM wikis w JOIN wiki_assignments a ON a.wiki_id=w.id JOIN agents g ON g.id=w.agent_id WHERE a.user_id=? AND g.active=1 ORDER BY w.name,w.id").bind(&user.id).fetch_all(pool).await?;
    Ok(Items {
        items: rows
            .into_iter()
            .map(|(id, agent_id, name)| Wiki { id, agent_id, name })
            .collect(),
    })
}
