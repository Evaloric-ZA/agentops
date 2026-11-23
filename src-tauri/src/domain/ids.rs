//! Deterministic identifier helpers.
//!
//! v0.1 has no randomness requirement, so ids are derived from a counter and
//! a prefix. That keeps seed data and simulator output reproducible across
//! runs and machines.

use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(1);

/// Next id for a prefix, e.g. `run-000001`.
pub fn next_id(prefix: &str) -> String {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{n:06}")
}

/// Reset the counter. Used by the simulator reset so a reset demo produces
/// the same identifiers it produced the first time.
pub fn reset_counter() {
    COUNTER.store(1, Ordering::Relaxed);
}

/// Ensure the counter is past every id already in the database, so a
/// freshly-opened database never re-issues an existing id.
pub fn observe_counter(value: u64) {
    COUNTER.fetch_max(value + 1, Ordering::Relaxed);
}

/// Parse the numeric suffix of an id like `run-000042` into `42`.
pub fn parse_suffix(id: &str) -> Option<u64> {
    let tail = id.rsplit('-').next()?;
    tail.parse::<u64>().ok()
}
