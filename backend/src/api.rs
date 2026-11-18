use crate::{auth, db, models::*, services};
use axum::{
    Json, Router,
    extract::{Path as RoutePath, Query, Request, State, rejection::JsonRejection},
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get, post},
};
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;
use std::{path::Path, sync::Arc};
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::{Expiry, Session, SessionManagerLayer, cookie::SameSite};
use tower_sessions_sqlx_store::SqliteStore;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub origins: Arc<Vec<String>>,
}

pub async fn application(
    pool: SqlitePool,
    origin: &str,
    static_dir: &Path,
) -> Result<Router, String> {
    if !static_dir.is_dir() || !static_dir.join("index.html").is_file() {
        return Err("Static directory must contain index.html.".into());
    }
    db::migrate(&pool)
        .await
        .map_err(|_| "Database migration failed.")?;
    let store = SqliteStore::new(pool.clone());
    store
        .migrate()
        .await
        .map_err(|_| "Session migration failed.")?;
    let mut origins = vec![origin.to_owned()];
    if let Ok(dev_origin) = std::env::var("SIPIBRAIN_DEV_ORIGIN") {
        if dev_origin != "http://127.0.0.1:5173" && dev_origin != "http://localhost:5173" {
            return Err("Development origin must be exact loopback Vite origin.".into());
        }
        origins.push(dev_origin);
    }
    let state = AppState {
        pool,
        origins: Arc::new(origins),
    };
    let sessions = SessionManagerLayer::new(store)
        .with_name("sipibrain.sid")
        .with_http_only(true)
        .with_same_site(SameSite::Strict)
        .with_secure(origin.starts_with("https://"))
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(8)));
    let api = Router::new()
        .route("/session", get(session_state))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/wikis", get(wikis))
        .route("/wikis/{wiki_id}/records", get(records).post(create_record))
        .route("/wikis/{wiki_id}/records/{record_id}", get(record))
        .route("/wikis/{wiki_id}/sources", post(create_source))
        .route("/wikis/{wiki_id}/sources/{source_id}", get(source))
        .route(
            "/wikis/{wiki_id}/records/{record_id}/proposals",
            post(propose),
        )
        .route("/wikis/{wiki_id}/conflicts", get(conflicts))
        .route(
            "/wikis/{wiki_id}/conflicts/{conflict_id}/resolve",
            post(resolve),
        )
        .route("/wikis/{wiki_id}/records/{record_id}/links", post(link))
        .route("/wikis/{wiki_id}/records/{record_id}/reviews", post(review))
        .route(
            "/wikis/{wiki_id}/maintenance",
            get(maintenance).post(set_maintenance),
        )
        .route("/wikis/{wiki_id}/events", get(events))
        .route("/admin/assignments", get(assignments).post(set_assignment))
        .route("/admin/wikis", get(admin_wikis).post(create_wiki))
        .route("/admin/wiki-activity", get(wiki_activity))
        .fallback(not_found)
        .layer(middleware::from_fn_with_state(state.clone(), protect))
        .layer(sessions)
        .with_state(state);
    Ok(Router::new()
        .route("/api", any(not_found))
        .nest("/api", api)
        .fallback_service(
            ServeDir::new(static_dir).fallback(ServeFile::new(static_dir.join("index.html"))),
        ))
}

async fn protect(
    State(state): State<AppState>,
    session: Session,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if !matches!(request.method().as_str(), "GET" | "HEAD" | "OPTIONS") {
        let origin = request
            .headers()
            .get("origin")
            .and_then(|v| v.to_str().ok());
        let token = request
            .headers()
            .get("X-CSRF-Token")
            .and_then(|v| v.to_str().ok());
        let expected = session
            .get::<String>("csrf")
            .await
            .map_err(auth::session_error)?;
        if !origin.is_some_and(|o| state.origins.iter().any(|allowed| allowed == o))
            || token.is_none()
            || expected.as_deref() != token
        {
            return Err(ApiError(ErrorCode::InvalidRequestOrigin));
        }
    }
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    Ok(response)
}
fn input<T: DeserializeOwned>(payload: Result<Json<T>, JsonRejection>) -> Result<T, ApiError> {
    payload
        .map(|Json(value)| value)
        .map_err(|_| ApiError(ErrorCode::InvalidInput))
}
async fn session_state(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<SessionState>, ApiError> {
    Ok(Json(SessionState {
        user: auth::current_user(&state.pool, &session).await?,
        csrf_token: auth::csrf(&session).await?,
    }))
}
async fn login(
    State(state): State<AppState>,
    session: Session,
    payload: Result<Json<LoginInput>, JsonRejection>,
) -> Result<Json<SessionState>, ApiError> {
    let input: LoginInput = input(payload)?;
    let user = auth::authenticate(&state.pool, &input.username, input.password).await?;
    session.cycle_id().await.map_err(auth::session_error)?;
    session
        .insert("user_id", &user.id)
        .await
        .map_err(auth::session_error)?;
    let csrf_token = auth::token();
    session
        .insert("csrf", &csrf_token)
        .await
        .map_err(auth::session_error)?;
    Ok(Json(SessionState {
        user: Some(user),
        csrf_token,
    }))
}
async fn logout(
    session: Session,
    payload: Result<Json<serde_json::Value>, JsonRejection>,
) -> Result<Json<SessionState>, ApiError> {
    let value = input(payload)?;
    if value != serde_json::json!({}) {
        return Err(ApiError(ErrorCode::InvalidInput));
    }
    session.flush().await.map_err(auth::session_error)?;
    Ok(Json(SessionState {
        user: None,
        csrf_token: auth::csrf(&session).await?,
    }))
}
async fn wikis(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<Items<Wiki>>, ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    Ok(Json(services::wikis(&state.pool, &user).await?))
}
async fn records(
    State(state): State<AppState>,
    session: Session,
    RoutePath(wiki_id): RoutePath<String>,
    query: Result<Query<RecordQuery>, axum::extract::rejection::QueryRejection>,
) -> Result<Json<Items<RecordSummary>>, ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    let Query(query) = query.map_err(|_| ApiError(ErrorCode::InvalidInput))?;
    Ok(Json(
        services::records(&state.pool, &user, &wiki_id, query).await?,
    ))
}
async fn assignments(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<Assignments>, ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    Ok(Json(services::assignments(&state.pool, &user).await?))
}
async fn admin_wikis(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<WikiAdminIndex>, ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    Ok(Json(services::admin_wikis(&state.pool, &user).await?))
}
async fn wiki_activity(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<Items<WikiActivitySummary>>, ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    Ok(Json(services::wiki_activity(&state.pool, &user).await?))
}
async fn create_wiki(
    State(state): State<AppState>,
    session: Session,
    payload: Result<Json<WikiCreateInput>, JsonRejection>,
) -> Result<(StatusCode, Json<WikiAdminItem>), ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    auth::require_admin(&user)?;
    Ok((
        StatusCode::CREATED,
        Json(services::create_wiki(&state.pool, &user, input(payload)?).await?),
    ))
}
async fn set_assignment(
    State(state): State<AppState>,
    session: Session,
    payload: Result<Json<AssignmentInput>, JsonRejection>,
) -> Result<Json<AssignmentResult>, ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    auth::require_admin(&user)?;
    Ok(Json(
        services::set_assignment(&state.pool, &user, input(payload)?).await?,
    ))
}
async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorBody {
            code: ErrorCode::NotFound,
            message: "Resource not found.".into(),
        }),
    )
}

async fn create_source(
    State(state): State<AppState>,
    session: Session,
    RoutePath(wiki): RoutePath<String>,
    payload: Result<Json<SourceInput>, JsonRejection>,
) -> Result<(StatusCode, Json<Source>), ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    Ok((
        StatusCode::CREATED,
        Json(services::create_source(&state.pool, &user, &wiki, input(payload)?).await?),
    ))
}
async fn source(
    State(state): State<AppState>,
    session: Session,
    RoutePath((wiki, source)): RoutePath<(String, String)>,
) -> Result<Json<Source>, ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    Ok(Json(
        services::get_source(&state.pool, &user, &wiki, &source).await?,
    ))
}
async fn create_record(
    State(state): State<AppState>,
    session: Session,
    RoutePath(wiki): RoutePath<String>,
    payload: Result<Json<RecordInput>, JsonRejection>,
) -> Result<(StatusCode, Json<RecordDetail>), ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    Ok((
        StatusCode::CREATED,
        Json(services::create_record(&state.pool, &user, &wiki, input(payload)?).await?),
    ))
}
async fn record(
    State(state): State<AppState>,
    session: Session,
    RoutePath((wiki, record)): RoutePath<(String, String)>,
) -> Result<Json<RecordDetail>, ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    Ok(Json(
        services::get_record(&state.pool, &user, &wiki, &record).await?,
    ))
}
async fn propose(
    State(state): State<AppState>,
    session: Session,
    RoutePath((wiki, record)): RoutePath<(String, String)>,
    payload: Result<Json<ProposalInput>, JsonRejection>,
) -> Result<(StatusCode, Json<ProposalResult>), ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    let (status, result) =
        services::propose(&state.pool, &user, &wiki, &record, input(payload)?).await?;
    Ok((
        StatusCode::from_u16(status).expect("fixed proposal status"),
        Json(result),
    ))
}
async fn conflicts(
    State(state): State<AppState>,
    session: Session,
    RoutePath(wiki): RoutePath<String>,
) -> Result<Json<Items<Conflict>>, ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    Ok(Json(services::conflicts(&state.pool, &user, &wiki).await?))
}
async fn resolve(
    State(state): State<AppState>,
    session: Session,
    RoutePath((wiki, conflict)): RoutePath<(String, String)>,
    payload: Result<Json<ResolutionInput>, JsonRejection>,
) -> Result<(StatusCode, Json<RecordDetail>), ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    auth::require_admin(&user)?;
    Ok((
        StatusCode::CREATED,
        Json(services::resolve(&state.pool, &user, &wiki, &conflict, input(payload)?).await?),
    ))
}
async fn link(
    State(state): State<AppState>,
    session: Session,
    RoutePath((wiki, record)): RoutePath<(String, String)>,
    payload: Result<Json<LinkInput>, JsonRejection>,
) -> Result<(StatusCode, Json<RecordLink>), ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    Ok((
        StatusCode::CREATED,
        Json(services::link(&state.pool, &user, &wiki, &record, input(payload)?).await?),
    ))
}
async fn review(
    State(state): State<AppState>,
    session: Session,
    RoutePath((wiki, record)): RoutePath<(String, String)>,
    payload: Result<Json<ReviewInput>, JsonRejection>,
) -> Result<(StatusCode, Json<KnowledgeReview>), ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    Ok((
        StatusCode::CREATED,
        Json(services::review(&state.pool, &user, &wiki, &record, input(payload)?).await?),
    ))
}
async fn maintenance(
    State(state): State<AppState>,
    session: Session,
    RoutePath(wiki): RoutePath<String>,
) -> Result<Json<Maintenance>, ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    Ok(Json(
        services::maintenance(&state.pool, &user, &wiki).await?,
    ))
}
async fn set_maintenance(
    State(state): State<AppState>,
    session: Session,
    RoutePath(wiki): RoutePath<String>,
    payload: Result<Json<MaintenanceInput>, JsonRejection>,
) -> Result<Json<Maintenance>, ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    Ok(Json(
        services::set_maintenance(&state.pool, &user, &wiki, input(payload)?).await?,
    ))
}
async fn events(
    State(state): State<AppState>,
    session: Session,
    RoutePath(wiki): RoutePath<String>,
) -> Result<Json<Items<WikiEvent>>, ApiError> {
    let user = auth::require_user(&state.pool, &session).await?;
    Ok(Json(services::events(&state.pool, &user, &wiki).await?))
}
