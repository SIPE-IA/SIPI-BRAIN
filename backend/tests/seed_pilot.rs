mod common;

use common::TestClient;
use serde_json::json;
use std::process::Command;

fn seed(root: &std::path::Path) {
    let result = Command::new(env!("CARGO_BIN_EXE_sipibrain"))
        .args(["seed-pilot", "--synthetic"])
        .env("SIPIBRAIN_DATA_DIR", root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "seed-pilot --synthetic must succeed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("pilot-project-alfa"));
}

#[tokio::test]
async fn explicit_synthetic_seed_repeats_without_overwriting_accounts_or_edits() {
    let mut c = TestClient::knowledge().await;
    sqlx::query("VACUUM INTO ?")
        .bind(c.root.join("sipibrain.db").to_string_lossy().as_ref())
        .execute(&c.pool)
        .await
        .unwrap();
    c.pool.close().await;
    c.pool = sipibrain::db::open(&c.root.join("sipibrain.db"))
        .await
        .unwrap();
    c.router = sipibrain::api::application(c.pool.clone(), common::ORIGIN, &c.root)
        .await
        .unwrap();
    let users_before: Vec<(String, String, String, i64, String)> =
        sqlx::query_as("SELECT id,username,password_hash,active,role FROM users ORDER BY id")
            .fetch_all(&c.pool)
            .await
            .unwrap();
    let assignments_before = c.assignment_count().await;
    seed(&c.root);
    let records: i64 = sqlx::query_scalar("SELECT count(*) FROM records")
        .fetch_one(&c.pool)
        .await
        .unwrap();
    assert_eq!(records, 5);
    let alfa:i64=sqlx::query_scalar("SELECT count(*) FROM records WHERE wiki_id='pilot-insight' AND project_id='pilot-project-alfa' AND kind='rfd'").fetch_one(&c.pool).await.unwrap();
    let beta:i64=sqlx::query_scalar("SELECT count(*) FROM records WHERE wiki_id='pilot-insight' AND project_id='pilot-project-beta' AND kind='rfd'").fetch_one(&c.pool).await.unwrap();
    let own_note:i64=sqlx::query_scalar("SELECT count(*) FROM records WHERE wiki_id='pilot-estimador' AND project_id='pilot-project-alfa' AND kind='note'").fetch_one(&c.pool).await.unwrap();
    assert_eq!((alfa, beta, own_note), (3, 1, 1));
    assert_eq!(
        c.assignment_count().await,
        assignments_before,
        "seed must not grant application users access"
    );
    let users_after:Vec<(String,String,String,i64,String)>=sqlx::query_as("SELECT id,username,password_hash,active,role FROM users WHERE id!='pilot-fixture-author' ORDER BY id").fetch_all(&c.pool).await.unwrap();
    assert_eq!(users_after, users_before);
    let fixture: (i64, String) =
        sqlx::query_as("SELECT active,password_hash FROM users WHERE id='pilot-fixture-author'")
            .fetch_one(&c.pool)
            .await
            .unwrap();
    assert_eq!(fixture.0, 0, "synthetic authors cannot log in");
    assert!(fixture.1.starts_with("$argon2id$"));
    let before = c.business_snapshot().await;
    seed(&c.root);
    assert_eq!(c.business_snapshot().await, before);
    c.session().await;
    c.login("admin").await;
    assert_eq!(
        c.post(
            "/api/admin/assignments",
            json!({"user_id":"one","wiki_id":"pilot-insight","action":"grant"})
        )
        .await
        .status,
        200
    );
    c.login("one").await;
    let record = c.get("/api/wikis/pilot-insight/records").await.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["title"] == "RFD-001")
        .unwrap()
        .clone();
    let path = format!(
        "/api/wikis/pilot-insight/records/{}/proposals",
        record["id"].as_str().unwrap()
    );
    assert_eq!(c.post(&path,json!({"submission_id":uuid::Uuid::new_v4().to_string(),"base_revision_id":record["head_revision_id"],"body_markdown":"A programmer-edited synthetic requirement","epistemic_kind":"decision","source_ids":[]})).await.status,201);
    let before = c.business_snapshot().await;
    seed(&c.root);
    assert_eq!(
        c.business_snapshot().await,
        before,
        "repeat seed must not restore the fixture over an edited record"
    );
    c.login("admin").await;
    for username in ["one", "two", "admin"] {
        for wiki in ["pilot-insight", "pilot-estimador"] {
            assert_eq!(
                c.post(
                    "/api/admin/assignments",
                    json!({"user_id":username,"wiki_id":wiki,"action":"grant"})
                )
                .await
                .status,
                200
            );
        }
    }
    c.pool.close().await;
    eprintln!(
        "browser-fixture-db: {}",
        c.root.join("sipibrain.db").display()
    );
}

#[test]
fn seed_requires_explicit_synthetic_flag_before_creating_data() {
    let root = tempfile::Builder::new()
        .prefix("sipibrain-seed-refusal-")
        .tempdir()
        .unwrap()
        .keep();
    eprintln!("scratch: {}", root.display());
    let result = Command::new(env!("CARGO_BIN_EXE_sipibrain"))
        .arg("seed-pilot")
        .env("SIPIBRAIN_DATA_DIR", root.join("data"))
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!root.join("data").exists());
}
