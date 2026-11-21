//! Pure task-family qualification attachments over existing evaluation evidence.
//!
//! An attachment binds one complete evaluated subject (Lattice definition
//! identity, Lattice profile revision, task family, and every closed component
//! with its declared, observed, or unavailable identity) to one independent
//! evidence class: `declared_support`, `contract_conformance`,
//! `task_family_evaluation`, or `deployment_admission`. Classes are separate
//! facts, never a ladder.
//!
//! Attachments are canonical JSON-subset bytes; their identity is
//! `blake3:<hex>` over those bytes. Strings are bounded tokens or `blake3:`
//! identities, so raw prompts, hidden tests, credentials, provider bodies, and
//! host paths cannot be represented. This module has no clock, filesystem,
//! network, process, or provider authority and adds no dependency beyond
//! `blake3`. A favorable attachment never activates a revision, widens
//! authority, bypasses revocation, or replaces per-candidate acceptance.

mod codec;
mod consume;
mod data;
mod json;
mod rules;

pub use codec::{attachment_identity, decode_attachment, encode_attachment};
pub use consume::{
    classify_applicability, compare_equal_budget, component_identity, cost_per_accepted,
    framed_identity, qualify, replay_attachments, review_qualification, validate_attachment,
};
pub use data::{
    Applicability, ApplicabilityPolicy, Attachment, AttachmentDocument, ComparisonRefusal,
    CompatibilityDecision, Component, ComponentBasis, ComponentKind, CostPerAccepted, Costs,
    CoveredTime, CurrentComponent, EqualBudgetComparison, EvidenceClass, EvidenceLabel,
    EvidenceSource, ExpectedSubject, Measures, ModelDeployment, ModelFacts, Outcomes, Provenance,
    Qualification, QualificationAnswer, QualificationSubject, Reference, Rejection,
    RejectionReason, ReplayReport, Scope, VersionBasis,
};

/// Attachment schema.
pub const ATTACHMENT_SCHEMA: &str = "animus.qualification-attachment.v1";
/// Subject schema.
pub const SUBJECT_SCHEMA: &str = "animus.qualification-subject.v1";
/// Largest canonical attachment.
pub const MAX_ATTACHMENT_BYTES: usize = 16_384;
/// Domain of [`component_identity`].
pub const COMPONENT_IDENTITY_DOMAIN: &str = "animus.qualification-component.v1";
pub(crate) const FIELD_QUALIFICATION: &str = "qualification";

#[cfg(test)]
mod tests;
