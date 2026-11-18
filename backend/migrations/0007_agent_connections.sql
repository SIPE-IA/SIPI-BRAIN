CREATE TABLE agent_connections (
    agent_id TEXT PRIMARY KEY NOT NULL REFERENCES agents(id),
    destination_id TEXT NOT NULL,
    expected_identity TEXT NOT NULL,
    revision TEXT NOT NULL
);
