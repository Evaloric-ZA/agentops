//! Team commands.

use crate::db::Database;
use crate::domain::TeamMember;
use crate::error::AppResult;

/// List the seeded team members.
pub fn list(db: &Database) -> AppResult<Vec<TeamMember>> {
    let mut stmt = db
        .conn()
        .prepare("SELECT id, name, role, on_shift, agent_ids FROM team_members ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(TeamMember {
            id: row.get(0)?,
            name: row.get(1)?,
            role: row.get(2)?,
            on_shift: row.get::<_, i64>(3)? != 0,
            agent_ids: crate::commands::audit::parse_json_array(&row.get::<_, String>(4)?),
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}
