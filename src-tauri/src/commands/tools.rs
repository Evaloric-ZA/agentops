//! Tool registry commands.

use crate::db::Database;
use crate::domain::Tool;
use crate::error::AppResult;

/// List every registered tool.
pub fn list(db: &Database) -> AppResult<Vec<Tool>> {
    let mut stmt = db.conn().prepare(
        "SELECT id, name, kind, status, latency_ms, success_rate, calls_last_hour
         FROM tools ORDER BY name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Tool {
            id: row.get(0)?,
            name: row.get(1)?,
            kind: row.get(2)?,
            status: row.get(3)?,
            latency_ms: row.get(4)?,
            success_rate: row.get(5)?,
            calls_last_hour: row.get(6)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}
