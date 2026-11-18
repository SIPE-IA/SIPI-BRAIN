mod common;

use common::{ORIGIN, TestClient};
use serde_json::json;

const AGENTS: &str = "/api/admin/agents";

#[tokio::test]
async fn register_requires_admin_and_persists_an_eligible_agent_without_access() {
    let mut c = TestClient::new().await;
    c.seed_accounts().await;
    for query in [
        "DELETE FROM wiki_assignments",
        "DELETE FROM wikis",
        "DELETE FROM agents",
    ] {
        sqlx::query(query).execute(&c.pool).await.unwrap();
    }
    c.session().await;
    c.login("one").await;
    let input = json!({"id":"  first  ","name":"  First agent  "});
    assert_eq!(c.get(AGENTS).await.status, 403);
    assert_eq!(c.post(AGENTS, input.clone()).await.status, 403);
    let mut guest = c.fork("one").await;
    guest.cookie.clear();
    assert_eq!(guest.get(AGENTS).await.status, 401);
    c.login("admin").await;
    assert_eq!(c.get(AGENTS).await.body, json!({"items":[]}));
    assert_eq!(
        c.request("POST", AGENTS, input.clone(), Some(ORIGIN), false)
            .await
            .status,
        403
    );
    assert_eq!(
        c.request(
            "POST",
            AGENTS,
            input.clone(),
            Some("http://invalid.test"),
            true
        )
        .await
        .status,
        403
    );
    for invalid in [
        json!({"id":" ","name":"Name"}),
        json!({"id":"ok","name":" "}),
        json!({"id":" . ","name":"Dot"}),
        json!({"id":"..","name":"Parent"}),
    ] {
        assert_eq!(c.post(AGENTS, invalid).await.status, 400);
    }
    assert_eq!(c.get(AGENTS).await.body, json!({"items":[]}));
    let before = c.get("/api/admin/assignments").await.body["items"].clone();
    let saved = c.post(AGENTS, input).await;
    assert_eq!(saved.status, 201);
    assert_eq!(
        saved.body,
        json!({"id":"first","name":"First agent","active":true,"connection":null})
    );
    let duplicate = c
        .post(AGENTS, json!({"id":"first","name":"Replacement"}))
        .await;
    assert_eq!(duplicate.status, 409);
    assert_eq!(duplicate.body["code"], "agent_already_exists");
    c.reopen().await;
    let listed = c.get(AGENTS).await;
    assert_eq!(listed.status, 200);
    assert!(
        listed.body["items"]
            .as_array()
            .unwrap()
            .contains(&saved.body)
    );
    assert!(
        c.get("/api/admin/wikis").await.body["eligible_agents"]
            .as_array()
            .unwrap()
            .contains(&json!({"id":"first","name":"First agent"}))
    );
    assert_eq!(c.get("/api/admin/assignments").await.body["items"], before);
    let saved_wiki = c
        .post(
            "/api/admin/wikis",
            json!({"agent_id":"first","name":"First wiki"}),
        )
        .await;
    assert_eq!(saved_wiki.status, 201);
    let wiki = saved_wiki.body["id"].as_str().unwrap();
    c.reopen().await;
    assert_eq!(
        c.get("/api/admin/wikis").await.body["items"],
        json!([saved_wiki.body])
    );
    assert_eq!(
        c.get(&format!("/api/wikis/{wiki}/records")).await.status,
        404
    );
    assert_eq!(c.get("/api/admin/assignments").await.body["items"], before);
}

#[tokio::test]
async fn competing_registrations_keep_one_row_and_one_conflict() {
    let mut a = TestClient::knowledge().await;
    a.login("admin").await;
    let mut b = a.fork("admin").await;
    let (a_reply, b_reply) = tokio::join!(
        a.post(AGENTS, json!({"id":"race","name":"First"})),
        b.post(AGENTS, json!({"id":"race","name":"Second"})),
    );
    let (ok, rejected) = if a_reply.status == 201 {
        (a_reply, b_reply)
    } else {
        (b_reply, a_reply)
    };
    assert_eq!(ok.status, 201);
    assert_eq!(rejected.status, 409);
    assert_eq!(rejected.body["code"], "agent_already_exists");
    assert_eq!(
        a.get(AGENTS).await.body["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| item["id"] == "race")
            .collect::<Vec<_>>(),
        vec![&ok.body]
    );
}
