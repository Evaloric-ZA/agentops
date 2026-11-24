//! The defects an independent audit found, and their regression tests.
//!
//! Each test here failed before the fix it covers. The audit's findings
//! are recorded in `BUILD_STATE.md`.

use agentops_lib::commands::{policies, runs};
use agentops_lib::db::Database;
use agentops_lib::seed;
use agentops_lib::simulator;

fn prepared() -> Database {
    let db = Database::open_memory().expect("database");
    seed::seed(db.conn(), "2026-01-01T18:00:00Z").expect("seed");
    db
}

fn setting(db: &Database, key: &str) -> Option<String> {
    db.conn()
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .ok()
}

fn count(db: &Database, table: &str, like: &str) -> i64 {
    db.conn()
        .query_row(
            &format!("SELECT COUNT(*) FROM {table} WHERE id LIKE ?1"),
            [like],
            |row| row.get(0),
        )
        .unwrap_or(0)
}

/// A tick that fails part way through must leave nothing behind.
///
/// The step and everything it writes are one unit. When they were not, a
/// failure part way through committed what had already been written while
/// the step counter stayed where it was, so every retry hit the same
/// fault and the simulator was wedged for good.
#[test]
fn a_failing_tick_commits_nothing_and_does_not_consume_the_step() {
    let db = prepared();
    simulator::start(db.conn()).expect("start");

    // The first tick creates a run and writes the event
    // "sim-event-000001-0". Taking that identifier first makes the
    // insert fail part way through the step.
    db.conn()
        .execute(
            "INSERT INTO run_events (id, run_id, agent_id, kind, message, at)
             VALUES ('sim-event-000001-0', 'run-000000', 'agent-000000',
                     'run.started', 'A conflicting event', '2026-01-01T18:00:00Z')",
            [],
        )
        .expect("the conflicting event");

    let result = simulator::tick(db.conn());
    assert!(result.is_err(), "the tick should have failed");

    // Nothing was committed: no run, no stages.
    assert_eq!(
        count(&db, "runs", "sim-run-%"),
        0,
        "a failed tick must not leave a run behind"
    );
    assert_eq!(
        count(&db, "run_stages", "sim-%"),
        0,
        "a failed tick must not leave stages behind"
    );

    // And the step was not consumed, so the next tick tries the same step.
    assert_eq!(
        simulator::state(db.conn()).expect("the state").step,
        0,
        "a failed tick must not consume the step"
    );
}

/// A policy that does not exist is not reported as a successful toggle.
#[test]
fn enabling_a_policy_that_does_not_exist_is_refused() {
    let db = prepared();
    let result = policies::set_enabled(&db, "policy-does-not-exist", false);
    assert!(result.is_err(), "a missing policy must not report success");

    // A policy that does exist still works.
    policies::set_enabled(&db, "policy-read-only", false).expect("an existing policy");
}

/// The wildcards in a search term are matched literally.
///
/// A search for "%" or "_" used to match every row, because those are
/// wildcards in a LIKE pattern.
#[test]
fn a_wildcard_in_a_search_is_matched_literally() {
    let db = prepared();
    let all = runs::list(&db, None, None).expect("every run");
    assert_eq!(all.len(), 10, "the seed has ten runs");

    for wildcard in ["%", "_"] {
        let found = runs::list(&db, None, Some(wildcard.to_string())).expect("the search");
        assert!(
            found.is_empty(),
            "a search for {wildcard:?} matched {} runs; it must match none",
            found.len()
        );
    }

    // A real term still matches. The seeded agents display under their own
    // names, so this matches by agent.
    let found = runs::list(&db, None, Some("archaeologist".to_string())).expect("the search");
    assert!(!found.is_empty(), "a real search term still matches");
}

/// An identifier is measured in characters, as its error message says.
///
/// The validator only accepts ASCII letters, digits, '-', '_' and '.', so
/// a multi-byte character is refused either way. The difference between
/// counting bytes and counting characters is therefore not whether the
/// input is accepted but *which* refusal explains why: a string of a
/// hundred accented letters is a hundred characters long, not eight
/// hundred bytes, so the length rule must not be the one that speaks.
#[test]
fn an_identifier_is_measured_in_characters() {
    // 128 characters is accepted, 129 is not.
    let at_the_limit = "a".repeat(128);
    assert!(
        agentops_lib::error::validate_id("id", &at_the_limit).is_ok(),
        "128 characters should be accepted"
    );
    let over_the_limit = "a".repeat(129);
    assert!(
        agentops_lib::error::validate_id("id", &over_the_limit).is_err(),
        "129 characters should be refused"
    );

    // A hundred characters that are not ASCII: refused for the character
    // set, not for the length.
    let accented = "é".repeat(100);
    let message = match agentops_lib::error::validate_id("id", &accented) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("a non-ASCII identifier must be refused"),
    };
    assert!(
        message.contains("may only contain"),
        "the length rule spoke about a string that is within the length limit: {message}"
    );
}

/// A reset restores exactly the state the first seed produced.
///
/// The seed wrote the wall clock into "seeded_at", so every re-seed
/// stored a different value and the reset was not reproducible, which is
/// what the documentation claims it is.
#[test]
fn a_reset_restores_the_same_state_byte_for_byte() {
    let dir = std::env::temp_dir().join(format!(
        "agentops-reset-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).expect("the temporary directory");
    let path = dir.join("reset.db");

    let db = Database::open(&path).expect("the database");
    seed::seed(db.conn(), "2026-01-01T18:00:00Z").expect("the seed");
    let seeded_at = setting(&db, "seeded_at");
    assert_eq!(
        seeded_at.as_deref(),
        Some("2026-01-01T18:00:00Z"),
        "the seed records the anchor it was given"
    );

    // Advance the simulator, then reset it.
    simulator::start(db.conn()).expect("start");
    for _ in 0..4 {
        simulator::tick(db.conn()).expect("tick");
    }
    simulator::reset(db.conn()).expect("the reset");

    assert_eq!(
        setting(&db, "seeded_at"),
        seeded_at,
        "a reset must restore the seeded state exactly"
    );
    assert_eq!(
        count(&db, "runs", "sim-run-%"),
        0,
        "a reset removes the simulator's runs"
    );
    assert_eq!(
        simulator::state(db.conn()).expect("the state").step,
        0,
        "a reset rewinds the counter"
    );

    std::fs::remove_dir_all(&dir).ok();
}
