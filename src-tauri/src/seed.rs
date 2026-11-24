//! Synthetic seed data.
//!
//! Everything here is invented. There are no real hosts, no real people, no
//! real model names, and no network addresses. The data is anchored to a
//! fixed instant so that "reset demo data" reproduces byte-for-byte the same
//! state it produced the first time.

use rusqlite::{params, Connection};

use crate::error::AppResult;
use crate::util::{now_rfc3339, to_rfc3339};

/// The seed anchor, in minutes before the anchor instant.
const SHIFT_START_MINUTES: i64 = 240;
/// How long a simulated shift lasts, in minutes.
const SHIFT_LENGTH_MINUTES: i64 = 240;

const AGENT_SEEDS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "code-archaeologist",
        "code-archaeologist",
        "local-sim",
        "standard",
        "trusted",
    ),
    (
        "documentation-writer",
        "documentation-writer",
        "local-sim",
        "standard",
        "limited",
    ),
    (
        "test-engineer",
        "test-engineer",
        "local-sim",
        "standard",
        "trusted",
    ),
    (
        "security-reviewer",
        "security-reviewer",
        "local-sim",
        "elevated",
        "trusted",
    ),
    (
        "dependency-planner",
        "dependency-planner",
        "local-sim",
        "standard",
        "limited",
    ),
    (
        "refactor-engineer",
        "refactor-engineer",
        "local-sim",
        "standard",
        "limited",
    ),
    (
        "build-engineer",
        "build-engineer",
        "local-sim",
        "standard",
        "trusted",
    ),
    (
        "migration-planner",
        "migration-planner",
        "local-sim",
        "elevated",
        "untrusted",
    ),
    (
        "service-validator",
        "service-validator",
        "local-sim",
        "standard",
        "trusted",
    ),
    (
        "release-reviewer",
        "release-reviewer",
        "local-sim",
        "elevated",
        "trusted",
    ),
];

const CAPABILITY_SEEDS: &[(&str, &str, &str, bool)] = &[
    (
        "code-archaeologist",
        "read-repository",
        "Read a local repository tree",
        true,
    ),
    (
        "code-archaeologist",
        "write-notes",
        "Write findings to the local notes store",
        true,
    ),
    (
        "documentation-writer",
        "read-repository",
        "Read a local repository tree",
        true,
    ),
    (
        "documentation-writer",
        "write-notes",
        "Write findings to the local notes store",
        true,
    ),
    (
        "test-engineer",
        "run-tests",
        "Run the local test suite",
        true,
    ),
    (
        "test-engineer",
        "read-repository",
        "Read a local repository tree",
        true,
    ),
    (
        "security-reviewer",
        "read-repository",
        "Read a local repository tree",
        true,
    ),
    (
        "security-reviewer",
        "audit-query",
        "Query the local audit log",
        true,
    ),
    (
        "dependency-planner",
        "read-repository",
        "Read a local repository tree",
        true,
    ),
    (
        "dependency-planner",
        "write-notes",
        "Write findings to the local notes store",
        false,
    ),
    (
        "refactor-engineer",
        "write-notes",
        "Write findings to the local notes store",
        true,
    ),
    ("build-engineer", "run-build", "Run the local build", true),
    (
        "build-engineer",
        "run-tests",
        "Run the local test suite",
        true,
    ),
    (
        "migration-planner",
        "read-repository",
        "Read a local repository tree",
        false,
    ),
    (
        "service-validator",
        "run-tests",
        "Run the local test suite",
        true,
    ),
    (
        "release-reviewer",
        "audit-query",
        "Query the local audit log",
        true,
    ),
    (
        "release-reviewer",
        "approve-change",
        "Approve a release gate",
        true,
    ),
];

/// A seeded tool: id, name, kind, status, latency, success rate, calls.
type ToolSeed = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    Option<i64>,
    f64,
    i64,
);

const TOOL_SEEDS: &[ToolSeed] = &[
    (
        "tool-index",
        "index-search",
        "index",
        "healthy",
        Some(42),
        0.99,
        1840,
    ),
    (
        "tool-lint",
        "lint-runner",
        "linter",
        "healthy",
        Some(120),
        0.97,
        620,
    ),
    (
        "tool-build",
        "build-runner",
        "builder",
        "degraded",
        Some(880),
        0.82,
        145,
    ),
    (
        "tool-package",
        "package-inspector",
        "packaging",
        "healthy",
        Some(210),
        0.95,
        92,
    ),
    (
        "tool-notes",
        "notes-store",
        "storage",
        "healthy",
        Some(8),
        0.99,
        2310,
    ),
    (
        "tool-net-check",
        "network-probe",
        "network",
        "offline",
        None,
        0.0,
        0,
    ),
];

const POLICY_SEEDS: &[(&str, &str, &str, &str, bool, &str)] = &[
    (
        "policy-read-only",
        "Read-only default",
        "workspace",
        "allow",
        true,
        "Agents may read the workspace by default.",
    ),
    (
        "policy-no-network",
        "No outbound network",
        "connector",
        "deny",
        true,
        "v0.1 binds no network listener and makes no outbound request.",
    ),
    (
        "policy-approve-writes",
        "Writes need approval",
        "workspace",
        "require_approval",
        true,
        "A write action raises an approval request for a reviewer.",
    ),
    (
        "policy-elevated-review",
        "Elevated review only",
        "host",
        "require_approval",
        true,
        "Elevated host classes always route to a human reviewer.",
    ),
    (
        "policy-untrusted-read",
        "Untrusted agents are read-only",
        "agent",
        "deny",
        true,
        "An untrusted agent may not write, only read.",
    ),
    (
        "policy-audit-all",
        "Audit every action",
        "audit",
        "allow",
        true,
        "Every simulator and approval action is written to the audit log.",
    ),
];

const TEAM_SEEDS: &[(&str, &str, &str, bool, &[&str])] = &[
    (
        "team-alpha",
        "Operator One",
        "operator",
        true,
        &["code-archaeologist", "test-engineer"],
    ),
    (
        "team-beta",
        "Reviewer One",
        "reviewer",
        true,
        &["security-reviewer", "release-reviewer"],
    ),
    (
        "team-gamma",
        "Administrator One",
        "administrator",
        false,
        &["build-engineer"],
    ),
    (
        "team-delta",
        "Operator Two",
        "operator",
        false,
        &["documentation-writer", "refactor-engineer"],
    ),
];

/// The tasks the seeded runs are working on.
const TASK_SEEDS: &[(&str, &str)] = &[
    ("code-archaeologist", "Map the local module graph"),
    ("documentation-writer", "Draft the module reference"),
    ("test-engineer", "Close the flaky test gap"),
    ("security-reviewer", "Review the dependency policy"),
    ("dependency-planner", "Plan the dependency refresh"),
    ("refactor-engineer", "Split the oversized module"),
    ("build-engineer", "Verify the release build"),
    ("migration-planner", "Draft the schema migration"),
    ("service-validator", "Validate the local service contract"),
    ("release-reviewer", "Prepare the release gate"),
];

/// Populate the database with synthetic data anchored at `anchor`.
///
/// The function is idempotent in the sense that it clears the seeded tables
/// first, so it can be used both for the first launch and for a reset.
pub fn seed(conn: &Connection, anchor: &str) -> AppResult<()> {
    let base = crate::util::parse_rfc3339(anchor)
        .ok_or_else(|| crate::error::AppError::Internal("invalid seed anchor"))?;

    let tx = conn.unchecked_transaction()?;
    tx.execute_batch(
        "DELETE FROM approval_decisions;
         DELETE FROM approvals;
         DELETE FROM run_events;
         DELETE FROM run_stages;
         DELETE FROM runs;
         DELETE FROM agent_capabilities;
         DELETE FROM agents;
         DELETE FROM tools;
         DELETE FROM policies;
         DELETE FROM team_members;
         DELETE FROM audit_events;",
    )?;

    let shift_start = base - chrono::Duration::minutes(SHIFT_START_MINUTES);

    // --- agents -----------------------------------------------------------
    for (index, (id, name, adapter, host_class, trust)) in AGENT_SEEDS.iter().enumerate() {
        let status = match index % 5 {
            0 => "working",
            1 => "idle",
            2 => "working",
            3 => "approval",
            _ => "failed",
        };
        let heartbeat = shift_start + chrono::Duration::minutes((index as i64) * 7 + 3);
        tx.execute(
            "INSERT INTO agents
                 (id, display_name, adapter, host_class, status, trust_level,
                  current_run_id, last_heartbeat)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7)",
            params![
                id,
                name,
                adapter,
                host_class,
                status,
                trust,
                to_rfc3339(heartbeat)
            ],
        )?;
    }

    for (agent_id, name, description, granted) in CAPABILITY_SEEDS {
        tx.execute(
            "INSERT INTO agent_capabilities (agent_id, name, description, granted)
             VALUES (?1, ?2, ?3, ?4)",
            params![agent_id, name, description, i64::from(*granted)],
        )?;
    }

    // --- runs, stages and events ------------------------------------------
    for (run_index, (agent_id, task)) in TASK_SEEDS.iter().enumerate() {
        let run_id = format!("run-{run_index:06}");
        let status = match run_index % 5 {
            0 => "running",
            1 => "succeeded",
            2 => "waiting",
            3 => "failed",
            _ => "succeeded",
        };
        let started = shift_start + chrono::Duration::minutes((run_index as i64) * 21);
        let duration = 12 + (run_index as i64 % 7) * 6;
        let ended = started + chrono::Duration::minutes(duration);
        let finished = status == "succeeded" || status == "failed";
        let model = if run_index % 3 == 0 {
            "sample-model-large"
        } else {
            "sample-model"
        };
        let compute = 1.5 + (run_index as f64 % 5.0) * 0.75;

        tx.execute(
            "INSERT INTO runs
                 (id, agent_id, task, status, started_at, ended_at,
                  duration_seconds, artifacts, model, compute_units)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                run_id,
                agent_id,
                task,
                status,
                to_rfc3339(started),
                if finished {
                    Some(to_rfc3339(ended))
                } else {
                    None
                },
                if finished { Some(duration * 60) } else { None },
                format!(r#"["artifact-{run_index}.txt"]"#),
                model,
                compute
            ],
        )?;

        let stage_names = ["plan", "execute", "verify"];
        for (ordinal, stage_name) in stage_names.iter().enumerate() {
            let stage_status = match (run_index + ordinal) % 4 {
                0 => "running",
                1 => "succeeded",
                2 => "succeeded",
                _ => "queued",
            };
            let stage_started = started + chrono::Duration::minutes((ordinal as i64) * 4);
            let stage_ended = if stage_status == "succeeded" {
                Some(to_rfc3339(stage_started + chrono::Duration::minutes(3)))
            } else {
                None
            };
            tx.execute(
                "INSERT INTO run_stages
                     (run_id, ordinal, name, status, started_at, ended_at, detail)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    run_id,
                    ordinal as i64,
                    stage_name,
                    stage_status,
                    to_rfc3339(stage_started),
                    stage_ended,
                    format!("{stage_name} stage for {task}")
                ],
            )?;
        }

        for (event_index, kind) in ["run.started", "stage.progress", "run.finished"]
            .iter()
            .enumerate()
        {
            let at = started + chrono::Duration::minutes(event_index as i64 * 5);
            tx.execute(
                "INSERT INTO run_events (id, run_id, agent_id, kind, message, at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    format!("event-{run_index:06}-{event_index}"),
                    run_id,
                    agent_id,
                    kind,
                    format!("{kind} for {task}"),
                    to_rfc3339(at)
                ],
            )?;
        }

        if status == "running" || status == "waiting" {
            tx.execute(
                "UPDATE agents SET current_run_id = ?1 WHERE id = ?2",
                params![run_id, agent_id],
            )?;
        }
    }

    // --- approvals --------------------------------------------------------
    let approval_specs: &[(&str, &str, &str, f64, &str)] = &[
        (
            "run-000002",
            "test-engineer",
            "high",
            0.82,
            "A test fixture writes outside the sample workspace.",
        ),
        (
            "run-000004",
            "security-reviewer",
            "medium",
            0.64,
            "A dependency recommendation needs a second reviewer.",
        ),
        (
            "run-000007",
            "migration-planner",
            "high",
            0.91,
            "A schema migration would remove a seeded column.",
        ),
    ];
    for (index, (run_id, agent_id, severity, confidence, reason)) in
        approval_specs.iter().enumerate()
    {
        let created = shift_start + chrono::Duration::minutes(40 + index as i64 * 55);
        tx.execute(
            "INSERT INTO approvals
                 (id, run_id, agent_id, severity, reason, evidence, confidence,
                  state, created_at, decided_at, comment)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8, NULL, NULL)",
            params![
                format!("approval-{index:06}"),
                run_id,
                agent_id,
                severity,
                reason,
                format!(r#"["evidence-{index}-a","evidence-{index}-b"]"#),
                confidence,
                to_rfc3339(created)
            ],
        )?;
    }

    // --- tools, policies, team, audit -------------------------------------
    for (id, name, kind, status, latency, rate, calls) in TOOL_SEEDS {
        tx.execute(
            "INSERT INTO tools
                 (id, name, kind, status, latency_ms, success_rate, calls_last_hour)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, name, kind, status, latency, rate, calls],
        )?;
    }

    for (id, name, scope, effect, enabled, detail) in POLICY_SEEDS {
        tx.execute(
            "INSERT INTO policies (id, name, scope, effect, enabled, detail)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name, scope, effect, i64::from(*enabled), detail],
        )?;
    }

    for (id, name, role, on_shift, agents) in TEAM_SEEDS {
        let list = agents
            .iter()
            .map(|a| format!("\"{a}\""))
            .collect::<Vec<_>>()
            .join(",");
        tx.execute(
            "INSERT INTO team_members (id, name, role, on_shift, agent_ids)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, name, role, i64::from(*on_shift), format!("[{list}]")],
        )?;
    }

    for (index, action) in [
        "simulator.shift_started",
        "simulator.run_created",
        "simulator.approval_requested",
        "simulator.run_failed",
        "simulator.run_completed",
    ]
    .iter()
    .enumerate()
    {
        let at = shift_start + chrono::Duration::minutes(index as i64 * 47);
        tx.execute(
            "INSERT INTO audit_events
                 (id, at, actor, action, subject, outcome, detail)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                format!("audit-seed-{index:06}"),
                to_rfc3339(at),
                "simulator",
                action,
                format!("sample-subject-{}", index),
                "recorded",
                "Synthetic demo event."
            ],
        )?;
    }

    tx.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('seed_anchor', ?1)",
        params![anchor],
    )?;
    tx.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('seeded_at', ?1)",
        params![anchor],
    )?;
    tx.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('shift_length_minutes', ?1)",
        params![SHIFT_LENGTH_MINUTES.to_string()],
    )?;

    tx.commit()?;
    Ok(())
}

/// The stored seed anchor, if the database has been seeded.
pub fn stored_anchor(conn: &Connection) -> AppResult<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = 'seed_anchor'")?;
    let mut rows = stmt.query([])?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get::<_, String>(0)?)),
        None => Ok(None),
    }
}

/// Ensure the database holds seed data, seeding it on first use.
pub fn ensure_seeded(conn: &Connection) -> AppResult<()> {
    match stored_anchor(conn)? {
        Some(_) => Ok(()),
        None => seed(conn, &now_rfc3339()),
    }
}
