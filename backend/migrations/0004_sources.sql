CREATE TABLE original_sources (
    id TEXT PRIMARY KEY,
    wiki_id TEXT NOT NULL REFERENCES wikis(id),
    body TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    origin_kind TEXT NOT NULL,
    origin_ref TEXT NOT NULL,
    project_id TEXT REFERENCES projects(id),
    contributor_id TEXT NOT NULL REFERENCES users(id),
    received_at TEXT NOT NULL
);
CREATE TABLE revisions (
    id TEXT PRIMARY KEY,
    record_id TEXT NOT NULL REFERENCES records(id),
    parent_revision_id TEXT REFERENCES revisions(id),
    body_markdown TEXT NOT NULL,
    author_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL,
    epistemic_kind TEXT NOT NULL CHECK(epistemic_kind IN ('verified_fact','decision','assumption'))
);
CREATE TABLE source_references (
    revision_id TEXT NOT NULL REFERENCES revisions(id),
    source_id TEXT NOT NULL REFERENCES original_sources(id),
    locator TEXT,
    PRIMARY KEY(revision_id,source_id)
);
CREATE TABLE record_links (
    source_record_id TEXT NOT NULL REFERENCES records(id),
    target_record_id TEXT NOT NULL REFERENCES records(id),
    relation TEXT NOT NULL,
    PRIMARY KEY(source_record_id,target_record_id,relation)
);
CREATE TRIGGER revision_parent_scope BEFORE INSERT ON revisions
WHEN NEW.parent_revision_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM revisions WHERE id=NEW.parent_revision_id AND record_id=NEW.record_id)
BEGIN SELECT RAISE(ABORT,'invalid parent'); END;
CREATE TRIGGER source_reference_scope BEFORE INSERT ON source_references
WHEN NOT EXISTS(SELECT 1 FROM revisions v JOIN records r ON r.id=v.record_id JOIN original_sources s ON s.wiki_id=r.wiki_id WHERE v.id=NEW.revision_id AND s.id=NEW.source_id)
BEGIN SELECT RAISE(ABORT,'invalid reference'); END;
CREATE TRIGGER record_link_scope BEFORE INSERT ON record_links
WHEN NOT EXISTS(SELECT 1 FROM records a JOIN records b ON b.wiki_id=a.wiki_id WHERE a.id=NEW.source_record_id AND b.id=NEW.target_record_id)
BEGIN SELECT RAISE(ABORT,'invalid link'); END;
CREATE TRIGGER original_source_no_update BEFORE UPDATE ON original_sources BEGIN SELECT RAISE(ABORT,'immutable source'); END;
CREATE TRIGGER original_source_no_delete BEFORE DELETE ON original_sources BEGIN SELECT RAISE(ABORT,'immutable source'); END;
CREATE TRIGGER revision_no_update BEFORE UPDATE ON revisions BEGIN SELECT RAISE(ABORT,'immutable revision'); END;
CREATE TRIGGER revision_no_delete BEFORE DELETE ON revisions BEGIN SELECT RAISE(ABORT,'immutable revision'); END;
CREATE TRIGGER reference_no_update BEFORE UPDATE ON source_references BEGIN SELECT RAISE(ABORT,'immutable reference'); END;
CREATE TRIGGER reference_no_delete BEFORE DELETE ON source_references BEGIN SELECT RAISE(ABORT,'immutable reference'); END;
