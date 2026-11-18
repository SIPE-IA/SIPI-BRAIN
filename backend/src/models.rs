use serde::{Deserialize, Serialize};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Unauthenticated,
    InvalidRequestOrigin,
    Forbidden,
    NotFound,
    InvalidInput,
    WikiAlreadyExists,
    AgentAlreadyExists,
    IdempotencyConflict,
    ConflictChanged,
    StorageBusy,
    InternalError,
    NotImplemented,
}

#[derive(Debug)]
pub struct ApiError(pub ErrorCode);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0 {
            ErrorCode::Unauthenticated => (StatusCode::UNAUTHORIZED, "Authentication required."),
            ErrorCode::InvalidRequestOrigin => {
                (StatusCode::FORBIDDEN, "Request origin or token invalid.")
            }
            ErrorCode::Forbidden => (StatusCode::FORBIDDEN, "Operation not permitted."),
            ErrorCode::NotFound => (StatusCode::NOT_FOUND, "Resource not found."),
            ErrorCode::InvalidInput => (StatusCode::BAD_REQUEST, "Invalid input."),
            ErrorCode::WikiAlreadyExists => {
                (StatusCode::CONFLICT, "This agent already has a wiki.")
            }
            ErrorCode::AgentAlreadyExists => {
                (StatusCode::CONFLICT, "This agent ID is already registered.")
            }
            ErrorCode::IdempotencyConflict => (
                StatusCode::CONFLICT,
                "Submission ID already used for different input.",
            ),
            ErrorCode::ConflictChanged => (
                StatusCode::CONFLICT,
                "Conflict changed; reload before deciding.",
            ),
            ErrorCode::StorageBusy => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Not saved; storage busy. Retry later.",
            ),
            ErrorCode::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Operation failed."),
            ErrorCode::NotImplemented => (StatusCode::NOT_IMPLEMENTED, "Not implemented."),
        };
        (
            status,
            Json(ErrorBody {
                code: self.0,
                message: message.into(),
            }),
        )
            .into_response()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Admin,
    Programmer,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub role: Role,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionState {
    pub user: Option<User>,
    pub csrf_token: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Items<T> {
    pub items: Vec<T>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Wiki {
    pub id: String,
    pub agent_id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct AgentSummary {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentRegistration {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct AgentOnboardingItem {
    pub id: String,
    pub name: String,
    pub active: bool,
    pub connection: Option<ConnectionSummary>,
}

#[derive(Clone, Serialize)]
pub struct ConnectionSummary {
    pub destination_id: String,
    pub expected_identity: String,
    pub revision: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionInput {
    pub destination_id: String,
    pub expected_identity: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionTestInput {
    pub revision: String,
}

#[derive(Serialize)]
pub struct WikiAdminItem {
    pub id: String,
    pub name: String,
    pub agent: AgentSummary,
}

#[derive(Serialize)]
pub struct WikiAdminIndex {
    pub items: Vec<WikiAdminItem>,
    pub eligible_agents: Vec<AgentSummary>,
}

#[derive(Serialize)]
pub struct WikiActivitySummary {
    #[serde(flatten)]
    pub wiki: WikiAdminItem,
    pub record_count: i64,
    pub source_count: i64,
    pub last_contribution_at: Option<String>,
    pub recent_activity: Vec<WikiActivityEntry>,
}

#[derive(Serialize)]
pub struct WikiActivityEntry {
    pub occurred_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WikiCreateInput {
    pub name: String,
    pub agent_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Assignment {
    pub user_id: String,
    pub wiki_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Assignments {
    pub users: Vec<User>,
    pub wikis: Vec<Wiki>,
    pub items: Vec<Assignment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentAction {
    Grant,
    Revoke,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssignmentInput {
    pub user_id: String,
    pub wiki_id: String,
    pub action: AssignmentAction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AssignmentResult {
    pub user_id: String,
    pub wiki_id: String,
    pub assigned: bool,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceInput {
    pub body: String,
    pub origin_kind: String,
    pub origin_ref: String,
    pub project_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Source {
    pub id: String,
    pub wiki_id: String,
    pub body: String,
    pub content_hash: String,
    pub origin_kind: String,
    pub origin_ref: String,
    pub project_id: Option<String>,
    pub contributor_id: String,
    pub received_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecordKind {
    Prompt,
    Decision,
    Note,
    Rfd,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EpistemicKind {
    VerifiedFact,
    Decision,
    Assumption,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecordState {
    Active,
    Conflicted,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecordInput {
    pub submission_id: String,
    pub kind: RecordKind,
    pub title: String,
    pub body_markdown: String,
    pub project_id: Option<String>,
    pub client_id: Option<String>,
    pub epistemic_kind: EpistemicKind,
    pub source_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RecordQuery {
    pub q: Option<String>,
    pub agent_id: Option<String>,
    pub client_id: Option<String>,
    pub project_id: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordSummary {
    pub id: String,
    pub wiki_id: String,
    pub kind: RecordKind,
    pub title: String,
    pub project_id: Option<String>,
    pub client_id: Option<String>,
    pub head_revision_id: String,
    pub generation: i64,
    pub state: RecordState,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Revision {
    pub id: String,
    pub record_id: String,
    pub parent_revision_id: Option<String>,
    pub body_markdown: String,
    pub author_id: String,
    pub created_at: String,
    pub epistemic_kind: EpistemicKind,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordDetail {
    #[serde(flatten)]
    pub summary: RecordSummary,
    pub revision: Revision,
    pub sources: Vec<Source>,
    pub links: Vec<RecordLink>,
    pub review: Option<KnowledgeReview>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalInput {
    pub submission_id: String,
    pub base_revision_id: String,
    pub body_markdown: String,
    pub epistemic_kind: EpistemicKind,
    pub source_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProposalState {
    Active,
    Pending,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProposalResult {
    pub record_id: String,
    pub proposal_id: String,
    pub revision_id: String,
    pub state: ProposalState,
    pub generation: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConflictProposal {
    pub id: String,
    pub submission_id: String,
    pub base_revision_id: String,
    pub revision: Revision,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictState {
    Pending,
    Resolved,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Conflict {
    pub id: String,
    pub record_id: String,
    pub base_revision_id: String,
    pub generation: i64,
    pub state: ConflictState,
    pub proposals: Vec<ConflictProposal>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResolutionInput {
    pub expected_generation: i64,
    pub body_markdown: String,
    pub epistemic_kind: EpistemicKind,
    pub reason: String,
    pub source_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordLink {
    pub source_record_id: String,
    pub target_record_id: String,
    pub relation: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LinkInput {
    pub target_record_id: String,
    pub relation: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceStatus {
    Pending,
    Reviewed,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MaintenanceInput {
    pub rules: String,
    pub status: MaintenanceStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Maintenance {
    pub wiki_id: String,
    pub rules: String,
    pub status: MaintenanceStatus,
    pub reviewed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewResult {
    Verified,
    NeedsReview,
    Outdated,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewInput {
    pub revision_id: String,
    pub result: ReviewResult,
    pub note: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnowledgeReview {
    pub revision_id: String,
    pub result: ReviewResult,
    pub reviewer_id: String,
    pub reviewed_at: String,
    pub note: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WikiEvent {
    pub id: String,
    pub wiki_id: String,
    pub actor_id: String,
    pub action: String,
    pub reference_id: String,
    pub created_at: String,
}
