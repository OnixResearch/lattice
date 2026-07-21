
impl NativeSessionUniquenessScope {
    /// Returns the stable scope label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::Workspace => "workspace",
            Self::Agent => "agent",
        }
    }
}

/// Native resume capability label.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeResumeCapability {
    /// Resume is supported.
    Supported,
    /// Resume is unsupported.
    Unsupported,
}

/// Borrowed native session reference before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeSessionReferenceDraft<'a> {
    /// Safe source label, if present.
    pub source_label: Option<&'a str>,
    /// Safe agent label, if present.
    pub agent_label: Option<&'a str>,
    /// Session reference kind.
    pub kind: NativeSessionReferenceKind,
    /// Lowercase BLAKE3 identity for the reference value.
    pub value_hash: Option<&'a str>,
    /// Freshness status.
    pub freshness: NativeSessionFreshness,
    /// Uniqueness scope.
    pub uniqueness_scope: NativeSessionUniquenessScope,
    /// Resume capability.
    pub resume_capability: NativeResumeCapability,
    /// Sensitive raw value, forbidden in reusable records.
    pub sensitive_value: Option<&'a str>,
}

/// Safe native session reference record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeSessionReference {
    source_label: String,
    agent_label: String,
    kind: NativeSessionReferenceKind,
    value_hash: crate::artifact::Hash,
    freshness: NativeSessionFreshness,
    uniqueness_scope: NativeSessionUniquenessScope,
    resume_capability: NativeResumeCapability,
}

impl NativeSessionReference {
    /// Validates safe native session reference metadata.
    ///
    /// # Errors
    ///
    /// Returns [`SessionRestoreError`] when required safe fields are missing,
    /// malformed, or sensitive.
    // r[impl agent_harness_runtime.session_restore.native_session_reference]
    pub fn validate(draft: &NativeSessionReferenceDraft<'_>) -> Result<Self, SessionRestoreError> {
        if draft.sensitive_value.is_some() {
            return Err(SessionRestoreError::InvalidNativeReference {
                reason: RestoreDegradedReason::SensitiveNativeReference,
            });
        }
        let source_label =
            draft
                .source_label
                .ok_or(SessionRestoreError::InvalidNativeReference {
                    reason: RestoreDegradedReason::MissingNativeReference,
                })?;
        let agent_label = draft
            .agent_label
            .ok_or(SessionRestoreError::InvalidNativeReference {
                reason: RestoreDegradedReason::MissingNativeReference,
            })?;
        let value_hash = draft
            .value_hash
            .ok_or(SessionRestoreError::InvalidNativeReference {
                reason: RestoreDegradedReason::MissingNativeReference,
            })?;
        Ok(Self {
            source_label: normalize(SESSION_SOURCE_FIELD, source_label)?,
            agent_label: normalize(AGENT_LABEL_FIELD, agent_label)?,
            kind: draft.kind,
            value_hash: crate::artifact::Hash::new(value_hash).map_err(|_| {
                SessionRestoreError::InvalidNativeReference {
                    reason: RestoreDegradedReason::MalformedNativeReference,
                }
            })?,
            freshness: draft.freshness,
            uniqueness_scope: draft.uniqueness_scope,
            resume_capability: draft.resume_capability,
        })
    }

    /// Returns source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns agent label.
    #[must_use]
    pub fn agent_label(&self) -> &str {
        self.agent_label.as_str()
    }

    /// Returns reference kind.
    #[must_use]
    pub const fn kind(&self) -> NativeSessionReferenceKind {
        self.kind
    }

    /// Returns value hash.
    #[must_use]
    pub const fn value_hash(&self) -> &crate::artifact::Hash {
        &self.value_hash
    }

    /// Returns freshness.
    #[must_use]
    pub const fn freshness(&self) -> NativeSessionFreshness {
        self.freshness
    }

    /// Returns uniqueness scope.
    #[must_use]
    pub const fn uniqueness_scope(&self) -> NativeSessionUniquenessScope {
        self.uniqueness_scope
    }

    /// Returns resume capability.
    #[must_use]
    pub const fn resume_capability(&self) -> NativeResumeCapability {
        self.resume_capability
    }
}

/// History redaction state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HistoryRedactionStatus {
    /// No history was present.
    NotPresent,
    /// History was redacted.
    Redacted,
    /// History is unredacted and unsafe for replay.
    Unredacted,
}

/// History storage classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HistoryStorageClass {
    /// No stored history.
    None,
    /// Ephemeral host storage.
    Ephemeral,
    /// Persistent host storage.
    Persistent,
}

/// Borrowed history replay metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistoryReplayDraft<'a> {
    /// Whether replay is enabled in the snapshot.
    pub enabled: bool,
    /// Bounded display line count.
    pub bounded_line_count: u32,
    /// Whether replay was truncated.
    pub truncated: bool,
    /// Redaction status.
    pub redaction_status: HistoryRedactionStatus,
    /// Storage class.
    pub storage_class: HistoryStorageClass,
    /// Whether the host tried to mark replay as model-visible context.
    pub model_visible: bool,
    /// Secret-looking raw history, forbidden.
    pub secret_history: Option<&'a str>,
}

/// Safe bounded history replay metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryReplay {
    enabled: bool,
    bounded_line_count: u32,
    truncated: bool,
    redaction_status: HistoryRedactionStatus,
    storage_class: HistoryStorageClass,
    model_visible: bool,
}

impl HistoryReplay {
    /// Validates bounded display-only history replay metadata.
    ///
    /// # Errors
    ///
    /// Returns [`SessionRestoreError`] when unsafe history is present.
    // r[impl agent_harness_runtime.session_restore.history_replay]
    pub fn validate(draft: &HistoryReplayDraft<'_>) -> Result<Self, SessionRestoreError> {
        if draft.secret_history.is_some()
            || (draft.enabled && draft.redaction_status == HistoryRedactionStatus::Unredacted)
        {
            return Err(SessionRestoreError::UnsafeHistoryReplay);
        }
        Ok(Self {
            enabled: draft.enabled,
            bounded_line_count: draft.bounded_line_count,
            truncated: draft.truncated,
            redaction_status: draft.redaction_status,
            storage_class: draft.storage_class,
            model_visible: draft.model_visible,
        })
    }

    /// Returns whether replay is enabled.
    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns bounded display line count.
    #[must_use]
    pub const fn bounded_line_count(&self) -> u32 {
        self.bounded_line_count
    }

    /// Returns whether replay was truncated.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    /// Returns redaction status.
    #[must_use]
    pub const fn redaction_status(&self) -> HistoryRedactionStatus {
        self.redaction_status
    }

    /// Returns storage classification.
    #[must_use]
    pub const fn storage_class(&self) -> HistoryStorageClass {
        self.storage_class
    }

    /// Returns whether replay may be model-visible.
    #[must_use]
    pub const fn model_visible(&self) -> bool {
        self.model_visible
    }
}
