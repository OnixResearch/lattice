const SELECTED_TRUSTED_SUMMARY: &str = "selected trusted lifecycle status";
const SELECTED_HEURISTIC_SUMMARY: &str = "selected heuristic fallback status";
const UNKNOWN_STATUS_SUMMARY: &str = "no current trusted status authority";
const MAX_EVIDENCE_SUMMARY_CHARS: usize = 256;
const EMPTY_CHAR_COUNT: usize = 0;

pub(super) const RAW_PROMPT_MARKER: &str = "RAW_PROMPT:";
pub(super) const TERMINAL_SNAPSHOT_MARKER: &str = "TERMINAL_SNAPSHOT:";
pub(super) const PROVIDER_PAYLOAD_MARKER: &str = "PROVIDER_PAYLOAD:";
pub(super) const API_KEY_MARKER: &str = "API_KEY=";
pub(super) const SECRET_KEY_PREFIX_MARKER: &str = "sk-";
pub(super) const UNBOUNDED_DETECTOR_MARKER: &str = "UNBOUNDED_DETECTOR_EVIDENCE:";

const UNSAFE_MARKERS: &[&str] = &[
    RAW_PROMPT_MARKER,
    TERMINAL_SNAPSHOT_MARKER,
    PROVIDER_PAYLOAD_MARKER,
    API_KEY_MARKER,
    SECRET_KEY_PREFIX_MARKER,
    UNBOUNDED_DETECTOR_MARKER,
];

/// Bounded redacted evidence summary for durable explanations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceSummary {
    text: String,
}

impl EvidenceSummary {
    /// Builds a bounded, redacted evidence summary.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError`] when the summary is empty, overlong, or
    /// contains an unsafe raw-evidence marker.
    pub fn new(text: &str) -> Result<Self, EvidenceError> {
        validate_summary(text)?;
        Ok(Self {
            text: String::from(text),
        })
    }

    pub(super) fn trusted_lifecycle() -> Self {
        Self {
            text: String::from(SELECTED_TRUSTED_SUMMARY),
        }
    }

    pub(super) fn heuristic_fallback() -> Self {
        Self {
            text: String::from(SELECTED_HEURISTIC_SUMMARY),
        }
    }

    pub(super) fn unknown() -> Self {
        Self {
            text: String::from(UNKNOWN_STATUS_SUMMARY),
        }
    }

    /// Returns the safe summary text.
    #[must_use]
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
}

/// Evidence-summary validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceError {
    /// Summary contained no user-visible characters.
    EmptySummary,
    /// Summary exceeded the bounded evidence limit.
    SummaryTooLong {
        /// Maximum accepted character count.
        max_chars: usize,
        /// Actual character count.
        actual_chars: usize,
    },
    /// Summary contained a raw or unsafe evidence marker.
    UnsafeEvidenceMarker {
        /// Marker that was detected.
        marker: &'static str,
    },
}

impl core::fmt::Display for EvidenceError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptySummary => write!(formatter, "status evidence summary must not be empty"),
            Self::SummaryTooLong {
                max_chars,
                actual_chars,
            } => write!(
                formatter,
                "status evidence summary must be at most {max_chars} characters; got {actual_chars}"
            ),
            Self::UnsafeEvidenceMarker { marker } => write!(
                formatter,
                "status evidence summary contains unsafe marker {marker}"
            ),
        }
    }
}

impl std::error::Error for EvidenceError {}

/// Validates a durable status evidence summary.
///
/// # Errors
///
/// Returns [`EvidenceError`] when the summary is empty, overlong, or contains a
/// raw prompt, terminal snapshot, provider payload, credential-looking value, or
/// unbounded detector evidence marker.
pub fn validate_summary(text: &str) -> Result<(), EvidenceError> {
    let visible_chars = text.trim().chars().count();
    assert!(visible_chars <= text.chars().count());
    if visible_chars == EMPTY_CHAR_COUNT {
        return Err(EvidenceError::EmptySummary);
    }

    let actual_chars = text.chars().count();
    assert!(actual_chars >= visible_chars);
    if actual_chars > MAX_EVIDENCE_SUMMARY_CHARS {
        return Err(EvidenceError::SummaryTooLong {
            max_chars: MAX_EVIDENCE_SUMMARY_CHARS,
            actual_chars,
        });
    }

    for marker in UNSAFE_MARKERS {
        if text.contains(marker) {
            return Err(EvidenceError::UnsafeEvidenceMarker { marker });
        }
    }

    Ok(())
}
