CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT 1 CHECK(active IN (0, 1))
);
CREATE TABLE wikis (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL UNIQUE REFERENCES agents(id),
    name TEXT NOT NULL
);
CREATE TABLE wiki_assignments (
    user_id TEXT NOT NULL REFERENCES users(id),
    wiki_id TEXT NOT NULL REFERENCES wikis(id),
    assigned_by TEXT NOT NULL REFERENCES users(id),
    assigned_at TEXT NOT NULL,
    PRIMARY KEY(user_id, wiki_id)
);
