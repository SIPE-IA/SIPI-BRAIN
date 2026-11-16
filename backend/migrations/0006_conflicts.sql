CREATE TABLE edit_proposals (
    id TEXT PRIMARY KEY,
    record_id TEXT NOT NULL REFERENCES records(id),
    base_revision_id TEXT NOT NULL REFERENCES revisions(id),
    proposed_revision_id TEXT NOT NULL UNIQUE REFERENCES revisions(id),
    submission_id TEXT NOT NULL,
    payload_hash TEXT NOT NULL,
    author_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('active','pending','resolved')),
    UNIQUE(author_id,submission_id)
);
CREATE TABLE edit_conflicts (
    id TEXT PRIMARY KEY,
    record_id TEXT NOT NULL REFERENCES records(id),
    base_revision_id TEXT NOT NULL REFERENCES revisions(id),
    generation INTEGER NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('pending','resolved'))
);
CREATE UNIQUE INDEX one_pending_conflict ON edit_conflicts(record_id) WHERE state='pending';
CREATE TABLE conflict_candidates (
    conflict_id TEXT NOT NULL REFERENCES edit_conflicts(id),
    proposal_id TEXT NOT NULL REFERENCES edit_proposals(id),
    PRIMARY KEY(conflict_id,proposal_id)
);
CREATE TABLE conflict_resolutions (
    id TEXT PRIMARY KEY,
    conflict_id TEXT NOT NULL UNIQUE REFERENCES edit_conflicts(id),
    expected_generation INTEGER NOT NULL,
    result_revision_id TEXT NOT NULL REFERENCES revisions(id),
    administrator_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL,
    reason TEXT NOT NULL
);
CREATE TRIGGER proposal_scope BEFORE INSERT ON edit_proposals
WHEN NOT EXISTS(SELECT 1 FROM revisions a JOIN revisions b ON b.record_id=a.record_id WHERE a.id=NEW.base_revision_id AND b.id=NEW.proposed_revision_id AND a.record_id=NEW.record_id)
BEGIN SELECT RAISE(ABORT,'invalid proposal'); END;
CREATE TRIGGER candidate_scope BEFORE INSERT ON conflict_candidates
WHEN NOT EXISTS(SELECT 1 FROM edit_conflicts c JOIN edit_proposals p ON p.record_id=c.record_id WHERE c.id=NEW.conflict_id AND p.id=NEW.proposal_id)
BEGIN SELECT RAISE(ABORT,'invalid candidate'); END;
CREATE TRIGGER proposal_no_rewrite BEFORE UPDATE OF id,record_id,base_revision_id,proposed_revision_id,submission_id,payload_hash,author_id,created_at ON edit_proposals BEGIN SELECT RAISE(ABORT,'immutable proposal'); END;
CREATE TRIGGER proposal_no_delete BEFORE DELETE ON edit_proposals BEGIN SELECT RAISE(ABORT,'immutable proposal'); END;
CREATE TRIGGER candidate_no_update BEFORE UPDATE ON conflict_candidates BEGIN SELECT RAISE(ABORT,'immutable candidate'); END;
CREATE TRIGGER candidate_no_delete BEFORE DELETE ON conflict_candidates BEGIN SELECT RAISE(ABORT,'immutable candidate'); END;
CREATE TRIGGER resolution_no_update BEFORE UPDATE ON conflict_resolutions BEGIN SELECT RAISE(ABORT,'immutable resolution'); END;
CREATE TRIGGER resolution_no_delete BEFORE DELETE ON conflict_resolutions BEGIN SELECT RAISE(ABORT,'immutable resolution'); END;
