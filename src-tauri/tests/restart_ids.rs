//! The identifier counter and the application restart.
//!
//! The identifier counter is a process-local atomic that starts at one in
//! every new process. The database, however, still holds the identifiers
//! the previous process minted. Without the counter being advanced past
//! what is already stored, the first decision of the new session is handed
//! an identifier that is already in use, and the write fails on the
//! primary key.
//!
//! The application is restarted far more often than it is reset, so this
//! has to work.
//!
//! This is an integration test rather than a unit test for two reasons.
//! It opens a database on disk, so the two sessions are genuinely two
//! states of the same file. And it is its own process, so resetting the
//! process-local counter here cannot race the unit tests, which share
//! theirs.

use agentops_lib::commands::approvals;
use agentops_lib::db::Database;
use agentops_lib::domain::ids;
use agentops_lib::seed;

/// A directory of its own for this test.
fn scratch_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "agentops-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).expect("the temporary directory");
    dir
}

#[test]
fn a_decision_still_works_after_the_application_restarts() {
    let dir = scratch_dir("approval-restart");
    let path = dir.join("restart.db");

    // The first session: open, seed and decide.
    {
        let db = Database::open(&path).expect("the first open");
        seed::seed(db.conn(), "2026-01-01T18:00:00Z").expect("the seed");
        approvals::decide(&db, "approval-000000", "approved", None).expect("the first decision");
    }

    // A new process: the counter starts again at one.
    ids::reset_counter();

    // The second session opens the same database and decides again.
    let db = Database::open(&path).expect("the second open");
    let result = approvals::decide(&db, "approval-000001", "rejected", None);
    assert!(
        result.is_ok(),
        "the first decision after a restart failed: {:?}",
        result.err()
    );

    // And the identifiers it wrote are new ones, not the previous
    // session's.
    let decisions: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM approval_decisions", [], |row| {
            row.get(0)
        })
        .expect("the decision count");
    assert_eq!(decisions, 2, "both decisions should be recorded");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn an_audit_event_still_works_after_the_application_restarts() {
    let dir = scratch_dir("audit-restart");
    let path = dir.join("restart.db");

    {
        let db = Database::open(&path).expect("the first open");
        seed::seed(db.conn(), "2026-01-01T18:00:00Z").expect("the seed");
        agentops_lib::commands::audit::record(
            &db,
            "operator",
            "test.action",
            "subject",
            "recorded",
            "the first event",
        )
        .expect("the first event");
    }

    ids::reset_counter();

    let db = Database::open(&path).expect("the second open");
    let result = agentops_lib::commands::audit::record(
        &db,
        "operator",
        "test.action",
        "subject",
        "recorded",
        "the second event",
    );
    assert!(
        result.is_ok(),
        "the first audit event after a restart failed: {:?}",
        result.err()
    );

    std::fs::remove_dir_all(&dir).ok();
}
