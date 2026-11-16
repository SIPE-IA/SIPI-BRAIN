mod common;

use common::TestClient;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::Barrier;

fn proposal(base: &Value, body: &str) -> Value {
    json!({"submission_id":uuid::Uuid::new_v4().to_string(),"base_revision_id":base["head_revision_id"],"body_markdown":body,"epistemic_kind":"decision","source_ids":[]})
}

#[tokio::test]
async fn concurrent_branches_are_preserved_idempotent_and_only_current_admin_can_resolve() {
    let mut c = TestClient::knowledge().await;
    let base = c.record("base antecedent", vec![]).await;
    let mut other = c.fork("two").await;
    let path = format!(
        "/api/wikis/insight/records/{}",
        base["id"].as_str().unwrap()
    );
    let proposal_path = format!("{path}/proposals");
    let a = proposal(&base, "branch one");
    let b = proposal(&base, "branch two");
    let barrier = Arc::new(Barrier::new(2));
    let first = async {
        barrier.wait().await;
        c.post(&proposal_path, a.clone()).await
    };
    let second = async {
        barrier.wait().await;
        other.post(&proposal_path, b.clone()).await
    };
    let (a_result, b_result) = tokio::join!(first, second);
    let mut statuses = vec![a_result.status, b_result.status];
    statuses.sort();
    assert_eq!(statuses, vec![201, 202]);
    assert!(a_result.body["state"] == "pending" || b_result.body["state"] == "pending");
    c.reopen().await;
    let pending = c.get("/api/wikis/insight/conflicts").await;
    assert_eq!(pending.status, 200);
    let conflict = pending.body["items"][0].clone();
    assert_eq!(conflict["proposals"].as_array().unwrap().len(), 2);
    let text = conflict.to_string();
    assert!(text.contains("branch one") && text.contains("branch two"));
    let detail = c.get(&path).await;
    assert_eq!(detail.body["state"], "conflicted");
    assert_eq!(detail.body["revision"]["body_markdown"], "base antecedent");
    let before = c.business_snapshot().await;
    let repeated = other.post(&proposal_path, b.clone()).await;
    assert!(matches!(repeated.status, 201 | 202));
    assert_eq!(repeated.body["proposal_id"], b_result.body["proposal_id"]);
    assert_eq!(c.business_snapshot().await, before);
    let mut reused = b.clone();
    reused["body_markdown"] = "changed payload".into();
    let refused = other.post(&proposal_path, reused).await;
    assert_eq!(refused.status, 409);
    assert_eq!(refused.body["code"], "idempotency_conflict");
    assert_eq!(c.business_snapshot().await, before);
    let resolve_path = format!(
        "/api/wikis/insight/conflicts/{}/resolve",
        conflict["id"].as_str().unwrap()
    );
    let mut resolution = json!({"expected_generation":conflict["generation"],"body_markdown":"manual result","epistemic_kind":"decision","reason":"Both candidates reviewed","source_ids":[]});
    assert_eq!(c.post(&resolve_path, resolution.clone()).await.status, 403);
    assert_eq!(c.business_snapshot().await, before);
    c.login("admin").await;
    assert_eq!(c.post(&resolve_path, resolution.clone()).await.status, 404);
    sqlx::query("INSERT INTO wiki_assignments(user_id,wiki_id,assigned_by,assigned_at) VALUES('admin','insight','admin','2026-11-16T00:00:00Z')").execute(&c.pool).await.unwrap();
    assert_eq!(
        other
            .post(&proposal_path, proposal(&base, "third candidate"))
            .await
            .status,
        202
    );
    let changed_before = c.business_snapshot().await;
    let obsolete = c.post(&resolve_path, resolution.clone()).await;
    assert_eq!(obsolete.status, 409);
    assert_eq!(obsolete.body["code"], "conflict_changed");
    assert_eq!(c.business_snapshot().await, changed_before);
    let current = c.get("/api/wikis/insight/conflicts").await.body["items"][0].clone();
    assert_eq!(current["proposals"].as_array().unwrap().len(), 3);
    resolution["expected_generation"] = current["generation"].clone();
    let resolved = c.post(&resolve_path, resolution).await;
    assert_eq!(resolved.status, 201);
    assert_eq!(resolved.body["state"], "active");
    assert_eq!(resolved.body["revision"]["body_markdown"], "manual result");
    assert_eq!(
        c.get("/api/wikis/insight/conflicts").await.body["items"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        other
            .post(&proposal_path, proposal(&base, "late branch"))
            .await
            .status,
        202
    );
    let previous: i64 = sqlx::query_scalar("SELECT count(*) FROM conflict_resolutions")
        .fetch_one(&c.pool)
        .await
        .unwrap();
    assert_eq!(
        previous, 1,
        "late proposals must preserve the prior administrator decision"
    );
    assert_eq!(
        c.get("/api/wikis/insight/conflicts").await.body["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn busy_writer_times_out_without_success_or_partial_proposal() {
    let mut c = TestClient::knowledge().await;
    let base = c.record("base", vec![]).await;
    let independent = sipibrain::db::open(&c.root.join("test.db")).await.unwrap();
    let before = c.business_snapshot().await;
    let lock = independent.begin_with("BEGIN IMMEDIATE").await.unwrap();
    let started = std::time::Instant::now();
    let result = c
        .post(
            &format!(
                "/api/wikis/insight/records/{}/proposals",
                base["id"].as_str().unwrap()
            ),
            proposal(&base, "not confirmed"),
        )
        .await;
    assert_eq!(result.status, 503);
    assert_eq!(result.body["code"], "storage_busy");
    assert!(started.elapsed() < std::time::Duration::from_secs(3));
    lock.rollback().await.unwrap();
    assert_eq!(c.business_snapshot().await, before);
}
