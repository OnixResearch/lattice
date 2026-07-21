//! Pure redaction-first session-tree projection, rendering, and receipts.
//!
//! Callers supply already-loaded validated metadata. This module performs no
//! filesystem, environment, clock, network, process, terminal, or persistence
//! effects and never accepts raw transcript or provider bodies.

mod core;
mod data;
#[path = "render.rs"]
mod encoding;
#[path = "receipt.rs"]
mod evidence;

pub use core::{
    project_session_tree_export, session_tree_export_policy_content_hash,
    validate_session_tree_export_policy,
};
pub use data::{
    SESSION_TREE_EXPORT_ARTIFACT_HASH_DOMAIN, SESSION_TREE_EXPORT_POLICY_HASH_DOMAIN,
    SESSION_TREE_EXPORT_POLICY_SCHEMA, SESSION_TREE_EXPORT_PROJECTION_HASH_DOMAIN,
    SESSION_TREE_EXPORT_RECEIPT_HASH_DOMAIN, SESSION_TREE_EXPORT_RECEIPT_SCHEMA,
    SESSION_TREE_EXPORT_SOURCE_HASH_DOMAIN, SessionTreeExportArtifact, SessionTreeExportEntry,
    SessionTreeExportEntryInput, SessionTreeExportEntryKind, SessionTreeExportError,
    SessionTreeExportFormat, SessionTreeExportIssue, SessionTreeExportPolicy,
    SessionTreeExportPolicyInput, SessionTreeExportProjection, SessionTreeExportReceipt,
    SessionTreeExportReceiptInput, SessionTreeExportSensitiveDraft, SessionTreeExportSourceInput,
    SessionTreeExportWriteEvidence,
};
pub use encoding::render_session_tree_export;
pub use evidence::project_session_tree_export_receipt;

#[cfg(test)]
mod tests;
