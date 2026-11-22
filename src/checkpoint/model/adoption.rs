#![allow(
    unknown_lints,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    reason = "the adoption records keep their versioned store fields and the adoption names explicit"
)]

//! Target-store records of an adopted checkpoint.

use lattice_workflow_contract::Identity;
use serde::Deserialize;
use serde::Serialize;

use super::CompletedNode;
use super::Consumed;
use super::RestartReceipt;

/// Target-store record of one adopted checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptionRecord {
    pub schema: String,
    pub execution: Identity,
    pub receipt: RestartReceipt,
    pub producing_incarnation: u32,
    pub consumed: Consumed,
    pub deadline_unix_ms: u64,
    pub cancellation_requested: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<Identity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    /// Checkpoint-complete nodes, whose outcome objects the target checkpoint root holds; the
    /// serving exchange restores them when it starts the adopted execution.
    pub completed_nodes: Vec<CompletedNode>,
}

/// Result of one atomic adoption attempt in the target store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdoptionOutcome {
    Adopted {
        record: Box<crate::workflow_exchange::ExecutionRecord>,
        adoption: Box<AdoptionRecord>,
    },
    Existing(AdoptionRecord),
}
