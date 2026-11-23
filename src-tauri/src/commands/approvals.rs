//! Approval queue commands.

use rusqlite::params;

use crate::db::Database;
use crate::domain::{Approval, ApprovalDecision, ApprovalState, Severity};
use crate::error::{AppError, AppResult};

fn parse_severity(value: &str) -> Severity {
    match value {
        "high" => Severity::High,
        "low" => Severity::Low,
        _ => Severity::Medium,
    }
}

fn parse_state(value: &str) -> ApprovalState {
    match value {
        "approved" => ApprovalState::Approved,
        "rejected" => ApprovalState::Rejected,
        _ => ApprovalState::Pending,
    }
}

/// List approvals, newest first, optionally filtered by state.
pub fn list(db: &Database, state: Option<String>) -> AppResult<Vec<Approval>> {
    let mut sql = String::from(
        "SELECT a.id, a.run_id, a.agent_id, ag.display_name, a.severity, a.reason,
                a.evidence, a.confidence, a.state, a.created_at, a.decided_at, a.comment
         FROM approvals a LEFT JOIN agents ag ON ag.id = a.agent_id
         WHERE 1 = 1",
    );
    let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(state) = state.as_deref().filter(|s| !s.is_empty() && *s != "all") {
        sql.push_str(" AND a.state = ?");
        args.push(Box::new(state.to_string()));
    }
    sql.push_str(" ORDER BY a.created_at DESC");

    let mut stmt = db.conn().prepare(&sql)?;
    let refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(|a| a.as_ref()).collect();
    let rows = stmt.query_map(refs.as_slice(), |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, f64>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, String>(9)?,
            row.get::<_, Option<String>>(10)?,
            row.get::<_, Option<String>>(11)?,
        ))
    })?;

    let mut out = Vec::new();
    for row in rows {
        let (
            id,
            run_id,
            agent_id,
            agent_name,
            severity,
            reason,
            evidence,
            confidence,
            state,
            created_at,
            decided_at,
            comment,
        ) = row?;
        out.push(Approval {
            id,
            run_id,
            agent_id,
            agent_name: agent_name.unwrap_or_else(|| "unknown agent".to_string()),
            severity: parse_severity(&severity),
            reason,
            evidence: crate::commands::audit::parse_json_array(&evidence),
            confidence,
            state: parse_state(&state),
            created_at,
            decided_at,
            comment,
        });
    }
    Ok(out)
}

/// Approve or reject an approval.
///
/// The decision and the state change happen in one transaction, so a
/// decision is never recorded without the state changing, or the other way
/// round. Deciding an already-decided approval is refused: a stale tab must
/// not be able to overwrite a decision that has already been recorded.
pub fn decide(
    db: &Database,
    approval_id: &str,
    outcome: &str,
    comment: Option<String>,
) -> AppResult<Approval> {
    let approval_id = crate::error::validate_id("approval id", approval_id)?;
    let outcome = match outcome.trim().to_ascii_lowercase().as_str() {
        "approved" => "approved",
        "rejected" => "rejected",
        other => {
            return Err(AppError::InvalidInput(format!(
                "outcome must be 'approved' or 'rejected', got '{other}'"
            )))
        }
    };
    let comment = comment
        .map(|c| crate::error::validate_text("comment", &c, 1000))
        .transpose()?;

    let tx = db.conn().unchecked_transaction()?;
    let current: Option<String> = tx
        .query_row(
            "SELECT state FROM approvals WHERE id = ?1",
            params![approval_id],
            |row| row.get(0),
        )
        .ok();
    match current {
        None => {
            return Err(AppError::NotFound("approval"));
        }
        Some(state) if state != "pending" => {
            return Err(AppError::Conflict(format!(
                "this approval was already {state}"
            )));
        }
        Some(_) => {}
    }

    let now = crate::util::now_rfc3339();
    tx.execute(
        "UPDATE approvals SET state = ?2, decided_at = ?3, comment = ?4 WHERE id = ?1",
        params![approval_id, outcome, now, comment],
    )?;
    let decision_id = format!(
        "decision-{}",
        crate::domain::ids::next_id("evt").trim_start_matches("evt-")
    );
    tx.execute(
        "INSERT INTO approval_decisions (id, approval_id, outcome, comment, at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![decision_id, approval_id, outcome, comment, now],
    )?;
    tx.execute(
        "INSERT INTO audit_events (id, at, actor, action, subject, outcome, detail)
         VALUES (?1, ?2, 'operator', ?3, ?4, ?5, ?6)",
        params![
            format!(
                "audit-{}",
                crate::domain::ids::next_id("evt").trim_start_matches("evt-")
            ),
            now,
            format!("approval.{outcome}"),
            approval_id,
            outcome,
            comment.unwrap_or_default()
        ],
    )?;
    // An approved run continues; a rejected run stops.
    //
    // A rejected run has to be given its end time and its duration. The
    // simulator's own failure path sets both, and without them the run
    // detail view reads "still running" for a run that was rejected.
    let approved = outcome == "approved";
    // The run is only moved on if it is still waiting. A decision is
    // recorded whatever the run has become — it is a fact about what the
    // operator did — but a run that has already failed must not be
    // resurrected by a late decision.
    let moved: usize = if approved {
        tx.execute(
            "UPDATE runs SET status = 'running'
             WHERE id = (SELECT run_id FROM approvals WHERE id = ?1)
               AND status IN ('waiting', 'running', 'queued')",
            params![approval_id],
        )?
    } else {
        tx.execute(
            "UPDATE runs SET status = 'failed', ended_at = ?2, duration_seconds = 120
             WHERE id = (SELECT run_id FROM approvals WHERE id = ?1)
               AND status IN ('waiting', 'running', 'queued')",
            params![approval_id, now],
        )?
    };
    if moved > 0 {
        if approved {
            // The agent was waiting on this decision, so it goes back to work.
            tx.execute(
                "UPDATE agents SET status = 'working'
                 WHERE id = (SELECT agent_id FROM approvals WHERE id = ?1)",
                params![approval_id],
            )?;
        } else {
            // A rejected run stops, so the agent stops with it. This mirrors
            // the simulator's own failure path.
            tx.execute(
                "UPDATE agents SET status = 'failed', current_run_id = NULL
                 WHERE id = (SELECT agent_id FROM approvals WHERE id = ?1)",
                params![approval_id],
            )?;
        }
    }
    tx.commit()?;

    list(db, None)?
        .into_iter()
        .find(|approval| approval.id == approval_id)
        .ok_or(AppError::NotFound("approval"))
}

/// The decisions recorded against an approval.
pub fn decisions(db: &Database, approval_id: &str) -> AppResult<Vec<ApprovalDecision>> {
    let approval_id = crate::error::validate_id("approval id", approval_id)?;
    let mut stmt = db.conn().prepare(
        "SELECT id, approval_id, outcome, comment, at FROM approval_decisions
         WHERE approval_id = ?1 ORDER BY at",
    )?;
    let rows = stmt.query_map([approval_id], |row| {
        Ok(ApprovalDecision {
            id: row.get(0)?,
            approval_id: row.get(1)?,
            outcome: row.get(2)?,
            comment: row.get(3)?,
            at: row.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed;

    fn prepared() -> Database {
        let db = Database::open_memory().expect("database");
        seed::seed(db.conn(), "2026-01-01T18:00:00Z").expect("seed");
        db
    }

    #[test]
    fn the_queue_lists_every_seeded_approval() {
        let db = prepared();
        let approvals = list(&db, None).expect("list");
        assert_eq!(approvals.len(), 3);
    }

    #[test]
    fn deciding_an_approval_records_the_decision_and_the_state_change() {
        let db = prepared();
        let approval = decide(
            &db,
            "approval-000000",
            "approved",
            Some("looks fine".into()),
        )
        .expect("decide");
        assert_eq!(approval.state, ApprovalState::Approved);
        assert_eq!(approval.comment.as_deref(), Some("looks fine"));
        assert!(approval.decided_at.is_some());
        let decisions = decisions(&db, "approval-000000").expect("decisions");
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].outcome, "approved");
    }

    #[test]
    fn deciding_twice_is_refused() {
        let db = prepared();
        decide(&db, "approval-000000", "approved", None).expect("first decision");
        let second = decide(&db, "approval-000000", "rejected", None);
        assert!(second.is_err(), "a second decision must be refused");
    }

    #[test]
    fn an_unknown_outcome_is_rejected() {
        let db = prepared();
        assert!(decide(&db, "approval-000000", "maybe", None).is_err());
    }

    #[test]
    fn a_missing_approval_is_not_found() {
        let db = prepared();
        assert!(decide(&db, "approval-999999", "approved", None).is_err());
    }
}
