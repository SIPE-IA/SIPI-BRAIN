use serde_json::json;
use sipibrain::models::{RecordInput, RecordKind, RecordQuery};

#[test]
fn record_dto_rejects_client_authority_and_keeps_optional_project_query() {
    let mut payload = json!({"submission_id":"test-id","kind":"prompt","title":"Title","body_markdown":"Content","epistemic_kind":"assumption","source_ids":[]});
    let input: RecordInput = serde_json::from_value(payload.clone()).unwrap();
    assert_eq!(input.kind, RecordKind::Prompt);
    assert!(input.project_id.is_none());
    payload["author_id"] = "admin".into();
    assert!(serde_json::from_value::<RecordInput>(payload).is_err());
    let query: RecordQuery = serde_json::from_value(json!({"project_id":"project-alfa"})).unwrap();
    assert_eq!(query.project_id.as_deref(), Some("project-alfa"));
}
