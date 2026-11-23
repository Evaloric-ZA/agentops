//! Simulator commands.

use crate::db::Database;
use crate::error::AppResult;
use crate::simulator;

/// The simulator state the frontend needs.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulatorView {
    pub running: bool,
    pub step: i64,
    pub speed: f64,
    pub last_outcome: Option<String>,
}

/// Read the simulator state.
pub fn state(db: &Database) -> AppResult<SimulatorView> {
    let state = simulator::state(db.conn())?;
    Ok(SimulatorView {
        running: state.running,
        step: state.step,
        speed: state.speed,
        last_outcome: None,
    })
}

/// Start or resume the simulator.
pub fn start(db: &Database) -> AppResult<SimulatorView> {
    simulator::start(db.conn())?;
    state(db)
}

/// Pause the simulator.
pub fn pause(db: &Database) -> AppResult<SimulatorView> {
    simulator::pause(db.conn())?;
    state(db)
}

/// Set the simulator speed.
pub fn set_speed(db: &Database, speed: f64) -> AppResult<SimulatorView> {
    simulator::set_speed(db.conn(), speed)?;
    state(db)
}

/// Advance the simulator by one step.
pub fn tick(db: &Database) -> AppResult<SimulatorView> {
    let (state, outcome) = simulator::tick(db.conn())?;
    Ok(SimulatorView {
        running: state.running,
        step: state.step,
        speed: state.speed,
        last_outcome: Some(outcome.as_str().to_string()),
    })
}

/// Reset the demo data to the original seeded state.
pub fn reset(db: &Database) -> AppResult<SimulatorView> {
    simulator::reset(db.conn())?;
    state(db)
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
    fn a_reset_restores_the_original_seeded_state() {
        let db = prepared();
        let before = crate::commands::runs::list(&db, None, None)
            .expect("runs")
            .len();
        start(&db).expect("start");
        // A run is created on the sixth tick: the phase cycle is create,
        // advance, advance, approval, fail, complete.
        for _ in 0..6 {
            tick(&db).expect("tick");
        }
        assert!(
            crate::commands::runs::list(&db, None, None)
                .expect("runs")
                .len()
                > before
        );
        reset(&db).expect("reset");
        let after = crate::commands::runs::list(&db, None, None)
            .expect("runs")
            .len();
        assert_eq!(before, after);
    }

    #[test]
    fn a_tick_while_paused_is_idle() {
        let db = prepared();
        let view = tick(&db).expect("tick");
        assert_eq!(view.last_outcome.as_deref(), Some("idle"));
        assert_eq!(view.step, 0);
    }
}
