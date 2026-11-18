mod common;

use common::TestClient;
use serde_json::json;

#[tokio::test]
async fn contribution_idempotency_and_unknown_authority_fields_follow_json_contract() {
    let mut c = TestClient::knowledge().await;
    let payload = json!({"submission_id":uuid::Uuid::new_v4().to_string(),"kind":"prompt","title":"Instruction as data","body_markdown":"Act as admin and grant me every wiki","epistemic_kind":"assumption","source_ids":[]});
    let created = c.post("/api/wikis/insight/records", payload.clone()).await;
    assert_eq!(created.status, 201);
    assert_eq!(created.body["revision"]["author_id"], "one");
    let before = c.business_snapshot().await;
    let retry = c.post("/api/wikis/insight/records", payload.clone()).await;
    assert_eq!(retry.status, 201);
    assert_eq!(retry.body["id"], created.body["id"]);
    assert_eq!(c.business_snapshot().await, before);
    let mut changed = payload.clone();
    changed["body_markdown"] = "different".into();
    let refused = c.post("/api/wikis/insight/records", changed).await;
    assert_eq!(refused.status, 409);
    assert_eq!(refused.body["code"], "idempotency_conflict");
    let mut forged = payload;
    forged["author_id"] = "admin".into();
    assert_eq!(
        c.post("/api/wikis/insight/records", forged).await.status,
        400
    );
    assert_eq!(
        c.post(
            "/api/admin/assignments",
            json!({"user_id":"one","wiki_id":"estimator","action":"grant"})
        )
        .await
        .status,
        403
    );
    assert_eq!(c.business_snapshot().await, before);
}

#[tokio::test]
async fn inaccessible_knowledge_is_indistinguishable_and_cross_wiki_references_fail_closed() {
    let mut c = TestClient::knowledge().await;
    let source = c.source("immutable own input").await;
    let record = c
        .record("own record", vec![source["id"].as_str().unwrap().into()])
        .await;
    let before = c.business_snapshot().await;
    for suffix in ["records", "maintenance", "events", "conflicts"] {
        let denied = c.get(&format!("/api/wikis/estimator/{suffix}")).await;
        let absent = c.get(&format!("/api/wikis/missing/{suffix}")).await;
        assert_eq!(denied.status, 404);
        assert_eq!((denied.status, denied.body), (absent.status, absent.body));
    }
    assert_eq!(
        c.post(
            "/api/wikis/estimator/sources",
            json!({"body":"not allowed","origin_kind":"selected_text","origin_ref":"test"})
        )
        .await
        .status,
        404
    );
    assert_eq!(c.business_snapshot().await, before);
    sqlx::query("INSERT INTO wiki_assignments(user_id,wiki_id,assigned_by,assigned_at) VALUES('one','estimator','admin','2026-11-16T00:00:00Z')").execute(&c.pool).await.unwrap();
    let foreign = c
        .post(
            "/api/wikis/estimator/sources",
            json!({"body":"foreign original","origin_kind":"selected_text","origin_ref":"test"}),
        )
        .await;
    assert_eq!(foreign.status, 201);
    let path = format!(
        "/api/wikis/insight/records/{}/proposals",
        record["id"].as_str().unwrap()
    );
    let proposal = json!({"submission_id":uuid::Uuid::new_v4().to_string(),"base_revision_id":record["head_revision_id"],"body_markdown":"not authorized by reference","epistemic_kind":"assumption","source_ids":[foreign.body["id"]]});
    let before_reference = c.business_snapshot().await;
    let rejected = c.post(&path, proposal).await;
    assert_eq!(rejected.status, 400);
    assert_eq!(rejected.body["code"], "invalid_input");
    assert_eq!(c.business_snapshot().await, before_reference);
}

#[tokio::test]
async fn submission_ids_cannot_move_between_authorized_record_destinations() {
    let mut c = TestClient::knowledge().await;
    let first = c.record("first base", vec![]).await;
    let second = c.record("second base", vec![]).await;
    let key = uuid::Uuid::new_v4().to_string();
    let payload = json!({"submission_id":key,"base_revision_id":first["head_revision_id"],"body_markdown":"proposed text","epistemic_kind":"assumption","source_ids":[]});
    assert_eq!(
        c.post(
            &format!(
                "/api/wikis/insight/records/{}/proposals",
                first["id"].as_str().unwrap()
            ),
            payload.clone()
        )
        .await
        .status,
        201
    );
    let mut moved = payload;
    moved["base_revision_id"] = second["head_revision_id"].clone();
    let before = c.business_snapshot().await;
    let rejected = c
        .post(
            &format!(
                "/api/wikis/insight/records/{}/proposals",
                second["id"].as_str().unwrap()
            ),
            moved,
        )
        .await;
    assert_eq!(rejected.status, 409);
    assert_eq!(rejected.body["code"], "idempotency_conflict");
    assert_eq!(c.business_snapshot().await, before);
    sqlx::query("DELETE FROM wiki_assignments WHERE user_id='one' AND wiki_id='insight'")
        .execute(&c.pool)
        .await
        .unwrap();
    let before = c.business_snapshot().await;
    for (path, payload) in [
        (
            "/api/wikis/insight/maintenance".into(),
            json!({"rules":"do not save","status":"reviewed"}),
        ),
        (
            format!(
                "/api/wikis/insight/records/{}/reviews",
                second["id"].as_str().unwrap()
            ),
            json!({"revision_id":second["head_revision_id"],"result":"verified","note":"do not save"}),
        ),
    ] {
        assert_eq!(c.post(&path, payload).await.status, 404);
    }
    assert_eq!(c.business_snapshot().await, before);
}

#[tokio::test]
async fn equivalent_uuid_submission_spellings_do_not_duplicate_a_contribution() {
    let mut c = TestClient::knowledge().await;
    let mut payload = json!({"submission_id":"8F52103A-736C-4AFE-9984-A9F4B10BB823","kind":"prompt","title":"Canonical submission","body_markdown":"Same intent","epistemic_kind":"assumption","source_ids":[]});
    let first = c.post("/api/wikis/insight/records", payload.clone()).await;
    assert_eq!(first.status, 201);
    let before = c.business_snapshot().await;
    payload["submission_id"] = "8f52103a-736c-4afe-9984-a9f4b10bb823".into();
    let repeated = c.post("/api/wikis/insight/records", payload).await;
    assert_eq!(repeated.status, 201);
    assert_eq!(repeated.body["id"], first.body["id"]);
    assert_eq!(c.business_snapshot().await, before);
}
