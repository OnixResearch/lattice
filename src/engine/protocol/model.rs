//! Lattice-owned projected role-protocol domain types.
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the protocol data contract uses local serialization and submodule vocabulary"
)]
#![allow(
    path_segment_repetition,
    reason = "public protocol DTO names keep domain meaning at external call sites"
)]

mod artifact;
mod session;

pub use artifact::*;
use serde::Deserialize;
use serde::Serialize;
pub use session::*;

pub const SESSION_SCHEMA: &str = "lattice.role-protocol-session.v1";
pub const ENVELOPE_SCHEMA: &str = "lattice.role-protocol-envelope.v1";
pub const OBSERVATION_SCHEMA: &str = "lattice.role-protocol-observation.v1";
pub const EVIDENCE_SCHEMA: &str = "lattice.role-protocol-evidence.v1";
pub const FROZEN_CHOREGRAPH_REVISION: &str = "0b2000edaf86d5a7a8601246afbc0d526f4a2a07";
pub const FROZEN_ADAPTER_IDENTITY: &str = "blake3:0afe63826d52e531da3924bb9a07945b93088a5636d2f35ae4141f297ea3c20c";
pub const FROZEN_TRELLIS_REVISION: &str = "d65974b14978e1a44b0f5067beb9ef6d83317dc8";
pub const CLAIM_BOUNDARY: &str = concat!(
    "local artifact admission, deterministic cursor decisions, durable attempt linkage, and bounded observations; ",
    "not transport confidentiality, endpoint identity truth, remote durability, external-role correctness, ",
    "exactly-once delivery, global progress, fault deadlock freedom, production readiness, or release eligibility"
);

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Identity(String);

impl Identity {
    pub fn new(reference: impl Into<String>) -> Result<Self, Error> {
        let reference = reference.into();
        if !super::identity::valid_reference(&reference) {
            return Err(Error::MalformedIdentity);
        }
        Ok(Self(reference))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RoleId(pub u32);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct LabelId(pub u32);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct PayloadTag(pub u32);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ValueId(pub u32);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Branch {
    pub label: LabelId,
    pub body: RoleProgram,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RoleProgram {
    End,
    Send {
        peer: RoleId,
        label: LabelId,
        payload_tag: PayloadTag,
        next: Box<RoleProgram>,
    },
    Receive {
        peer: RoleId,
        label: LabelId,
        payload_tag: PayloadTag,
        next: Box<RoleProgram>,
    },
    SendOwned {
        peer: RoleId,
        label: LabelId,
        payload_tag: PayloadTag,
        value: ValueId,
        next: Box<RoleProgram>,
    },
    ReceiveOwned {
        peer: RoleId,
        label: LabelId,
        payload_tag: PayloadTag,
        value: ValueId,
        next: Box<RoleProgram>,
    },
    InternalChoice {
        branches: Vec<Branch>,
    },
    Offer {
        from: RoleId,
        branches: Vec<Branch>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedValue {
    pub value: ValueId,
    pub payload_tag: PayloadTag,
    pub owner: RoleId,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoleCursor {
    pub role: RoleId,
    pub local_artifact: Identity,
    pub protocol: RoleProgram,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MessageEnvelope {
    pub schema: String,
    pub identity: Identity,
    pub attempt: Identity,
    pub protocol: Identity,
    pub complete_local_set: Identity,
    pub session: Identity,
    pub source_role: RoleId,
    pub destination_role: RoleId,
    pub source_instance: Identity,
    pub destination_instance: Identity,
    pub source_local_artifact: Identity,
    pub destination_local_artifact: Identity,
    pub step: Identity,
    pub label: LabelId,
    pub payload_tag: PayloadTag,
    pub value: Option<ValueId>,
    pub payload_reference: Identity,
    pub schema_set: Identity,
    pub adapter_cohort: Identity,
    pub authority_reference: Identity,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionKind {
    Transfer,
    OwnedTransfer,
    Choice,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActionRequest {
    pub source: RoleId,
    pub label: LabelId,
    pub payload_reference: Identity,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedMessage {
    pub destination: RoleId,
    pub draft_identity: Identity,
    pub adapter_cohort: Identity,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedAction {
    pub identity: Identity,
    pub sequence: u64,
    pub step: Identity,
    pub kind: ActionKind,
    pub request: ActionRequest,
    pub payload_tag: PayloadTag,
    pub value: Option<ValueId>,
    pub messages: Vec<PlannedMessage>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Issue {
    UnsupportedCohort,
    IdentityMismatch,
    IncompleteArtifacts,
    InvalidPlacement,
    MissingRole,
    UnexpectedAction,
    WrongPeer,
    WrongStep,
    WrongLabel,
    WrongPayload,
    WrongValue,
    FormerOwner,
    DuplicateMessage,
    ReorderedMessage,
    AttemptInFlight,
    AttemptNotUnknown,
    AcknowledgementRequired,
    ObservationMismatch,
    AuthorityDenied,
    PersistenceFailed,
    AdapterFailed,
    UnsafeDiagnostic,
}

impl Issue {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnsupportedCohort => "protocol.unsupported-cohort",
            Self::IdentityMismatch => "protocol.identity-mismatch",
            Self::IncompleteArtifacts => "protocol.incomplete-artifacts",
            Self::InvalidPlacement => "protocol.invalid-placement",
            Self::MissingRole => "protocol.missing-role",
            Self::UnexpectedAction => "protocol.unexpected-action",
            Self::WrongPeer => "protocol.wrong-peer",
            Self::WrongStep => "protocol.wrong-step",
            Self::WrongLabel => "protocol.wrong-label",
            Self::WrongPayload => "protocol.wrong-payload",
            Self::WrongValue => "protocol.wrong-value",
            Self::FormerOwner => "protocol.former-owner",
            Self::DuplicateMessage => "protocol.duplicate-message",
            Self::ReorderedMessage => "protocol.reordered-message",
            Self::AttemptInFlight => "protocol.attempt-in-flight",
            Self::AttemptNotUnknown => "protocol.attempt-not-unknown",
            Self::AcknowledgementRequired => "protocol.acknowledgement-required",
            Self::ObservationMismatch => "protocol.observation-mismatch",
            Self::AuthorityDenied => "protocol.authority-denied",
            Self::PersistenceFailed => "protocol.persistence-failed",
            Self::AdapterFailed => "protocol.adapter-failed",
            Self::UnsafeDiagnostic => "protocol.unsafe-diagnostic",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    MalformedIdentity,
    Serialization,
    Issue(Issue),
}

impl From<Issue> for Error {
    fn from(issue: Issue) -> Self {
        Self::Issue(issue)
    }
}
