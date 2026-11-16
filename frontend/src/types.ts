export type Role = 'admin' | 'programmer'
export interface User { id: string; username: string; role: Role }
export interface SessionState { user: User | null; csrf_token: string }
export interface LoginInput { username: string; password: string }
export interface ApiErrorBody { code: string; message: string }
export interface Items<T> { items: T[] }
export interface Wiki { id: string; agent_id: string; name: string }
export interface Assignment { user_id: string; wiki_id: string }
export interface AssignmentIndex extends Items<Assignment> { users: User[]; wikis: Wiki[] }
export interface AssignmentInput extends Assignment { action: 'grant' | 'revoke' }
export interface AssignmentResult extends Assignment { assigned: boolean }
export type RecordKind = 'prompt' | 'decision' | 'note' | 'rfd'
export type EpistemicKind = 'verified_fact' | 'decision' | 'assumption'
export interface SourceInput {
  body: string
  origin_kind: string
  origin_ref: string
  project_id?: string | null
}
export interface Source extends SourceInput {
  id: string
  wiki_id: string
  content_hash: string
  project_id: string | null
  contributor_id: string
  received_at: string
}
export interface RecordInput {
  submission_id: string
  kind: RecordKind
  title: string
  body_markdown: string
  project_id?: string | null
  client_id?: string | null
  epistemic_kind: EpistemicKind
  source_ids: string[]
}
export interface RecordQuery {
  q?: string
  agent_id?: string
  client_id?: string
  project_id?: string
  limit?: number
  offset?: number
}
export interface RecordSummary {
  id: string
  wiki_id: string
  kind: RecordKind
  title: string
  project_id: string | null
  client_id: string | null
  head_revision_id: string
  generation: number
  state: 'active' | 'conflicted'
}
export interface Revision {
  id: string
  record_id: string
  parent_revision_id: string | null
  body_markdown: string
  author_id: string
  created_at: string
  epistemic_kind: EpistemicKind
}
export interface RecordLink { source_record_id: string; target_record_id: string; relation: string }
export type ReviewResult = 'verified' | 'needs_review' | 'outdated'
export interface KnowledgeReviewInput { revision_id: string; result: ReviewResult; note: string }
export interface KnowledgeReview extends KnowledgeReviewInput { reviewer_id: string; reviewed_at: string }
export interface RecordDetail extends RecordSummary {
  revision: Revision
  sources: Source[]
  links: RecordLink[]
  review: KnowledgeReview | null
}
export interface ProposalInput {
  submission_id: string
  base_revision_id: string
  body_markdown: string
  epistemic_kind: EpistemicKind
  source_ids: string[]
}
export interface ProposalResult {
  record_id: string
  proposal_id: string
  revision_id: string
  state: 'active' | 'pending'
  generation: number
}
export interface Proposal {
  id: string
  submission_id: string
  base_revision_id: string
  revision: Revision
}
export interface Conflict {
  id: string
  record_id: string
  base_revision_id: string
  generation: number
  state: 'pending' | 'resolved'
  proposals: Proposal[]
}
export interface ResolutionInput {
  expected_generation: number
  body_markdown: string
  epistemic_kind: EpistemicKind
  reason: string
  source_ids: string[]
}
export interface MaintenanceInput { rules: string; status: 'pending' | 'reviewed' }
export interface WikiMaintenance extends MaintenanceInput { wiki_id: string; reviewed_at: string | null }
export interface WikiEvent {
  id: string
  wiki_id: string
  actor_id: string
  action: string
  reference_id: string
  created_at: string
}
