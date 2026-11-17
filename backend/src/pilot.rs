use sqlx::SqlitePool;

use crate::{auth, db, models::*, services};

pub const INSIGHT: &str = "pilot-insight";
pub const ESTIMADOR: &str = "pilot-estimador";
pub const ALFA: &str = "pilot-project-alfa";
pub const BETA: &str = "pilot-project-beta";
const AUTHOR: &str = "pilot-fixture-author";
const USERNAME: &str = "__synthetic_fixture__";

async fn prepare(pool: &SqlitePool) -> Result<User, ApiError> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id=?)")
        .bind(AUTHOR)
        .fetch_one(pool)
        .await?;
    // No generated credential is retained or printed; the author is disabled after seeding.
    let hash = if exists {
        String::new()
    } else {
        auth::hash_password(auth::token()).await?
    };
    db::write(pool,move|conn| {let hash=hash.clone();Box::pin(async move {
        sqlx::query("INSERT INTO users(id,username,password_hash,active,role) VALUES(?,?,?,0,'programmer') ON CONFLICT DO NOTHING").bind(AUTHOR).bind(USERNAME).bind(hash).execute(&mut *conn).await?;
        let expected:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id=? AND username=? AND role='programmer')").bind(AUTHOR).bind(USERNAME).fetch_one(&mut *conn).await?;
        if !expected {return Err(ApiError(ErrorCode::InvalidInput));}
        for (agent,name) in [(INSIGHT,"SIPI Insight"),(ESTIMADOR,"SIPI Estimador")] {
            sqlx::query("INSERT INTO agents(id,name,active) VALUES(?,?,1) ON CONFLICT(id) DO NOTHING").bind(agent).bind(name).execute(&mut *conn).await?;
            sqlx::query("INSERT INTO wikis(id,agent_id,name) VALUES(?,?,?) ON CONFLICT(id) DO NOTHING").bind(agent).bind(agent).bind(name).execute(&mut *conn).await?;
            let valid:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM wikis w JOIN agents a ON a.id=w.agent_id WHERE w.id=? AND a.id=? AND w.name=? AND a.name=? AND a.active=1)").bind(agent).bind(agent).bind(name).bind(name).fetch_one(&mut *conn).await?;
            if !valid {return Err(ApiError(ErrorCode::InvalidInput));}
        }
        for (client,name,project,project_name) in [("pilot-client-alfa","Cliente Alfa",ALFA,"Alfa"),("pilot-client-beta","Cliente Beta",BETA,"Beta")] {
            sqlx::query("INSERT INTO clients(id,display_name) VALUES(?,?) ON CONFLICT(id) DO NOTHING").bind(client).bind(name).execute(&mut *conn).await?;
            sqlx::query("INSERT INTO projects(id,name,client_id) VALUES(?,?,?) ON CONFLICT(id) DO NOTHING").bind(project).bind(project_name).bind(client).execute(&mut *conn).await?;
            let valid:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM projects p JOIN clients c ON c.id=p.client_id WHERE p.id=? AND p.name=? AND c.id=? AND c.display_name=?)").bind(project).bind(project_name).bind(client).bind(name).fetch_one(&mut *conn).await?;
            if !valid {return Err(ApiError(ErrorCode::InvalidInput));}
        }
        sqlx::query("UPDATE users SET active=1 WHERE id=?").bind(AUTHOR).execute(&mut *conn).await?;
        for wiki in [INSIGHT,ESTIMADOR] {
            sqlx::query("INSERT INTO wiki_assignments(user_id,wiki_id,assigned_by,assigned_at) VALUES(?,?,?,?) ON CONFLICT(user_id,wiki_id) DO NOTHING").bind(AUTHOR).bind(wiki).bind(AUTHOR).bind(db::now()).execute(&mut *conn).await?;
        }
        Ok(User{id:AUTHOR.into(),username:USERNAME.into(),role:Role::Programmer})
    })}).await
}

pub async fn seed(pool: &SqlitePool) -> Result<(), ApiError> {
    let author = prepare(pool).await?;
    let result = contribute(pool, &author).await;
    let cleanup = db::write(pool, |conn| {
        Box::pin(async move {
            sqlx::query("UPDATE users SET active=0 WHERE id=?")
                .bind(AUTHOR)
                .execute(&mut *conn)
                .await?;
            sqlx::query("DELETE FROM wiki_assignments WHERE user_id=?")
                .bind(AUTHOR)
                .execute(&mut *conn)
                .await?;
            Ok(())
        })
    })
    .await;
    result.and(cleanup)
}

async fn contribute(pool: &SqlitePool, author: &User) -> Result<(), ApiError> {
    for (key, wiki, project, client, kind, title, body) in [
        (
            "59e4a260-6c4d-4f26-a000-000000000001",
            INSIGHT,
            ALFA,
            "pilot-client-alfa",
            RecordKind::Rfd,
            "RFD-001",
            "Synthetic functional requirement 001 for project Alfa.",
        ),
        (
            "59e4a260-6c4d-4f26-a000-000000000002",
            INSIGHT,
            ALFA,
            "pilot-client-alfa",
            RecordKind::Rfd,
            "RFD-002",
            "Synthetic functional requirement 002 for project Alfa.",
        ),
        (
            "59e4a260-6c4d-4f26-a000-000000000003",
            INSIGHT,
            ALFA,
            "pilot-client-alfa",
            RecordKind::Rfd,
            "RFD-003",
            "Synthetic functional requirement 003 for project Alfa.",
        ),
        (
            "59e4a260-6c4d-4f26-a000-000000000004",
            INSIGHT,
            BETA,
            "pilot-client-beta",
            RecordKind::Rfd,
            "RFD-004",
            "Synthetic functional requirement 004 for project Beta.",
        ),
        (
            "59e4a260-6c4d-4f26-a000-000000000005",
            ESTIMADOR,
            ALFA,
            "pilot-client-alfa",
            RecordKind::Note,
            "Estimator Alfa note",
            "Synthetic estimate context for project Alfa; no external client data.",
        ),
    ] {
        services::create_record(
            pool,
            author,
            wiki,
            RecordInput {
                submission_id: key.into(),
                kind,
                title: title.into(),
                body_markdown: body.into(),
                project_id: Some(project.into()),
                client_id: Some(client.into()),
                epistemic_kind: EpistemicKind::Assumption,
                source_ids: vec![],
            },
        )
        .await?;
    }
    Ok(())
}
