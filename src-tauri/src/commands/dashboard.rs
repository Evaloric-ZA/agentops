//! The Night Shift summary: four cards plus the three lower panels.

use rusqlite::params;

use crate::db::Database;
use crate::domain::{
    AgentStatusCount, ComputeByModel, DashboardSummary, DisagreementRow, ToolHealthRow,
};
use crate::error::AppResult;

/// Build the whole Night Shift summary in one pass.
pub fn summary(db: &Database) -> AppResult<DashboardSummary> {
    let runs_total: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM runs", [], |row| row.get(0))?;
    let finished_autonomously: i64 = db.conn().query_row(
        "SELECT COUNT(*) FROM runs WHERE status = 'succeeded' AND id NOT IN
             (SELECT run_id FROM approvals)",
        [],
        |row| row.get(0),
    )?;
    let waiting_on_you: i64 = db.conn().query_row(
        "SELECT COUNT(*) FROM approvals WHERE state = 'pending'",
        [],
        |row| row.get(0),
    )?;
    let compute_used_units: f64 = db.conn().query_row(
        "SELECT COALESCE(SUM(compute_units), 0) FROM runs",
        [],
        |row| row.get(0),
    )?;

    let mut stmt = db
        .conn()
        .prepare("SELECT status, COUNT(*) FROM agents GROUP BY status ORDER BY status")?;
    let rows = stmt.query_map([], |row| {
        Ok(AgentStatusCount {
            status: row.get(0)?,
            count: row.get(1)?,
        })
    })?;
    let mut agent_status_counts = Vec::new();
    for row in rows {
        agent_status_counts.push(row?);
    }

    let mut stmt = db.conn().prepare(
        "SELECT model, COALESCE(SUM(compute_units), 0) FROM runs
         GROUP BY model ORDER BY model",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ComputeByModel {
            model: row.get(0)?,
            units: row.get(1)?,
        })
    })?;
    let mut compute_by_model = Vec::new();
    for row in rows {
        compute_by_model.push(row?);
    }

    // Reviewer disagreement is derived from the seeded evidence: a run that
    // raised more than one approval with different severities is a run the
    // reviewers did not agree on.
    let mut stmt = db.conn().prepare(
        "SELECT a.run_id, COALESCE(ag.display_name, a.run_id) AS subject,
                COUNT(*) AS reviewers,
                CASE WHEN COUNT(DISTINCT a.severity) > 1
                     THEN 'split severity' ELSE 'aligned' END AS split
         FROM approvals a LEFT JOIN agents ag ON ag.id = a.agent_id
         GROUP BY a.run_id HAVING COUNT(*) > 0
         ORDER BY reviewers DESC, a.run_id LIMIT 10",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(DisagreementRow {
            run_id: row.get(0)?,
            subject: row.get(1)?,
            reviewers: row.get(2)?,
            split: row.get(3)?,
        })
    })?;
    let mut reviewer_disagreement = Vec::new();
    for row in rows {
        reviewer_disagreement.push(row?);
    }

    let mut stmt = db
        .conn()
        .prepare("SELECT id, name, status, success_rate, latency_ms FROM tools ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(ToolHealthRow {
            tool_id: row.get(0)?,
            name: row.get(1)?,
            status: row.get(2)?,
            success_rate: row.get(3)?,
            latency_ms: row.get(4)?,
        })
    })?;
    let mut tool_health = Vec::new();
    for row in rows {
        tool_health.push(row?);
    }

    Ok(DashboardSummary {
        runs_total,
        finished_autonomously,
        waiting_on_you,
        compute_used_units,
        agent_status_counts,
        compute_by_model,
        reviewer_disagreement,
        tool_health,
    })
}

/// The Night Shift timeline: one row per run, positioned in the shift window.
pub fn timeline(db: &Database) -> AppResult<Vec<crate::domain::TimelineEntry>> {
    let mut stmt = db.conn().prepare(
        "SELECT r.id, r.agent_id, a.display_name, r.status, r.task, r.started_at,
                COALESCE(r.ended_at, r.started_at)
         FROM runs r JOIN agents a ON a.id = r.agent_id
         ORDER BY r.started_at",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
        ))
    })?;

    let mut raw = Vec::new();
    for row in rows {
        raw.push(row?);
    }
    let earliest = raw
        .iter()
        .filter_map(|r| crate::util::parse_rfc3339(&r.5))
        .min();
    let latest = raw
        .iter()
        .filter_map(|r| crate::util::parse_rfc3339(&r.6))
        .max();

    let mut out = Vec::new();
    for (id, agent_id, agent_name, status, task, started, ended) in raw {
        let start_minute = earliest
            .and_then(|base| {
                crate::util::parse_rfc3339(&started).map(|dt| (dt - base).num_minutes())
            })
            .unwrap_or(0);
        let end_minute = earliest
            .and_then(|base| crate::util::parse_rfc3339(&ended).map(|dt| (dt - base).num_minutes()))
            .unwrap_or(start_minute);
        let severity: Option<String> = db
            .conn()
            .query_row(
                "SELECT severity FROM approvals WHERE run_id = ?1 ORDER BY created_at LIMIT 1",
                params![id],
                |row| row.get(0),
            )
            .ok();
        out.push(crate::domain::TimelineEntry {
            run_id: id,
            agent_id,
            agent_name,
            status,
            task,
            start_minute,
            end_minute: end_minute.max(start_minute),
            severity,
        });
    }
    let _ = latest;
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
    fn the_summary_counts_the_seeded_records() {
        let db = prepared();
        let summary = summary(&db).expect("summary");
        assert_eq!(summary.runs_total, 10);
        assert_eq!(summary.waiting_on_you, 3);
        assert!(summary.compute_used_units > 0.0);
        assert!(!summary.agent_status_counts.is_empty());
        assert!(!summary.tool_health.is_empty());
    }

    #[test]
    fn the_timeline_positions_every_run() {
        let db = prepared();
        let timeline = timeline(&db).expect("timeline");
        assert_eq!(timeline.len(), 10);
        for entry in &timeline {
            assert!(entry.end_minute >= entry.start_minute);
        }
    }
}
