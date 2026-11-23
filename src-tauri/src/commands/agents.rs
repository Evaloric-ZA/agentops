//! Agent registry commands.

use rusqlite::params;

use crate::db::Database;
use crate::domain::{Agent, AgentCapability, AgentStatus, TrustLevel};
use crate::error::{AppError, AppResult};

fn parse_status(value: &str) -> AgentStatus {
    match value {
        "working" => AgentStatus::Working,
        "approval" => AgentStatus::Approval,
        "failed" => AgentStatus::Failed,
        "offline" => AgentStatus::Offline,
        _ => AgentStatus::Idle,
    }
}

fn parse_trust(value: &str) -> TrustLevel {
    match value {
        "trusted" => TrustLevel::Trusted,
        "untrusted" => TrustLevel::Untrusted,
        _ => TrustLevel::Limited,
    }
}

/// List every agent, with its capabilities.
pub fn list(db: &Database) -> AppResult<Vec<Agent>> {
    let mut stmt = db.conn().prepare(
        "SELECT id, display_name, adapter, host_class, status, trust_level,
                current_run_id, last_heartbeat
         FROM agents ORDER BY display_name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, String>(7)?,
        ))
    })?;

    let mut agents = Vec::new();
    for row in rows {
        let (id, display_name, adapter, host_class, status, trust, current, heartbeat) = row?;
        agents.push(Agent {
            capabilities: capabilities_for(db, &id)?,
            id,
            display_name,
            adapter,
            host_class,
            status: parse_status(&status),
            trust_level: parse_trust(&trust),
            current_run_id: current,
            last_heartbeat: heartbeat,
        });
    }
    Ok(agents)
}

/// Get one agent by id.
pub fn get(db: &Database, id: &str) -> AppResult<Agent> {
    let id = crate::error::validate_id("agent id", id)?;
    list(db)?
        .into_iter()
        .find(|agent| agent.id == id)
        .ok_or(AppError::NotFound("agent"))
}

/// The capabilities of one agent.
fn capabilities_for(db: &Database, agent_id: &str) -> AppResult<Vec<AgentCapability>> {
    let mut stmt = db.conn().prepare(
        "SELECT name, description, granted FROM agent_capabilities
         WHERE agent_id = ?1 ORDER BY name",
    )?;
    let rows = stmt.query_map([agent_id], |row| {
        Ok(AgentCapability {
            name: row.get(0)?,
            description: row.get(1)?,
            granted: row.get::<_, i64>(2)? != 0,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// The event history of one agent.
pub fn events(db: &Database, agent_id: &str) -> AppResult<Vec<crate::domain::RunEvent>> {
    let agent_id = crate::error::validate_id("agent id", agent_id)?;
    let mut stmt = db.conn().prepare(
        "SELECT id, run_id, agent_id, kind, message, at FROM run_events
         WHERE agent_id = ?1 ORDER BY at DESC LIMIT 100",
    )?;
    let rows = stmt.query_map([agent_id], |row| {
        Ok(crate::domain::RunEvent {
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

/// Set an agent's status. Used by the simulator and by the local controls.
pub fn set_status(db: &Database, agent_id: &str, status: &str) -> AppResult<()> {
    let agent_id = crate::error::validate_id("agent id", agent_id)?;
    let allowed = ["working", "idle", "approval", "failed", "offline"];
    if !allowed.contains(&status) {
        return Err(AppError::InvalidInput(format!(
            "unknown agent status: {status}"
        )));
    }
    db.conn().execute(
        "UPDATE agents SET status = ?2 WHERE id = ?1",
        params![agent_id, status],
    )?;
    Ok(())
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
    fn the_registry_lists_every_seeded_agent() {
        let db = prepared();
        let agents = list(&db).expect("list");
        assert_eq!(agents.len(), 10);
        // Sorted by display name.
        let names: Vec<&str> = agents.iter().map(|a| a.display_name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
    }

    #[test]
    fn a_missing_agent_is_not_found() {
        let db = prepared();
        assert!(get(&db, "agent-nope").is_err());
    }

    #[test]
    fn a_malformed_agent_id_is_rejected_before_it_reaches_sql() {
        let db = prepared();
        assert!(get(&db, "agent; DROP TABLE agents").is_err());
    }

    #[test]
    fn an_unknown_status_is_rejected() {
        let db = prepared();
        assert!(set_status(&db, "test-engineer", "nonsense").is_err());
    }
}
