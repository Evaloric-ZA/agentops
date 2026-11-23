//! Run registry commands.

use crate::db::Database;
use crate::domain::{Run, RunEvent, RunStage, RunStatus};
use crate::error::{AppError, AppResult};

fn parse_status(value: &str) -> RunStatus {
    match value {
        "queued" => RunStatus::Queued,
        "running" => RunStatus::Running,
        "waiting" => RunStatus::Waiting,
        "failed" => RunStatus::Failed,
        _ => RunStatus::Succeeded,
    }
}

/// List runs, newest first, optionally filtered by status and searched by
/// task or agent name.
pub fn list(db: &Database, status: Option<String>, search: Option<String>) -> AppResult<Vec<Run>> {
    let mut sql = String::from(
        "SELECT r.id, r.agent_id, a.display_name, r.task, r.status, r.started_at,
                r.ended_at, r.duration_seconds, r.artifacts
         FROM runs r JOIN agents a ON a.id = r.agent_id
         WHERE 1 = 1",
    );
    let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(status) = status.as_deref().filter(|s| !s.is_empty() && *s != "all") {
        sql.push_str(" AND r.status = ?");
        args.push(Box::new(status.to_string()));
    }
    if let Some(search) = search.as_deref().filter(|s| !s.trim().is_empty()) {
        sql.push_str(" AND (r.task LIKE ? OR a.display_name LIKE ?)");
        let pattern = format!("%{}%", search.trim());
        args.push(Box::new(pattern.clone()));
        args.push(Box::new(pattern));
    }
    sql.push_str(" ORDER BY r.started_at DESC");

    let mut stmt = db.conn().prepare(&sql)?;
    let refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(|a| a.as_ref()).collect();
    let rows = stmt.query_map(refs.as_slice(), |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, Option<i64>>(7)?,
            row.get::<_, String>(8)?,
        ))
    })?;

    let mut out = Vec::new();
    for row in rows {
        let (id, agent_id, agent_name, task, status, started, ended, duration, artifacts) = row?;
        out.push(Run {
            stages: stages_for(db, &id)?,
            id,
            agent_id,
            agent_name,
            task,
            status: parse_status(&status),
            started_at: started,
            ended_at: ended,
            duration_seconds: duration,
            artifacts: crate::commands::audit::parse_json_array(&artifacts),
        });
    }
    Ok(out)
}

/// The stages of one run, in order.
fn stages_for(db: &Database, run_id: &str) -> AppResult<Vec<RunStage>> {
    let mut stmt = db.conn().prepare(
        "SELECT name, status, started_at, ended_at, detail FROM run_stages
         WHERE run_id = ?1 ORDER BY ordinal",
    )?;
    let rows = stmt.query_map([run_id], |row| {
        Ok(RunStage {
            name: row.get(0)?,
            status: parse_status(&row.get::<_, String>(1)?),
            started_at: row.get(2)?,
            ended_at: row.get(3)?,
            detail: row.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Get one run with its stages.
pub fn get(db: &Database, run_id: &str) -> AppResult<Run> {
    let run_id = crate::error::validate_id("run id", run_id)?;
    list(db, None, None)?
        .into_iter()
        .find(|run| run.id == run_id)
        .ok_or(AppError::NotFound("run"))
}

/// The event timeline of one run.
pub fn events(db: &Database, run_id: &str) -> AppResult<Vec<RunEvent>> {
    let run_id = crate::error::validate_id("run id", run_id)?;
    let mut stmt = db.conn().prepare(
        "SELECT id, run_id, agent_id, kind, message, at FROM run_events
         WHERE run_id = ?1 ORDER BY at",
    )?;
    let rows = stmt.query_map([run_id], |row| {
        Ok(RunEvent {
            id: row.get(0)?,
            run_id: row.get(1)?,
            agent_id: row.get(2)?,
            kind: row.get(3)?,
            message: row.get(4)?,
            at: row.get(5)?,
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
    fn runs_are_listed_newest_first() {
        let db = prepared();
        let runs = list(&db, None, None).expect("list");
        assert_eq!(runs.len(), 10);
        let times: Vec<&str> = runs.iter().map(|r| r.started_at.as_str()).collect();
        let mut sorted = times.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        assert_eq!(times, sorted);
    }

    #[test]
    fn a_status_filter_narrows_the_result() {
        let db = prepared();
        let running = list(&db, Some("running".into()), None).expect("list");
        assert!(running.iter().all(|r| r.status == RunStatus::Running));
        assert!(!running.is_empty());
    }

    #[test]
    fn a_search_matches_the_task_and_the_agent_name() {
        let db = prepared();
        // "archaeolog" is in the agent name "code-archaeologist".
        let by_agent = list(&db, None, Some("archaeolog".into())).expect("list");
        assert!(!by_agent.is_empty());
        assert!(by_agent
            .iter()
            .all(|r| r.task.to_lowercase().contains("archaeolog")
                || r.agent_name.to_lowercase().contains("archaeolog")));
        // "module graph" is in the task text.
        let by_task = list(&db, None, Some("module graph".into())).expect("list");
        assert!(!by_task.is_empty());
        assert!(by_task
            .iter()
            .all(|r| r.task.to_lowercase().contains("module graph")
                || r.agent_name.to_lowercase().contains("module graph")));
    }

    #[test]
    fn an_unknown_run_is_not_found() {
        let db = prepared();
        assert!(get(&db, "run-999999").is_err());
    }
}
