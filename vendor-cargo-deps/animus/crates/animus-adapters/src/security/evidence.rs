/// Unsafe material class rejected by security review receipt validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Leak {
    /// Raw prompt text appeared in a safe field.
    RawPrompt,
    /// Credential-like material appeared in a safe field.
    Credential,
    /// Private full path material appeared in a safe field.
    PrivatePath,
    /// Raw provider payload material appeared in a safe field.
    ProviderPayload,
    /// Raw tool arguments appeared in a safe field.
    RawToolArguments,
    /// Raw tool output appeared in a safe field.
    RawToolOutput,
}

impl Leak {
    /// Returns the stable leak label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RawPrompt => "raw_prompt",
            Self::Credential => "credential",
            Self::PrivatePath => "private_path",
            Self::ProviderPayload => "provider_payload",
            Self::RawToolArguments => "raw_tool_arguments",
            Self::RawToolOutput => "raw_tool_output",
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Draft<'a> {
    pub(super) input: &'a super::Review,
    pub(super) scan_report: &'a super::Report,
    pub(super) advisory_reviews: &'a [super::AdvisoryReview],
    pub(super) status: super::Status,
    pub(super) issue: super::Issue,
}

/// Safe security review receipt metadata emitted before dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    status: super::Status,
    issue: super::Issue,
    action_class: super::ActionClass,
    risk_class: super::RiskClass,
    finding_count: u32,
    advisory_count: u32,
    evidence_hashes: super::Collection<super::Text>,
    hash: super::Text,
}

impl Receipt {
    pub(super) fn new(draft: Draft<'_>) -> Self {
        let finding_count = super::receipt_count(draft.scan_report.findings().len());
        let advisory_count = super::receipt_count(draft.advisory_reviews.len());
        let receipt_hash = super::hash_parts(&[
            super::HASH_SCHEMA,
            draft.status.as_str(),
            draft.issue.as_str(),
            draft.input.action_class().as_str(),
            draft.input.primary_evidence_hash(),
        ]);
        Self {
            status: draft.status,
            issue: draft.issue,
            action_class: draft.input.action_class(),
            risk_class: draft.input.risk_class(),
            finding_count,
            advisory_count,
            evidence_hashes: draft.input.evidence_hashes().to_vec(),
            hash: receipt_hash,
        }
    }

    /// Returns the review decision status.
    #[must_use]
    pub const fn status(&self) -> super::Status {
        self.status
    }

    /// Returns the review decision issue.
    #[must_use]
    pub const fn issue(&self) -> super::Issue {
        self.issue
    }

    /// Returns the reviewed action class.
    #[must_use]
    pub const fn action_class(&self) -> super::ActionClass {
        self.action_class
    }

    /// Returns the policy risk class used for fail behavior.
    #[must_use]
    pub const fn risk_class(&self) -> super::RiskClass {
        self.risk_class
    }

    /// Returns the number of deterministic findings summarized by the receipt.
    #[must_use]
    pub const fn finding_count(&self) -> u32 {
        self.finding_count
    }

    /// Returns the number of advisory reviews summarized by the receipt.
    #[must_use]
    pub const fn advisory_count(&self) -> u32 {
        self.advisory_count
    }

    /// Returns redacted evidence identities included in the receipt.
    #[must_use]
    pub fn evidence_hashes(&self) -> &[super::Text] {
        self.evidence_hashes.as_slice()
    }

    /// Returns the deterministic BLAKE3 receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.hash.as_str()
    }
}

/// Validates rendered review receipt text for known raw-material sentinels.
///
/// # Errors
///
/// Returns [`Leak`] when the text contains prompt, credential, private path,
/// provider payload, raw tool argument, or raw tool output markers.
pub fn validate_text(text: &str) -> Result<(), Leak> {
    if let Some(leak) = detect_leak(text) {
        return Err(leak);
    }
    Ok(())
}

pub(super) fn detect_leak(text: &str) -> Option<Leak> {
    debug_assert!(!super::RAW_PROMPT_MARKERS.is_empty());
    debug_assert!(!super::CREDENTIAL_MARKERS.is_empty());
    let lower = text.to_ascii_lowercase();
    if contains_marker(&lower, super::RAW_PROMPT_MARKERS) {
        return Some(Leak::RawPrompt);
    }
    if contains_marker(&lower, super::CREDENTIAL_MARKERS) {
        return Some(Leak::Credential);
    }
    if contains_marker(&lower, super::PRIVATE_PATH_MARKERS) {
        return Some(Leak::PrivatePath);
    }
    if contains_marker(&lower, super::PROVIDER_PAYLOAD_MARKERS) {
        return Some(Leak::ProviderPayload);
    }
    if contains_marker(&lower, super::RAW_TOOL_ARGUMENT_MARKERS) {
        return Some(Leak::RawToolArguments);
    }
    if contains_marker(&lower, super::RAW_TOOL_OUTPUT_MARKERS) {
        return Some(Leak::RawToolOutput);
    }
    None
}

fn contains_marker(text: &str, markers: &[&str]) -> bool {
    markers
        .iter()
        .map(|marker| marker.to_ascii_lowercase())
        .any(|marker| text.contains(marker.as_str()))
}
