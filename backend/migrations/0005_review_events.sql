CREATE TABLE wiki_maintenance (
    wiki_id TEXT PRIMARY KEY REFERENCES wikis(id),
    rules TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending','reviewed')),
    reviewed_at TEXT
);
CREATE TABLE knowledge_reviews (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    revision_id TEXT NOT NULL REFERENCES revisions(id),
    result TEXT NOT NULL CHECK(result IN ('verified','needs_review','outdated')),
    reviewer_id TEXT NOT NULL REFERENCES users(id),
    reviewed_at TEXT NOT NULL,
    note TEXT NOT NULL
);
CREATE TABLE wiki_events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE,
    wiki_id TEXT NOT NULL REFERENCES wikis(id),
    actor_id TEXT NOT NULL REFERENCES users(id),
    action TEXT NOT NULL,
    reference_id TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE TRIGGER review_no_update BEFORE UPDATE ON knowledge_reviews BEGIN SELECT RAISE(ABORT,'immutable review'); END;
CREATE TRIGGER review_no_delete BEFORE DELETE ON knowledge_reviews BEGIN SELECT RAISE(ABORT,'immutable review'); END;
CREATE TRIGGER event_no_update BEFORE UPDATE ON wiki_events BEGIN SELECT RAISE(ABORT,'immutable event'); END;
CREATE TRIGGER event_no_delete BEFORE DELETE ON wiki_events BEGIN SELECT RAISE(ABORT,'immutable event'); END;
