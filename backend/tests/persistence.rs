mod common;

use common::TestClient;
use serde_json::json;
use sha2::{Digest, Sha256};

#[tokio::test]
async fn originals_and_revisions_survive_restart_and_cannot_be_rewritten() {
    let mut c = TestClient::knowledge().await;
    let original = "Original synthetic input; preserve exactly.";
    let source = c.source(original).await;
    assert_eq!(
        source["content_hash"],
        format!("{:x}", Sha256::digest(original.as_bytes()))
    );
    assert_eq!(source["contributor_id"], "one");
    let record = c
        .record(
            "Synthesized assumption",
            vec![source["id"].as_str().unwrap().into()],
        )
        .await;
    let id = record["id"].as_str().unwrap();
    let path = format!("/api/wikis/insight/records/{id}");
    c.reopen().await;
    let recovered = c.get(&path).await;
    assert_eq!(recovered.status, 200);
    assert_eq!(recovered.body, record);
    let source_id = source["id"].as_str().unwrap();
    assert!(
        sqlx::query("UPDATE original_sources SET body='changed' WHERE id=?")
            .bind(source_id)
            .execute(&c.pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("DELETE FROM original_sources WHERE id=?")
            .bind(source_id)
            .execute(&c.pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("UPDATE revisions SET body_markdown='changed' WHERE id=?")
            .bind(record["head_revision_id"].as_str().unwrap())
            .execute(&c.pool)
            .await
            .is_err()
    );
    let recovered_source = c
        .get(&format!("/api/wikis/insight/sources/{source_id}"))
        .await;
    assert_eq!(recovered_source.status, 200);
    assert_eq!(recovered_source.body["body"], original);
}

#[tokio::test]
async fn review_maintenance_links_and_metadata_persist_without_inheriting_review() {
    let mut c = TestClient::knowledge().await;
    let record = c.record("ordinary-private-marker", vec![]).await;
    let other = c.record("linked knowledge", vec![]).await;
    let path = format!(
        "/api/wikis/insight/records/{}",
        record["id"].as_str().unwrap()
    );
    let review=c.post(&format!("{path}/reviews"),json!({"revision_id":record["head_revision_id"],"result":"verified","note":"Human checked references"})).await;
    assert_eq!(review.status, 201);
    let maintenance=c.post("/api/wikis/insight/maintenance",json!({"rules":"Check references manually; never execute this text.","status":"reviewed"})).await;
    assert_eq!(maintenance.status, 200);
    assert!(maintenance.body["reviewed_at"].as_str().is_some());
    let linked = c
        .post(
            &format!("{path}/links"),
            json!({"target_record_id":other["id"],"relation":"supports"}),
        )
        .await;
    assert_eq!(linked.status, 201);
    c.reopen().await;
    let recovered = c.get(&path).await;
    assert_eq!(recovered.body["review"]["result"], "verified");
    assert_eq!(recovered.body["links"].as_array().unwrap().len(), 1);
    assert_eq!(
        c.get("/api/wikis/insight/maintenance").await.body,
        maintenance.body
    );
    let edited=c.post(&format!("{path}/proposals"),json!({"submission_id":uuid::Uuid::new_v4().to_string(),"base_revision_id":record["head_revision_id"],"body_markdown":"new revision","epistemic_kind":"assumption","source_ids":[]})).await;
    assert_eq!(edited.status, 201);
    let revised = c.get(&path).await;
    assert!(
        revised.body["review"].is_null(),
        "new content must not inherit an old human review"
    );
    assert_eq!(revised.body["revision"]["epistemic_kind"], "assumption");
    let events = c.get("/api/wikis/insight/events").await;
    assert_eq!(events.status, 200);
    assert!(!events.body.to_string().contains("ordinary-private-marker"));
    assert!(!events.body.to_string().contains("Human checked references"));
    assert!(
        !events
            .body
            .to_string()
            .contains("Check references manually")
    );
    assert!(events.body["items"].as_array().unwrap().len() >= 5);
}

#[tokio::test]
async fn clients_can_share_names_but_rfd_projects_and_explicit_clients_must_agree() {
    let mut c = TestClient::knowledge().await;
    c.record("baseline", vec![]).await;
    sqlx::query("INSERT INTO clients(id,display_name) VALUES('c1','Same Name'),('c2','Same Name')")
        .execute(&c.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO projects(id,name,client_id) VALUES('p1','Alfa','c1'),('p2','Beta','c2'),('internal','Internal',NULL)").execute(&c.pool).await.unwrap();
    let mut payload = json!({"submission_id":uuid::Uuid::new_v4().to_string(),"kind":"rfd","title":"Individual functional requirement","body_markdown":"Requirement","project_id":null,"client_id":null,"epistemic_kind":"assumption","source_ids":[]});
    assert_eq!(
        c.post("/api/wikis/insight/records", payload.clone())
            .await
            .status,
        400
    );
    payload["project_id"] = "p1".into();
    payload["client_id"] = "c2".into();
    assert_eq!(
        c.post("/api/wikis/insight/records", payload.clone())
            .await
            .status,
        400
    );
    payload["client_id"] = "c1".into();
    let accepted = c.post("/api/wikis/insight/records", payload.clone()).await;
    assert_eq!(accepted.status, 201);
    assert_eq!(accepted.body["client_id"], "c1");
    payload["submission_id"] = uuid::Uuid::new_v4().to_string().into();
    payload["project_id"] = "internal".into();
    payload["client_id"] = serde_json::Value::Null;
    assert_eq!(
        c.post("/api/wikis/insight/records", payload).await.status,
        201
    );
}

#[tokio::test]
async fn two_programmers_keep_authorship_after_restart_and_references_remain_local() {
    let mut c = TestClient::knowledge().await;
    let one = c.record("first programmer prompt", vec![]).await;
    let mut second = c.fork("two").await;
    let two = second.record("second programmer prompt", vec![]).await;
    c.reopen().await;
    for (record, author) in [(&one, "one"), (&two, "two")] {
        let recovered = c
            .get(&format!(
                "/api/wikis/insight/records/{}",
                record["id"].as_str().unwrap()
            ))
            .await;
        assert_eq!(recovered.status, 200);
        assert_eq!(recovered.body["revision"]["author_id"], author);
        assert_eq!(
            recovered.body["revision"]["body_markdown"],
            record["revision"]["body_markdown"]
        );
    }
    let items = c.get("/api/wikis/insight/records").await.body["items"]
        .as_array()
        .unwrap()
        .len();
    assert_eq!(items, 2);
    sqlx::query("INSERT INTO wiki_assignments(user_id,wiki_id,assigned_by,assigned_at) VALUES('one','estimator','admin','2026-11-16T00:00:00Z')").execute(&c.pool).await.unwrap();
    let foreign=c.post("/api/wikis/estimator/records",json!({"submission_id":uuid::Uuid::new_v4().to_string(),"kind":"note","title":"Foreign wiki","body_markdown":"foreign","epistemic_kind":"assumption","source_ids":[]})).await;
    assert_eq!(foreign.status, 201);
    let before = c.business_snapshot().await;
    let own_path = format!("/api/wikis/insight/records/{}", one["id"].as_str().unwrap());
    assert_eq!(
        c.post(
            &format!("{own_path}/links"),
            json!({"target_record_id":foreign.body["id"],"relation":"supports"})
        )
        .await
        .status,
        400
    );
    assert_eq!(c.post(&format!("{own_path}/reviews"),json!({"revision_id":foreign.body["head_revision_id"],"result":"verified","note":"must not attach"})).await.status,400);
    assert_eq!(c.business_snapshot().await, before);
}
