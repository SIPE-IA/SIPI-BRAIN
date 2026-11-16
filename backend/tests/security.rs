mod common;

use common::{ORIGIN, TestClient};
use serde_json::{Value, json};

#[tokio::test]
async fn prelogin_has_session_bound_csrf_and_hardened_cookie() {
    let mut c = TestClient::new().await;
    let reply = c.session().await;
    assert!(reply.body["user"].is_null());
    assert!(c.csrf.len() >= 32);
    assert!(reply.cookie_header.contains("HttpOnly"));
    assert!(
        reply.cookie_header.contains("SameSite=Strict")
            || reply.cookie_header.contains("SameSite=Lax")
    );
    assert!(reply.cookie_header.contains("Max-Age="));
    assert!(!reply.cookie_header.to_lowercase().contains("domain="));
}

#[tokio::test]
async fn unauthenticated_knowledge_is_denied_as_json() {
    let mut c = TestClient::new().await;
    let reply = c
        .request("GET", "/api/wikis", Value::Null, None, false)
        .await;
    assert_eq!(reply.status, 401);
    assert_eq!(reply.body["code"], "unauthenticated");
}

#[tokio::test]
async fn login_rejects_missing_csrf_and_foreign_or_missing_origin() {
    let mut c = TestClient::new().await;
    let input = json!({"username":"one","password":"not-echo-this"});
    let missing = c
        .request("POST", "/api/login", input.clone(), Some(ORIGIN), false)
        .await;
    assert_eq!(missing.status, 403, "login needs prelogin CSRF");
    c.session().await;
    for origin in [
        None,
        Some("http://evil.invalid"),
        Some("http://127.0.0.1:8080.evil.invalid"),
    ] {
        let reply = c
            .request("POST", "/api/login", input.clone(), origin, true)
            .await;
        assert_eq!(reply.status, 403);
        assert_eq!(reply.body["code"], "invalid_request_origin");
        assert!(!reply.body.to_string().contains("not-echo-this"));
    }
}

#[tokio::test]
async fn invalid_credentials_are_generic_and_unknown_authority_fields_rejected() {
    let mut c = TestClient::new().await;
    c.session().await;
    c.seed_accounts().await;
    let wrong = c
        .request(
            "POST",
            "/api/login",
            json!({"username":"one","password":"private-wrong-value"}),
            Some(ORIGIN),
            true,
        )
        .await;
    assert_eq!(wrong.status, 401);
    assert_eq!(wrong.body["code"], "unauthenticated");
    assert!(!wrong.body.to_string().contains("private-wrong-value"));
    let forged = c
        .request(
            "POST",
            "/api/login",
            json!({"username":"one","password":common::PASSWORD,"role":"admin"}),
            Some(ORIGIN),
            true,
        )
        .await;
    assert_eq!(forged.status, 400);
    assert_eq!(forged.body["code"], "invalid_input");
}

#[tokio::test]
async fn login_rotates_session_persists_on_reopen_and_logout_invalidates() {
    let mut c = TestClient::new().await;
    c.session().await;
    c.seed_accounts().await;
    let prelogin_cookie = c.cookie.clone();
    let login = c.login("one").await;
    assert_eq!(login.body["user"]["role"], "programmer");
    assert_ne!(c.cookie, prelogin_cookie);
    let authenticated_cookie = c.cookie.clone();
    let persisted: i64 = sqlx::query_scalar("SELECT count(*) FROM tower_sessions")
        .fetch_one(&c.pool)
        .await
        .unwrap();
    assert!(
        persisted > 0,
        "session must live in SQLite, not MemoryStore"
    );
    c.reopen().await;
    assert_eq!(c.session().await.body["user"]["id"], "one");
    c.cookie = prelogin_cookie;
    assert!(c.session().await.body["user"].is_null());
    c.cookie = authenticated_cookie.clone();
    c.session().await;
    let denied = c
        .request("POST", "/api/logout", json!({}), None, true)
        .await;
    assert_eq!(denied.status, 403);
    assert_eq!(c.session().await.body["user"]["id"], "one");
    let logout = c
        .request("POST", "/api/logout", json!({}), Some(ORIGIN), true)
        .await;
    assert_eq!(logout.status, 200);
    assert!(logout.body["user"].is_null());
    c.cookie = authenticated_cookie;
    assert!(c.session().await.body["user"].is_null());
}

#[tokio::test]
async fn assignments_require_admin_and_admin_content_still_requires_assignment() {
    let mut c = TestClient::new().await;
    c.session().await;
    c.seed_accounts().await;
    c.login("one").await;
    let before = c.assignment_count().await;
    let attempted = c
        .request(
            "POST",
            "/api/admin/assignments",
            json!({"user_id":"one","wiki_id":"estimator","action":"grant"}),
            Some(ORIGIN),
            true,
        )
        .await;
    assert_eq!(attempted.status, 403);
    assert_eq!(c.assignment_count().await, before);
    assert_eq!(
        c.request("GET", "/api/admin/assignments", Value::Null, None, false)
            .await
            .status,
        403
    );
    let wikis = c
        .request("GET", "/api/wikis", Value::Null, None, false)
        .await;
    assert_eq!(wikis.status, 200);
    assert_eq!(wikis.body["items"].as_array().unwrap().len(), 1);
    assert_eq!(wikis.body["items"][0]["id"], "insight");
    let foreign = c
        .request(
            "GET",
            "/api/wikis/estimator/records",
            Value::Null,
            None,
            false,
        )
        .await;
    let absent = c
        .request("GET", "/api/wikis/absent/records", Value::Null, None, false)
        .await;
    assert_eq!((foreign.status, foreign.body), (absent.status, absent.body));
    c.login("admin").await;
    assert_eq!(
        c.request(
            "GET",
            "/api/wikis/insight/records",
            Value::Null,
            None,
            false
        )
        .await
        .status,
        404
    );
    let granted = c
        .request(
            "POST",
            "/api/admin/assignments",
            json!({"user_id":"two","wiki_id":"insight","action":"grant"}),
            Some(ORIGIN),
            true,
        )
        .await;
    assert_eq!(granted.status, 200);
    assert_eq!(granted.body["assigned"], true);
    let before_invalid = c.assignment_count().await;
    let invalid = c
        .request(
            "POST",
            "/api/admin/assignments",
            json!({"user_id":"absent","wiki_id":"insight","action":"grant"}),
            Some(ORIGIN),
            true,
        )
        .await;
    assert_eq!(invalid.status, 400);
    assert_eq!(c.assignment_count().await, before_invalid);

    let revoked = c
        .request(
            "POST",
            "/api/admin/assignments",
            json!({"user_id":"two","wiki_id":"insight","action":"revoke"}),
            Some(ORIGIN),
            true,
        )
        .await;
    assert_eq!(revoked.status, 200);
    assert_eq!(revoked.body["assigned"], false);
    assert_eq!(c.assignment_count().await, before_invalid - 1);
}

#[tokio::test]
async fn accounts_use_salted_phc_hashes_and_inactive_sessions_lose_authority() {
    let mut c = TestClient::new().await;
    c.session().await;
    let admin = sipibrain::auth::create_user(&c.pool, "admin", common::PASSWORD.into(), true)
        .await
        .unwrap();
    assert!(
        sipibrain::auth::create_user(&c.pool, "second-admin", common::PASSWORD.into(), true)
            .await
            .is_err()
    );
    assert!(
        sipibrain::auth::create_user(&c.pool, "admin", common::PASSWORD.into(), false)
            .await
            .is_err()
    );
    sipibrain::auth::create_user(&c.pool, "other", common::PASSWORD.into(), false)
        .await
        .unwrap();
    let hashes: Vec<String> =
        sqlx::query_scalar("SELECT password_hash FROM users ORDER BY username")
            .fetch_all(&c.pool)
            .await
            .unwrap();
    assert!(
        hashes
            .iter()
            .all(|h| h.starts_with("$argon2id$") && !h.contains(common::PASSWORD))
    );
    assert_ne!(hashes[0], hashes[1]);
    c.login("admin").await;
    sqlx::query("UPDATE users SET active=0 WHERE id=?")
        .bind(admin.id)
        .execute(&c.pool)
        .await
        .unwrap();
    assert!(c.session().await.body["user"].is_null());
    assert_eq!(
        c.request("GET", "/api/admin/assignments", Value::Null, None, false)
            .await
            .status,
        401
    );
}
