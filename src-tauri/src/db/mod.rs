//! SQLite persistence.
//!
//! One connection behind a mutex. v0.1 is a single-user local console, so a
//! pool would be complexity without benefit; the schema, not the driver,
//! is what keeps the data honest. Foreign keys are ON, writes that touch
//! more than one table run in a transaction, and every query is
//! parameterised.

pub mod migrations;
pub mod records;

use rusqlite::{Connection, OpenFlags};

use crate::error::{AppError, AppResult};

/// The shared database handle.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (creating if needed) the database at `path` and bring the schema
    /// up to date.
    pub fn open(path: &std::path::Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|_| AppError::Internal("could not create the data directory"))?;
        }
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )
        .map_err(|_| AppError::Internal("could not open the database"))?;
        let mut db = Database { conn };
        db.configure()?;
        db.migrate()?;
        Ok(db)
    }

    /// Open an in-memory database. Used by the tests.
    pub fn open_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|_| AppError::Internal("could not open an in-memory database"))?;
        let mut db = Database { conn };
        db.configure()?;
        db.migrate()?;
        Ok(db)
    }

    fn configure(&self) -> AppResult<()> {
        self.conn
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;",
            )
            .map_err(AppError::Database)
    }

    /// Apply every migration that has not been applied yet.
    pub fn migrate(&mut self) -> AppResult<()> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS schema_migrations (
                     version    INTEGER PRIMARY KEY,
                     name       TEXT NOT NULL,
                     applied_at TEXT NOT NULL
                 );",
            )
            .map_err(AppError::Database)?;

        for migration in migrations::MIGRATIONS {
            let applied: i64 = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1",
                    [migration.version],
                    |row| row.get(0),
                )
                .map_err(AppError::Database)?;
            if applied > 0 {
                continue;
            }
            let tx = self.conn.transaction().map_err(AppError::Database)?;
            tx.execute_batch(migration.sql)
                .map_err(AppError::Database)?;
            tx.execute(
                "INSERT INTO schema_migrations (version, name, applied_at)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    migration.version,
                    migration.name,
                    crate::util::now_rfc3339()
                ],
            )
            .map_err(AppError::Database)?;
            tx.commit().map_err(AppError::Database)?;
        }
        Ok(())
    }

    /// Direct access to the connection, for the command layer.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// The schema version currently applied.
    pub fn schema_version(&self) -> AppResult<i64> {
        self.conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .map_err(AppError::Database)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_apply_once_and_are_recorded() {
        let mut db = Database::open_memory().expect("in-memory database");
        assert_eq!(db.schema_version().expect("version"), 1);
        // Applying again must be a no-op, not an error.
        db.migrate().expect("re-running migrations is safe");
        assert_eq!(db.schema_version().expect("version"), 1);
        let count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .expect("count");
        assert_eq!(count, 1);
    }

    #[test]
    fn foreign_keys_are_enforced() {
        let db = Database::open_memory().expect("in-memory database");
        let result = db.conn().execute(
            "INSERT INTO runs (id, agent_id, task, status, started_at)
             VALUES ('run-x', 'agent-missing', 'task', 'running', '2026-01-01T00:00:00Z')",
            [],
        );
        assert!(
            result.is_err(),
            "a run referencing a missing agent must be rejected by the foreign key"
        );
    }
}
