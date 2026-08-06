use super::HANDOFF_DETAIL_FIELD;
use super::HANDOFF_STATUS_FIELD;
use super::HANDOFF_SUMMARY_FIELD;
use super::MIN_DETAIL_BOUND_CHARS;
use super::SubagentError;
use super::SubagentReturnMode;
use super::Text;
use super::required_text;

/// Untrusted subagent handoff data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubagentHandoffDraft<'a> {
    /// Return mode label.
    pub return_mode: &'a str,
    /// Child status label.
    pub status: &'a str,
    /// Bounded child summary.
    pub summary: &'a str,
    /// Optional bounded detail reference.
    pub detail_ref: Option<&'a str>,
    /// Observed detail character count.
    pub detail_chars: u32,
    /// Configured detail character bound.
    pub max_detail_chars: u32,
}

/// Validated parent handoff record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentHandoff {
    return_mode: SubagentReturnMode,
    status: Text,
    summary: Text,
    detail_ref: Option<Text>,
}

impl SubagentHandoff {
    /// Returns the return mode.
    #[must_use]
    pub const fn return_mode(&self) -> SubagentReturnMode {
        self.return_mode
    }

    /// Returns the child status label.
    #[must_use]
    pub fn status(&self) -> &str {
        self.status.as_str()
    }

    /// Returns the bounded summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }

    /// Returns the imported detail reference, when allowed by mode and bounds.
    #[must_use]
    pub fn detail_ref(&self) -> Option<&str> {
        self.detail_ref.as_deref()
    }
}

/// Builds a parent handoff record according to return-mode bounds.
///
/// # Errors
///
/// Returns [`SubagentError`] when status or summary is malformed, full detail is
/// unbounded, or full detail exceeds its configured bound.
///
/// # Panics
///
/// Panics if stable handoff field names are accidentally emptied at compile time.
pub fn build_subagent_handoff(draft: &SubagentHandoffDraft<'_>) -> Result<SubagentHandoff, SubagentError> {
    assert!(!HANDOFF_STATUS_FIELD.as_str().is_empty(), "handoff status field is stable");
    assert!(!HANDOFF_SUMMARY_FIELD.as_str().is_empty(), "handoff summary field is stable");
    let return_mode = SubagentReturnMode::parse(draft.return_mode)?;
    let status = required_text(HANDOFF_STATUS_FIELD, draft.status)?;
    let summary = required_text(HANDOFF_SUMMARY_FIELD, draft.summary)?;
    let detail_ref = match return_mode {
        SubagentReturnMode::SummaryOnly | SubagentReturnMode::StatusPlusSummary => None,
        SubagentReturnMode::FullDetailWithBounds => {
            if draft.max_detail_chars < MIN_DETAIL_BOUND_CHARS || draft.detail_chars > draft.max_detail_chars {
                return Err(SubagentError::DetailBoundsExceeded);
            }
            Some(required_text(HANDOFF_DETAIL_FIELD, draft.detail_ref.ok_or(SubagentError::MissingDetailReference)?)?)
        }
    };
    Ok(SubagentHandoff {
        return_mode,
        status,
        summary,
        detail_ref,
    })
}
