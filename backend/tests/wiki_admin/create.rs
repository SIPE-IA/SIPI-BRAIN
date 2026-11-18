use crate::common::TestClient;
use serde_json::{Value, json};

const PATH: &str = "/api/admin/wikis";

async fn eligible(client: &TestClient, id: &str) {
    sqlx::query("INSERT INTO agents(id,name) VALUES(?,?)")
        .bind(id)
        .bind(id)
        .execute(&client.pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn admin_creates_once_and_reopened_wiki_uses_existing_assignments() {
    let mut client = TestClient::knowledge().await;
    assert_eq!(client.get(PATH).await.status, 403);
    client.login("admin").await;
    assert_eq!(client.get(PATH).await.body["eligible_agents"], json!([]));
    eligible(&client, "new-agent").await;
    assert_eq!(
        client.get(PATH).await.body["eligible_agents"],
        json!([{"id":"new-agent","name":"new-agent"}])
    );
    let input = json!({"name":"  New Wiki  ","agent_id":"new-agent"});
    let saved = client.post(PATH, input.clone()).await;
    assert_eq!(saved.status, 201);
    let id = saved.body["id"].as_str().unwrap();
    assert_eq!(
        saved.body,
        json!({"id":id,"name":"New Wiki","agent":{"id":"new-agent","name":"new-agent"}})
    );
    let duplicate = client
        .post(PATH, json!({"name":"Other","agent_id":"new-agent"}))
        .await;
    assert_eq!(duplicate.status, 409);
    assert_eq!(duplicate.body["code"], "wiki_already_exists");
    client.reopen().await;
    let list = client.get(PATH).await.body;
    assert_eq!(list["eligible_agents"], json!([]));
    assert_eq!(
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| **item == saved.body)
            .count(),
        1
    );
    let record_path = format!("/api/wikis/{id}/records");
    let mut programmer = client.fork("one").await;
    assert_eq!(programmer.get(&record_path).await.status, 404);
    assert_eq!(client.get(&record_path).await.status, 404);
    assert_eq!(
        client
            .post(
                "/api/admin/assignments",
                json!({"action":"grant","user_id":"one","wiki_id":id})
            )
            .await
            .status,
        200
    );
    assert_eq!(programmer.get(&record_path).await.status, 200);
    assert_eq!(client.get(&record_path).await.status, 404);
}

#[tokio::test]
async fn rejected_or_failed_creation_does_not_change_wikis_and_race_has_one_winner() {
    let mut client = TestClient::knowledge().await;
    eligible(&client, "new-agent").await;
    eligible(&client, "disabled-agent").await;
    sqlx::query("UPDATE agents SET active=0 WHERE id='disabled-agent'")
        .execute(&client.pool)
        .await
        .unwrap();
    let input = json!({"name":"New Wiki","agent_id":"new-agent"});
    let mut anonymous = TestClient::new().await;
    anonymous.session().await;
    assert_eq!(anonymous.get(PATH).await.status, 401);
    assert_eq!(anonymous.post(PATH, input.clone()).await.status, 401);
    assert_eq!(client.post(PATH, input.clone()).await.status, 403);
    client.login("admin").await;
    for (origin, csrf) in [(None, true), (Some(crate::common::ORIGIN), false)] {
        let reply = client
            .request("POST", PATH, input.clone(), origin, csrf)
            .await;
        assert_eq!(reply.status, 403);
        assert_eq!(reply.body["code"], "invalid_request_origin");
    }
    for invalid in [
        json!({"name":"  ","agent_id":"new-agent"}),
        json!({"name":"Wiki","agent_id":"missing"}),
        json!({"name":"Wiki","agent_id":"disabled-agent"}),
    ] {
        assert_eq!(client.post(PATH, invalid).await.status, 400);
    }
    sqlx::query("CREATE TRIGGER test_wiki_failure BEFORE INSERT ON wikis WHEN NEW.name='Fail' BEGIN SELECT RAISE(ABORT,'synthetic failure'); END")
        .execute(&client.pool).await.unwrap();
    assert_eq!(
        client
            .post(PATH, json!({"name":"Fail","agent_id":"new-agent"}))
            .await
            .status,
        500
    );
    let list = client.get(PATH).await.body;
    assert_eq!(list["items"].as_array().unwrap().len(), 2);
    assert_eq!(
        list["eligible_agents"],
        json!([{"id":"new-agent","name":"new-agent"}])
    );

    let mut other = client.fork("admin").await;
    let (a, b) = tokio::join!(
        client.post(PATH, input),
        other.post(PATH, json!({"name":"Other","agent_id":"new-agent"}))
    );
    let mut statuses = [a.status, b.status];
    statuses.sort();
    assert_eq!(statuses, [201, 409]);
    let saved: Value = if a.status == 201 { a.body } else { b.body };
    let items = client.get(PATH).await.body["items"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(
        items
            .into_iter()
            .filter(|item| item["agent"]["id"] == "new-agent")
            .collect::<Vec<_>>(),
        vec![saved]
    );
}
