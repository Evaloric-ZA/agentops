//! The deterministic agent simulator.
//!
//! The simulator is the only thing that produces "live" activity in v0.1.
//! It is driven by an explicit tick from the frontend rather than by a
//! background thread, so the operator controls the pace and the frontend
//! never has to render faster than it can keep up.
//!
//! Determinism comes from the persisted step counter: the same step always
//! produces the same event. Two runs from the same seed produce the same
//! sequence.

use rusqlite::{params, Connection};

use crate::error::AppResult;
use crate::util::{now_rfc3339, to_rfc3339};
use chrono::Utc;

/// The stage names a simulated run passes through.
const STAGES: [&str; 4] = ["plan", "execute", "verify", "report"];

/// How many phases one run's cycle has: create, advance, advance,
/// approval, fail, complete.
const PHASES: i64 = 6;

/// The simulated tasks, chosen to read as synthetic work.
const TASKS: [&str; 6] = [
    "Reconcile the sample inventory",
    "Rebuild the sample index",
    "Summarise the sample findings",
    "Verify the sample contract",
    "Rotate the sample fixtures",
    "Publish the sample digest",
];

/// What one simulator tick does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickOutcome {
    /// A new run was created.
    Created,
    /// A run moved to its next stage.
    Advanced,
    /// A run raised an approval request.
    Approval,
    /// A run failed.
    Failed,
    /// A run finished.
    Completed,
    /// Nothing happened on this tick.
    Idle,
}

impl TickOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            TickOutcome::Created => "created",
            TickOutcome::Advanced => "advanced",
            TickOutcome::Approval => "approval",
            TickOutcome::Failed => "failed",
            TickOutcome::Completed => "completed",
            TickOutcome::Idle => "idle",
        }
    }
}

/// The current simulator state.
#[derive(Debug, Clone)]
pub struct SimulatorState {
    pub running: bool,
    pub step: i64,
    pub speed: f64,
}

/// Read the simulator state from the settings table.
pub fn state(conn: &Connection) -> AppResult<SimulatorState> {
    Ok(SimulatorState {
        running: setting(conn, "simulator_running")?.as_deref() == Some("true"),
        step: setting(conn, "simulator_step")?
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0),
        speed: setting(conn, "simulator_speed")?
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(1.0),
    })
}

/// Set a simulator setting.
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

fn setting(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = ?1")?;
    let mut rows = stmt.query([key])?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get::<_, String>(0)?)),
        None => Ok(None),
    }
}

/// Start (or resume) the simulator.
pub fn start(conn: &Connection) -> AppResult<SimulatorState> {
    set_setting(conn, "simulator_running", "true")?;
    audit(
        conn,
        "simulator",
        "simulator.started",
        "simulator",
        "recorded",
        "Simulator started.",
    )?;
    state(conn)
}

/// Pause the simulator. The step counter is kept, so a resume continues
/// exactly where it stopped.
pub fn pause(conn: &Connection) -> AppResult<SimulatorState> {
    set_setting(conn, "simulator_running", "false")?;
    audit(
        conn,
        "simulator",
        "simulator.paused",
        "simulator",
        "recorded",
        "Simulator paused.",
    )?;
    state(conn)
}

/// Set the simulator speed, clamped to a range the UI can safely render.
pub fn set_speed(conn: &Connection, speed: f64) -> AppResult<SimulatorState> {
    if !speed.is_finite() {
        return Err(crate::error::AppError::InvalidInput(
            "simulator speed must be a finite number".into(),
        ));
    }
    let clamped = speed.clamp(0.25, 4.0);
    set_setting(conn, "simulator_speed", &format!("{clamped:.2}"))?;
    state(conn)
}

/// Advance the simulator by one step.
///
/// The step counter selects what happens, so the sequence is fixed:
/// step % 6 == 0 creates a run, 1 and 2 advance it, 3 raises an approval,
/// 4 fails it, and 5 completes it.
pub fn tick(conn: &Connection) -> AppResult<(SimulatorState, TickOutcome)> {
    let current = state(conn)?;
    if !current.running {
        return Ok((current, TickOutcome::Idle));
    }
    let step = current.step + 1;
    let outcome = apply_step(conn, step)?;
    set_setting(conn, "simulator_step", &step.to_string())?;
    Ok((state(conn)?, outcome))
}

/// Apply one step of the simulation.
fn apply_step(conn: &Connection, step: i64) -> AppResult<TickOutcome> {
    // The phases are counted from the first tick, so the cycle starts with
    // the create phase. The step counter starts at zero and the first tick
    // is step one, so counting from the step number would begin the
    // sequence at the second phase and leave the first five ticks with
    // nothing to do.
    let phase = (step - 1).rem_euclid(PHASES);
    let agent = pick_agent(conn, step)?;
    match phase {
        0 => create_run(conn, &agent, step),
        1 | 2 => advance_run(conn, &agent, step),
        3 => raise_approval(conn, &agent, step),
        4 => fail_run(conn, &agent, step),
        5 => complete_run(conn, &agent, step),
        _ => Ok(TickOutcome::Idle),
    }
}

/// Choose the agent for a step, round-robin over the seeded agents.
fn pick_agent(conn: &Connection, step: i64) -> AppResult<String> {
    let mut stmt = conn.prepare("SELECT id FROM agents ORDER BY id")?;
    let mut rows = stmt.query([])?;
    let mut ids = Vec::new();
    while let Some(row) = rows.next()? {
        ids.push(row.get::<_, String>(0)?);
    }
    if ids.is_empty() {
        return Err(crate::error::AppError::Internal("no agents are seeded"));
    }
    // The agent is chosen by the cycle, not by the step. The phase cycle
    // is create, advance, advance, approval, fail, complete, and those six
    // phases belong to one run, so they must belong to one agent. Choosing
    // the agent by the step instead spread the six phases over six
    // unrelated agents, and every phase after the first found no run of
    // its own to act on.
    let cycle = (step - 1).div_euclid(PHASES);
    let index = cycle.rem_euclid(ids.len() as i64) as usize;
    Ok(ids[index].clone())
}

/// Whether a run is waiting on a human decision.
///
/// A run that is waiting on a reviewer must not be failed or completed
/// behind the reviewer's back. The simulator leaves it waiting, and the
/// operator's decision is what moves it on.
fn waiting_on_a_decision(conn: &Connection, run_id: &str) -> AppResult<bool> {
    let pending: i64 = conn.query_row(
        "SELECT COUNT(*) FROM approvals WHERE run_id = ?1 AND state = 'pending'",
        params![run_id],
        |row| row.get(0),
    )?;
    Ok(pending > 0)
}

/// The run currently owned by an agent, if any.
///
/// Only the simulator's own runs are considered. The seeded runs are the
/// fixed baseline the operator sees, and the simulator must not advance,
/// fail or complete them: a seeded run that already carries an approval
/// would end up with two, and the seeded baseline would change underneath
/// the operator as the demonstration ran.
fn active_run(conn: &Connection, agent_id: &str) -> AppResult<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT id FROM runs
         WHERE agent_id = ?1 AND id LIKE 'sim-run-%'
           AND status IN ('queued', 'running', 'waiting')
         ORDER BY started_at DESC LIMIT 1",
    )?;
    let mut rows = stmt.query([agent_id])?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get::<_, String>(0)?)),
        None => Ok(None),
    }
}

fn create_run(conn: &Connection, agent_id: &str, step: i64) -> AppResult<TickOutcome> {
    let task = TASKS[(step as usize) % TASKS.len()];
    let run_id = format!("sim-run-{step:06}");
    let now = Utc::now();
    conn.execute(
        "INSERT INTO runs
             (id, agent_id, task, status, started_at, ended_at, duration_seconds,
              artifacts, model, compute_units)
         VALUES (?1, ?2, ?3, 'running', ?4, NULL, NULL, '[]', 'sample-model', ?5)",
        params![
            run_id,
            agent_id,
            task,
            to_rfc3339(now),
            0.5 + (step % 4) as f64 * 0.25
        ],
    )?;
    for (ordinal, stage) in STAGES.iter().enumerate() {
        conn.execute(
            "INSERT INTO run_stages
                 (run_id, ordinal, name, status, started_at, ended_at, detail)
             VALUES (?1, ?2, ?3, 'queued', ?4, NULL, '')",
            params![run_id, ordinal as i64, stage, to_rfc3339(now)],
        )?;
    }
    conn.execute(
        "INSERT INTO run_events (id, run_id, agent_id, kind, message, at)
         VALUES (?1, ?2, ?3, 'run.started', ?4, ?5)",
        params![
            format!("sim-event-{step:06}-0"),
            run_id,
            agent_id,
            format!("Simulated run started: {task}"),
            to_rfc3339(now)
        ],
    )?;
    conn.execute(
        "UPDATE agents SET status = 'working', current_run_id = ?1, last_heartbeat = ?2
         WHERE id = ?3",
        params![run_id, to_rfc3339(now), agent_id],
    )?;
    audit(
        conn,
        "simulator",
        "simulator.run_created",
        &run_id,
        "recorded",
        &format!("Created simulated run for {agent_id}."),
    )?;
    Ok(TickOutcome::Created)
}

fn advance_run(conn: &Connection, agent_id: &str, step: i64) -> AppResult<TickOutcome> {
    let run_id = match active_run(conn, agent_id)? {
        Some(id) => id,
        None => return Ok(TickOutcome::Idle),
    };
    let now = Utc::now();
    // SQLite does not accept ORDER BY / LIMIT in an UPDATE, so the next
    // queued stage is selected first and then updated by its ordinal.
    let next_stage: Option<i64> = conn
        .query_row(
            "SELECT ordinal FROM run_stages
             WHERE run_id = ?1 AND status = 'queued' ORDER BY ordinal LIMIT 1",
            params![run_id],
            |row| row.get(0),
        )
        .ok();
    if let Some(ordinal) = next_stage {
        conn.execute(
            "UPDATE run_stages
             SET status = 'running', started_at = ?2, detail = 'Simulated stage progress'
             WHERE run_id = ?1 AND ordinal = ?3",
            params![run_id, to_rfc3339(now), ordinal],
        )?;
    }
    conn.execute(
        "INSERT INTO run_events (id, run_id, agent_id, kind, message, at)
         VALUES (?1, ?2, ?3, 'stage.progress', 'Simulated stage progress', ?4)",
        params![
            format!("sim-event-{step:06}-p"),
            run_id,
            agent_id,
            to_rfc3339(now)
        ],
    )?;
    Ok(TickOutcome::Advanced)
}

fn raise_approval(conn: &Connection, agent_id: &str, step: i64) -> AppResult<TickOutcome> {
    let run_id = match active_run(conn, agent_id)? {
        Some(id) => id,
        None => return Ok(TickOutcome::Idle),
    };
    let now = Utc::now();
    let approval_id = format!("sim-approval-{step:06}");
    conn.execute(
        "INSERT INTO approvals
             (id, run_id, agent_id, severity, reason, evidence, confidence,
              state, created_at, decided_at, comment)
         VALUES (?1, ?2, ?3, 'medium', ?4, ?5, 0.71, 'pending', ?6, NULL, NULL)",
        params![
            approval_id,
            run_id,
            agent_id,
            "A simulated action needs a reviewer decision.",
            r#"["sim-evidence-a","sim-evidence-b"]"#,
            to_rfc3339(now)
        ],
    )?;
    conn.execute(
        "UPDATE runs SET status = 'waiting' WHERE id = ?1",
        params![run_id],
    )?;
    conn.execute(
        "UPDATE agents SET status = 'approval' WHERE id = ?1",
        params![agent_id],
    )?;
    conn.execute(
        "INSERT INTO run_events (id, run_id, agent_id, kind, message, at)
         VALUES (?1, ?2, ?3, 'approval.requested', 'Simulated approval requested', ?4)",
        params![
            format!("sim-event-{step:06}-a"),
            run_id,
            agent_id,
            to_rfc3339(now)
        ],
    )?;
    audit(
        conn,
        "simulator",
        "simulator.approval_requested",
        &approval_id,
        "recorded",
        "Simulated approval requested.",
    )?;
    Ok(TickOutcome::Approval)
}

fn fail_run(conn: &Connection, agent_id: &str, step: i64) -> AppResult<TickOutcome> {
    let run_id = match active_run(conn, agent_id)? {
        Some(id) => id,
        None => return Ok(TickOutcome::Idle),
    };
    if waiting_on_a_decision(conn, &run_id)? {
        // The run is waiting on the operator, so it waits.
        return Ok(TickOutcome::Idle);
    }
    let now = Utc::now();
    conn.execute(
        "UPDATE runs SET status = 'failed', ended_at = ?2, duration_seconds = 120
         WHERE id = ?1",
        params![run_id, to_rfc3339(now)],
    )?;
    conn.execute(
        "UPDATE run_stages SET status = 'failed', ended_at = ?2
         WHERE run_id = ?1 AND status IN ('queued', 'running')",
        params![run_id, to_rfc3339(now)],
    )?;
    conn.execute(
        "INSERT INTO run_events (id, run_id, agent_id, kind, message, at)
         VALUES (?1, ?2, ?3, 'run.failed', 'Simulated failure', ?4)",
        params![
            format!("sim-event-{step:06}-f"),
            run_id,
            agent_id,
            to_rfc3339(now)
        ],
    )?;
    conn.execute(
        "UPDATE agents SET status = 'failed', current_run_id = NULL WHERE id = ?1",
        params![agent_id],
    )?;
    audit(
        conn,
        "simulator",
        "simulator.run_failed",
        &run_id,
        "recorded",
        "Simulated run failed.",
    )?;
    Ok(TickOutcome::Failed)
}

fn complete_run(conn: &Connection, agent_id: &str, step: i64) -> AppResult<TickOutcome> {
    let run_id = match active_run(conn, agent_id)? {
        Some(id) => id,
        None => return Ok(TickOutcome::Idle),
    };
    if waiting_on_a_decision(conn, &run_id)? {
        // The run is waiting on the operator, so it waits.
        return Ok(TickOutcome::Idle);
    }
    let now = Utc::now();
    conn.execute(
        "UPDATE runs SET status = 'succeeded', ended_at = ?2, duration_seconds = 180
         WHERE id = ?1",
        params![run_id, to_rfc3339(now)],
    )?;
    conn.execute(
        "UPDATE run_stages SET status = 'succeeded', ended_at = ?2
         WHERE run_id = ?1 AND status IN ('queued', 'running')",
        params![run_id, to_rfc3339(now)],
    )?;
    conn.execute(
        "INSERT INTO run_events (id, run_id, agent_id, kind, message, at)
         VALUES (?1, ?2, ?3, 'run.completed', 'Simulated run completed', ?4)",
        params![
            format!("sim-event-{step:06}-c"),
            run_id,
            agent_id,
            to_rfc3339(now)
        ],
    )?;
    conn.execute(
        "UPDATE agents SET status = 'idle', current_run_id = NULL, last_heartbeat = ?2
         WHERE id = ?1",
        params![agent_id, to_rfc3339(now)],
    )?;
    audit(
        conn,
        "simulator",
        "simulator.run_completed",
        &run_id,
        "recorded",
        "Simulated run completed.",
    )?;
    Ok(TickOutcome::Completed)
}

/// Write an audit record.
pub fn audit(
    conn: &Connection,
    actor: &str,
    action: &str,
    subject: &str,
    outcome: &str,
    detail: &str,
) -> AppResult<()> {
    let id = format!(
        "audit-{}",
        crate::domain::ids::next_id("evt").trim_start_matches("evt-")
    );
    conn.execute(
        "INSERT INTO audit_events (id, at, actor, action, subject, outcome, detail)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, now_rfc3339(), actor, action, subject, outcome, detail],
    )?;
    Ok(())
}

/// Restore the demo to its original seeded state.
///
/// The simulation creates its own runs as it goes, and the seed is restored
/// over the top of them, so rewinding means restoring the seed rather than
/// just the step counter: otherwise a replay would start from a different
/// state and produce a different sequence. The identifier counter is reset
/// too, so a replay reproduces the same identifiers as well as the same
/// events.
pub fn reset(conn: &Connection) -> AppResult<()> {
    let anchor = crate::seed::stored_anchor(conn)?
        .ok_or_else(|| crate::error::AppError::Internal("the database has no seed anchor"))?;
    crate::seed::seed(conn, &anchor)?;
    crate::domain::ids::reset_counter();
    set_setting(conn, "simulator_step", "0")?;
    set_setting(conn, "simulator_running", "false")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::seed;

    fn prepared() -> Database {
        let db = Database::open_memory().expect("database");
        seed::seed(db.conn(), "2026-01-01T18:00:00Z").expect("seed");
        db
    }

    #[test]
    fn a_paused_simulator_does_nothing() {
        let db = prepared();
        let (_, outcome) = tick(db.conn()).expect("tick");
        assert_eq!(outcome, TickOutcome::Idle);
    }

    #[test]
    fn the_same_step_sequence_produces_the_same_result() {
        let db = prepared();
        start(db.conn()).expect("start");
        let first: Vec<TickOutcome> = (0..6).map(|_| tick(db.conn()).expect("tick").1).collect();
        // Restore and replay: the outcome sequence must be identical.
        reset(db.conn()).expect("reset");
        start(db.conn()).expect("start");
        let second: Vec<TickOutcome> = (0..6).map(|_| tick(db.conn()).expect("tick").1).collect();
        assert_eq!(first, second);
    }

    #[test]
    fn the_speed_is_clamped_to_a_renderable_range() {
        let db = prepared();
        let state = set_speed(db.conn(), 100.0).expect("speed");
        assert_eq!(state.speed, 4.0);
        let state = set_speed(db.conn(), 0.01).expect("speed");
        assert_eq!(state.speed, 0.25);
    }

    #[test]
    fn a_non_finite_speed_is_rejected() {
        let db = prepared();
        assert!(set_speed(db.conn(), f64::NAN).is_err());
    }
}
