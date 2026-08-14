//! Protocol attempts, events, and durable session DTOs.

use serde::Deserialize;
use serde::Serialize;

use super::ActionKind;
use super::ActionRequest;
use super::Identity;
use super::Issue;
use super::MessageEnvelope;
use super::OwnedValue;
use super::PayloadTag;
use super::RoleCursor;
use super::SessionPlan;
use super::ValueId;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionStatus {
    Active,
    Complete,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttemptState {
    Prepared,
    Authorized,
    DispatchedUnknown,
    Committed,
    Failed,
    AbandonedUnknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub schema: String,
    pub identity: Identity,
    pub attempt: Identity,
    pub message: Identity,
    pub evidence_reference: Identity,
    pub accepted: bool,
    pub issue: Option<Issue>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub identity: Identity,
    pub action: Identity,
    pub sequence: u64,
    pub step: Identity,
    pub kind: ActionKind,
    pub request: ActionRequest,
    pub payload_tag: PayloadTag,
    pub value: Option<ValueId>,
    pub messages: Vec<MessageEnvelope>,
    pub authorization_references: Vec<Identity>,
    pub state: AttemptState,
    pub observations: Vec<Observation>,
    pub outcome: Option<Identity>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum SessionEvent {
    Prepared {
        attempt: Attempt,
    },
    Authorized {
        attempt: Identity,
        references: Vec<Identity>,
    },
    DispatchedUnknown {
        attempt: Identity,
    },
    Observed {
        attempt: Identity,
        observation: Observation,
    },
    Committed {
        attempt: Identity,
        outcome: Identity,
    },
    Failed {
        attempt: Identity,
        outcome: Identity,
    },
    AbandonedUnknown {
        attempt: Identity,
        outcome: Identity,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionRecord {
    pub schema: String,
    pub plan: SessionPlan,
    pub version: u64,
    pub status: SessionStatus,
    pub cursors: Vec<RoleCursor>,
    pub owners: Vec<OwnedValue>,
    pub attempts: Vec<Attempt>,
    pub events: Vec<SessionEvent>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryRecovery {
    pub recovered: SessionRecord,
    pub prepared: SessionRecord,
}
