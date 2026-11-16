CREATE TABLE clients (id TEXT PRIMARY KEY, display_name TEXT NOT NULL);
CREATE TABLE projects (id TEXT PRIMARY KEY, name TEXT NOT NULL, client_id TEXT REFERENCES clients(id));
CREATE TABLE records (
    id TEXT PRIMARY KEY,
    wiki_id TEXT NOT NULL REFERENCES wikis(id),
    kind TEXT NOT NULL CHECK(kind IN ('prompt','decision','note','rfd')),
    title TEXT NOT NULL CHECK(length(trim(title))>0),
    project_id TEXT REFERENCES projects(id),
    client_id TEXT REFERENCES clients(id),
    head_revision_id TEXT NOT NULL REFERENCES revisions(id) DEFERRABLE INITIALLY DEFERRED,
    generation INTEGER NOT NULL CHECK(generation>0),
    state TEXT NOT NULL CHECK(state IN ('active','conflicted')),
    CHECK(kind!='rfd' OR project_id IS NOT NULL)
);
CREATE INDEX records_by_wiki ON records(wiki_id,id);
CREATE TRIGGER record_client_insert BEFORE INSERT ON records
WHEN NEW.project_id IS NOT NULL AND NEW.client_id IS NOT NULL
AND NOT EXISTS(SELECT 1 FROM projects WHERE id=NEW.project_id AND client_id=NEW.client_id)
BEGIN SELECT RAISE(ABORT,'invalid association'); END;
CREATE TRIGGER record_client_update BEFORE UPDATE OF project_id,client_id ON records
WHEN NEW.project_id IS NOT NULL AND NEW.client_id IS NOT NULL
AND NOT EXISTS(SELECT 1 FROM projects WHERE id=NEW.project_id AND client_id=NEW.client_id)
BEGIN SELECT RAISE(ABORT,'invalid association'); END;
CREATE TABLE record_submissions (
    author_id TEXT NOT NULL REFERENCES users(id),
    submission_id TEXT NOT NULL,
    payload_hash TEXT NOT NULL,
    record_id TEXT NOT NULL REFERENCES records(id),
    PRIMARY KEY(author_id,submission_id)
);
