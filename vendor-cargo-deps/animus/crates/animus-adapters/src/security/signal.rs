/// Deterministic scanner completion state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScannerStatus {
    /// Scanner completed and findings are authoritative deterministic evidence.
    Complete,
    /// Scanner was unavailable before review.
    Unavailable,
    /// Scanner timed out before review.
    Timeout,
    /// Scanner output was malformed and ignored.
    Malformed,
}

impl ScannerStatus {
    /// Returns the stable scanner status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Unavailable => "unavailable",
            Self::Timeout => "timeout",
            Self::Malformed => "malformed",
        }
    }

    pub(super) const fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }
}

/// Deterministic security finding category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Category {
    /// Prompt or instruction injection was detected.
    PromptInjection,
    /// Destructive host mutation was detected.
    DestructiveAction,
    /// Exfiltration or unknown-domain upload was detected.
    Exfiltration,
    /// Privilege escalation was detected.
    PrivilegeEscalation,
    /// Obfuscated download-and-execute behavior was detected.
    ObfuscatedRemoteExecution,
    /// Mutation outside the declared workspace or task scope was detected.
    OutOfScopeMutation,
}

impl Category {
    /// Returns the stable category label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PromptInjection => "prompt_injection",
            Self::DestructiveAction => "destructive_action",
            Self::Exfiltration => "exfiltration",
            Self::PrivilegeEscalation => "privilege_escalation",
            Self::ObfuscatedRemoteExecution => "obfuscated_remote_execution",
            Self::OutOfScopeMutation => "out_of_scope_mutation",
        }
    }

    pub(super) const fn safe_explanation(self) -> &'static str {
        match self {
            Self::PromptInjection => "instruction-injection marker detected",
            Self::DestructiveAction => "destructive action marker detected",
            Self::Exfiltration => "exfiltration or credential-access marker detected",
            Self::PrivilegeEscalation => "privilege escalation marker detected",
            Self::ObfuscatedRemoteExecution => "download-and-execute marker detected",
            Self::OutOfScopeMutation => "out-of-scope mutation marker detected",
        }
    }
}

/// Finding confidence bucket.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Confidence {
    /// Medium-confidence deterministic match.
    Medium,
    /// High-confidence deterministic match.
    High,
}

impl Confidence {
    /// Returns the stable confidence label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// Security policy disposition for one finding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Disposition {
    /// Finding is warning evidence only.
    Warn,
    /// Finding blocks dispatch before host effects.
    Block,
}

impl Disposition {
    /// Returns the stable disposition label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warn => "warn",
            Self::Block => "block",
        }
    }

    pub(super) const fn blocks_dispatch(self) -> bool {
        matches!(self, Self::Block)
    }
}

/// Deterministic finding with safe metadata and redacted evidence identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    id: super::Text,
    category: Category,
    confidence: Confidence,
    disposition: Disposition,
    safe_explanation: super::Text,
    evidence_hash: super::Text,
}

impl Finding {
    pub(super) fn new(
        category: Category,
        confidence: Confidence,
        disposition: Disposition,
        evidence_hash: &str,
    ) -> Self {
        let id = super::hash_parts(&[super::HASH_SCHEMA, "finding", category.as_str(), evidence_hash]);
        Self {
            id,
            category,
            confidence,
            disposition,
            safe_explanation: super::Text::from(category.safe_explanation()),
            evidence_hash: super::Text::from(evidence_hash),
        }
    }

    /// Returns the stable BLAKE3 finding identifier.
    #[must_use]
    pub fn finding_id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns the finding category.
    #[must_use]
    pub const fn category(&self) -> Category {
        self.category
    }

    /// Returns the deterministic confidence bucket.
    #[must_use]
    pub const fn confidence(&self) -> Confidence {
        self.confidence
    }

    /// Returns the policy disposition.
    #[must_use]
    pub const fn disposition(&self) -> Disposition {
        self.disposition
    }

    /// Returns a safe explanation without raw suspicious content.
    #[must_use]
    pub fn safe_explanation(&self) -> &str {
        self.safe_explanation.as_str()
    }

    /// Returns the redacted evidence hash associated with this finding.
    #[must_use]
    pub fn evidence_hash(&self) -> &str {
        self.evidence_hash.as_str()
    }

    pub(super) const fn blocks_dispatch(&self) -> bool {
        self.disposition.blocks_dispatch()
    }
}

/// Deterministic scanner report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    status: ScannerStatus,
    findings: super::Collection<Finding>,
}

impl Report {
    pub(super) fn complete(findings: super::Collection<Finding>) -> Self {
        Self {
            status: ScannerStatus::Complete,
            findings,
        }
    }

    /// Builds an unavailable scanner report for fail-behavior tests or hosts.
    ///
    /// # Panics
    ///
    /// Panics when `status` is [`ScannerStatus::Complete`]. Use
    /// [`super::scan_review_input`] for completed deterministic reports.
    #[must_use]
    pub fn unavailable(status: ScannerStatus) -> Self {
        assert!(!status.is_complete(), "unavailable scan report must not use complete status");
        Self {
            status,
            findings: super::Collection::new(),
        }
    }

    /// Returns scanner completion status.
    #[must_use]
    pub const fn status(&self) -> ScannerStatus {
        self.status
    }

    /// Returns deterministic findings.
    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        self.findings.as_slice()
    }

    pub(super) fn has_blocking_finding(&self) -> bool {
        self.findings.iter().any(Finding::blocks_dispatch)
    }
}
