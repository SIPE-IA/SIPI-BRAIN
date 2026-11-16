#![allow(dead_code)]

use std::{path::PathBuf, str::FromStr};

use argon2::{Argon2, password_hash::PasswordHasher};
use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tower::ServiceExt;

pub const ORIGIN: &str = "http://127.0.0.1:8080";
pub const PASSWORD: &str = "Synthetic-test-only-42!";

pub struct TestClient {
    pub root: PathBuf,
    pub pool: SqlitePool,
    pub router: Router,
    pub cookie: String,
    pub csrf: String,
}

pub struct Reply {
    pub status: u16,
    pub body: Value,
    pub cookie_header: String,
}

impl TestClient {
    pub async fn knowledge() -> Self {
        let mut client = Self::new().await;
        client.session().await;
        client.seed_accounts().await;
        sqlx::query("INSERT INTO wiki_assignments(user_id,wiki_id,assigned_by,assigned_at) VALUES('two','insight','admin','2026-11-16T00:00:00Z')").execute(&client.pool).await.unwrap();
        client.login("one").await;
        client
    }

    pub async fn fork(&self, username: &str) -> Self {
        let pool = sipibrain::db::open(&self.root.join("test.db"))
            .await
            .unwrap();
        let router = sipibrain::api::application(pool.clone(), ORIGIN, &self.root)
            .await
            .unwrap();
        let mut client = Self {
            root: self.root.clone(),
            pool,
            router,
            cookie: String::new(),
            csrf: String::new(),
        };
        client.session().await;
        client.login(username).await;
        client
    }

    pub async fn post(&mut self, path: &str, body: Value) -> Reply {
        self.request("POST", path, body, Some(ORIGIN), true).await
    }

    pub async fn get(&mut self, path: &str) -> Reply {
        self.request("GET", path, Value::Null, None, false).await
    }

    pub async fn source(&mut self, body: &str) -> Value {
        let reply = self.post("/api/wikis/insight/sources",json!({"body":body,"origin_kind":"selected_text","origin_ref":"synthetic fixture","project_id":null})).await;
        assert_eq!(
            reply.status, 201,
            "selected original source must be persisted"
        );
        reply.body
    }

    pub async fn record(&mut self, body: &str, sources: Vec<String>) -> Value {
        let reply = self.post("/api/wikis/insight/records",json!({"submission_id":uuid::Uuid::new_v4().to_string(),"kind":"prompt","title":"Synthetic prompt","body_markdown":body,"project_id":null,"client_id":null,"epistemic_kind":"assumption","source_ids":sources})).await;
        assert_eq!(
            reply.status, 201,
            "ordinary assigned contribution must commit"
        );
        reply.body
    }

    pub async fn business_snapshot(&self) -> Value {
        let records: Vec<(String, String, i64, String)> =
            sqlx::query_as("SELECT id,head_revision_id,generation,state FROM records ORDER BY id")
                .fetch_all(&self.pool)
                .await
                .unwrap();
        let revisions: i64 = sqlx::query_scalar("SELECT count(*) FROM revisions")
            .fetch_one(&self.pool)
            .await
            .unwrap();
        let proposals: i64 = sqlx::query_scalar("SELECT count(*) FROM edit_proposals")
            .fetch_one(&self.pool)
            .await
            .unwrap();
        let resolutions: i64 = sqlx::query_scalar("SELECT count(*) FROM conflict_resolutions")
            .fetch_one(&self.pool)
            .await
            .unwrap();
        json!({"records":records,"revisions":revisions,"proposals":proposals,"resolutions":resolutions})
    }

    pub async fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("sipibrain-test-")
            .tempdir()
            .unwrap()
            .keep();
        eprintln!("scratch: {}", root.display());
        std::fs::write(root.join("index.html"), "<div id=app>fixture</div>").unwrap();
        let options =
            SqliteConnectOptions::from_str(&format!("sqlite://{}", root.join("test.db").display()))
                .unwrap()
                .create_if_missing(true)
                .foreign_keys(true);
        let pool = SqlitePool::connect_with(options).await.unwrap();
        let router = sipibrain::api::application(pool.clone(), ORIGIN, &root)
            .await
            .unwrap();
        Self {
            root,
            pool,
            router,
            cookie: String::new(),
            csrf: String::new(),
        }
    }

    pub async fn reopen(&mut self) {
        self.pool.close().await;
        self.pool = sipibrain::db::open(&self.root.join("test.db"))
            .await
            .unwrap();
        self.router = sipibrain::api::application(self.pool.clone(), ORIGIN, &self.root)
            .await
            .unwrap();
    }

    pub async fn request(
        &mut self,
        method: &str,
        path: &str,
        body: Value,
        origin: Option<&str>,
        csrf: bool,
    ) -> Reply {
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json");
        if !self.cookie.is_empty() {
            builder = builder.header("cookie", &self.cookie);
        }
        if let Some(origin) = origin {
            builder = builder.header("origin", origin);
        }
        if csrf {
            builder = builder.header("X-CSRF-Token", &self.csrf);
        }
        let body = if method == "GET" {
            Body::empty()
        } else {
            Body::from(body.to_string())
        };
        let response = self
            .router
            .clone()
            .oneshot(builder.body(body).unwrap())
            .await
            .unwrap();
        let status = response.status().as_u16();
        let cookie_header = response
            .headers()
            .get("set-cookie")
            .map(|v| v.to_str().unwrap().to_owned())
            .unwrap_or_default();
        if !cookie_header.is_empty() {
            self.cookie = cookie_header.split(';').next().unwrap().to_owned();
        }
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| json!({"non_json": String::from_utf8_lossy(&bytes)}));
        Reply {
            status,
            body,
            cookie_header,
        }
    }

    pub async fn session(&mut self) -> Reply {
        let reply = self
            .request("GET", "/api/session", Value::Null, None, false)
            .await;
        assert_eq!(
            reply.status, 200,
            "GET /api/session must establish prelogin session"
        );
        self.csrf = reply.body["csrf_token"].as_str().unwrap().to_owned();
        reply
    }

    pub async fn login(&mut self, username: &str) -> Reply {
        let reply = self
            .request(
                "POST",
                "/api/login",
                json!({"username":username,"password":PASSWORD}),
                Some(ORIGIN),
                true,
            )
            .await;
        assert_eq!(
            reply.status, 200,
            "login must authenticate an active seeded user"
        );
        self.csrf = reply.body["csrf_token"].as_str().unwrap().to_owned();
        reply
    }

    pub async fn seed_accounts(&self) {
        let hash = Argon2::default()
            .hash_password(PASSWORD.as_bytes())
            .unwrap()
            .to_string();
        for (id, role) in [
            ("admin", "admin"),
            ("one", "programmer"),
            ("two", "programmer"),
        ] {
            sqlx::query(
                "INSERT INTO users(id,username,password_hash,active,role) VALUES(?,?,?,1,?)",
            )
            .bind(id)
            .bind(id)
            .bind(&hash)
            .bind(role)
            .execute(&self.pool)
            .await
            .unwrap();
        }
        for id in ["insight", "estimator"] {
            sqlx::query("INSERT INTO agents(id,name,active) VALUES(?,?,1)")
                .bind(id)
                .bind(id)
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("INSERT INTO wikis(id,agent_id,name) VALUES(?,?,?)")
                .bind(id)
                .bind(id)
                .bind(id)
                .execute(&self.pool)
                .await
                .unwrap();
        }
        sqlx::query("INSERT INTO wiki_assignments(user_id,wiki_id,assigned_by,assigned_at) VALUES('one','insight','admin','2026-11-16T00:00:00Z')").execute(&self.pool).await.unwrap();
    }

    pub async fn assignment_count(&self) -> i64 {
        sqlx::query_scalar("SELECT count(*) FROM wiki_assignments")
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }
}
