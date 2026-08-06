//! Explicit Animus and Tile mapping and snapshot-first stream profiles.

/// Canonical semantic status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticStatus {
    /// Unknown.
    Unknown,
    /// Idle.
    Idle,
    /// Running.
    Running,
    /// Waiting for input or dependency.
    Waiting,
    /// Explicitly blocked.
    Blocked,
    /// Completed.
    Done,
    /// Failed.
    Failed,
}

/// Mapping diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MappingError {
    /// Native status is unsupported without loss.
    LossyStatus,
    /// Positional selector scope or revision drifted.
    StaleSelector,
    /// Snapshot-first ordering was violated.
    StreamOrder,
    /// Optional feature was not negotiated.
    UnsupportedFeature,
}

/// Map an Animus status exactly.
///
/// # Errors
/// Returns a loss diagnostic for unknown labels.
pub fn map_animus_status(value: &str) -> Result<SemanticStatus, MappingError> {
    match value {
        "unknown" => Ok(SemanticStatus::Unknown),
        "idle" => Ok(SemanticStatus::Idle),
        "running" => Ok(SemanticStatus::Running),
        "waiting" => Ok(SemanticStatus::Waiting),
        "blocked" => Ok(SemanticStatus::Blocked),
        "done" => Ok(SemanticStatus::Done),
        "failed" => Ok(SemanticStatus::Failed),
        _ => Err(MappingError::LossyStatus),
    }
}

/// Map Tile's narrower vocabulary without silently coercing waiting or failure.
///
/// # Errors
/// Returns a loss diagnostic for labels outside Tile's declared vocabulary.
pub fn map_tile_status(value: &str) -> Result<SemanticStatus, MappingError> {
    match value {
        "unknown" => Ok(SemanticStatus::Unknown),
        "idle" => Ok(SemanticStatus::Idle),
        "working" => Ok(SemanticStatus::Running),
        "blocked" => Ok(SemanticStatus::Blocked),
        "done" => Ok(SemanticStatus::Done),
        _ => Err(MappingError::LossyStatus),
    }
}

/// Validate a revision-scoped positional selector.
///
/// # Errors
/// Returns stale when either declared scope or revision differs.
pub fn validate_scoped_index(
    expected_scope: &str,
    observed_scope: &str,
    expected_revision: u64,
    observed_revision: u64,
) -> Result<(), MappingError> {
    if expected_scope == observed_scope && expected_revision == observed_revision {
        Ok(())
    } else {
        Err(MappingError::StaleSelector)
    }
}

/// Snapshot-first stream state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamState {
    /// No snapshot accepted.
    AwaitingSnapshot,
    /// Snapshot accepted; incremental updates allowed.
    Active,
}

/// Stream item kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamItem {
    /// Initial redaction-safe snapshot.
    Snapshot,
    /// Incremental update.
    Incremental,
}

/// Advance a negotiated snapshot-first stream.
///
/// # Errors
/// Returns unsupported or ordering failures before state mutation.
pub fn advance_stream(negotiated: bool, state: StreamState, item: StreamItem) -> Result<StreamState, MappingError> {
    if !negotiated {
        return Err(MappingError::UnsupportedFeature);
    }
    match (state, item) {
        (StreamState::AwaitingSnapshot, StreamItem::Snapshot) | (StreamState::Active, StreamItem::Incremental) => {
            Ok(StreamState::Active)
        }
        _ => Err(MappingError::StreamOrder),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mappings_and_streams_preserve_declared_semantics() {
        assert_eq!(map_animus_status("waiting"), Ok(SemanticStatus::Waiting));
        assert_eq!(map_tile_status("working"), Ok(SemanticStatus::Running));
        assert_eq!(advance_stream(true, StreamState::AwaitingSnapshot, StreamItem::Snapshot), Ok(StreamState::Active));
    }
    #[test]
    fn loss_stale_scope_and_incremental_first_fail_closed() {
        assert_eq!(map_tile_status("failed"), Err(MappingError::LossyStatus));
        assert_eq!(validate_scoped_index("tab:1", "tab:2", 1, 1), Err(MappingError::StaleSelector));
        assert_eq!(
            advance_stream(true, StreamState::AwaitingSnapshot, StreamItem::Incremental),
            Err(MappingError::StreamOrder)
        );
        assert_eq!(
            advance_stream(false, StreamState::AwaitingSnapshot, StreamItem::Snapshot),
            Err(MappingError::UnsupportedFeature)
        );
    }
}
