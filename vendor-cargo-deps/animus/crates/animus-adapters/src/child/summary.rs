use super::ID_FIELD;
use super::MIN_TURN_COUNT;
use super::OUTCOME_CANCELLATION;
use super::OUTCOME_DENIED_TOOL;
use super::OUTCOME_FAILURE;
use super::OUTCOME_KIND_FIELD;
use super::OUTCOME_MALFORMED_SUMMARY;
use super::OUTCOME_PARTIAL_PARALLEL_SUCCESS;
use super::OUTCOME_SUCCESS;
use super::OUTCOME_TIMEOUT;
use super::OUTCOME_WRITE_CONFLICT;
use super::SubagentError;
use super::Text;
use super::optional_text;
use super::parse_value;
use super::required_text;

const EVENT_KIND_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_event_kind");
const EVENT_PARENT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_parent_id");
const EVENT_STATUS_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_event_status");
const EVENT_EVIDENCE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_event_evidence");

const EVENT_START: &str = "start";
const EVENT_STOP: &str = "stop";
const EVENT_CONTINUATION: &str = "continuation";
const EVENT_FAILURE: &str = "failure";
const EVENT_AGGREGATION: &str = "aggregation";

/// Child outcome kind used for aggregation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentOutcomeKind {
    /// Child completed successfully.
    Success,
    /// Child failed.
    Failure,
    /// Child timed out.
    Timeout,
    /// Child was cancelled.
    Cancellation,
    /// Child returned a malformed summary.
    MalformedSummary,
    /// Child requested a denied tool.
    DeniedTool,
    /// Aggregate-only partial parallel success status.
    PartialParallelSuccess,
    /// Child or aggregate encountered a write conflict.
    WriteConflict,
}

impl SubagentOutcomeKind {
    /// Parses a stable outcome kind label.
    ///
    /// # Errors
    ///
    /// Returns [`SubagentError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, SubagentError> {
        parse_value(OUTCOME_KIND_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            OUTCOME_SUCCESS => Some(Self::Success),
            OUTCOME_FAILURE => Some(Self::Failure),
            OUTCOME_TIMEOUT => Some(Self::Timeout),
            OUTCOME_CANCELLATION => Some(Self::Cancellation),
            OUTCOME_MALFORMED_SUMMARY => Some(Self::MalformedSummary),
            OUTCOME_DENIED_TOOL => Some(Self::DeniedTool),
            OUTCOME_PARTIAL_PARALLEL_SUCCESS => Some(Self::PartialParallelSuccess),
            OUTCOME_WRITE_CONFLICT => Some(Self::WriteConflict),
            _ => None,
        }
    }
}

/// Untrusted child outcome fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubagentOutcomeDraft<'a> {
    /// Child run identity.
    pub child_id: &'a str,
    /// Outcome kind label.
    pub kind: &'a str,
    /// Whether the child summary passed schema validation.
    pub summary_valid: bool,
    /// Whether the child attempted or encountered a parallel write conflict.
    pub write_conflict: bool,
}

/// Aggregated child outcome report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentAggregateReport {
    status: SubagentOutcomeKind,
    success_count: u32,
    failure_count: u32,
    write_conflict: bool,
}

impl SubagentAggregateReport {
    /// Returns aggregate status.
    #[must_use]
    pub const fn status(&self) -> SubagentOutcomeKind {
        self.status
    }

    /// Returns successful child count.
    #[must_use]
    pub const fn success_count(&self) -> u32 {
        self.success_count
    }

    /// Returns failed child count.
    #[must_use]
    pub const fn failure_count(&self) -> u32 {
        self.failure_count
    }

    /// Returns whether a write conflict was observed.
    #[must_use]
    pub const fn write_conflict(&self) -> bool {
        self.write_conflict
    }
}

/// Aggregates subagent outcomes without treating failures as success.
///
/// # Errors
///
/// Returns [`SubagentError`] when the outcome list is empty or fields are malformed.
///
/// # Panics
///
/// Panics if the non-empty outcome validation above is weakened.
pub fn aggregate_subagent_outcomes(
    drafts: &[SubagentOutcomeDraft<'_>],
) -> Result<SubagentAggregateReport, SubagentError> {
    if drafts.is_empty() {
        return Err(SubagentError::EmptyCollection {
            field: ID_FIELD.as_str(),
        });
    }
    assert!(!drafts.is_empty(), "outcome list was checked above");
    assert!(!drafts.is_empty(), "outcome slice has a first item");
    let mut success_count = 0_u32;
    let mut failure_count = 0_u32;
    let mut has_write_conflict = false;
    for draft in drafts {
        required_text(ID_FIELD, draft.child_id)?;
        let kind = SubagentOutcomeKind::parse(draft.kind)?;
        if kind == SubagentOutcomeKind::Success && draft.summary_valid && !draft.write_conflict {
            success_count = success_count.saturating_add(MIN_TURN_COUNT);
        } else {
            failure_count = failure_count.saturating_add(MIN_TURN_COUNT);
        }
        has_write_conflict =
            has_write_conflict || draft.write_conflict || matches!(kind, SubagentOutcomeKind::WriteConflict);
    }
    let status = classify_aggregate(AggregateCounts {
        success_count,
        failure_count,
        has_write_conflict,
    });
    Ok(SubagentAggregateReport {
        status,
        success_count,
        failure_count,
        write_conflict: has_write_conflict,
    })
}

#[derive(Clone, Copy)]
struct AggregateCounts {
    success_count: u32,
    failure_count: u32,
    has_write_conflict: bool,
}

fn classify_aggregate(counts: AggregateCounts) -> SubagentOutcomeKind {
    if counts.has_write_conflict {
        return SubagentOutcomeKind::WriteConflict;
    }
    if counts.success_count > 0 && counts.failure_count > 0 {
        return SubagentOutcomeKind::PartialParallelSuccess;
    }
    if counts.success_count > 0 {
        return SubagentOutcomeKind::Success;
    }
    SubagentOutcomeKind::Failure
}

/// Borrowed subagent runtime event fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubagentEventDraft<'a> {
    /// Child run identity.
    pub child_id: &'a str,
    /// Parent run identity or correlation label.
    pub parent_id: &'a str,
    /// Stable event kind label.
    pub kind: &'a str,
    /// Bounded status label.
    pub status: &'a str,
    /// Optional bounded evidence reference.
    pub evidence_ref: Option<&'a str>,
}

/// Runtime report event for subagent lifecycle and aggregation evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentEvent {
    child_id: Text,
    parent_id: Text,
    kind: SubagentEventKind,
    status: Text,
    evidence_ref: Option<Text>,
}

impl SubagentEvent {
    /// Validates subagent event metadata without importing raw child logs.
    ///
    /// # Errors
    ///
    /// Returns [`SubagentError`] when required fields are empty or the event
    /// kind is unsupported.
    pub fn validate(draft: &SubagentEventDraft<'_>) -> Result<Self, SubagentError> {
        let child_id = required_text(ID_FIELD, draft.child_id)?;
        let parent_id = required_text(EVENT_PARENT_FIELD, draft.parent_id)?;
        let kind = parse_value(EVENT_KIND_FIELD, draft.kind, SubagentEventKind::parse)?;
        let status = required_text(EVENT_STATUS_FIELD, draft.status)?;
        let evidence_ref = optional_text(EVENT_EVIDENCE_FIELD, draft.evidence_ref)?;
        Ok(Self {
            child_id,
            parent_id,
            kind,
            status,
            evidence_ref,
        })
    }

    /// Returns the child identity.
    #[must_use]
    pub fn child_id(&self) -> &str {
        self.child_id.as_str()
    }

    /// Returns the parent identity.
    #[must_use]
    pub fn parent_id(&self) -> &str {
        self.parent_id.as_str()
    }

    /// Returns the event kind.
    #[must_use]
    pub const fn kind(&self) -> SubagentEventKind {
        self.kind
    }

    /// Returns the bounded status label.
    #[must_use]
    pub fn status(&self) -> &str {
        self.status.as_str()
    }

    /// Returns the optional bounded evidence reference.
    #[must_use]
    pub fn evidence_ref(&self) -> Option<&str> {
        self.evidence_ref.as_deref()
    }
}

/// Subagent runtime event kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentEventKind {
    /// Child run started.
    Start,
    /// Child run stopped successfully or intentionally.
    Stop,
    /// Child requested or entered a continuation.
    Continuation,
    /// Child run failed, was cancelled, or timed out.
    Failure,
    /// Parent aggregated child outcomes.
    Aggregation,
}

impl SubagentEventKind {
    fn parse(value: &str) -> Option<Self> {
        match value {
            EVENT_START => Some(Self::Start),
            EVENT_STOP => Some(Self::Stop),
            EVENT_CONTINUATION => Some(Self::Continuation),
            EVENT_FAILURE => Some(Self::Failure),
            EVENT_AGGREGATION => Some(Self::Aggregation),
            _unsupported => None,
        }
    }

    /// Returns the stable event label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => EVENT_START,
            Self::Stop => EVENT_STOP,
            Self::Continuation => EVENT_CONTINUATION,
            Self::Failure => EVENT_FAILURE,
            Self::Aggregation => EVENT_AGGREGATION,
        }
    }
}
