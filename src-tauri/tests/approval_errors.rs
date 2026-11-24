//! The approval error handling.
//!
//! A missing approval and a broken database are different problems, and
//! the operator has to be told which one they have. The lookup used to
//! discard the error from its query, so every database fault became
//! "record not found" and sent the operator looking for an approval that
//! was really there.
//!
//! The failures here are manufactured on the test's own database, so the
//! production code carries no test hook: a column is removed to break the
//! lookup, and a trigger is installed to break a later write.

use agentops_lib::commands::approvals;
use agentops_lib::db::Database;
use agentops_lib::error::AppError;
use agentops_lib::seed;

fn prepared() -> Database {
    let db = Database::open_memory().expect("database");
    seed::seed(db.conn(), "2026-01-01T18:00:00Z").expect("seed");
    db
}

fn scalar(db: &Database, sql: &str, argument: &str) -> String {
    db.conn()
        .query_row(sql, [argument], |row| row.get(0))
        .unwrap_or_else(|_| panic!("the query failed: {sql}"))
}

fn count(db: &Database, sql: &str, argument: &str) -> i64 {
    db.conn()
        .query_row(sql, [argument], |row| row.get(0))
        .unwrap_or(0)
}

/// The pending approval the tests work with, read out of the database
/// rather than guessed.
fn a_pending_approval(db: &Database) -> (String, String, String) {
    let approval_id: String = db
        .conn()
        .query_row(
            "SELECT id FROM approvals WHERE state = 'pending' ORDER BY id LIMIT 1",
            [],
            |row| row.get(0),
        )
        .expect("a pending approval");
    let run_id = scalar(
        db,
        "SELECT run_id FROM approvals WHERE id = ?1",
        &approval_id,
    );
    let agent_id = scalar(
        db,
        "SELECT agent_id FROM approvals WHERE id = ?1",
        &approval_id,
    );
    (approval_id, run_id, agent_id)
}

/// A missing approval is reported as a missing approval.
#[test]
fn a_missing_approval_is_reported_as_not_found() {
    let db = prepared();
    let error = approvals::decide(&db, "approval-does-not-exist", "approved", None)
        .expect_err("a missing approval must be refused");
    assert!(
        matches!(error, AppError::NotFound("approval")),
        "a missing approval must be reported as not found, got {error:?}"
    );
}

/// A broken lookup is not reported as a missing approval.
///
/// The column the lookup reads is removed from the test's own database,
/// so the query fails with a real SQLite error rather than returning no
/// row. This is the case that used to be reported as "record not found".
#[test]
fn a_broken_lookup_is_not_reported_as_a_missing_approval() {
    let db = prepared();
    let (approval_id, run_id, agent_id) = a_pending_approval(&db);
    let run_status_before = scalar(&db, "SELECT status FROM runs WHERE id = ?1", &run_id);
    let agent_status_before = scalar(&db, "SELECT status FROM agents WHERE id = ?1", &agent_id);

    // Break the lookup: the column it reads is no longer there.
    db.conn()
        .execute_batch(
            "DROP INDEX idx_approvals_state;
             ALTER TABLE approvals DROP COLUMN state;",
        )
        .expect("the damaged schema");

    let error = approvals::decide(&db, &approval_id, "approved", Some("looks fine".into()))
        .expect_err("a broken database must be refused");

    // 1. The operation returns a database error.
    assert!(
        matches!(error, AppError::Database(_)),
        "a database fault must stay a database error, got {error:?}"
    );
    // 2. It is not the approval's not-found error.
    assert!(
        !matches!(error, AppError::NotFound(_)),
        "a database fault must not be reported as a missing approval, got {error:?}"
    );
    assert_eq!(error.code(), "db_error");

    // 3. No approval decision is committed.
    assert_eq!(
        count(
            &db,
            "SELECT COUNT(*) FROM approval_decisions WHERE approval_id = ?1",
            &approval_id,
        ),
        0,
        "no decision may be committed"
    );
    // 4. No successful audit record is committed.
    assert_eq!(
        count(
            &db,
            "SELECT COUNT(*) FROM audit_events WHERE action LIKE ?1",
            "approval.%",
        ),
        0,
        "no audit record may be committed"
    );
    // 5. The associated run state is unchanged, and so is the agent's.
    assert_eq!(
        scalar(&db, "SELECT status FROM runs WHERE id = ?1", &run_id),
        run_status_before,
        "the run must not have moved"
    );
    assert_eq!(
        scalar(&db, "SELECT status FROM agents WHERE id = ?1", &agent_id),
        agent_status_before,
        "the agent must not have moved"
    );
    // 6. The rollback succeeded and the connection is still usable.
    assert_eq!(
        count(&db, "SELECT COUNT(*) FROM pragma_transaction_state", ""),
        0,
        "the transaction must have been rolled back"
    );
    assert_eq!(
        scalar(&db, "SELECT id FROM runs WHERE id = ?1", &run_id),
        run_id,
        "the connection must still be usable"
    );
}

/// A failure after the approval has been updated rolls everything back.
///
/// The trigger makes the insert of the decision fail, after the approval
/// has already been updated inside the transaction. The transaction must
/// therefore roll back: nothing is committed, the approval is still
/// pending and the run is where it was.
#[test]
fn a_failure_after_the_approval_is_updated_rolls_everything_back() {
    let db = prepared();
    let (approval_id, run_id, agent_id) = a_pending_approval(&db);
    let run_status_before = scalar(&db, "SELECT status FROM runs WHERE id = ?1", &run_id);
    let agent_status_before = scalar(&db, "SELECT status FROM agents WHERE id = ?1", &agent_id);

    // Make the insert of the decision fail. The statement is aborted, so
    // the error is a genuine SQLite error and not a missing row.
    db.conn()
        .execute_batch(
            "CREATE TRIGGER block_the_decision BEFORE INSERT ON approval_decisions
             BEGIN SELECT RAISE(ABORT, 'the decision table is blocked'); END;",
        )
        .expect("the trigger");

    let error = approvals::decide(&db, &approval_id, "approved", Some("looks fine".into()))
        .expect_err("a broken database must be refused");

    assert!(
        matches!(error, AppError::Database(_)),
        "a database fault must stay a database error, got {error:?}"
    );
    assert!(
        !matches!(error, AppError::NotFound(_)),
        "a database fault must not be reported as a missing approval, got {error:?}"
    );

    assert_eq!(
        count(
            &db,
            "SELECT COUNT(*) FROM approval_decisions WHERE approval_id = ?1",
            &approval_id,
        ),
        0,
        "no decision may be committed"
    );
    assert_eq!(
        count(
            &db,
            "SELECT COUNT(*) FROM audit_events WHERE action LIKE ?1",
            "approval.%",
        ),
        0,
        "no audit record may be committed"
    );
    // The update to the approval was rolled back with everything else.
    assert_eq!(
        scalar(
            &db,
            "SELECT state FROM approvals WHERE id = ?1",
            &approval_id
        ),
        "pending",
        "the approval must still be pending"
    );
    assert_eq!(
        scalar(&db, "SELECT status FROM runs WHERE id = ?1", &run_id),
        run_status_before,
        "the run must not have moved"
    );
    assert_eq!(
        scalar(&db, "SELECT status FROM agents WHERE id = ?1", &agent_id),
        agent_status_before,
        "the agent must not have moved"
    );
    assert_eq!(
        count(&db, "SELECT COUNT(*) FROM pragma_transaction_state", ""),
        0,
        "the transaction must have been rolled back"
    );
}
