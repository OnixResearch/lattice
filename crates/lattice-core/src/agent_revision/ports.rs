//! Port facts: the shared qualification-port contract (Animus #18) and the
//! preparation and cleanup observations of the runtime owner (Kiln #49).
//!
//! These are values only. The shell's injected ports produce them; the core
//! never asks for them.

// r[impl lattice.agent_revisions.ports]

/// Independent classes of qualification evidence, never a ladder.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    DeclaredSupport,
    ContractConformance,
    TaskFamilyEvaluation,
    DeploymentAdmission,
}

/// Why an attachment does not qualify the queried subject.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unqualified {
    SubjectMismatch,
    ScopeMismatch,
    Stale,
    Unusable,
    InsufficientClass,
    Conflicting,
    Malformed,
}

/// One qualification question: exact subject keys, scope and the class the
/// promotion policy requires.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    /// `blake3:` definition identity (`lattice-worker-definition-v1`).
    pub definition_identity: String,
    /// Bare 64-hex profile revision, verbatim.
    pub profile_revision: String,
    pub task_family: String,
    pub required_class: EvidenceClass,
    /// The shell's clock reading for the validator's freshness check; the
    /// core never reads or compares it.
    pub now_ms: u64,
}

/// The qualification port's answer.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Verdict {
    /// Exact subject, scope, current applicability and required class.
    Qualified(Attachment),
    NotQualified {
        reason: Unqualified,
    },
    /// No producer is installed.
    Unavailable {},
}

/// The producer's provenance label for the evidence. Recorded with the
/// admitted qualification; never a promotion condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Fixture,
    Live,
}

// r[impl lattice.agent_revisions.attachments.subject]
/// The subject keys an attachment was produced for, as the producer decoded
/// them, never copied from the question.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subject {
    /// `blake3:<hex>` definition identity.
    pub definition_identity: String,
    /// Bare 64 lowercase hex profile revision.
    pub profile_revision: String,
    pub task_family: String,
}

/// The facts a favorable answer carries.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attachment {
    /// BLAKE3 over the canonical attachment bytes, spelled `blake3:<hex>`.
    pub qualification_ref: String,
    /// The subject the attachment qualifies.
    pub subject: Subject,
    pub evidence_class: EvidenceClass,
    pub protocol_identity: String,
    pub verifier_identity: String,
    /// Producer-stated epoch milliseconds; recorded verbatim, never compared
    /// with a clock here.
    pub observed_at_ms: u64,
    /// `fixture` or `live`, as the producer labelled the evidence.
    pub provenance: Provenance,
}

impl Attachment {
    /// Canonical references and subject spellings.
    #[must_use]
    pub fn is_canonical(&self) -> bool {
        let references = [
            &self.qualification_ref,
            &self.protocol_identity,
            &self.verifier_identity,
            &self.subject.definition_identity,
        ];
        let checks = [
            references.iter().all(|value| super::is_reference(value)),
            super::is_bare_digest(&self.subject.profile_revision),
            crate::execution_binding::capability::is_name(&self.subject.task_family),
        ];
        checks.iter().all(|check| *check)
    }
}

/// What the preparation owner observed for one ticket.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Prepared {
    /// The bundle is materialized; the artifacts are retained references.
    Prepared { artifacts: Vec<String> },
    /// Preparation failed; `artifacts` lists resources still occupied.
    Failed { artifacts: Vec<String> },
    /// No outcome is known; every resource is treated as occupied.
    Unknown {},
}

/// What the preparation owner observed for one cleanup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cleanup {
    Released,
    Failed,
    Unknown,
}

/// Bounded, distinct canonical references.
pub(crate) fn is_reference_set(values: &[String]) -> bool {
    let distinct = values.iter().collect::<std::collections::BTreeSet<_>>();
    let is_canonical = values.iter().all(|value| super::is_reference(value));
    is_canonical && distinct.len() == values.len() && super::is_within(values.len(), super::MAX_REFERENCES)
}
