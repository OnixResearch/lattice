pub use crate::status::evidence::SAFE_EVIDENCE_HASH_HEX_CHARS;

/// Optional native host session reference.
pub type AgentSessionReference = crate::status::identity::Reference;
/// Borrowed session-reference input.
pub type AgentSessionReferenceDraft<'a> = crate::status::identity::ReferenceDraft<'a>;
/// Supported session reference kind.
pub type AgentSessionReferenceKind = crate::status::identity::Kind;
/// Validated semantic status observation supplied by a host adapter.
pub type AgentStatusObservation = crate::status::report::Observation;
/// Borrowed status observation input.
pub type AgentStatusObservationDraft<'a> = crate::status::report::ObservationDraft<'a>;
/// Observation authority class supplied by a host integration.
pub type AgentStatusObservationKind = crate::status::lifecycle::ObservationKind;
/// Presentation-only status metadata.
pub type AgentStatusPresentation = crate::status::view::Presentation;
/// Borrowed presentation metadata input.
pub type AgentStatusPresentationDraft<'a> = crate::status::view::PresentationDraft<'a>;
/// Batch returned by a host status-observation adapter.
pub type HostStatusObservationBatch = crate::status::report::HostObservationBatch;
/// Request sent to a host status-observation adapter.
pub type HostStatusRequest = crate::status::report::HostRequest;
/// Semantic lifecycle state shared by runtime hosts.
pub type SemanticAgentState = crate::status::lifecycle::SemanticAgent;
/// Logical monotonic sequence used for deterministic freshness checks.
pub type StatusSequence = crate::status::lifecycle::Sequence;
/// Logical TTL for an observation.
pub type StatusTtl = crate::status::lifecycle::Ttl;
