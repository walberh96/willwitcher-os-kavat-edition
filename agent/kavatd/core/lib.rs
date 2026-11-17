//! kavatd: the daemon between the agent and the system.
//!
//! The agent never gets a shell. It gets tools: each has a name, typed parameters, a
//! permission level and one exact command line, declared in a TOML file. A call is checked
//! against the tool, let through or held for a confirmation by its level, run with a
//! timeout and written to the log.
//!
//! This is the skeleton of phase 4: the registry, the gate, the log and the socket are
//! here; the model, the search model and the skills are marked stubs in `model.rs`.

pub mod daemon;
pub mod log;
pub mod model;
pub mod registry;
pub mod run;
pub mod serve;
