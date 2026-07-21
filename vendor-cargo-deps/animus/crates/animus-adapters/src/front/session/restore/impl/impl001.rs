use alloc::string::{String, ToString};
use alloc::vec::Vec;

const RESTORE_TARGET_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("restore_target_label");
const SESSION_SOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("session_source_label");
const AGENT_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("session_agent_label");
const MAX_RESTORE_TARGETS: usize = 16;
const ZERO_RESTORE_ITEMS: usize = 0;
const CANONICAL_SEPARATOR: &str = "|";
const CANONICAL_VERSION_PREFIX: &str = "session-restore-receipt-v1";
const CLAIM_PROCESS_SURVIVED: u8 = 1 << 0;
const CLAIM_LAYOUT_RECONSTRUCTED: u8 = 1 << 1;
const CLAIM_DISPLAY_REPLAYED: u8 = 1 << 2;
const CLAIM_NATIVE_RESUME_REQUESTED: u8 = 1 << 3;
const CLAIM_HANDOFF_REQUESTED: u8 = 1 << 4;
const CLAIM_MODEL_CONTEXT_ADMITTED: u8 = 1 << 5;
const CAPABILITY_LIVE_PROCESS: u8 = 1 << 0;
const CAPABILITY_SNAPSHOT_LAYOUT: u8 = 1 << 1;
const CAPABILITY_HISTORY_REPLAY: u8 = 1 << 2;
const CAPABILITY_NATIVE_RESUME: u8 = 1 << 3;
const CAPABILITY_HANDOFF_TRANSFER: u8 = 1 << 4;
const EFFECT_SPAWNED_PROCESS: u8 = 1 << 0;
const EFFECT_READ_FILES: u8 = 1 << 1;
const EFFECT_OPENED_PTY: u8 = 1 << 2;
const EFFECT_CONSTRUCTED_SHELL_COMMAND: u8 = 1 << 3;
const EFFECT_INSPECTED_ENVIRONMENT: u8 = 1 << 4;
const EMPTY_BITS: u8 = 0;

const _: () = assert!(MAX_RESTORE_TARGETS > ZERO_RESTORE_ITEMS);

/// Restore mode requested by a host snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestoreMode {
    /// Live process persistence.
    LiveProcess,
    /// Snapshot layout reconstruction.
    SnapshotLayout,
    /// Bounded display history replay.
    HistoryReplay,
    /// Native agent conversation resume.
    NativeSessionResume,
    /// Handoff-style process transfer.
    HandoffTransfer,
    /// Safe fallback shell or replacement target.
    Fallback,
}

impl RestoreMode {
    /// Returns the stable restore-mode label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LiveProcess => "live-process",
            Self::SnapshotLayout => "snapshot-layout",
            Self::HistoryReplay => "history-replay",
            Self::NativeSessionResume => "native-session-resume",
            Self::HandoffTransfer => "handoff-transfer",
            Self::Fallback => "fallback",
        }
    }
}

const fn has_bit(bits: u8, bit: u8) -> bool {
    bits & bit == bit
}

const fn capability_bit(mode: RestoreMode) -> u8 {
    match mode {
        RestoreMode::LiveProcess => CAPABILITY_LIVE_PROCESS,
        RestoreMode::SnapshotLayout => CAPABILITY_SNAPSHOT_LAYOUT,
        RestoreMode::HistoryReplay => CAPABILITY_HISTORY_REPLAY,
        RestoreMode::NativeSessionResume => CAPABILITY_NATIVE_RESUME,
        RestoreMode::HandoffTransfer => CAPABILITY_HANDOFF_TRANSFER,
        RestoreMode::Fallback => EMPTY_BITS,
    }
}

/// Restore target status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestoreOutcomeStatus {
    /// Requested restore capability was supported.
    Restored,
    /// Restore degraded to a safe fallback.
    Degraded,
}

impl RestoreOutcomeStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Restored => "restored",
            Self::Degraded => "degraded",
        }
    }
}

/// Degraded restore reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestoreDegradedReason {
    /// Host does not support the requested capability.
    UnsupportedCapability,
    /// Native session reference was missing.
    MissingNativeReference,
    /// Native session reference was malformed.
    MalformedNativeReference,
    /// Native session reference was duplicated.
    DuplicatedNativeReference,
    /// Native session reference was stale.
    StaleNativeReference,
    /// Native session reference was unsupported for resume.
    UnsupportedNativeReference,
    /// Native session reference contained sensitive material.
    SensitiveNativeReference,
    /// History replay was disabled by policy.
    HistoryReplayDisabled,
    /// History replay was unsafe.
    UnsafeHistoryReplay,
    /// Handoff success was not supported by host evidence.
    FailedHandoffClaim,
    /// Explicit safe fallback.
    SafeFallback,
}
