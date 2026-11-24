//! Policy commands.

use rusqlite::params;

use crate::db::Database;
use crate::domain::Policy;
use crate::error::{AppError, AppResult};

/// List every policy.
pub fn list(db: &Database) -> AppResult<Vec<Policy>> {
    let mut stmt = db.conn().prepare(
        "SELECT id, name, scope, effect, enabled, detail FROM policies ORDER BY scope, name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Policy {
            id: row.get(0)?,
            name: row.get(1)?,
            scope: row.get(2)?,
            effect: row.get(3)?,
            enabled: row.get::<_, i64>(4)? != 0,
            detail: row.get(5)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Enable or disable a policy.
pub fn set_enabled(db: &Database, policy_id: &str, enabled: bool) -> AppResult<()> {
    let policy_id = crate::error::validate_id("policy id", policy_id)?;
    let changed = db.conn().execute(
        "UPDATE policies SET enabled = ?2 WHERE id = ?1",
        params![policy_id, i64::from(enabled)],
    )?;
    if changed == 0 {
        // Nothing matched, so there is no such policy. Reporting success
        // would show a successful toggle for a policy that is not there.
        return Err(AppError::NotFound("policy"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed;

    #[test]
    fn policies_are_listed_and_can_be_toggled() {
        let db = Database::open_memory().expect("database");
        seed::seed(db.conn(), "2026-01-01T18:00:00Z").expect("seed");
        let before = list(&db).expect("list");
        assert_eq!(before.len(), 6);
        let target = &before[0];
        set_enabled(&db, &target.id, !target.enabled).expect("toggle");
        let after = list(&db).expect("list");
        let updated = after.iter().find(|p| p.id == target.id).expect("policy");
        assert_eq!(updated.enabled, !target.enabled);
    }
}
