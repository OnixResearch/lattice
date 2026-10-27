const PRIVACY_LOCAL_ONLY: &str = "local_only";
const PRIVACY_REMOTE_ALLOWED: &str = "remote_allowed";
const FALLBACK_ELIGIBLE: &str = "eligible";
const FALLBACK_INELIGIBLE: &str = "ineligible";

const EMPTY_COUNT: u32 = 0;
const MAX_CONTEXT_PRESSURE_PER_MILLE: u16 = 1_000;
const EMPTY_METADATA_SIGNAL_COUNT: usize = 4;

const UNSAFE_SENTINELS: &[&str] = &[
    "prompt:",
    "provider payload",
    "api_key",
    "credential",
    "secret",
    "\n",
];

/// Bounded request metadata consumed by the default router signal projector.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestMetadata {
    prompt_char_estimate: u32,
    tool_call_count: u32,
    contains_code: bool,
    requires_structured_output: bool,
    context_window_fill_per_mille: u16,
    privacy_policy: PrivacyPolicy,
    fallback_hint: FallbackHint,
}

impl RequestMetadata {
    /// Builds bounded request metadata from host-supplied safe fields.
    ///
    /// # Errors
    ///
    /// Returns [`SignalProjectionError`] when metadata is empty, privacy is
    /// missing or unsupported, fallback is unsupported, context pressure is out
    /// of range, or a label resembles an unsafe raw payload.
    pub fn new(draft: RequestMetadataDraft<'_>) -> Result<Self, SignalProjectionError> {
        validate_non_empty(&draft)?;
        validate_context_pressure(draft.context_window_fill_per_mille)?;
        reject_unsafe_label(UnsafeLabelField::PrivacyPolicy, draft.privacy_policy)?;
        reject_unsafe_label(UnsafeLabelField::FallbackHint, draft.fallback_hint)?;
        Ok(Self {
            prompt_char_estimate: draft.prompt_char_estimate,
            tool_call_count: draft.tool_call_count,
            contains_code: draft.contains_code,
            requires_structured_output: draft.requires_structured_output,
            context_window_fill_per_mille: draft.context_window_fill_per_mille,
            privacy_policy: PrivacyPolicy::parse(draft.privacy_policy)?,
            fallback_hint: FallbackHint::parse(draft.fallback_hint)?,
        })
    }

    /// Returns the bounded prompt character estimate.
    #[must_use]
    pub const fn prompt_char_estimate(&self) -> u32 {
        self.prompt_char_estimate
    }

    /// Returns the bounded tool-call count.
    #[must_use]
    pub const fn tool_call_count(&self) -> u32 {
        self.tool_call_count
    }

    /// Returns whether code-like content was observed by the host.
    #[must_use]
    pub const fn contains_code(&self) -> bool {
        self.contains_code
    }

    /// Returns whether structured output was requested.
    #[must_use]
    pub const fn requires_structured_output(&self) -> bool {
        self.requires_structured_output
    }

    /// Returns context-window fill in per-mille units.
    #[must_use]
    pub const fn context_window_fill_per_mille(&self) -> u16 {
        self.context_window_fill_per_mille
    }

    pub(crate) const fn privacy_signal(&self) -> &'static str {
        self.privacy_policy.as_signal()
    }

    pub(crate) const fn fallback_signal(&self) -> &'static str {
        self.fallback_hint.as_signal()
    }
}

/// Borrowed fields used to build [`RequestMetadata`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestMetadataDraft<'a> {
    /// Bounded prompt character estimate supplied by the host.
    pub prompt_char_estimate: u32,
    /// Bounded count of declared tool calls.
    pub tool_call_count: u32,
    /// Whether the host observed code-like content without passing raw text.
    pub contains_code: bool,
    /// Whether the request demands structured output.
    pub requires_structured_output: bool,
    /// Context-window fill in per-mille units, from 0 to 1000 inclusive.
    pub context_window_fill_per_mille: u16,
    /// Privacy/locality policy label.
    pub privacy_policy: &'a str,
    /// Fallback eligibility hint label.
    pub fallback_hint: &'a str,
}

/// Error returned while projecting request metadata into router signals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SignalProjectionError {
    /// Metadata carried no useful bounded signal.
    EmptyMetadata,
    /// Privacy policy label was missing.
    MissingPrivacyPolicy,
    /// Privacy policy label was not supported.
    UnsupportedPrivacyPolicy,
    /// Fallback hint label was not supported.
    UnsupportedFallbackHint,
    /// Context-window fill exceeded the per-mille range.
    ContextWindowOutOfRange {
        /// Maximum accepted context fill per-mille value.
        max_per_mille: u16,
        /// Actual context fill per-mille value.
        actual_per_mille: u16,
    },
    /// A label looked like raw prompt/provider/credential material.
    UnsafePayload {
        /// Field that carried unsafe-looking text.
        field: &'static str,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PrivacyPolicy {
    LocalOnly,
    RemoteAllowed,
}

impl PrivacyPolicy {
    fn parse(value: &str) -> Result<Self, SignalProjectionError> {
        match value.trim() {
            "" => Err(SignalProjectionError::MissingPrivacyPolicy),
            PRIVACY_LOCAL_ONLY => Ok(Self::LocalOnly),
            PRIVACY_REMOTE_ALLOWED => Ok(Self::RemoteAllowed),
            _ => Err(SignalProjectionError::UnsupportedPrivacyPolicy),
        }
    }

    const fn as_signal(self) -> &'static str {
        match self {
            Self::LocalOnly => PRIVACY_LOCAL_ONLY,
            Self::RemoteAllowed => PRIVACY_REMOTE_ALLOWED,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FallbackHint {
    Eligible,
    Ineligible,
}

impl FallbackHint {
    fn parse(value: &str) -> Result<Self, SignalProjectionError> {
        match value.trim() {
            FALLBACK_ELIGIBLE => Ok(Self::Eligible),
            FALLBACK_INELIGIBLE => Ok(Self::Ineligible),
            _ => Err(SignalProjectionError::UnsupportedFallbackHint),
        }
    }

    const fn as_signal(self) -> &'static str {
        match self {
            Self::Eligible => FALLBACK_ELIGIBLE,
            Self::Ineligible => FALLBACK_INELIGIBLE,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UnsafeLabelField {
    PrivacyPolicy,
    FallbackHint,
}

impl UnsafeLabelField {
    const fn as_str(self) -> &'static str {
        match self {
            Self::PrivacyPolicy => "privacy_policy",
            Self::FallbackHint => "fallback_hint",
        }
    }
}

fn validate_non_empty(draft: &RequestMetadataDraft<'_>) -> Result<(), SignalProjectionError> {
    let present_signals = [
        draft.prompt_char_estimate > EMPTY_COUNT,
        draft.tool_call_count > EMPTY_COUNT,
        draft.contains_code,
        draft.requires_structured_output,
    ];
    assert_eq!(present_signals.len(), EMPTY_METADATA_SIGNAL_COUNT);
    if !present_signals.iter().any(|signal_present| *signal_present) {
        return Err(SignalProjectionError::EmptyMetadata);
    }
    Ok(())
}

fn validate_context_pressure(value: u16) -> Result<(), SignalProjectionError> {
    if value > MAX_CONTEXT_PRESSURE_PER_MILLE {
        return Err(SignalProjectionError::ContextWindowOutOfRange {
            max_per_mille: MAX_CONTEXT_PRESSURE_PER_MILLE,
            actual_per_mille: value,
        });
    }
    Ok(())
}

fn reject_unsafe_label(
    field: UnsafeLabelField,
    label_text: &str,
) -> Result<(), SignalProjectionError> {
    if UNSAFE_SENTINELS
        .iter()
        .any(|sentinel| label_text.contains(sentinel))
    {
        return Err(SignalProjectionError::UnsafePayload {
            field: field.as_str(),
        });
    }
    Ok(())
}
