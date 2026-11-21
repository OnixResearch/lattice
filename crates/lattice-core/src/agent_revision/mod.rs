//! Pure lifecycle of immutable agent revisions (Lattice #25).
//!
//! A revision is the existing profile revision of an implementation-attempt
//! request, an opaque bare 64-hex value. Per agent definition this core keeps
//! separately the desired revision and its change generation, preparation
//! attempts bound to their desired generation, qualification admitted for a
//! named task-family scope, the last usable revision, the revision selected
//! for new sessions and the revision pinned to each session. Several
//! revisions coexist.
//!
//! The core reads no files, environment, clock, process or network. Every
//! transition is a function from the current lifecycle and one command to a
//! new lifecycle and an outcome, or to a refusal from a closed vocabulary
//! that leaves the lifecycle unchanged. Preparation and qualification facts
//! arrive only as injected port observations. A favorable qualification is
//! never promotion authority: promotion and rollback are explicit commands
//! that also revalidate current revocation and compare the rollout
//! generation.

// r[impl lattice.agent_revisions.core]

pub mod facts;
pub mod framing;
pub mod identity;
pub mod ops;
pub mod ports;
pub mod projection;
pub mod state;

/// Owner of qualification attachments.
pub const QUALIFICATION_OWNER: &str = "animus#18";
/// Owner of runtime preparation and cleanup.
pub const PREPARATION_OWNER: &str = "kiln#49";
/// Revisions one definition retains. A new desired revision first forgets
/// every revision nothing references, so the bound is never exhausted for
/// good.
pub const MAX_CANDIDATES: u32 = 64;
/// Live sessions one definition pins at once. A closed session is forgotten
/// once none of its checkpoints is held.
pub const MAX_SESSIONS: u32 = 1024;
/// The most recent promotion and rollback receipts kept for idempotency; the
/// oldest is forgotten first. A forgotten request can never match the
/// current rollout generation again, so it is refused, never reapplied.
pub const MAX_RECEIPTS: u32 = 256;
/// Qualifications admitted per revision and task-family scope.
pub const MAX_QUALIFICATIONS: u32 = 256;
/// Revocations of each kind recorded for targets the lifecycle does not
/// reference. Revoking a referenced target is never refused.
pub const MAX_REVOCATIONS: u32 = 1024;
/// Artifacts or holds one revision records.
pub const MAX_REFERENCES: u32 = 128;
/// Bytes of a session or request identifier.
pub const MAX_TOKEN_BYTES: u32 = 256;
const REFERENCE_PREFIX: &str = "blake3:";
const HEX_DIGITS: usize = 64;

/// Exactly 64 lowercase hexadecimal digits: the bare profile revision.
#[must_use]
pub fn is_bare_digest(value: &str) -> bool {
    value.len() == HEX_DIGITS && value.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

/// `blake3:` plus 64 lowercase hexadecimal digits.
#[must_use]
pub fn is_reference(value: &str) -> bool {
    value.strip_prefix(REFERENCE_PREFIX).is_some_and(is_bare_digest)
}

/// A session or request identifier: 1 to 256 printable ASCII bytes.
#[must_use]
pub fn is_token(value: &str) -> bool {
    let is_printable = value.bytes().all(|byte| byte.is_ascii_graphic());
    let is_bounded = u32::try_from(value.len()).is_ok_and(|bytes| bytes <= MAX_TOKEN_BYTES);
    let checks = [!value.is_empty(), is_printable, is_bounded];
    checks.iter().all(|check| *check)
}

/// Within a bound expressed as `u32`.
pub(crate) fn is_within(count: usize, bound: u32) -> bool {
    u32::try_from(count).is_ok_and(|count| count <= bound)
}

/// Why a command left the lifecycle unchanged. The vocabulary is closed.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "code", rename_all = "snake_case", deny_unknown_fields)]
pub enum Refusal {
    /// A revision, reference, token, bundle or policy is malformed.
    Malformed,
    /// The bundle or query names another definition.
    DefinitionMismatch,
    UnknownRevision,
    /// Preparation may start only for the current desired revision.
    NotDesired,
    NotPrepared,
    /// A late result bound to an older desired generation.
    PreparationSuperseded,
    PreparationFailed,
    /// Preparation outcome is unknown; resources are treated as occupied.
    PreparationUncertain,
    /// An earlier attempt must be reconciled first.
    PreparationInFlight,
    /// No attempt is bound to the ticket's generation, or no open attempt
    /// remains to reissue.
    StaleTicket,
    /// A second, different result for one attempt.
    ResultConflict,
    RevisionRevoked,
    QualificationRevoked,
    SessionRevoked,
    QualificationUnavailable {
        owner: String,
    },
    NotQualified {
        reason: crate::agent_revision::ports::Unqualified,
    },
    InsufficientClass,
    /// The named reference differs from the current answer, or is bound to
    /// another revision or task family.
    QualificationMismatch,
    AlreadySelected,
    /// Rollback targets only a revision selected before.
    NotPreviouslySelected,
    /// The expected rollout generation is not current.
    GenerationConflict,
    /// One request identifier reused for a different command.
    RequestConflict,
    NoSelection,
    /// A session may pin only the revision selected for new sessions.
    RevisionNotSelected,
    /// A session identifier is already pinned to another revision.
    SessionConflict,
    UnknownSession,
    SessionClosed,
    /// Cleanup refused: live sessions, checkpoints, recovery evidence,
    /// selection or an uncertain preparation still reference the revision.
    Retained,
    CapacityExhausted,
    GenerationExhausted,
}
