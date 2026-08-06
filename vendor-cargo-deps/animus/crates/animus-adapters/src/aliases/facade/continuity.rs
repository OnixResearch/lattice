pub use crate::ledger::SessionLedgerService;
/// Neutral ledger entry owned by Animus adapter boundaries.
pub type LedgerEntry = crate::ledger::Entry;
/// Validated neutral ledger entry identifier.
pub type LedgerEntryId = crate::ledger::EntryId;
/// Kind of neutral ledger entry.
pub type LedgerEntryKind = crate::ledger::EntryKind;
/// Summary semantics for first-class summary entries.
pub type LedgerSummaryKind = crate::ledger::SummaryKind;
/// One append-only ledger entry with an optional parent link.
pub type LedgerNode = crate::ledger::Node;
/// Operator-facing label attached to one ledger entry.
pub type LedgerLabel = crate::ledger::Label;
/// In-memory session tree snapshot selected by an active leaf.
pub type LedgerSnapshot = crate::ledger::Snapshot;
/// Safe replay metadata counts.
pub type LedgerReplayCounts = crate::ledger::ReplayCounts;
/// Replay errors that fail closed without exposing raw adapter-owned payloads.
pub type LedgerReplayError = crate::ledger::Error;
/// Successful replay result for neutral entries.
pub type LedgerReplayReport = crate::ledger::Report;
/// Replay request sent to an injected session-ledger service.
pub type LedgerReplayRequest = crate::ledger::Request;
/// Model-visible context item produced by replay.
pub type ModelContextItem = crate::ledger::ContextItem;
/// Model-visible replay role.
pub type ModelContextRole = crate::ledger::ContextRole;

/// Builds the fail-closed outcome for an absent session-ledger service.
#[must_use]
pub fn missing_session_ledger_service(request: &LedgerReplayRequest) -> crate::ServiceOutcome<LedgerReplayReport> {
    crate::ledger::missing_service(request)
}

/// Builds the typed error for a missing session-ledger service.
#[must_use]
pub fn missing_session_ledger_service_error(request: &LedgerReplayRequest) -> LedgerReplayError {
    crate::ledger::missing_service_error(request)
}

/// Builds the typed error for a missing session in the ledger service.
#[must_use]
pub fn missing_session_ledger_session_error(request: &LedgerReplayRequest) -> LedgerReplayError {
    crate::ledger::missing_session_error(request)
}

/// Builds the typed error for an unavailable session-ledger service.
#[must_use]
pub fn unavailable_session_ledger_service_error(request: &LedgerReplayRequest) -> LedgerReplayError {
    crate::ledger::unavailable_service_error(request)
}

/// Replays neutral entries into model-visible context plus safe counts.
///
/// # Errors
///
/// Returns [`LedgerReplayError`] when the ledger is empty or contains an
/// unsupported or adapter-owned entry marker.
pub fn replay_ledger(entries: &[LedgerEntry]) -> Result<LedgerReplayReport, LedgerReplayError> {
    crate::ledger::replay(entries)
}

/// Replays the selected active branch of a session tree snapshot.
///
/// # Errors
///
/// Returns [`LedgerReplayError`] when tree shape validation fails, the active
/// branch contains unsupported entries, a tool result is orphaned, or a summary
/// has no prior model-visible span to replace.
pub fn replay_session_tree(snapshot: &LedgerSnapshot) -> Result<LedgerReplayReport, LedgerReplayError> {
    crate::ledger::replay_snapshot(snapshot)
}
