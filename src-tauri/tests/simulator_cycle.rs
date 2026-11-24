//! The simulator's cycle, and the seeded baseline it must not disturb.
//!
//! The phase cycle is create, advance, advance, approval, fail, complete,
//! and the six phases belong to one run. The simulator creates its own
//! runs: the seeded ones are the fixed baseline the operator sees, and the
//! simulator must not advance, fail or complete them.

use agentops_lib::commands::approvals;
use agentops_lib::db::Database;
use agentops_lib::seed;
use agentops_lib::simulator;
use agentops_lib::simulator::TickOutcome;

fn prepared() -> Database {
    let db = Database::open_memory().expect("database");
    seed::seed(db.conn(), "2026-01-01T18:00:00Z").expect("seed");
    db
}

/// The seeded runs, as a snapshot of their identifiers and statuses.
fn seeded_runs(db: &Database) -> Vec<(String, String, Option<String>)> {
    db.conn()
        .prepare("SELECT id, status, ended_at FROM runs WHERE id LIKE 'run-%' ORDER BY id")
        .expect("prepare")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .expect("query")
        .filter_map(|r| r.ok())
        .collect()
}

#[test]
fn the_cycle_creates_one_run_and_walks_it_through_its_phases() {
    let db = prepared();
    simulator::start(db.conn()).expect("start");

    let outcomes: Vec<TickOutcome> = (0..4)
        .map(|_| simulator::tick(db.conn()).expect("tick").1)
        .collect();

    assert_eq!(
        outcomes,
        vec![
            TickOutcome::Created,
            TickOutcome::Advanced,
            TickOutcome::Advanced,
            TickOutcome::Approval,
        ],
        "the cycle is create, advance, advance, approval"
    );

    // One run, created by the simulator.
    let simulated: i64 = db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM runs WHERE id LIKE 'sim-run-%'",
            [],
            |row| row.get(0),
        )
        .expect("the count");
    assert_eq!(simulated, 1, "the cycle creates exactly one run");
}

#[test]
fn the_simulator_never_touches_a_seeded_run() {
    let db = prepared();
    let before = seeded_runs(&db);

    simulator::start(db.conn()).expect("start");
    for _ in 0..12 {
        simulator::tick(db.conn()).expect("tick");
    }

    assert_eq!(
        seeded_runs(&db),
        before,
        "the seeded runs are the fixed baseline and must not change"
    );
}

#[test]
fn a_seeded_run_never_gains_a_second_approval() {
    let db = prepared();
    simulator::start(db.conn()).expect("start");
    for _ in 0..12 {
        simulator::tick(db.conn()).expect("tick");
    }

    let doubled: i64 = db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM (
                 SELECT run_id FROM approvals GROUP BY run_id HAVING COUNT(*) > 1
             )",
            [],
            |row| row.get(0),
        )
        .expect("the count");
    assert_eq!(
        doubled, 0,
        "no run should carry more than one approval: the reviewer panel counts them"
    );
}

#[test]
fn a_run_waiting_on_a_decision_is_not_failed_behind_the_operator() {
    let db = prepared();
    simulator::start(db.conn()).expect("start");
    for _ in 0..4 {
        simulator::tick(db.conn()).expect("tick");
    }

    // The fourth tick raised an approval, so the run is waiting.
    let waiting: i64 = db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM runs WHERE id LIKE 'sim-run-%' AND status = 'waiting'",
            [],
            |row| row.get(0),
        )
        .expect("the count");
    assert_eq!(waiting, 1, "the run should be waiting on the operator");

    // The next two ticks are the fail and complete phases. Neither may
    // touch a run that is waiting on a human decision.
    for _ in 0..2 {
        let outcome = simulator::tick(db.conn()).expect("tick").1;
        assert_eq!(
            outcome,
            TickOutcome::Idle,
            "a run waiting on a decision must not be failed or completed"
        );
    }

    let still_waiting: i64 = db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM runs WHERE id LIKE 'sim-run-%' AND status = 'waiting'",
            [],
            |row| row.get(0),
        )
        .expect("the count");
    assert_eq!(still_waiting, 1, "the run is still waiting on the operator");
}

#[test]
fn deciding_an_approval_moves_the_run_and_the_agent_on() {
    let db = prepared();
    simulator::start(db.conn()).expect("start");
    for _ in 0..4 {
        simulator::tick(db.conn()).expect("tick");
    }

    let (approval_id, agent_id): (String, String) = db
        .conn()
        .query_row(
            "SELECT id, agent_id FROM approvals WHERE id LIKE 'sim-approval-%'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("the approval");

    let agent_status: String = db
        .conn()
        .query_row(
            "SELECT status FROM agents WHERE id = ?1",
            [&agent_id],
            |row| row.get(0),
        )
        .expect("the agent status");
    assert_eq!(
        agent_status, "approval",
        "the agent is waiting on a decision"
    );

    approvals::decide(&db, &approval_id, "approved", None).expect("the decision");

    // The run is running again, and it is no longer waiting.
    let run_status: String = db
        .conn()
        .query_row(
            "SELECT status FROM runs WHERE id = (
                 SELECT run_id FROM approvals WHERE id = ?1
             )",
            [&approval_id],
            |row| row.get(0),
        )
        .expect("the run status");
    assert_eq!(run_status, "running", "an approved run continues");

    // The agent is no longer stuck on the approval.
    let agent_status: String = db
        .conn()
        .query_row(
            "SELECT status FROM agents WHERE id = ?1",
            [&agent_id],
            |row| row.get(0),
        )
        .expect("the agent status");
    assert_eq!(
        agent_status, "working",
        "a decided approval must not leave the agent waiting"
    );
}

#[test]
fn a_rejected_run_is_ended_rather_than_left_running() {
    let db = prepared();
    simulator::start(db.conn()).expect("start");
    for _ in 0..4 {
        simulator::tick(db.conn()).expect("tick");
    }

    let (approval_id, agent_id): (String, String) = db
        .conn()
        .query_row(
            "SELECT id, agent_id FROM approvals WHERE id LIKE 'sim-approval-%'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("the approval");

    approvals::decide(&db, &approval_id, "rejected", None).expect("the decision");

    // The run has ended, with an end time and a duration. Without them
    // the run detail view reads "still running" for a rejected run.
    let (status, ended_at, duration): (String, Option<String>, Option<i64>) = db
        .conn()
        .query_row(
            "SELECT status, ended_at, duration_seconds FROM runs WHERE id = (
                 SELECT run_id FROM approvals WHERE id = ?1
             )",
            [&approval_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("the run");
    assert_eq!(status, "failed", "a rejected run stops");
    assert!(ended_at.is_some(), "a rejected run has an end time");
    assert!(duration.is_some(), "a rejected run has a duration");

    let agent_status: String = db
        .conn()
        .query_row(
            "SELECT status FROM agents WHERE id = ?1",
            [&agent_id],
            |row| row.get(0),
        )
        .expect("the agent status");
    assert_eq!(
        agent_status, "failed",
        "an agent whose run was rejected is not left waiting"
    );
}
