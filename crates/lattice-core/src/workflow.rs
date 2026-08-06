//! Host-capability-free workflow metadata decisions.
//!
//! This module is part of the compiler-enforced Lattice core: it owns the
//! workflow metadata DTO and deterministic constructors that accept explicit
//! temporal facts. It performs no store, clock, filesystem, process, HTTP,
//! async-runtime, or UI work. Clock-reading conveniences belong to the shell.

use serde::Deserialize;
use serde::Serialize;

/// Metadata about a workflow (stored alongside the graph).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMeta {
    pub name: String,
    pub description: String,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
}

impl WorkflowMeta {
    /// Shell convenience constructor that reads the wall clock and delegates
    /// to the deterministic core constructor.
    pub fn new(name: impl Into<String>) -> Self {
        Self::new_at(name, chrono::Utc::now())
    }

    /// Deterministic constructor: created and modified derive only from a
    /// supplied temporal fact, never from an ambient clock.
    pub fn new_at(name: impl Into<String>, now: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            created: now,
            modified: now,
        }
    }
}

/// Error for stale or contradictory temporal observations supplied to core
/// constructors and transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalError {
    /// The supplied timestamp precedes the workflow creation time and is
    /// contradictory.
    StaleModified {
        created: chrono::DateTime<chrono::Utc>,
        supplied: chrono::DateTime<chrono::Utc>,
    },
}

#[cfg(test)]
mod tests {
    use super::WorkflowMeta;

    fn utc(rfc3339: &str) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339(rfc3339).expect("valid rfc3339").with_timezone(&chrono::Utc)
    }

    #[test]
    fn new_at_derives_created_and_modified_from_supplied_time() {
        let fixed = utc("2026-01-02T03:04:05Z");
        let meta = WorkflowMeta::new_at("timed", fixed);
        assert_eq!(meta.created, fixed);
        assert_eq!(meta.modified, fixed);
    }
}
