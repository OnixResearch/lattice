//! Reconnect and reconcile rule table `lattice.workflow-exchange-recovery.v1`.
//!
//! A pure decision table: given the command that was in flight, the observed
//! client outcome, and gateway evidence about the endpoint incarnation, it names
//! the only admissible next step. It never retries by itself and never turns an
//! unknown outcome into success or failure. Consumers implement the same table
//! from `fixtures/exchange-recovery-fixtures-v1.json`.

#![allow(
    unknown_lints,
    non_trait_imports,
    fragile_exhaustive_enum_match,
    reason = "the closed rule table matches every command and outcome class explicitly so a new class cannot fall through"
)]

use serde::Deserialize;
use serde::Serialize;

use crate::ClientFailure;
use crate::ExchangeCommand;
use crate::ExchangeResponse;
use crate::ExchangeResult;
use crate::RejectionCode;

pub const EXCHANGE_RECOVERY_SCHEMA: &str = "lattice.workflow-exchange-recovery.v1";

/// Every closed rejection code, in wire order, for exhaustive rule-table fixtures.
pub const REJECTION_CODES: [RejectionCode; 10] = [
    RejectionCode::InvalidSchema,
    RejectionCode::InvalidIdentity,
    RejectionCode::InvalidContract,
    RejectionCode::InvalidOperationResult,
    RejectionCode::FrameTooLarge,
    RejectionCode::LogTooLarge,
    RejectionCode::PeerDenied,
    RejectionCode::HandlerUnavailable,
    RejectionCode::NotFound,
    RejectionCode::EncodingFailure,
];

/// The exchange command that was in flight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandKind {
    Dispatch,
    RecoverDispatch,
    Observe,
    Control,
    ReadLog,
    Reconcile,
}

impl CommandKind {
    pub const ALL: [Self; 6] = [
        Self::Dispatch,
        Self::RecoverDispatch,
        Self::Observe,
        Self::Control,
        Self::ReadLog,
        Self::Reconcile,
    ];

    #[must_use]
    pub const fn of(command: &ExchangeCommand) -> Self {
        match command {
            ExchangeCommand::Dispatch { .. } => Self::Dispatch,
            ExchangeCommand::RecoverDispatch { .. } => Self::RecoverDispatch,
            ExchangeCommand::Observe { .. } => Self::Observe,
            ExchangeCommand::Control { .. } => Self::Control,
            ExchangeCommand::ReadLog { .. } => Self::ReadLog,
            ExchangeCommand::Reconcile { .. } => Self::Reconcile,
        }
    }
}

/// Serializable mirror of [`ClientFailure`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    InvalidProfile,
    InvalidRequest,
    ConnectFailed,
    NotAccepted,
    UnknownAfterWrite,
}

impl FailureClass {
    pub const ALL: [Self; 5] = [
        Self::InvalidProfile,
        Self::InvalidRequest,
        Self::ConnectFailed,
        Self::NotAccepted,
        Self::UnknownAfterWrite,
    ];
}

impl From<ClientFailure> for FailureClass {
    fn from(failure: ClientFailure) -> Self {
        match failure {
            ClientFailure::InvalidProfile => Self::InvalidProfile,
            ClientFailure::InvalidRequest => Self::InvalidRequest,
            ClientFailure::ConnectFailed => Self::ConnectFailed,
            ClientFailure::NotAccepted => Self::NotAccepted,
            ClientFailure::UnknownAfterWrite => Self::UnknownAfterWrite,
        }
    }
}

/// What the client observed for one exchange attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "class", rename_all = "snake_case")]
pub enum ExchangeOutcome {
    Failure(FailureClass),
    Success,
    Rejected(RejectionCode),
}

impl ExchangeOutcome {
    #[must_use]
    pub fn of(result: &Result<ExchangeResponse, ClientFailure>) -> Self {
        match result {
            Ok(response) => match &response.result {
                ExchangeResult::Success(_) => Self::Success,
                ExchangeResult::Rejected(rejection) => Self::Rejected(rejection.code),
            },
            Err(failure) => Self::Failure(FailureClass::from(*failure)),
        }
    }
}

/// The only admissible next step after one exchange outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryStep {
    /// The response is the result; nothing to recover.
    Complete,
    /// The request was not accepted: send the identical request bytes again.
    ResendIdentical,
    /// A dispatch outcome is unknown: send `RecoverDispatch` with the identical workflow request.
    RecoverDispatch,
    /// Recovery found no acceptance on the same endpoint incarnation: send the original dispatch
    /// again.
    ResendDispatch,
    /// A control outcome is unknown: observe, then decide with [`control_after_observe`].
    ObserveThenCompare,
    /// A read-only command failed: retry the same command within the caller's bound.
    RetryBounded,
    /// A closed rejection is the answer: surface it and stop.
    StopRejected,
    /// The control's expected sequence is behind the observation: never resend it.
    StopSuperseded,
    /// The outcome stays unknown: stop without redispatch and reconcile through other evidence.
    StopUnknown,
    /// Nothing was sent because the local profile or request is invalid.
    StopInvalid,
}

/// C6 rule table.
///
/// `same_incarnation` is gateway evidence, not wire data: the guest endpoint's
/// environment and attempt are unchanged and no restore from an older snapshot
/// happened since the original dispatch was written.
// r[impl lattice.workflow_interchange.reconnect_rules]
#[must_use]
pub const fn next_step(command: CommandKind, outcome: ExchangeOutcome, same_incarnation: bool) -> RecoveryStep {
    match outcome {
        ExchangeOutcome::Success => RecoveryStep::Complete,
        ExchangeOutcome::Rejected(RejectionCode::NotFound) => match command {
            CommandKind::RecoverDispatch if same_incarnation => RecoveryStep::ResendDispatch,
            CommandKind::RecoverDispatch => RecoveryStep::StopUnknown,
            CommandKind::Dispatch
            | CommandKind::Observe
            | CommandKind::Control
            | CommandKind::ReadLog
            | CommandKind::Reconcile => RecoveryStep::StopRejected,
        },
        ExchangeOutcome::Rejected(_) => RecoveryStep::StopRejected,
        ExchangeOutcome::Failure(FailureClass::InvalidProfile | FailureClass::InvalidRequest) => {
            RecoveryStep::StopInvalid
        }
        ExchangeOutcome::Failure(FailureClass::ConnectFailed | FailureClass::NotAccepted) => match command {
            CommandKind::Dispatch | CommandKind::Control => RecoveryStep::ResendIdentical,
            CommandKind::RecoverDispatch | CommandKind::Observe | CommandKind::ReadLog | CommandKind::Reconcile => {
                RecoveryStep::RetryBounded
            }
        },
        ExchangeOutcome::Failure(FailureClass::UnknownAfterWrite) => match command {
            CommandKind::Dispatch => RecoveryStep::RecoverDispatch,
            CommandKind::Control => RecoveryStep::ObserveThenCompare,
            CommandKind::RecoverDispatch | CommandKind::Observe | CommandKind::ReadLog | CommandKind::Reconcile => {
                RecoveryStep::RetryBounded
            }
        },
    }
}

/// Second half of the control rule: compare the control's `expected_sequence`
/// with the sequence observed after an unknown control outcome.
///
/// An applied control advances the sequence, so an equal sequence means the
/// control was not applied and the identical bytes may be resent. A higher
/// sequence means the control is behind and must never be resent. A lower
/// sequence means the observation predates the control's basis, so the endpoint
/// is not the one the control was written to.
// r[impl lattice.workflow_interchange.reconnect_rules]
#[must_use]
#[allow(
    unknown_lints,
    ambiguous_params,
    reason = "both sequences are named at every call; the comparison is the published C6 control rule"
)]
pub const fn control_after_observe(expected_sequence: u64, observed_sequence: u64) -> RecoveryStep {
    if observed_sequence == expected_sequence {
        RecoveryStep::ResendIdentical
    } else if observed_sequence > expected_sequence {
        RecoveryStep::StopSuperseded
    } else {
        RecoveryStep::StopUnknown
    }
}
