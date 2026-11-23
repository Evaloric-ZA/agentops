//! The Tauri command surface.
//!
//! These are the only functions the frontend can call. Each one validates
//! its input in Rust and returns a domain type or a structured error. The
//! frontend never receives a raw SQLite error or a stack trace.
//!
//! `rusqlite::Connection` is `Send` but not `Sync`, so the shared state
//! holds it behind a mutex. The lock is taken once per command and held for
//! the whole command, which is what keeps a read and the write that follows
//! it atomic with respect to other commands.

use std::sync::{Mutex, MutexGuard};

use tauri::State;

use crate::commands;
use crate::db::Database;
use crate::domain::{
    Agent, AppSettings, Approval, AuditEvent, DashboardSummary, Policy, Run, RunEvent, TeamMember,
    TimelineEntry, Tool,
};
use crate::error::AppResult;

/// The application state held by Tauri.
pub struct AppState {
    pub db: Mutex<Database>,
}

type Shared<'a> = State<'a, AppState>;

/// Take the database lock, recovering from a poisoned lock rather than
/// panicking: a previous command that panicked must not take the whole
/// application down with it.
fn lock(state: &AppState) -> MutexGuard<'_, Database> {
    state
        .db
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

// --- dashboard ------------------------------------------------------------

#[tauri::command]
pub fn dashboard_summary(state: Shared<'_>) -> AppResult<DashboardSummary> {
    commands::dashboard::summary(&lock(&state))
}

#[tauri::command]
pub fn dashboard_timeline(state: Shared<'_>) -> AppResult<Vec<TimelineEntry>> {
    commands::dashboard::timeline(&lock(&state))
}

// --- agents ---------------------------------------------------------------

#[tauri::command]
pub fn list_agents(state: Shared<'_>) -> AppResult<Vec<Agent>> {
    commands::agents::list(&lock(&state))
}

#[tauri::command]
pub fn get_agent(state: Shared<'_>, agent_id: String) -> AppResult<Agent> {
    commands::agents::get(&lock(&state), &agent_id)
}

#[tauri::command]
pub fn agent_events(state: Shared<'_>, agent_id: String) -> AppResult<Vec<RunEvent>> {
    commands::agents::events(&lock(&state), &agent_id)
}

// --- runs -----------------------------------------------------------------

#[tauri::command]
pub fn list_runs(
    state: Shared<'_>,
    status: Option<String>,
    search: Option<String>,
) -> AppResult<Vec<Run>> {
    commands::runs::list(&lock(&state), status, search)
}

#[tauri::command]
pub fn get_run(state: Shared<'_>, run_id: String) -> AppResult<Run> {
    commands::runs::get(&lock(&state), &run_id)
}

#[tauri::command]
pub fn run_events(state: Shared<'_>, run_id: String) -> AppResult<Vec<RunEvent>> {
    commands::runs::events(&lock(&state), &run_id)
}

// --- approvals ------------------------------------------------------------

#[tauri::command]
pub fn list_approvals(state: Shared<'_>, state_filter: Option<String>) -> AppResult<Vec<Approval>> {
    commands::approvals::list(&lock(&state), state_filter)
}

#[tauri::command]
pub fn decide_approval(
    state: Shared<'_>,
    approval_id: String,
    outcome: String,
    comment: Option<String>,
) -> AppResult<Approval> {
    commands::approvals::decide(&lock(&state), &approval_id, &outcome, comment)
}

// --- governance -----------------------------------------------------------

#[tauri::command]
pub fn list_audit(state: Shared<'_>, limit: Option<i64>) -> AppResult<Vec<AuditEvent>> {
    commands::audit::list(&lock(&state), limit)
}

#[tauri::command]
pub fn list_policies(state: Shared<'_>) -> AppResult<Vec<Policy>> {
    commands::policies::list(&lock(&state))
}

#[tauri::command]
pub fn set_policy_enabled(state: Shared<'_>, policy_id: String, enabled: bool) -> AppResult<()> {
    commands::policies::set_enabled(&lock(&state), &policy_id, enabled)
}

#[tauri::command]
pub fn list_tools(state: Shared<'_>) -> AppResult<Vec<Tool>> {
    commands::tools::list(&lock(&state))
}

#[tauri::command]
pub fn list_team(state: Shared<'_>) -> AppResult<Vec<TeamMember>> {
    commands::team::list(&lock(&state))
}

// --- settings and simulator ----------------------------------------------

#[tauri::command]
pub fn get_settings(state: Shared<'_>) -> AppResult<AppSettings> {
    commands::settings::get(&lock(&state))
}

#[tauri::command]
pub fn update_settings(state: Shared<'_>, settings: AppSettings) -> AppResult<AppSettings> {
    commands::settings::update(&lock(&state), &settings)
}

#[tauri::command]
pub fn simulator_state(state: Shared<'_>) -> AppResult<commands::simulator::SimulatorView> {
    commands::simulator::state(&lock(&state))
}

#[tauri::command]
pub fn simulator_start(state: Shared<'_>) -> AppResult<commands::simulator::SimulatorView> {
    commands::simulator::start(&lock(&state))
}

#[tauri::command]
pub fn simulator_pause(state: Shared<'_>) -> AppResult<commands::simulator::SimulatorView> {
    commands::simulator::pause(&lock(&state))
}

#[tauri::command]
pub fn simulator_tick(state: Shared<'_>) -> AppResult<commands::simulator::SimulatorView> {
    commands::simulator::tick(&lock(&state))
}

#[tauri::command]
pub fn simulator_set_speed(
    state: Shared<'_>,
    speed: f64,
) -> AppResult<commands::simulator::SimulatorView> {
    commands::simulator::set_speed(&lock(&state), speed)
}

#[tauri::command]
pub fn simulator_reset(state: Shared<'_>) -> AppResult<commands::simulator::SimulatorView> {
    commands::simulator::reset(&lock(&state))
}
