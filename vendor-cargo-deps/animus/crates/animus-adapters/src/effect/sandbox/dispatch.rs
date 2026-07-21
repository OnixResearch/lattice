/// Sandbox dispatch status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxDispatchStatus {
    /// Dispatch may continue to host execution.
    Ready,
    /// Dispatch must fail closed before host execution.
    Denied,
}

impl SandboxDispatchStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Denied => "denied",
        }
    }
}

/// Sandbox dispatch issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxDispatchIssue {
    /// No issue was found.
    None,
    /// Descriptor contains conflicting constraints.
    Conflict,
    /// Policy denied at least one requested constraint class.
    PolicyDenied,
    /// Platform lacks required capabilities.
    UnsupportedPlatformCapability,
    /// Required enforcement could only be partial.
    PartialEnforcement,
}

impl SandboxDispatchIssue {
    /// Returns the stable issue label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Conflict => "conflict",
            Self::PolicyDenied => "policy_denied",
            Self::UnsupportedPlatformCapability => "unsupported_platform_capability",
            Self::PartialEnforcement => "partial_enforcement",
        }
    }
}

/// Pure sandbox dispatch decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxDispatchDecision {
    status: SandboxDispatchStatus,
    issue: SandboxDispatchIssue,
    unsupported_capabilities: alloc::vec::Vec<SandboxPlatformCapability>,
}

impl SandboxDispatchDecision {
    /// Returns the dispatch status.
    #[must_use]
    pub const fn status(&self) -> SandboxDispatchStatus {
        self.status
    }

    /// Returns the dispatch issue.
    #[must_use]
    pub const fn issue(&self) -> SandboxDispatchIssue {
        self.issue
    }

    /// Returns whether host execution can start.
    #[must_use]
    pub const fn dispatch_allowed(&self) -> bool {
        matches!(self.status, SandboxDispatchStatus::Ready)
    }

    /// Returns unsupported platform capability labels.
    #[must_use]
    pub fn unsupported_capabilities(&self) -> &[SandboxPlatformCapability] {
        self.unsupported_capabilities.as_slice()
    }
}

/// Evaluates policy and platform support for a normalized sandbox descriptor.
#[must_use]
pub fn evaluate_sandbox_dispatch(
    descriptor: &SandboxDescriptor,
    policy: &SandboxPolicy,
    platform: &SandboxPlatformSupport,
) -> SandboxDispatchDecision {
    debug_assert!(!descriptor.descriptor_id().is_empty());
    debug_assert!(!policy.policy_id().is_empty());

    let is_policy_denied = descriptor
        .constraint_classes()
        .into_iter()
        .any(|class| !policy.allows(class));
    if is_policy_denied {
        return denied(SandboxDispatchIssue::PolicyDenied, alloc::vec::Vec::new());
    }
    let unsupported_capability_set = descriptor
        .required_platform_capabilities()
        .into_iter()
        .filter(|capability| !platform.supports(*capability))
        .collect::<alloc::vec::Vec<_>>();
    if unsupported_capability_set.is_empty() {
        return SandboxDispatchDecision {
            status: SandboxDispatchStatus::Ready,
            issue: SandboxDispatchIssue::None,
            unsupported_capabilities: unsupported_capability_set,
        };
    }
    let issue = if descriptor.enforcement_requirement() == SandboxEnforcementRequirement::Required {
        SandboxDispatchIssue::UnsupportedPlatformCapability
    } else {
        SandboxDispatchIssue::PartialEnforcement
    };
    denied(issue, unsupported_capability_set)
}

/// Safe sandbox dispatch receipt with hashes, labels, and bounded counts only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxDispatchReceipt {
    status: SandboxDispatchStatus,
    issue: SandboxDispatchIssue,
    descriptor_hash: super::SafeHash,
    platform_capability_labels: alloc::vec::Vec<alloc::string::String>,
    unsupported_capability_labels: alloc::vec::Vec<alloc::string::String>,
    cow_summary_count: u32,
    output_hash: Option<super::SafeHash>,
    receipt_hash: super::SafeHash,
}

impl SandboxDispatchReceipt {
    /// Builds a redacted sandbox dispatch receipt.
    #[must_use]
    pub fn new(draft: SandboxDispatchReceiptDraft<'_>) -> Self {
        let platform_capability_labels = draft
            .platform
            .capabilities()
            .iter()
            .map(|capability| alloc::string::String::from(capability.as_str()))
            .collect::<alloc::vec::Vec<_>>();
        let unsupported_capability_label_set = draft
            .decision
            .unsupported_capabilities()
            .iter()
            .map(|capability| alloc::string::String::from(capability.as_str()))
            .collect::<alloc::vec::Vec<_>>();
        let receipt_hash_input = ReceiptHashInput {
            descriptor: draft.descriptor,
            decision: draft.decision,
            platform: draft.platform,
            cow_summary_count: draft.cow_summary_count,
            output_hash: draft.output_hash,
        };
        let receipt_hash = receipt_hash(&receipt_hash_input);
        Self {
            status: draft.decision.status(),
            issue: draft.decision.issue(),
            descriptor_hash: draft.descriptor.descriptor_hash().clone(),
            platform_capability_labels,
            unsupported_capability_labels: unsupported_capability_label_set,
            cow_summary_count: draft.cow_summary_count,
            output_hash: draft.output_hash.cloned(),
            receipt_hash,
        }
    }

    /// Returns dispatch status.
    #[must_use]
    pub const fn status(&self) -> SandboxDispatchStatus {
        self.status
    }

    /// Returns dispatch issue.
    #[must_use]
    pub const fn issue(&self) -> SandboxDispatchIssue {
        self.issue
    }

    /// Returns normalized descriptor hash.
    #[must_use]
    pub const fn descriptor_hash(&self) -> &super::SafeHash {
        &self.descriptor_hash
    }

    /// Returns platform capability labels.
    #[must_use]
    pub fn platform_capability_labels(&self) -> &[alloc::string::String] {
        self.platform_capability_labels.as_slice()
    }

    /// Returns unsupported capability labels.
    #[must_use]
    pub fn unsupported_capability_labels(&self) -> &[alloc::string::String] {
        self.unsupported_capability_labels.as_slice()
    }

    /// Returns bounded COW summary count.
    #[must_use]
    pub const fn cow_summary_count(&self) -> u32 {
        self.cow_summary_count
    }

    /// Returns optional bounded output hash.
    #[must_use]
    pub const fn output_hash(&self) -> Option<&super::SafeHash> {
        self.output_hash.as_ref()
    }

    /// Returns deterministic receipt hash.
    #[must_use]
    pub const fn receipt_hash(&self) -> &super::SafeHash {
        &self.receipt_hash
    }
}

/// Borrowed sandbox receipt input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SandboxDispatchReceiptDraft<'a> {
    /// Normalized descriptor.
    pub descriptor: &'a SandboxDescriptor,
    /// Pure dispatch decision.
    pub decision: &'a SandboxDispatchDecision,
    /// Platform support evaluated for dispatch.
    pub platform: &'a SandboxPlatformSupport,
    /// Bounded COW summary count.
    pub cow_summary_count: u32,
    /// Optional safe output hash.
    pub output_hash: Option<&'a super::SafeHash>,
}

/// Sandbox receipt leak class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxReceiptLeak {
    /// Raw prompt text appeared in evidence.
    RawPrompt,
    /// Credential-looking material appeared in evidence.
    Credential,
    /// Absolute path dump appeared in evidence.
    FullPath,
    /// Raw network payload appeared in evidence.
    RawNetworkPayload,
    /// Raw file content appeared in evidence.
    RawFileContent,
    /// Raw process output appeared in evidence.
    RawProcessOutput,
    /// Unbounded event stream appeared in evidence.
    UnboundedEventStream,
}

impl SandboxReceiptLeak {
    /// Returns the stable leak label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RawPrompt => "raw_prompt",
            Self::Credential => "credential",
            Self::FullPath => "full_path",
            Self::RawNetworkPayload => "raw_network_payload",
            Self::RawFileContent => "raw_file_content",
            Self::RawProcessOutput => "raw_process_output",
            Self::UnboundedEventStream => "unbounded_event_stream",
        }
    }
}

/// Validates rendered sandbox receipt evidence for unsafe sentinels.
///
/// # Errors
///
/// Returns [`SandboxReceiptLeak`] when raw prompt, credential, absolute path,
/// network payload, file body, process output, or unbounded event markers appear.
pub fn validate_sandbox_receipt_text(text: &str) -> Result<(), SandboxReceiptLeak> {
    first_leak(text).map_or(Ok(()), Err)
}
