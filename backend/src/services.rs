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

use crate::access;
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqliteConnection, sqlite::SqliteRow};

async fn admin_wiki_items(conn: &mut SqliteConnection) -> Result<Vec<WikiAdminItem>, ApiError> {
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT w.id,w.name,g.id,g.name FROM wikis w JOIN agents g ON g.id=w.agent_id ORDER BY w.name,w.id",
    )
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, agent_id, agent_name)| WikiAdminItem {
            id,
            name,
            agent: AgentSummary {
                id: agent_id,
                name: agent_name,
            },
        })
        .collect())
}

pub async fn admin_wikis(pool: &SqlitePool, user: &User) -> Result<WikiAdminIndex, ApiError> {
    let mut tx = pool.begin().await?;
    access::admin_on(&mut tx, &user.id).await?;
    let items = admin_wiki_items(&mut tx).await?;
    let agents: Vec<(String, String)> = sqlx::query_as(
        "SELECT g.id,g.name FROM agents g WHERE g.active=1
         AND NOT EXISTS(SELECT 1 FROM wikis w WHERE w.agent_id=g.id) ORDER BY g.name,g.id",
    )
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(WikiAdminIndex {
        items,
        eligible_agents: agents
            .into_iter()
            .map(|(id, name)| AgentSummary { id, name })
            .collect(),
    })
}

pub async fn admin_agents(
    pool: &SqlitePool,
    user: &User,
) -> Result<Items<AgentOnboardingItem>, ApiError> {
    let mut tx = pool.begin().await?;
    access::admin_on(&mut tx, &user.id).await?;
    let rows: Vec<AgentProjection> =
        sqlx::query_as("SELECT a.id,a.name,a.active,c.destination_id,c.expected_identity,c.revision
                       FROM agents a LEFT JOIN agent_connections c ON c.agent_id=a.id ORDER BY a.name,a.id")
            .fetch_all(&mut *tx)
            .await?;
    tx.commit().await?;
    Ok(Items {
        items: rows.into_iter().map(|(id, name, active, destination, identity, revision)| {
            let connection = destination.zip(identity).zip(revision).map(|((destination_id, expected_identity), revision)| {
                ConnectionSummary { destination_id, expected_identity, revision }
            });
            AgentOnboardingItem { id, name, active, connection }
        }).collect(),
    })
}

type AgentProjection = (String, String, bool, Option<String>, Option<String>, Option<String>);

pub async fn connection_config(
    pool: &SqlitePool, user: &User, agent: &str,
) -> Result<Option<ConnectionSummary>, ApiError> {
    let mut tx = pool.begin().await?;
    access::admin_on(&mut tx, &user.id).await?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM agents WHERE id=?)")
        .bind(agent).fetch_one(&mut *tx).await?;
    if !exists {
        return Err(ApiError(ErrorCode::InvalidInput));
    }
    let config: Option<(String, String, String)> =
        sqlx::query_as("SELECT destination_id,expected_identity,revision FROM agent_connections WHERE agent_id=?")
            .bind(agent).fetch_optional(&mut *tx).await?;
    tx.commit().await?;
    Ok(config.map(|(destination_id, expected_identity, revision)| ConnectionSummary { destination_id, expected_identity, revision }))
}

pub async fn save_connection(
    pool: &SqlitePool, user: &User, agent: &str, config: ConnectionInput,
) -> Result<ConnectionSummary, ApiError> {
    text(&config.expected_identity)?;
    db::write(pool, |conn| {
        let (user, agent) = (user.id.clone(), agent.to_owned());
        let (destination_id, expected_identity) = (config.destination_id.clone(), config.expected_identity.clone());
        Box::pin(async move {
            access::admin_on(conn, &user).await?;
            let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM agents WHERE id=?)")
                .bind(&agent).fetch_one(&mut *conn).await?;
            if !exists {
                return Err(ApiError(ErrorCode::InvalidInput));
            }
            let revision: String = sqlx::query_scalar(
                "INSERT INTO agent_connections(agent_id,destination_id,expected_identity,revision)
                 VALUES(?,?,?,?) ON CONFLICT(agent_id) DO UPDATE SET
                   revision=CASE WHEN destination_id=excluded.destination_id
                                  AND expected_identity=excluded.expected_identity
                                 THEN revision ELSE excluded.revision END,
                   destination_id=excluded.destination_id,expected_identity=excluded.expected_identity
                 RETURNING revision")
                .bind(agent).bind(&destination_id).bind(&expected_identity).bind(id())
                .fetch_one(conn).await?;
            Ok(ConnectionSummary { destination_id, expected_identity, revision })
        })
    }).await
}

pub async fn register_agent(
    pool: &SqlitePool,
    user: &User,
    input: AgentRegistration,
) -> Result<AgentOnboardingItem, ApiError> {
    let id = input.id.trim();
    let name = input.name.trim();
    db::write(pool, |conn| {
        let (id, name, user_id) = (id.to_owned(), name.to_owned(), user.id.clone());
        Box::pin(async move {
            access::admin_on(conn, &user_id).await?;
            text(&id)?;
            if matches!(id.as_str(), "." | "..") {
                return Err(ApiError(ErrorCode::InvalidInput));
            }
            text(&name)?;
            let result = sqlx::query("INSERT INTO agents(id,name) VALUES(?,?) ON CONFLICT(id) DO NOTHING")
                .bind(&id)
                .bind(&name)
                .execute(conn)
                .await?;
            if result.rows_affected() == 0 {
                return Err(ApiError(ErrorCode::AgentAlreadyExists));
            }
            Ok(AgentOnboardingItem { id, name, active: true, connection: None })
        })
    }).await
}

pub async fn wiki_activity(
    pool: &SqlitePool,
    user: &User,
) -> Result<Items<WikiActivitySummary>, ApiError> {
    let mut tx = pool.begin().await?;
    access::admin_on(&mut tx, &user.id).await?;
    let wikis = admin_wiki_items(&mut tx).await?;
    let mut items = Vec::with_capacity(wikis.len());
    for wiki in wikis {
        let can_read = match access::assignment_on(&mut tx, &user.id, &wiki.id).await {
            Ok(()) => true,
            Err(ApiError(ErrorCode::NotFound)) => false,
            Err(error) => return Err(error),
        };
        let (record_count, source_count): (i64, i64) = sqlx::query_as(
            "SELECT (SELECT count(*) FROM records WHERE wiki_id=?),
                    (SELECT count(*) FROM original_sources WHERE wiki_id=?)",
        )
        .bind(&wiki.id)
        .bind(&wiki.id)
        .fetch_one(&mut *tx)
        .await?;
        // db::now writes UTC; removing Z orders whole seconds before their fractional parts.
        let events: Vec<(String, Option<String>)> = sqlx::query_as(
            "SELECT e.created_at, CASE WHEN ? THEN u.username END
             FROM wiki_events e JOIN users u ON u.id=e.actor_id
             WHERE e.wiki_id=? AND e.action IN
               ('record_created','source_created','revision_saved','proposal_pending','conflict_resolved')
             ORDER BY rtrim(e.created_at,'Z') DESC,e.sequence DESC LIMIT 10",
        )
        .bind(can_read)
        .bind(&wiki.id)
        .fetch_all(&mut *tx)
        .await?;
        items.push(WikiActivitySummary {
            wiki,
            record_count,
            source_count,
            last_contribution_at: events.first().map(|(date, _)| date.clone()),
            recent_activity: events
                .into_iter()
                .map(|(occurred_at, author_name)| WikiActivityEntry {
                    occurred_at,
                    author_name,
                })
                .collect(),
        });
    }
    tx.commit().await?;
    Ok(Items { items })
}

pub async fn create_wiki(
    pool: &SqlitePool,
    user: &User,
    input: WikiCreateInput,
) -> Result<WikiAdminItem, ApiError> {
    auth::require_admin(user)?;
    let user_id = &user.id;
    let name = input.name.trim();
    let agent_id = input.agent_id.as_str();
    let wiki_id = id();
    db::write(pool, |conn| {
        let wiki_id = wiki_id.clone();
        let user_id = user_id.clone();
        let name = name.to_owned();
        let agent_id = agent_id.to_owned();
        Box::pin(async move {
            access::admin_on(conn, &user_id).await?;
            text(&name)?;
            let agent_name: String = sqlx::query_scalar("SELECT name FROM agents WHERE id=? AND active=1")
                .bind(&agent_id)
                .fetch_optional(&mut *conn)
                .await?
                .ok_or(ApiError(ErrorCode::InvalidInput))?;
            let result = sqlx::query("INSERT INTO wikis(id,agent_id,name) VALUES(?,?,?) ON CONFLICT(agent_id) DO NOTHING")
                .bind(&wiki_id)
                .bind(&agent_id)
                .bind(&name)
                .execute(conn)
                .await?;
            if result.rows_affected() == 0 {
                return Err(ApiError(ErrorCode::WikiAlreadyExists));
            }
            Ok(WikiAdminItem { id: wiki_id, name, agent: AgentSummary { id: agent_id, name: agent_name } })
        })
    }).await
}

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn text(value: &str) -> Result<(), ApiError> {
    if value.trim().is_empty() {
        Err(ApiError(ErrorCode::InvalidInput))
    } else {
        Ok(())
    }
}
fn enum_value<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .expect("enum serializes")
        .as_str()
        .expect("string enum")
        .into()
}
fn parse_enum<T: DeserializeOwned>(value: String) -> Result<T, ApiError> {
    serde_json::from_value(serde_json::Value::String(value))
        .map_err(|_| ApiError(ErrorCode::InternalError))
}
fn fingerprint(value: impl Serialize) -> Result<String, ApiError> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|_| ApiError(ErrorCode::InvalidInput))?)
    ))
}
fn submission(value: &str) -> Result<String, ApiError> {
    uuid::Uuid::parse_str(value)
        .map(|uuid| uuid.to_string())
        .map_err(|_| ApiError(ErrorCode::InvalidInput))
}

fn summary(row: &SqliteRow) -> Result<RecordSummary, ApiError> {
    Ok(RecordSummary {
        id: row.try_get("id")?,
        wiki_id: row.try_get("wiki_id")?,
        kind: parse_enum(row.try_get("kind")?)?,
        title: row.try_get("title")?,
        project_id: row.try_get("project_id")?,
        client_id: row.try_get("client_id")?,
        head_revision_id: row.try_get("head_revision_id")?,
        generation: row.try_get("generation")?,
        state: parse_enum(row.try_get("state")?)?,
    })
}
fn revision(row: &SqliteRow) -> Result<Revision, ApiError> {
    Ok(Revision {
        id: row.try_get("id")?,
        record_id: row.try_get("record_id")?,
        parent_revision_id: row.try_get("parent_revision_id")?,
        body_markdown: row.try_get("body_markdown")?,
        author_id: row.try_get("author_id")?,
        created_at: row.try_get("created_at")?,
        epistemic_kind: parse_enum(row.try_get("epistemic_kind")?)?,
    })
}
fn source_row(row: &SqliteRow) -> Result<Source, ApiError> {
    Ok(Source {
        id: row.try_get("id")?,
        wiki_id: row.try_get("wiki_id")?,
        body: row.try_get("body")?,
        content_hash: row.try_get("content_hash")?,
        origin_kind: row.try_get("origin_kind")?,
        origin_ref: row.try_get("origin_ref")?,
        project_id: row.try_get("project_id")?,
        contributor_id: row.try_get("contributor_id")?,
        received_at: row.try_get("received_at")?,
    })
}
async fn record_on(
    conn: &mut SqliteConnection,
    wiki: &str,
    record: &str,
) -> Result<RecordSummary, ApiError> {
    let row = sqlx::query("SELECT * FROM records WHERE wiki_id=? AND id=?")
        .bind(wiki)
        .bind(record)
        .fetch_optional(conn)
        .await?
        .ok_or(ApiError(ErrorCode::NotFound))?;
    summary(&row)
}
async fn source_ids_on(
    conn: &mut SqliteConnection,
    wiki: &str,
    sources: &[String],
) -> Result<(), ApiError> {
    let mut seen = std::collections::HashSet::new();
    for source in sources {
        if !seen.insert(source) {
            return Err(ApiError(ErrorCode::InvalidInput));
        }
        let present: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM original_sources WHERE wiki_id=? AND id=?)",
        )
        .bind(wiki)
        .bind(source)
        .fetch_one(&mut *conn)
        .await?;
        if !present {
            return Err(ApiError(ErrorCode::InvalidInput));
        }
    }
    Ok(())
}
async fn project_on(
    conn: &mut SqliteConnection,
    project: Option<&str>,
    client: Option<&str>,
) -> Result<(), ApiError> {
    if let Some(project) = project {
        let row: Option<(Option<String>,)> =
            sqlx::query_as("SELECT client_id FROM projects WHERE id=?")
                .bind(project)
                .fetch_optional(&mut *conn)
                .await?;
        let (actual,) = row.ok_or(ApiError(ErrorCode::InvalidInput))?;
        if client.is_some() && actual.as_deref() != client {
            return Err(ApiError(ErrorCode::InvalidInput));
        }
    }
    if let Some(client) = client {
        let present: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM clients WHERE id=?)")
            .bind(client)
            .fetch_one(&mut *conn)
            .await?;
        if !present {
            return Err(ApiError(ErrorCode::InvalidInput));
        }
    }
    Ok(())
}
async fn event(
    conn: &mut SqliteConnection,
    wiki: &str,
    actor: &str,
    action: &str,
    reference: &str,
) -> Result<(), ApiError> {
    sqlx::query("INSERT INTO wiki_events(id,wiki_id,actor_id,action,reference_id,created_at) VALUES(?,?,?,?,?,?)").bind(id()).bind(wiki).bind(actor).bind(action).bind(reference).bind(db::now()).execute(conn).await?;
    Ok(())
}
async fn add_revision(
    conn: &mut SqliteConnection,
    revision: &Revision,
    sources: &[String],
) -> Result<(), ApiError> {
    sqlx::query("INSERT INTO revisions(id,record_id,parent_revision_id,body_markdown,author_id,created_at,epistemic_kind) VALUES(?,?,?,?,?,?,?)").bind(&revision.id).bind(&revision.record_id).bind(&revision.parent_revision_id).bind(&revision.body_markdown).bind(&revision.author_id).bind(&revision.created_at).bind(enum_value(&revision.epistemic_kind)).execute(&mut *conn).await?;
    for source in sources {
        sqlx::query("INSERT INTO source_references(revision_id,source_id) VALUES(?,?)")
            .bind(&revision.id)
            .bind(source)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}
async fn detail_on(
    conn: &mut SqliteConnection,
    wiki: &str,
    record: &str,
) -> Result<RecordDetail, ApiError> {
    let summary = record_on(conn, wiki, record).await?;
    let row = sqlx::query("SELECT * FROM revisions WHERE id=? AND record_id=?")
        .bind(&summary.head_revision_id)
        .bind(record)
        .fetch_one(&mut *conn)
        .await?;
    let revision = revision(&row)?;
    let sources=sqlx::query("SELECT s.* FROM original_sources s JOIN source_references f ON f.source_id=s.id WHERE f.revision_id=? AND s.wiki_id=? ORDER BY s.id").bind(&revision.id).bind(wiki).fetch_all(&mut *conn).await?.iter().map(source_row).collect::<Result<Vec<_>,_>>()?;
    let rows:Vec<(String,String,String)>=sqlx::query_as("SELECT source_record_id,target_record_id,relation FROM record_links WHERE source_record_id=? ORDER BY target_record_id,relation").bind(record).fetch_all(&mut *conn).await?;
    let links = rows
        .into_iter()
        .map(
            |(source_record_id, target_record_id, relation)| RecordLink {
                source_record_id,
                target_record_id,
                relation,
            },
        )
        .collect();
    let review = sqlx::query(
        "SELECT * FROM knowledge_reviews WHERE revision_id=? ORDER BY sequence DESC LIMIT 1",
    )
    .bind(&revision.id)
    .fetch_optional(&mut *conn)
    .await?
    .map(|row| -> Result<_, ApiError> {
        Ok(KnowledgeReview {
            revision_id: row.try_get("revision_id")?,
            result: parse_enum(row.try_get("result")?)?,
            reviewer_id: row.try_get("reviewer_id")?,
            reviewed_at: row.try_get("reviewed_at")?,
            note: row.try_get("note")?,
        })
    })
    .transpose()?;
    Ok(RecordDetail {
        summary,
        revision,
        sources,
        links,
        review,
    })
}

pub async fn create_source(
    pool: &SqlitePool,
    user: &User,
    wiki: &str,
    input: SourceInput,
) -> Result<Source, ApiError> {
    let user = user.clone();
    let wiki = wiki.to_owned();
    db::write(pool, move |conn| { let user=user.clone();let wiki=wiki.clone();let input=input.clone();Box::pin(async move {
        access::assignment_on(conn,&user.id,&wiki).await?;
        text(&input.body)?;text(&input.origin_kind)?;text(&input.origin_ref)?;
        project_on(conn,input.project_id.as_deref(),None).await?;
        let source=Source{id:id(),wiki_id:wiki,content_hash:format!("{:x}",Sha256::digest(input.body.as_bytes())),body:input.body,origin_kind:input.origin_kind,origin_ref:input.origin_ref,project_id:input.project_id,contributor_id:user.id,received_at:db::now()};
        sqlx::query("INSERT INTO original_sources(id,wiki_id,body,content_hash,origin_kind,origin_ref,project_id,contributor_id,received_at) VALUES(?,?,?,?,?,?,?,?,?)").bind(&source.id).bind(&source.wiki_id).bind(&source.body).bind(&source.content_hash).bind(&source.origin_kind).bind(&source.origin_ref).bind(&source.project_id).bind(&source.contributor_id).bind(&source.received_at).execute(&mut *conn).await?;
        event(conn,&source.wiki_id,&source.contributor_id,"source_created",&source.id).await?;
        Ok(source)
    })}).await
}

pub async fn get_source(
    pool: &SqlitePool,
    user: &User,
    wiki: &str,
    source: &str,
) -> Result<Source, ApiError> {
    let mut tx = pool.begin().await?;
    access::assignment_on(&mut tx, &user.id, wiki).await?;
    let row = sqlx::query("SELECT * FROM original_sources WHERE wiki_id=? AND id=?")
        .bind(wiki)
        .bind(source)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ApiError(ErrorCode::NotFound))?;
    let result = source_row(&row)?;
    tx.commit().await?;
    Ok(result)
}

pub async fn create_record(
    pool: &SqlitePool,
    user: &User,
    wiki: &str,
    input: RecordInput,
) -> Result<RecordDetail, ApiError> {
    let user = user.clone();
    let wiki = wiki.to_owned();
    db::write(pool,move|conn| {let user=user.clone();let wiki=wiki.clone();let mut input=input.clone();Box::pin(async move {
        access::assignment_on(conn,&user.id,&wiki).await?;
        input.submission_id=submission(&input.submission_id)?;text(&input.title)?;text(&input.body_markdown)?;
        let hash=fingerprint((&wiki,&input))?;
        let previous:Option<(String,String)>=sqlx::query_as("SELECT payload_hash,record_id FROM record_submissions WHERE author_id=? AND submission_id=?").bind(&user.id).bind(&input.submission_id).fetch_optional(&mut *conn).await?;
        if let Some((previous_hash,record))=previous {
            if previous_hash!=hash {return Err(ApiError(ErrorCode::IdempotencyConflict));}
            return detail_on(conn,&wiki,&record).await;
        }
        if input.kind==RecordKind::Rfd && input.project_id.is_none() {return Err(ApiError(ErrorCode::InvalidInput));}
        project_on(conn,input.project_id.as_deref(),input.client_id.as_deref()).await?;
        source_ids_on(conn,&wiki,&input.source_ids).await?;
        let record=id();let revision=Revision{id:id(),record_id:record.clone(),parent_revision_id:None,body_markdown:input.body_markdown,author_id:user.id.clone(),created_at:db::now(),epistemic_kind:input.epistemic_kind};
        sqlx::query("INSERT INTO records(id,wiki_id,kind,title,project_id,client_id,head_revision_id,generation,state) VALUES(?,?,?,?,?,?,?,1,'active')").bind(&record).bind(&wiki).bind(enum_value(&input.kind)).bind(&input.title).bind(&input.project_id).bind(&input.client_id).bind(&revision.id).execute(&mut *conn).await?;
        add_revision(conn,&revision,&input.source_ids).await?;
        sqlx::query("INSERT INTO record_submissions(author_id,submission_id,payload_hash,record_id) VALUES(?,?,?,?)").bind(&user.id).bind(&input.submission_id).bind(hash).bind(&record).execute(&mut *conn).await?;
        event(conn,&wiki,&user.id,"record_created",&record).await?;
        detail_on(conn,&wiki,&record).await
    })}).await
}

pub async fn get_record(
    pool: &SqlitePool,
    user: &User,
    wiki: &str,
    record: &str,
) -> Result<RecordDetail, ApiError> {
    let mut tx = pool.begin().await?;
    access::assignment_on(&mut tx, &user.id, wiki).await?;
    let result = detail_on(&mut tx, wiki, record).await?;
    tx.commit().await?;
    Ok(result)
}

pub async fn records(
    pool: &SqlitePool,
    user: &User,
    wiki: &str,
    query: RecordQuery,
) -> Result<Items<RecordSummary>, ApiError> {
    let mut tx = pool.begin().await?;
    access::assignment_on(&mut tx, &user.id, wiki).await?;
    // Advanced query behavior belongs to US3; never silently ignore an explicit filter.
    if query.q.is_some()
        || query.agent_id.is_some()
        || query.client_id.is_some()
        || query.project_id.is_some()
    {
        return Err(ApiError(ErrorCode::InvalidInput));
    }
    let limit = query.limit.unwrap_or(100);
    if limit == 0 || limit > 100 {
        return Err(ApiError(ErrorCode::InvalidInput));
    }
    let rows = sqlx::query("SELECT * FROM records WHERE wiki_id=? ORDER BY id LIMIT ? OFFSET ?")
        .bind(wiki)
        .bind(limit)
        .bind(query.offset.unwrap_or(0))
        .fetch_all(&mut *tx)
        .await?;
    let items = rows.iter().map(summary).collect::<Result<Vec<_>, _>>()?;
    tx.commit().await?;
    Ok(Items { items })
}

async fn proposal_result(
    conn: &mut SqliteConnection,
    proposal_id: &str,
) -> Result<(u16, ProposalResult), ApiError> {
    let row=sqlx::query("SELECT p.id,p.record_id,p.proposed_revision_id,r.generation,r.state FROM edit_proposals p JOIN records r ON r.id=p.record_id WHERE p.id=?").bind(proposal_id).fetch_one(conn).await?;
    let pending = row.try_get::<String, _>("state")? == "conflicted";
    Ok((
        if pending { 202 } else { 201 },
        ProposalResult {
            record_id: row.try_get("record_id")?,
            proposal_id: row.try_get("id")?,
            revision_id: row.try_get("proposed_revision_id")?,
            state: if pending {
                ProposalState::Pending
            } else {
                ProposalState::Active
            },
            generation: row.try_get("generation")?,
        },
    ))
}

pub async fn propose(
    pool: &SqlitePool,
    user: &User,
    wiki: &str,
    record: &str,
    input: ProposalInput,
) -> Result<(u16, ProposalResult), ApiError> {
    let user = user.clone();
    let wiki = wiki.to_owned();
    let record = record.to_owned();
    db::write(pool,move|conn| {let user=user.clone();let wiki=wiki.clone();let record=record.clone();let mut input=input.clone();Box::pin(async move {
        access::assignment_on(conn,&user.id,&wiki).await?;
        let current=record_on(conn,&wiki,&record).await?;
        input.submission_id=submission(&input.submission_id)?;text(&input.body_markdown)?;
        let hash=fingerprint((&wiki,&record,&input))?;
        let previous:Option<(String,String)>=sqlx::query_as("SELECT payload_hash,id FROM edit_proposals WHERE author_id=? AND submission_id=?").bind(&user.id).bind(&input.submission_id).fetch_optional(&mut *conn).await?;
        if let Some((previous_hash,proposal))=previous {
            if previous_hash!=hash {return Err(ApiError(ErrorCode::IdempotencyConflict));}
            return proposal_result(conn,&proposal).await;
        }
        let valid_base:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM revisions WHERE id=? AND record_id=?)").bind(&input.base_revision_id).bind(&record).fetch_one(&mut *conn).await?;
        if !valid_base {return Err(ApiError(ErrorCode::InvalidInput));}
        source_ids_on(conn,&wiki,&input.source_ids).await?;
        let revision=Revision{id:id(),record_id:record.clone(),parent_revision_id:Some(input.base_revision_id.clone()),body_markdown:input.body_markdown,author_id:user.id.clone(),created_at:db::now(),epistemic_kind:input.epistemic_kind};
        add_revision(conn,&revision,&input.source_ids).await?;
        let proposal=id();let generation=current.generation+1;
        let active=current.state==RecordState::Active && current.head_revision_id==input.base_revision_id;
        sqlx::query("INSERT INTO edit_proposals(id,record_id,base_revision_id,proposed_revision_id,submission_id,payload_hash,author_id,created_at,state) VALUES(?,?,?,?,?,?,?,?,?)").bind(&proposal).bind(&record).bind(&input.base_revision_id).bind(&revision.id).bind(&input.submission_id).bind(hash).bind(&user.id).bind(&revision.created_at).bind(if active {"active"}else{"pending"}).execute(&mut *conn).await?;
        if active {
            sqlx::query("UPDATE records SET head_revision_id=?,generation=?,state='active' WHERE id=?").bind(&revision.id).bind(generation).bind(&record).execute(&mut *conn).await?;
        } else {
            let open:Option<String>=sqlx::query_scalar("SELECT id FROM edit_conflicts WHERE record_id=? AND state='pending'").bind(&record).fetch_optional(&mut *conn).await?;
            let conflict=if let Some(conflict)=open {
                sqlx::query("UPDATE edit_conflicts SET generation=? WHERE id=?").bind(generation).bind(&conflict).execute(&mut *conn).await?;
                sqlx::query("UPDATE records SET generation=? WHERE id=?").bind(generation).bind(&record).execute(&mut *conn).await?;
                conflict
            } else {
                let conflict=id();
                sqlx::query("INSERT INTO edit_conflicts(id,record_id,base_revision_id,generation,state) VALUES(?,?,?,?,'pending')").bind(&conflict).bind(&record).bind(&input.base_revision_id).bind(generation).execute(&mut *conn).await?;
                let existing:Option<String>=sqlx::query_scalar("SELECT id FROM edit_proposals WHERE record_id=? AND proposed_revision_id=?").bind(&record).bind(&current.head_revision_id).fetch_optional(&mut *conn).await?;
                let candidate=if let Some(existing)=existing {existing} else {
                    // A late edit also preserves a previous manual resolution as a candidate.
                    let previous=sqlx::query("SELECT author_id,created_at FROM revisions WHERE id=? AND record_id=?").bind(&current.head_revision_id).bind(&record).fetch_one(&mut *conn).await?;
                    let candidate=id();
                    sqlx::query("INSERT INTO edit_proposals(id,record_id,base_revision_id,proposed_revision_id,submission_id,payload_hash,author_id,created_at,state) VALUES(?,?,?,?,?,?,?,?,'pending')").bind(&candidate).bind(&record).bind(&input.base_revision_id).bind(&current.head_revision_id).bind(id()).bind(fingerprint((&current.head_revision_id,&input.base_revision_id))?).bind(previous.try_get::<String,_>("author_id")?).bind(previous.try_get::<String,_>("created_at")?).execute(&mut *conn).await?;
                    candidate
                };
                sqlx::query("INSERT INTO conflict_candidates(conflict_id,proposal_id) VALUES(?,?)").bind(&conflict).bind(&candidate).execute(&mut *conn).await?;
                sqlx::query("UPDATE edit_proposals SET state='pending' WHERE id=?").bind(&candidate).execute(&mut *conn).await?;
                sqlx::query("UPDATE records SET head_revision_id=?,generation=?,state='conflicted' WHERE id=?").bind(&input.base_revision_id).bind(generation).bind(&record).execute(&mut *conn).await?;
                conflict
            };
            sqlx::query("INSERT INTO conflict_candidates(conflict_id,proposal_id) VALUES(?,?)").bind(&conflict).bind(&proposal).execute(&mut *conn).await?;
        }
        event(conn,&wiki,&user.id,if active {"revision_saved"}else{"proposal_pending"},&proposal).await?;
        proposal_result(conn,&proposal).await
    })}).await
}

pub async fn conflicts(
    pool: &SqlitePool,
    user: &User,
    wiki: &str,
) -> Result<Items<Conflict>, ApiError> {
    let mut tx = pool.begin().await?;
    access::assignment_on(&mut tx, &user.id, wiki).await?;
    let rows=sqlx::query("SELECT c.* FROM edit_conflicts c JOIN records r ON r.id=c.record_id WHERE r.wiki_id=? AND c.state='pending' ORDER BY c.id").bind(wiki).fetch_all(&mut *tx).await?;
    let mut items = vec![];
    for row in rows {
        let conflict_id: String = row.try_get("id")?;
        let candidates=sqlx::query("SELECT p.id AS proposal_id,p.submission_id,p.base_revision_id,v.* FROM conflict_candidates c JOIN edit_proposals p ON p.id=c.proposal_id JOIN revisions v ON v.id=p.proposed_revision_id WHERE c.conflict_id=? ORDER BY p.created_at,p.id").bind(&conflict_id).fetch_all(&mut *tx).await?;
        let proposals = candidates
            .iter()
            .map(|row| -> Result<_, ApiError> {
                Ok(ConflictProposal {
                    id: row.try_get("proposal_id")?,
                    submission_id: row.try_get("submission_id")?,
                    base_revision_id: row.try_get("base_revision_id")?,
                    revision: revision(row)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        items.push(Conflict {
            id: conflict_id,
            record_id: row.try_get("record_id")?,
            base_revision_id: row.try_get("base_revision_id")?,
            generation: row.try_get("generation")?,
            state: ConflictState::Pending,
            proposals,
        });
    }
    tx.commit().await?;
    Ok(Items { items })
}

pub async fn resolve(
    pool: &SqlitePool,
    user: &User,
    wiki: &str,
    conflict: &str,
    input: ResolutionInput,
) -> Result<RecordDetail, ApiError> {
    auth::require_admin(user)?;
    let user = user.clone();
    let wiki = wiki.to_owned();
    let conflict = conflict.to_owned();
    db::write(pool,move|conn| {let user=user.clone();let wiki=wiki.clone();let conflict=conflict.clone();let input=input.clone();Box::pin(async move {
        access::admin_on(conn,&user.id).await?;access::assignment_on(conn,&user.id,&wiki).await?;
        text(&input.body_markdown)?;text(&input.reason)?;
        let row=sqlx::query("SELECT c.* FROM edit_conflicts c JOIN records r ON r.id=c.record_id WHERE c.id=? AND r.wiki_id=?").bind(&conflict).bind(&wiki).fetch_optional(&mut *conn).await?.ok_or(ApiError(ErrorCode::NotFound))?;
        if row.try_get::<String,_>("state")?!="pending" || row.try_get::<i64,_>("generation")?!=input.expected_generation {return Err(ApiError(ErrorCode::ConflictChanged));}
        let record:String=row.try_get("record_id")?;
        source_ids_on(conn,&wiki,&input.source_ids).await?;
        let revision=Revision{id:id(),record_id:record.clone(),parent_revision_id:Some(row.try_get("base_revision_id")?),body_markdown:input.body_markdown,author_id:user.id.clone(),created_at:db::now(),epistemic_kind:input.epistemic_kind};
        add_revision(conn,&revision,&input.source_ids).await?;
        sqlx::query("INSERT INTO conflict_resolutions(id,conflict_id,expected_generation,result_revision_id,administrator_id,created_at,reason) VALUES(?,?,?,?,?,?,?)").bind(id()).bind(&conflict).bind(input.expected_generation).bind(&revision.id).bind(&user.id).bind(&revision.created_at).bind(&input.reason).execute(&mut *conn).await?;
        sqlx::query("UPDATE edit_conflicts SET state='resolved' WHERE id=?").bind(&conflict).execute(&mut *conn).await?;
        sqlx::query("UPDATE edit_proposals SET state='resolved' WHERE id IN(SELECT proposal_id FROM conflict_candidates WHERE conflict_id=?)").bind(&conflict).execute(&mut *conn).await?;
        sqlx::query("UPDATE records SET head_revision_id=?,generation=generation+1,state='active' WHERE id=?").bind(&revision.id).bind(&record).execute(&mut *conn).await?;
        event(conn,&wiki,&user.id,"conflict_resolved",&conflict).await?;
        detail_on(conn,&wiki,&record).await
    })}).await
}

pub async fn link(
    pool: &SqlitePool,
    user: &User,
    wiki: &str,
    record: &str,
    input: LinkInput,
) -> Result<RecordLink, ApiError> {
    let user = user.clone();
    let wiki = wiki.to_owned();
    let record = record.to_owned();
    db::write(pool,move|conn| {let user=user.clone();let wiki=wiki.clone();let record=record.clone();let input=input.clone();Box::pin(async move {
        access::assignment_on(conn,&user.id,&wiki).await?;record_on(conn,&wiki,&record).await?;
        text(&input.relation)?;
        let target:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM records WHERE id=? AND wiki_id=?)").bind(&input.target_record_id).bind(&wiki).fetch_one(&mut *conn).await?;
        if !target {return Err(ApiError(ErrorCode::InvalidInput));}
        sqlx::query("INSERT INTO record_links(source_record_id,target_record_id,relation) VALUES(?,?,?) ON CONFLICT DO NOTHING").bind(&record).bind(&input.target_record_id).bind(&input.relation).execute(&mut *conn).await?;
        event(conn,&wiki,&user.id,"record_linked",&record).await?;
        Ok(RecordLink{source_record_id:record,target_record_id:input.target_record_id,relation:input.relation})
    })}).await
}

async fn maintenance_on(conn: &mut SqliteConnection, wiki: &str) -> Result<Maintenance, ApiError> {
    let row = sqlx::query("SELECT * FROM wiki_maintenance WHERE wiki_id=?")
        .bind(wiki)
        .fetch_optional(conn)
        .await?;
    match row {
        Some(row) => Ok(Maintenance {
            wiki_id: wiki.into(),
            rules: row.try_get("rules")?,
            status: parse_enum(row.try_get("status")?)?,
            reviewed_at: row.try_get("reviewed_at")?,
        }),
        None => Ok(Maintenance {
            wiki_id: wiki.into(),
            rules: String::new(),
            status: MaintenanceStatus::Pending,
            reviewed_at: None,
        }),
    }
}

pub async fn maintenance(
    pool: &SqlitePool,
    user: &User,
    wiki: &str,
) -> Result<Maintenance, ApiError> {
    let mut tx = pool.begin().await?;
    access::assignment_on(&mut tx, &user.id, wiki).await?;
    let result = maintenance_on(&mut tx, wiki).await?;
    tx.commit().await?;
    Ok(result)
}

pub async fn set_maintenance(
    pool: &SqlitePool,
    user: &User,
    wiki: &str,
    input: MaintenanceInput,
) -> Result<Maintenance, ApiError> {
    let user = user.clone();
    let wiki = wiki.to_owned();
    db::write(pool,move|conn| {let user=user.clone();let wiki=wiki.clone();let input=input.clone();Box::pin(async move {
        access::assignment_on(conn,&user.id,&wiki).await?;
        let reviewed_at=if input.status==MaintenanceStatus::Reviewed {Some(db::now())}else{None};
        sqlx::query("INSERT INTO wiki_maintenance(wiki_id,rules,status,reviewed_at) VALUES(?,?,?,?) ON CONFLICT(wiki_id) DO UPDATE SET rules=excluded.rules,status=excluded.status,reviewed_at=excluded.reviewed_at").bind(&wiki).bind(input.rules).bind(enum_value(&input.status)).bind(reviewed_at).execute(&mut *conn).await?;
        event(conn,&wiki,&user.id,"maintenance_updated",&wiki).await?;maintenance_on(conn,&wiki).await
    })}).await
}

pub async fn review(
    pool: &SqlitePool,
    user: &User,
    wiki: &str,
    record: &str,
    input: ReviewInput,
) -> Result<KnowledgeReview, ApiError> {
    let user = user.clone();
    let wiki = wiki.to_owned();
    let record = record.to_owned();
    db::write(pool,move|conn| {let user=user.clone();let wiki=wiki.clone();let record=record.clone();let input=input.clone();Box::pin(async move {
        access::assignment_on(conn,&user.id,&wiki).await?;record_on(conn,&wiki,&record).await?;
        let valid:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM revisions WHERE id=? AND record_id=?)").bind(&input.revision_id).bind(&record).fetch_one(&mut *conn).await?;
        if !valid {return Err(ApiError(ErrorCode::InvalidInput));}
        let review=KnowledgeReview{revision_id:input.revision_id,result:input.result,reviewer_id:user.id.clone(),reviewed_at:db::now(),note:input.note};
        sqlx::query("INSERT INTO knowledge_reviews(revision_id,result,reviewer_id,reviewed_at,note) VALUES(?,?,?,?,?)").bind(&review.revision_id).bind(enum_value(&review.result)).bind(&review.reviewer_id).bind(&review.reviewed_at).bind(&review.note).execute(&mut *conn).await?;
        event(conn,&wiki,&user.id,"knowledge_reviewed",&review.revision_id).await?;Ok(review)
    })}).await
}

pub async fn events(
    pool: &SqlitePool,
    user: &User,
    wiki: &str,
) -> Result<Items<WikiEvent>, ApiError> {
    let mut tx = pool.begin().await?;
    access::assignment_on(&mut tx, &user.id, wiki).await?;
    let rows = sqlx::query("SELECT * FROM wiki_events WHERE wiki_id=? ORDER BY sequence")
        .bind(wiki)
        .fetch_all(&mut *tx)
        .await?;
    let items = rows
        .iter()
        .map(|row| -> Result<_, ApiError> {
            Ok(WikiEvent {
                id: row.try_get("id")?,
                wiki_id: row.try_get("wiki_id")?,
                actor_id: row.try_get("actor_id")?,
                action: row.try_get("action")?,
                reference_id: row.try_get("reference_id")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    tx.commit().await?;
    Ok(Items { items })
}
