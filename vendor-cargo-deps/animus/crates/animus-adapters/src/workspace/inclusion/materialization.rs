/// Ignored-file materialization outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IgnoredFileMaterializationStatus {
    /// File may be materialized by the host adapter.
    Materialized,
    /// File must not be copied.
    CopyDenied,
}

impl IgnoredFileMaterializationStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Materialized => STATUS_MATERIALIZED,
            Self::CopyDenied => STATUS_COPY_DENIED,
        }
    }
}

/// Denial reason for ignored-file materialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IgnoredFileDenialReason {
    /// No denial occurred.
    None,
    /// No matching allowlist entry was present.
    MissingAllowlist,
    /// Secret-bearing file lacked redaction approval.
    SecretWithoutApproval,
    /// Allowlist entry redaction class did not match the request classification.
    RedactionRejected,
}

impl IgnoredFileDenialReason {
    /// Returns the stable denial-reason label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => REASON_NONE,
            Self::MissingAllowlist => REASON_MISSING_ALLOWLIST,
            Self::SecretWithoutApproval => REASON_SECRET_WITHOUT_APPROVAL,
            Self::RedactionRejected => REASON_REDACTION_REJECTED,
        }
    }
}

/// Request to decide whether an ignored or local-only file can be materialized.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IgnoredFileMaterializationRequest<'a> {
    /// Safe file label; not a raw absolute path.
    pub file_label: &'a str,
    /// Host classification for the requested file.
    pub classification: IgnoredFileRedactionClass,
    /// Explicit allowlist entries available to this workspace operation.
    pub allowlist: &'a [IgnoredFileAllowlistEntry],
}

/// Pure ignored-file materialization decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IgnoredFileMaterializationDecision {
    file_label: alloc::string::String,
    status: IgnoredFileMaterializationStatus,
    denial_reason: IgnoredFileDenialReason,
    provenance: Option<alloc::string::String>,
}

impl IgnoredFileMaterializationDecision {
    /// Returns the safe file label.
    #[must_use]
    pub fn file_label(&self) -> &str {
        self.file_label.as_str()
    }

    /// Returns whether materialization is allowed.
    #[must_use]
    pub const fn status(&self) -> IgnoredFileMaterializationStatus {
        self.status
    }

    /// Returns the denial reason, if any.
    #[must_use]
    pub const fn denial_reason(&self) -> IgnoredFileDenialReason {
        self.denial_reason
    }

    /// Returns safe provenance for approved materialization.
    #[must_use]
    pub fn provenance(&self) -> Option<&str> {
        self.provenance.as_deref()
    }
}

/// Plans ignored-file materialization without copying data.
///
/// # Errors
///
/// Returns [`crate::AdapterContractError`] when the request file label is empty or malformed.
pub fn plan_ignored_file_materialization(
    request: IgnoredFileMaterializationRequest<'_>,
) -> Result<IgnoredFileMaterializationDecision, crate::AdapterContractError> {
    debug_assert!(request
        .allowlist
        .iter()
        .all(|entry| !entry.file_label().is_empty()));
    debug_assert!(matches!(
        request.classification,
        IgnoredFileRedactionClass::Public
            | IgnoredFileRedactionClass::LocalConfig
            | IgnoredFileRedactionClass::Secret
    ));

    let file_label =
        crate::validation::normalized_adapter_field(INCLUDE_FILE_FIELD, request.file_label)?;
    let Some(entry) = request
        .allowlist
        .iter()
        .find(|candidate| candidate.file_label() == file_label.as_str())
    else {
        return Ok(denied_decision(
            file_label,
            IgnoredFileDenialReason::MissingAllowlist,
        ));
    };
    if entry.redaction_class() != request.classification {
        return Ok(denied_decision(
            file_label,
            IgnoredFileDenialReason::RedactionRejected,
        ));
    }
    if request.classification == IgnoredFileRedactionClass::Secret
        && entry.approval() != IgnoredFileRedactionApproval::Approved
    {
        return Ok(denied_decision(
            file_label,
            IgnoredFileDenialReason::SecretWithoutApproval,
        ));
    }
    if entry.approval() != IgnoredFileRedactionApproval::Approved {
        return Ok(denied_decision(
            file_label,
            IgnoredFileDenialReason::RedactionRejected,
        ));
    }
    Ok(IgnoredFileMaterializationDecision {
        file_label,
        status: IgnoredFileMaterializationStatus::Materialized,
        denial_reason: IgnoredFileDenialReason::None,
        provenance: Some(alloc::string::String::from(entry.provenance())),
    })
}

fn denied_decision(
    file_label: alloc::string::String,
    denial_reason: IgnoredFileDenialReason,
) -> IgnoredFileMaterializationDecision {
    IgnoredFileMaterializationDecision {
        file_label,
        status: IgnoredFileMaterializationStatus::CopyDenied,
        denial_reason,
        provenance: None,
    }
}
