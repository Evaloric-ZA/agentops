//! The typed command surface the frontend calls.
//!
//! Every command takes validated input and returns a domain type. The
//! frontend never touches SQL, the filesystem or the simulator internals.

pub mod agents;
pub mod approvals;
pub mod audit;
pub mod dashboard;
pub mod policies;
pub mod runs;
pub mod settings;
pub mod simulator;
pub mod team;
pub mod tools;
