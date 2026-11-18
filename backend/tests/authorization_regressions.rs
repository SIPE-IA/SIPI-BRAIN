mod common;
use common::TestClient;
use serde_json::{Value, json};
use std::time::Duration;

fn proposal(base: &Value, body: &str) -> Value {
    json!({"submission_id":uuid::Uuid::new_v4().to_string(),"base_revision_id":base["head_revision_id"],"body_markdown":body,"epistemic_kind":"assumption","source_ids":[]})
}

#[tokio::test]
async fn revoked_assignment_is_rechecked_after_waiting_for_writer_lock() {
    let mut c = TestClient::knowledge().await;
    let base = c.record("antecedent", vec![]).await;
    let pool = sipibrain::db::open(&c.root.join("test.db")).await.unwrap();
    let mut lock = pool.begin_with("BEGIN IMMEDIATE").await.unwrap();
    sqlx::query("DELETE FROM wiki_assignments WHERE user_id='one' AND wiki_id='insight'")
        .execute(&mut *lock)
        .await
        .unwrap();
    let path = format!(
        "/api/wikis/insight/records/{}/proposals",
        base["id"].as_str().unwrap()
    );
    let release = async {
        tokio::time::sleep(Duration::from_millis(70)).await;
        lock.commit().await.unwrap();
    };
    let request = async { c.post(&path, proposal(&base, "must not commit")).await };
    let (_, reply) = tokio::join!(release, request);
    assert_eq!(reply.status, 404);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM edit_proposals")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    let state: (String, i64, String) =
        sqlx::query_as("SELECT head_revision_id,generation,state FROM records WHERE id=?")
            .bind(base["id"].as_str().unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        state,
        (
            base["head_revision_id"].as_str().unwrap().into(),
            1,
            "active".into()
        )
    );
}

#[tokio::test]
async fn administrator_without_assignment_cannot_read_or_mutate_local_knowledge() {
    let mut c = TestClient::knowledge().await;
    let source = c.source("original private text").await;
    let base = c
        .record("ordinary text", vec![source["id"].as_str().unwrap().into()])
        .await;
    c.login("admin").await;
    let before = c.business_snapshot().await;
    let root = "/api/wikis/insight";
    for suffix in [
        format!("sources/{}", source["id"].as_str().unwrap()),
        format!("records/{}", base["id"].as_str().unwrap()),
        "records".into(),
        "conflicts".into(),
        "events".into(),
        "maintenance".into(),
    ] {
        let reply = c.get(&format!("{root}/{suffix}")).await;
        assert_eq!(reply.status, 404);
        assert_eq!(reply.body["code"], "not_found");
        assert!(!reply.body.to_string().contains("original private text"));
    }
    for (path, payload) in [
        (
            format!("{root}/sources"),
            json!({"body":"not allowed","origin_kind":"text","origin_ref":"fixture"}),
        ),
        (
            format!("{root}/records/{}/proposals", base["id"].as_str().unwrap()),
            proposal(&base, "not allowed"),
        ),
        (
            format!("{root}/records/{}/reviews", base["id"].as_str().unwrap()),
            json!({"revision_id":base["head_revision_id"],"result":"verified","note":"not allowed"}),
        ),
        (
            format!("{root}/maintenance"),
            json!({"rules":"not allowed","status":"reviewed"}),
        ),
    ] {
        assert_eq!(c.post(&path, payload).await.status, 404);
    }
    assert_eq!(c.business_snapshot().await, before);
}

#[tokio::test]
async fn pending_proposals_and_links_do_not_cross_records_or_reveal_other_wiki() {
    let mut c = TestClient::knowledge().await;
    let a = c.record("first", vec![]).await;
    let b = c.record("second", vec![]).await;
    let before = c.business_snapshot().await;
    let path = format!(
        "/api/wikis/insight/records/{}/proposals",
        a["id"].as_str().unwrap()
    );
    assert_eq!(
        c.post(&path, proposal(&b, "wrong record")).await.status,
        400
    );
    let link = format!(
        "/api/wikis/insight/records/{}/links",
        a["id"].as_str().unwrap()
    );
    assert_eq!(
        c.post(
            &link,
            json!({"target_record_id":"unavailable","relation":"related"})
        )
        .await
        .status,
        400
    );
    let forged = format!(
        "/api/wikis/estimator/records/{}/reviews",
        a["id"].as_str().unwrap()
    );
    assert_eq!(
        c.post(
            &forged,
            json!({"revision_id":a["head_revision_id"],"result":"verified","note":"not allowed"})
        )
        .await
        .status,
        404
    );
    assert_eq!(c.business_snapshot().await, before);
}
