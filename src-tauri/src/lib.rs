//! AgentOps v0.1.
//!
//! A local, synthetic agent operations console. Nothing in this application
//! talks to the network, to a real model, or to a real agent.

pub mod commands;
pub mod commands_api;
pub mod db;
pub mod domain;
pub mod error;
pub mod seed;
pub mod simulator;
pub mod util;

use tauri::Manager;

use commands_api::AppState;
use db::Database;
use error::{AppError, AppResult};

/// Build and run the desktop application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|_| AppError::Internal("could not resolve the data directory"))?;
            let db_path = data_dir.join("agentops.db");
            let database = Database::open(&db_path)?;
            seed::ensure_seeded(database.conn())?;
            app.manage(AppState {
                db: std::sync::Mutex::new(database),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands_api::dashboard_summary,
            commands_api::dashboard_timeline,
            commands_api::list_agents,
            commands_api::get_agent,
            commands_api::agent_events,
            commands_api::list_runs,
            commands_api::get_run,
            commands_api::run_events,
            commands_api::list_approvals,
            commands_api::decide_approval,
            commands_api::list_audit,
            commands_api::list_policies,
            commands_api::set_policy_enabled,
            commands_api::list_tools,
            commands_api::list_team,
            commands_api::get_settings,
            commands_api::update_settings,
            commands_api::simulator_state,
            commands_api::simulator_start,
            commands_api::simulator_pause,
            commands_api::simulator_tick,
            commands_api::simulator_set_speed,
            commands_api::simulator_reset,
        ])
        .run(tauri::generate_context!());

    // The runtime could not start, so there is no window to report into.
    // A panic here would print a backtrace that says nothing about the
    // cause, so the failure is printed and the process leaves with a
    // failure code instead.
    if let Err(error) = result {
        eprintln!("the AgentOps application could not start: {error}");
        std::process::exit(1);
    }
}

/// The application state type, re-exported for the command module.
pub type State = commands_api::AppState;

/// Convenience alias used by the command module.
pub type AppResultOf<T> = AppResult<T>;
