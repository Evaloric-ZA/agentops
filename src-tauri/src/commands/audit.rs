//! Audit log commands, and the small JSON helpers shared with other modules.

use rusqlite::params;

use crate::db::Database;
use crate::domain::AuditEvent;
use crate::error::AppResult;

/// List the most recent audit events, newest first.
pub fn list(db: &Database, limit: Option<i64>) -> AppResult<Vec<AuditEvent>> {
    let limit = limit.unwrap_or(200).clamp(1, 1000);
    let mut stmt = db.conn().prepare(
        "SELECT id, at, actor, action, subject, outcome, detail
         FROM audit_events ORDER BY at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit], |row| {
        Ok(AuditEvent {
            id: row.get(0)?,
            at: row.get(1)?,
            actor: row.get(2)?,
            action: row.get(3)?,
            subject: row.get(4)?,
            outcome: row.get(5)?,
            detail: row.get(6)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Record an audit event.
pub fn record(
    db: &Database,
    actor: &str,
    action: &str,
    subject: &str,
    outcome: &str,
    detail: &str,
) -> AppResult<()> {
    let actor = crate::error::validate_text("actor", actor, 64)?;
    let action = crate::error::validate_text("action", action, 64)?;
    let subject = crate::error::validate_text("subject", subject, 128)?;
    let outcome = crate::error::validate_text("outcome", outcome, 32)?;
    let detail = crate::error::validate_text("detail", detail, 512)?;
    let id = format!(
        "audit-{}",
        crate::domain::ids::next_id("evt").trim_start_matches("evt-")
    );
    db.conn().execute(
        "INSERT INTO audit_events (id, at, actor, action, subject, outcome, detail)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            crate::util::now_rfc3339(),
            actor,
            action,
            subject,
            outcome,
            detail
        ],
    )?;
    Ok(())
}

/// Parse a JSON array stored as TEXT into a list of strings.
///
/// A malformed value yields an empty list rather than an error: a corrupt
/// row must not take down the whole page. The list endpoints are read
/// paths, and a read path that cannot fail is more useful to an operator
/// than one that refuses to render.
pub fn parse_json_array(raw: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(raw).unwrap_or_default()
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
    fn the_audit_log_is_listed_newest_first() {
        let db = prepared();
        let events = list(&db, None).expect("list");
        assert_eq!(events.len(), 5);
        let times: Vec<&str> = events.iter().map(|e| e.at.as_str()).collect();
        let mut sorted = times.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        assert_eq!(times, sorted);
    }

    #[test]
    fn a_recorded_event_appears_in_the_log() {
        let db = prepared();
        record(
            &db,
            "operator",
            "approval.granted",
            "approval-000000",
            "granted",
            "ok",
        )
        .expect("record");
        let events = list(&db, None).expect("list");
        assert_eq!(events.len(), 6);
        assert_eq!(events[0].action, "approval.granted");
    }

    #[test]
    fn a_malformed_json_array_reads_as_empty_not_as_an_error() {
        assert!(parse_json_array("not json").is_empty());
        assert!(parse_json_array("").is_empty());
        assert_eq!(
            parse_json_array("[\"a\",\"b\"]"),
            vec!["a".to_string(), "b".to_string()]
        );
    }
}
