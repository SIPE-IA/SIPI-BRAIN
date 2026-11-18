mod common;
use common::TestClient;
use serde_json::json;

#[tokio::test]
async fn seed_collision_preserves_real_username_password_role_and_assignments() {
    let mut c = TestClient::knowledge().await;
    let user = sipibrain::auth::create_user(
        &c.pool,
        "__synthetic_fixture__",
        common::PASSWORD.into(),
        false,
    )
    .await
    .unwrap();
    let row_before: (String, i64, String) =
        sqlx::query_as("SELECT password_hash,active,role FROM users WHERE id=?")
            .bind(&user.id)
            .fetch_one(&c.pool)
            .await
            .unwrap();
    let before = c.business_snapshot().await;
    assert!(sipibrain::pilot::seed(&c.pool).await.is_err());
    assert_eq!(c.business_snapshot().await, before);
    let row_after: (String, i64, String) =
        sqlx::query_as("SELECT password_hash,active,role FROM users WHERE id=?")
            .bind(&user.id)
            .fetch_one(&c.pool)
            .await
            .unwrap();
    assert_eq!(row_before, row_after);
    let authenticated = c.login("__synthetic_fixture__").await;
    assert_eq!(authenticated.body["user"]["id"], user.id);
    assert_eq!(authenticated.body["user"]["role"], "programmer");
    assert_eq!(
        c.get("/api/wikis").await.body["items"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

#[tokio::test]
async fn seed_author_cannot_authenticate_with_fixture_password_or_grant_existing_users_access() {
    let mut c = TestClient::knowledge().await;
    let count = c.assignment_count().await;
    sipibrain::pilot::seed(&c.pool).await.unwrap();
    assert_eq!(c.assignment_count().await, count);
    let active: i64 =
        sqlx::query_scalar("SELECT active FROM users WHERE id='pilot-fixture-author'")
            .fetch_one(&c.pool)
            .await
            .unwrap();
    assert_eq!(active, 0);
    let before = c.business_snapshot().await;
    assert_eq!(
        c.post(
            "/api/login",
            json!({"username":"__synthetic_fixture__","password":common::PASSWORD})
        )
        .await
        .status,
        401
    );
    assert_eq!(c.get("/api/wikis/pilot-insight/records").await.status, 404);
    assert_eq!(c.business_snapshot().await, before);
}
