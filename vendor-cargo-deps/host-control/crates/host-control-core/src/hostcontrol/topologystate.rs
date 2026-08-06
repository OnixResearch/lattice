/// Borrowed topology-resource summary before validation.
// r[impl agent_harness_runtime.host_control.topology_dtos]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostTopologyResourceDraft<'a> {
    /// Resource selector.
    pub selector: &'a HostSelector,
    /// Optional host label.
    pub label: Option<&'a str>,
    /// Optional cwd summary supplied by the host.
    pub cwd_summary: Option<&'a str>,
    /// Optional provenance summary supplied by the host.
    pub provenance_summary: Option<&'a str>,
    /// Whether this resource has host focus.
    pub focused: bool,
    /// Optional semantic status label.
    pub status: Option<&'a str>,
}

/// Validated topology-resource summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostTopologyResource {
    selector: HostSelector,
    label: Option<String>,
    cwd_summary: Option<String>,
    provenance_summary: Option<String>,
    focused: bool,
    status: Option<HostSemanticStatus>,
}

impl HostTopologyResource {
    /// Validates a topology-resource summary.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when text fields or semantic status labels are invalid.
    pub fn validate(draft: HostTopologyResourceDraft<'_>) -> Result<Self, HostControlError> {
        Ok(Self {
            selector: draft.selector.clone(),
            label: optional_text(LABEL_FIELD, draft.label)?,
            cwd_summary: optional_text(CWD_FIELD, draft.cwd_summary)?,
            provenance_summary: optional_text(PROVENANCE_FIELD, draft.provenance_summary)?,
            focused: draft.focused,
            status: draft.status.map(HostSemanticStatus::parse).transpose()?,
        })
    }

    /// Returns the resource selector.
    #[must_use]
    pub const fn selector(&self) -> &HostSelector {
        &self.selector
    }

    /// Returns the optional label.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns the optional cwd summary.
    #[must_use]
    pub fn cwd_summary(&self) -> Option<&str> {
        self.cwd_summary.as_deref()
    }

    /// Returns the optional provenance summary.
    #[must_use]
    pub fn provenance_summary(&self) -> Option<&str> {
        self.provenance_summary.as_deref()
    }

    /// Returns focus state.
    #[must_use]
    pub const fn focused(&self) -> bool {
        self.focused
    }

    /// Returns semantic status when present.
    #[must_use]
    pub const fn status(&self) -> Option<HostSemanticStatus> {
        self.status
    }
}

/// Borrowed topology snapshot before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostTopologySnapshotDraft<'a> {
    /// Host identity.
    pub host_id: &'a str,
    /// Current topology revision.
    pub revision: u64,
    /// Known selectors in the snapshot.
    pub selectors: &'a [HostSelector],
    /// Existing labels in the topology.
    pub labels: &'a [&'a str],
    /// Supported operation labels.
    pub supported_operations: &'a [&'a str],
}

/// Host topology snapshot used for pure command validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostTopologySnapshot {
    host_id: String,
    revision: u64,
    selectors: Vec<HostSelector>,
    labels: Vec<String>,
    supported_operations: Vec<HostOperation>,
}

impl HostTopologySnapshot {
    /// Validates a topology snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when text fields or operation labels are invalid.
    pub fn validate(draft: HostTopologySnapshotDraft<'_>) -> Result<Self, HostControlError> {
        Ok(Self {
            host_id: required_text(HOST_ID_FIELD, draft.host_id)?,
            revision: draft.revision,
            selectors: draft.selectors.to_vec(),
            labels: collect_labels(draft.labels)?,
            supported_operations: draft
                .supported_operations
                .iter()
                .map(|operation| HostOperation::parse(operation))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    /// Returns host identity.
    #[must_use]
    pub fn host_id(&self) -> &str {
        self.host_id.as_str()
    }

    /// Returns current topology revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the topology selectors.
    #[must_use]
    pub fn selectors(&self) -> &[HostSelector] {
        self.selectors.as_slice()
    }

    fn supports(&self, operation: HostOperation) -> bool {
        self.supported_operations.contains(&operation)
    }

    fn has_label(&self, label: &str) -> bool {
        self.labels.iter().any(|existing| existing == label)
    }

    fn contains_selector(&self, selector: &HostSelector) -> bool {
        self.selectors.iter().any(|known| {
            known.host_id == selector.host_id
                && known.resource_id == selector.resource_id
                && known.kind == selector.kind
        })
    }
}

/// Borrowed topology command before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TopologyCommandDraft<'a> {
    /// Operation label.
    pub operation: &'a str,
    /// Optional selector required by most topology operations.
    pub selector: Option<&'a HostSelector>,
    /// Optional label, used by rename-style operations.
    pub label: Option<&'a str>,
}

/// Validated topology command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologyCommand {
    operation: HostOperation,
    selector: Option<HostSelector>,
    label: Option<String>,
}

impl TopologyCommand {
    /// Validates a topology command against a host snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when the operation, selector, or label is invalid.
    pub fn validate(
        snapshot: &HostTopologySnapshot,
        draft: TopologyCommandDraft<'_>,
    ) -> Result<Self, HostControlError> {
        let operation = HostOperation::parse(draft.operation)?;
        if !snapshot.supports(operation) {
            return Err(HostControlError::UnsupportedOperation);
        }
        let selector = validate_selector(snapshot, operation, draft.selector)?;
        let label = match draft.label {
            Some(label) => Some(required_text(LABEL_FIELD, label)?),
            None => None,
        };
        if operation == HostOperation::Rename
            && label
                .as_ref()
                .is_some_and(|value| snapshot.has_label(value))
        {
            return Err(HostControlError::DuplicateLabel);
        }
        Ok(Self {
            operation,
            selector,
            label,
        })
    }

    /// Returns the command operation.
    #[must_use]
    pub const fn operation(&self) -> HostOperation {
        self.operation
    }

    /// Returns the target selector when present.
    #[must_use]
    pub const fn selector(&self) -> Option<&HostSelector> {
        self.selector.as_ref()
    }

    /// Returns the command label when present.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}