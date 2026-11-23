//! Versioned schema migrations.
//!
//! Migrations are applied in order, each inside a transaction, and recorded
//! in `schema_migrations`. A migration that has already been applied is
//! never re-applied.

/// One migration: a version number and the SQL that brings the schema to it.
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "initial_schema",
    sql: r#"
CREATE TABLE agents (
    id                TEXT PRIMARY KEY,
    display_name      TEXT NOT NULL,
    adapter           TEXT NOT NULL,
    host_class        TEXT NOT NULL,
    status            TEXT NOT NULL,
    trust_level       TEXT NOT NULL,
    current_run_id    TEXT,
    last_heartbeat    TEXT NOT NULL
);

CREATE TABLE agent_capabilities (
    agent_id          TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    name              TEXT NOT NULL,
    description       TEXT NOT NULL,
    granted           INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (agent_id, name)
);

CREATE TABLE runs (
    id                TEXT PRIMARY KEY,
    agent_id          TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    task              TEXT NOT NULL,
    status            TEXT NOT NULL,
    started_at        TEXT NOT NULL,
    ended_at          TEXT,
    duration_seconds  INTEGER,
    artifacts         TEXT NOT NULL DEFAULT '[]',
    model             TEXT NOT NULL DEFAULT 'sample-model',
    compute_units     REAL NOT NULL DEFAULT 0
);

CREATE TABLE run_stages (
    run_id            TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    ordinal           INTEGER NOT NULL,
    name              TEXT NOT NULL,
    status            TEXT NOT NULL,
    started_at        TEXT NOT NULL,
    ended_at          TEXT,
    detail            TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (run_id, ordinal)
);

CREATE TABLE run_events (
    id                TEXT PRIMARY KEY,
    run_id            TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    agent_id          TEXT NOT NULL,
    kind              TEXT NOT NULL,
    message           TEXT NOT NULL,
    at                TEXT NOT NULL
);

CREATE TABLE approvals (
    id                TEXT PRIMARY KEY,
    run_id            TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    agent_id          TEXT NOT NULL,
    severity          TEXT NOT NULL,
    reason            TEXT NOT NULL,
    evidence          TEXT NOT NULL DEFAULT '[]',
    confidence        REAL NOT NULL DEFAULT 0,
    state             TEXT NOT NULL,
    created_at        TEXT NOT NULL,
    decided_at        TEXT,
    comment           TEXT
);

CREATE TABLE approval_decisions (
    id                TEXT PRIMARY KEY,
    approval_id       TEXT NOT NULL REFERENCES approvals(id) ON DELETE CASCADE,
    outcome           TEXT NOT NULL,
    comment           TEXT,
    at                TEXT NOT NULL
);

CREATE TABLE tools (
    id                TEXT PRIMARY KEY,
    name              TEXT NOT NULL,
    kind              TEXT NOT NULL,
    status            TEXT NOT NULL,
    latency_ms        INTEGER,
    success_rate      REAL NOT NULL DEFAULT 0,
    calls_last_hour   INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE policies (
    id                TEXT PRIMARY KEY,
    name              TEXT NOT NULL,
    scope             TEXT NOT NULL,
    effect            TEXT NOT NULL,
    enabled           INTEGER NOT NULL DEFAULT 1,
    detail            TEXT NOT NULL DEFAULT ''
);

CREATE TABLE team_members (
    id                TEXT PRIMARY KEY,
    name              TEXT NOT NULL,
    role              TEXT NOT NULL,
    on_shift          INTEGER NOT NULL DEFAULT 0,
    agent_ids         TEXT NOT NULL DEFAULT '[]'
);

CREATE TABLE audit_events (
    id                TEXT PRIMARY KEY,
    at                TEXT NOT NULL,
    actor             TEXT NOT NULL,
    action            TEXT NOT NULL,
    subject           TEXT NOT NULL,
    outcome           TEXT NOT NULL,
    detail            TEXT NOT NULL DEFAULT ''
);

CREATE TABLE app_settings (
    key               TEXT PRIMARY KEY,
    value             TEXT NOT NULL
);

-- The schema_migrations table itself is created by the migration runner
-- before any migration runs, so it is deliberately NOT declared here.

CREATE INDEX idx_runs_agent ON runs(agent_id);
CREATE INDEX idx_runs_status ON runs(status);
CREATE INDEX idx_runs_started ON runs(started_at);
CREATE INDEX idx_events_run ON run_events(run_id);
CREATE INDEX idx_events_at ON run_events(at);
CREATE INDEX idx_approvals_state ON approvals(state);
CREATE INDEX idx_approvals_created ON approvals(created_at);
CREATE INDEX idx_audit_at ON audit_events(at);
"#,
}];
