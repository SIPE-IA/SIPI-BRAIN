mod common;
use common::{ORIGIN, TestClient};
use serde_json::json;

const CATALOG: &str = "/api/admin/connection-destinations";
const CONNECTION: &str = "/api/admin/agents/insight/connection";
const VERIFY: &str = "/api/admin/agents/insight/connection/test";

#[tokio::test]
async fn default_catalog_denies_unapproved_configuration_and_has_no_side_effects() {
    let mut c = TestClient::knowledge().await;
    assert_eq!(c.get(CATALOG).await.status, 403);
    assert_eq!(
        c.request("PUT", CONNECTION, json!({}), Some(ORIGIN), true)
            .await
            .status,
        403
    );
    assert_eq!(c.post(VERIFY, json!({"revision":"none"})).await.status, 403);
    c.login("admin").await;
    assert_eq!(c.get(CATALOG).await.body, json!({"items":[]}));
    let before = c.business_snapshot().await;
    let config = json!({"destination_id":"unapproved","expected_identity":"insight"});
    assert_eq!(
        c.request("PUT", CONNECTION, config.clone(), Some(ORIGIN), false)
            .await
            .body["code"],
        "invalid_request_origin"
    );
    assert_eq!(
        c.request("PUT", CONNECTION, config, Some(ORIGIN), true)
            .await
            .status,
        400
    );
    assert_eq!(
        c.post(VERIFY, json!({"revision":"none"})).await.body,
        json!({"status":"not_testable","revision":"none"})
    );
    assert_eq!(c.business_snapshot().await, before);
}

#[cfg(feature = "local-connection-fixture")]
mod fixture {
    use super::*;
    use serde_json::{Value, json};
    use sipibrain::{agent_connection::Catalog, api::application_with_catalog};
    use std::{path::Path, time::Duration};
    use tokio::{io::AsyncWriteExt, net::TcpListener, time::timeout};

    fn publish(path: &Path, version: u64, entries: &[Value]) {
        let tmp = path.with_extension("tmp");
        std::fs::write(
            &tmp,
            json!({"version": version, "destinations": entries}).to_string(),
        )
        .unwrap();
        std::fs::rename(tmp, path).unwrap();
    }

    async fn save(c: &mut TestClient, identity: &str) -> Value {
        let response = c
            .request(
                "PUT",
                CONNECTION,
                json!({
                    "destination_id":"fixture","expected_identity":identity
                }),
                Some(ORIGIN),
                true,
            )
            .await;
        assert_eq!(response.status, 200);
        response.body
    }

    #[tokio::test]
    async fn verification_preserves_business_data_and_fails_closed_on_revocation() {
        let mut c = TestClient::knowledge().await;
        c.login("admin").await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let path = c.root.join("catalog.json");
        let entry = json!({"id":"fixture","label":"Fixture","port":listener.local_addr().unwrap().port(),"version":"a"});
        publish(&path, 1, std::slice::from_ref(&entry));
        let catalog = Catalog::fixture(path.clone());
        c.router = application_with_catalog(c.pool.clone(), ORIGIN, &c.root, catalog.clone())
            .await
            .unwrap();
        assert_eq!(
            c.get(CATALOG).await.body,
            json!({"items":[{"id":"fixture","label":"Fixture"}]})
        );
        let first = save(&mut c, "first").await;
        assert_eq!(save(&mut c, "first").await, first);
        let saved = save(&mut c, "expected").await;
        assert_ne!(saved["revision"], first["revision"]);
        let before = c.business_snapshot().await;
        assert_eq!(
            c.get("/api/admin/agents").await.body["items"]
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item["id"] == "insight")
                .unwrap()["connection"],
            saved
        );
        let test = json!({"revision":saved["revision"]});

        for (identity, outcome) in [
            ("expected", "verified"),
            ("wrong", "identity_unverified"),
            ("", "unreachable"),
        ] {
            let respond = async {
                let (mut socket, _) = timeout(Duration::from_secs(3), listener.accept())
                    .await
                    .unwrap()
                    .unwrap();
                if identity.is_empty() {
                    tokio::time::sleep(Duration::from_millis(9200)).await;
                } else {
                    socket
                        .write_all(serde_json::to_string(identity).unwrap().as_bytes())
                        .await
                        .unwrap();
                    socket.shutdown().await.unwrap();
                }
            };
            let (reply, _) = tokio::join!(c.post(VERIFY, test.clone()), respond);
            assert_eq!(reply.status, 200);
            assert_eq!(
                reply.body,
                json!({"status":outcome,"revision":saved["revision"]})
            );
            assert_eq!(c.business_snapshot().await, before);
        }

        let redirected_to = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let redirect = async {
            let (mut socket, _) = listener.accept().await.unwrap();
            socket
                .write_all(
                    json!({"redirect":redirected_to.local_addr().unwrap().to_string()})
                        .to_string()
                        .as_bytes(),
                )
                .await
                .unwrap();
            socket.shutdown().await.unwrap();
        };
        let (reply, _) = tokio::join!(
            c.post(VERIFY, test.clone()),
            timeout(Duration::from_secs(3), redirect)
        );
        assert_eq!(reply.body["status"], "not_testable");
        assert!(
            timeout(Duration::from_millis(40), redirected_to.accept())
                .await
                .is_err()
        );

        for (revoked, reapproved) in [(2, None), (4, Some(5))] {
            let revoke_while_waiting = async {
                let (mut socket, _) = timeout(Duration::from_secs(3), listener.accept())
                    .await
                    .unwrap()
                    .unwrap();
                publish(&path, revoked, &[]);
                catalog.destinations().await;
                if let Some(version) = reapproved {
                    let mut new = entry.clone();
                    new["version"] = json!("b");
                    publish(&path, version, &[new]);
                    catalog.destinations().await;
                }
                socket.write_all(br#""expected""#).await.unwrap();
                socket.shutdown().await.unwrap();
            };
            let (reply, _) = tokio::join!(c.post(VERIFY, test.clone()), revoke_while_waiting);
            assert_eq!(reply.status, 200);
            assert_eq!(
                reply.body,
                json!({"status":"not_testable","revision":saved["revision"]})
            );
            if reapproved.is_none() {
                assert_eq!(c.get(CATALOG).await.body, json!({"items":[]}));
                let rejected = c
                    .request(
                        "PUT",
                        CONNECTION,
                        json!({"destination_id":"fixture","expected_identity":"new"}),
                        Some(ORIGIN),
                        true,
                    )
                    .await;
                assert_eq!(rejected.status, 400);
                assert_eq!(
                    c.post(VERIFY, test.clone()).await.body["status"],
                    "not_testable"
                );
                assert!(
                    timeout(Duration::from_millis(40), listener.accept())
                        .await
                        .is_err()
                );
                let mut new = entry.clone();
                new["version"] = json!("b");
                publish(&path, 3, &[new]);
            }
        }
        let gate = catalog.lock().await;
        let mut queued = Box::pin(c.post(VERIFY, test.clone()));
        assert!(
            std::future::poll_fn(|cx| {
                std::task::Poll::Ready(std::future::Future::poll(queued.as_mut(), cx).is_pending())
            })
            .await
        );
        publish(&path, 6, &[]);
        drop(gate);
        assert_eq!(queued.await.body["status"], "not_testable");
        assert!(
            timeout(Duration::from_millis(40), listener.accept())
                .await
                .is_err()
        );
        assert_eq!(c.business_snapshot().await, before);
    }
}
