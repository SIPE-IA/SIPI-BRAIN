use crate::common::TestClient;
use serde_json::{Value, json};

const PATH: &str = "/api/admin/wiki-activity";
const INSIGHT: &str = "/api/wikis/insight";
const ESTIMATOR: &str = "/api/wikis/estimator";

fn wiki(body: &Value, id: &str) -> Value {
    body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap()
        .clone()
}

#[tokio::test]
async fn dates_are_global_for_admin_but_author_and_content_follow_current_assignment() {
    let mut writer = TestClient::knowledge().await;
    assert_eq!(writer.get(PATH).await.status, 403);
    let mut admin = writer.fork("admin").await;
    let empty = admin.get(PATH).await;
    assert_eq!(empty.status, 200);
    assert_eq!(
        wiki(&empty.body, "insight"),
        json!({
            "id":"insight","name":"insight","agent":{"id":"insight","name":"insight"},
            "record_count":0,"source_count":0,"last_contribution_at":null,"recent_activity":[]
        })
    );
    for (user_id, wiki_id) in [("admin", "insight"), ("two", "estimator")] {
        assert_eq!(
            admin
                .post(
                    "/api/admin/assignments",
                    json!({"action":"grant","user_id":user_id,"wiki_id":wiki_id})
                )
                .await
                .status,
            200
        );
    }
    let mut second = writer.fork("two").await;
    let other = second.post(&format!("{ESTIMATOR}/records"), json!({
        "submission_id":uuid::Uuid::new_v4(),"kind":"note","title":"private estimator title",
        "body_markdown":"private estimator body","epistemic_kind":"assumption","source_ids":[]
    })).await;
    assert_eq!(other.status, 201);
    let baseline = wiki(&admin.get(PATH).await.body, "estimator");
    assert_eq!(baseline["record_count"], 1);
    assert_eq!(baseline["recent_activity"].as_array().unwrap().len(), 1);
    assert!(baseline["recent_activity"][0]["occurred_at"].is_string());
    assert_eq!(baseline["recent_activity"][0].as_object().unwrap().len(), 1);

    let first = writer.source("synthetic-original-A").await;
    let second = writer.source("synthetic-original-B").await;
    let source_ids = vec![
        first["id"].as_str().unwrap().to_owned(),
        second["id"].as_str().unwrap().to_owned(),
    ];
    let saved = writer
        .record("synthetic-private-body", source_ids.clone())
        .await;
    let summary = &saved;
    assert!(summary["id"].is_string(), "{saved}");
    let record_path = format!("{INSIGHT}/records/{}", summary["id"].as_str().unwrap());
    let updated = admin.get(PATH).await;
    let insight = wiki(&updated.body, "insight");
    assert_eq!(wiki(&updated.body, "estimator"), baseline);
    assert_eq!(insight["record_count"], 1);
    assert_eq!(insight["source_count"], 2);
    assert_eq!(insight["recent_activity"].as_array().unwrap().len(), 3);
    assert_eq!(
        insight["last_contribution_at"],
        insight["recent_activity"][0]["occurred_at"]
    );
    for entry in insight["recent_activity"].as_array().unwrap() {
        assert_eq!(entry["author_name"], "one");
        assert_eq!(entry.as_object().unwrap().len(), 2);
    }
    assert!(!updated.body.to_string().contains("synthetic-private-body"));
    assert_eq!(admin.get(&record_path).await.status, 200);
    let unauthorized = format!("{ESTIMATOR}/records");
    assert_eq!(admin.get(&unauthorized).await.status, 404);

    let proposal = writer
        .post(
            &format!("{record_path}/proposals"),
            json!({
                "submission_id":uuid::Uuid::new_v4(),
                "base_revision_id":summary["head_revision_id"],
                "body_markdown":"synthetic revision",
                "epistemic_kind":"assumption",
                "source_ids":source_ids
            }),
        )
        .await;
    assert_eq!(proposal.status, 201, "{}", proposal.body);
    let revised = wiki(&admin.get(PATH).await.body, "insight");
    assert_eq!(revised["record_count"], 1);
    assert_eq!(revised["source_count"], 2);
    assert_eq!(revised["recent_activity"].as_array().unwrap().len(), 4);

    assert_eq!(
        admin
            .post(
                "/api/admin/assignments",
                json!({"action":"revoke","user_id":"admin","wiki_id":"insight"})
            )
            .await
            .status,
        200
    );
    let after = wiki(&admin.get(PATH).await.body, "insight");
    assert_eq!(
        after["last_contribution_at"],
        revised["last_contribution_at"]
    );
    for (a, b) in after["recent_activity"]
        .as_array()
        .unwrap()
        .iter()
        .zip(revised["recent_activity"].as_array().unwrap())
    {
        assert_eq!(*a, json!({"occurred_at":b["occurred_at"]}));
    }
    assert_eq!(admin.get(&record_path).await.status, 404);
}

#[tokio::test]
async fn activity_is_bounded_ordered_and_storage_failure_is_not_empty() {
    let writer = TestClient::knowledge().await;
    let mut admin = writer.fork("admin").await;
    sqlx::query("INSERT INTO wiki_events(id,wiki_id,actor_id,action,reference_id,created_at) VALUES
        ('base','insight','one','record_created','private-ref','2026-01-01T00:00:10Z'),
        ('fraction','insight','one','revision_saved','private-ref','2026-01-01T00:00:10.1Z'),
        ('non-contribution','insight','admin','maintenance_updated','private-ref','2026-01-01T00:00:59Z')")
        .execute(&admin.pool).await.unwrap();
    for i in 0..10 {
        sqlx::query("INSERT INTO wiki_events(id,wiki_id,actor_id,action,reference_id,created_at) VALUES (?,'insight','two','source_created','private-ref','2026-01-01T00:00:10Z')")
            .bind(format!("fixture-{i}")).execute(&admin.pool).await.unwrap();
    }
    assert_eq!(
        admin
            .post(
                "/api/admin/assignments",
                json!({"action":"grant","user_id":"admin","wiki_id":"insight"})
            )
            .await
            .status,
        200
    );
    let body = admin.get(PATH).await.body;
    let item = wiki(&body, "insight");
    assert_eq!(item["last_contribution_at"], "2026-01-01T00:00:10.1Z");
    let recent = item["recent_activity"].as_array().unwrap();
    assert_eq!(recent.len(), 10);
    assert_eq!(
        recent[0],
        json!({"occurred_at":"2026-01-01T00:00:10.1Z","author_name":"one"})
    );
    for event in &recent[1..] {
        assert_eq!(
            *event,
            json!({"occurred_at":"2026-01-01T00:00:10Z","author_name":"two"})
        );
    }
    assert!(!body.to_string().contains("private-ref"));
    sqlx::query("DROP TABLE wiki_events")
        .execute(&admin.pool)
        .await
        .unwrap();
    let failure = admin.get(PATH).await;
    assert_eq!(failure.status, 500);
    assert_eq!(failure.body["code"], "internal_error");
}
