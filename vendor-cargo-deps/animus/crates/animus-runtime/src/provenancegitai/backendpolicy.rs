/// Injected version-control provenance backend contract.
pub trait VersionControlProvenanceBackend {
    /// Returns the safe backend kind label.
    fn backend_kind(&self) -> &str;

    /// Records a before/after mutating-tool checkpoint.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when the backend is unavailable, unsupported,
    /// or refuses the safe checkpoint metadata.
    fn record_checkpoint(
        &mut self,
        checkpoint: &ProvenanceToolCheckpoint,
    ) -> Result<ProvenanceToolCheckpointReceipt, ProvenanceError>;
}

/// Backend requirement mode for provenance checkpoint policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvenanceBackendRequirement {
    /// Provenance is explicitly disabled for the tool action.
    Disabled,
    /// Provenance is recorded when a compatible backend is injected.
    Optional,
    /// Provenance must be recorded before complete evidence can be claimed.
    Required,
}

impl ProvenanceBackendRequirement {
    /// Parses a backend requirement label.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when the label is unsupported.
    pub fn parse(value: &str) -> Result<Self, ProvenanceError> {
        let value = validate_safe_text("backend_requirement", value.to_owned())?;
        match value.as_str() {
            PROVENANCE_GIT_AI_BACKEND_DISABLED => Ok(Self::Disabled),
            PROVENANCE_GIT_AI_BACKEND_OPTIONAL => Ok(Self::Optional),
            PROVENANCE_GIT_AI_BACKEND_REQUIRED => Ok(Self::Required),
            other => Err(ProvenanceError::new(
                ProvenanceErrorKind::InvalidPolicy,
                "backend_requirement",
                other,
                "unknown provenance backend requirement",
            )),
        }
    }

    /// Returns the backend requirement label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => PROVENANCE_GIT_AI_BACKEND_DISABLED,
            Self::Optional => PROVENANCE_GIT_AI_BACKEND_OPTIONAL,
            Self::Required => PROVENANCE_GIT_AI_BACKEND_REQUIRED,
        }
    }
}

/// Borrowed mutating-tool policy fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProvenanceToolPolicyDraft<'a> {
    /// Raw tool name to classify.
    pub tool_name: &'a str,
    /// Canonical mutation kind label.
    pub mutation_kind: &'a str,
    /// File-identity field names required by the policy.
    pub file_identity_fields: &'a [&'a str],
    /// Backend requirement label.
    pub backend_requirement: &'a str,
    /// Receipt redaction marker label.
    pub receipt_redaction: &'a str,
}

/// Validated mutating-tool policy entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceToolPolicy {
    /// Raw tool name to classify.
    pub tool_name: Text,
    /// Canonical mutation kind.
    pub mutation_kind: ProvenanceMutationKind,
    /// File-identity field names required by the policy.
    pub file_identity_fields: Collection<Text>,
    /// Backend requirement.
    pub backend_requirement: ProvenanceBackendRequirement,
    /// Receipt redaction marker label.
    pub receipt_redaction: Text,
    /// BLAKE3 policy entry hash over safe fields.
    pub policy_hash: Text,
}

impl ProvenanceToolPolicy {
    /// Validates a mutating-tool policy entry.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when a tool name, mutation kind, file field,
    /// backend requirement, or receipt redaction marker is unsupported.
    pub fn validate(draft: &ProvenanceToolPolicyDraft<'_>) -> Result<Self, ProvenanceError> {
        let tool_name = validate_safe_text("policy_tool_name", draft.tool_name.to_owned())?;
        let mutation_kind = ProvenanceMutationKind::parse(draft.mutation_kind)?;
        if !CANONICAL_MUTATION_KINDS.contains(&mutation_kind.as_str()) {
            return Err(ProvenanceError::new(
                ProvenanceErrorKind::InvalidPolicy,
                "mutation_kind",
                mutation_kind.as_str(),
                "mutation kind is not canonical",
            ));
        }
        let file_identity_fields = validate_file_identity_fields(draft.file_identity_fields)?;
        let backend_requirement = ProvenanceBackendRequirement::parse(draft.backend_requirement)?;
        let receipt_redaction = validate_receipt_redaction(draft.receipt_redaction)?;
        let policy_hash = tool_policy_hash(ToolPolicyHashInput {
            tool_name: tool_name.as_str(),
            mutation_kind: mutation_kind.as_str(),
            file_identity_fields: file_identity_fields.as_slice(),
            backend_requirement: backend_requirement.as_str(),
            receipt_redaction: receipt_redaction.as_str(),
        });
        Ok(Self {
            tool_name,
            mutation_kind,
            file_identity_fields,
            backend_requirement,
            receipt_redaction,
            policy_hash,
        })
    }
}

/// Records a checkpoint according to backend policy.
///
/// # Errors
///
/// Returns [`ProvenanceError`] when required provenance has no backend, a backend
/// is unsupported, or receipt construction fails.
pub fn record_provenance_checkpoint(
    requirement: ProvenanceBackendRequirement,
    backend: Option<&mut dyn VersionControlProvenanceBackend>,
    checkpoint: &ProvenanceToolCheckpoint,
) -> Result<ProvenanceToolCheckpointReceipt, ProvenanceError> {
    match (requirement, backend) {
        (ProvenanceBackendRequirement::Disabled, _) => ProvenanceToolCheckpointReceipt::from_checkpoint(
            PROVENANCE_GIT_AI_DISABLED_STATUS,
            PROVENANCE_GIT_AI_BACKEND_DISABLED,
            checkpoint,
        ),
        (ProvenanceBackendRequirement::Optional, None) => ProvenanceToolCheckpointReceipt::from_checkpoint(
            PROVENANCE_GIT_AI_DISABLED_STATUS,
            PROVENANCE_GIT_AI_BACKEND_OPTIONAL,
            checkpoint,
        ),
        (ProvenanceBackendRequirement::Optional | ProvenanceBackendRequirement::Required, Some(backend)) => {
            backend.record_checkpoint(checkpoint)
        }
        (ProvenanceBackendRequirement::Required, None) => Err(ProvenanceError::new(
            ProvenanceErrorKind::UnavailableBackend,
            "provenance_backend",
            PROVENANCE_GIT_AI_DEFAULT_BACKEND_LABEL,
            "required provenance backend was not injected",
        )),
    }
}

/// Finds a matching mutating-tool policy entry.
#[must_use]
pub fn classify_provenance_tool_policy<'a>(
    policies: &'a [ProvenanceToolPolicy],
    tool_name: &str,
) -> Option<&'a ProvenanceToolPolicy> {
    policies.iter().find(|policy| policy.tool_name == tool_name)
}

/// Validates that reusable provenance DTO field names remain VCS-neutral.
///
/// # Errors
///
/// Returns [`ProvenanceError`] if a required field names Git notes, commit SHAs,
/// branch, stash, rebase, Git command, or Git hook semantics.
pub fn validate_vcs_neutral_field_names(fields: &[&str]) -> Result<(), ProvenanceError> {
    for field in fields {
        let checked = validate_safe_text("core_field_name", (*field).to_owned())?;
        let lowered = checked.to_ascii_lowercase();
        if let Some(needle) = GIT_TERM_NEEDLES
            .iter()
            .copied()
            .find(|needle| lowered.contains(needle))
        {
            return Err(ProvenanceError::new(
                ProvenanceErrorKind::GitTermInCoreDto,
                "core_field_name",
                needle,
                "Git-specific field belongs in an optional compatibility adapter",
            ));
        }
    }
    Ok(())
}
fn validate_file_identity_fields(values: &[&str]) -> Result<Collection<Text>, ProvenanceError> {
    if values.is_empty() {
        return Err(ProvenanceError::new(
            ProvenanceErrorKind::InvalidPolicy,
            "file_identity_fields",
            UNKNOWN_SAFE_FIELD,
            "file identity fields must not be empty",
        ));
    }
    let fields = validate_safe_list("file_identity_field", values)?;
    validate_vcs_neutral_field_names(values)?;
    Ok(fields)
}

fn validate_receipt_redaction(value: &str) -> Result<Text, ProvenanceError> {
    let value = validate_safe_text("receipt_redaction", value.to_owned())?;
    if SAFE_RECEIPT_REDACTIONS.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(ProvenanceError::new(
            ProvenanceErrorKind::InvalidPolicy,
            "receipt_redaction",
            value,
            "unsupported receipt redaction marker",
        ))
    }
}

#[derive(Clone, Copy)]
struct CheckpointReceiptHashInput<'a> {
    status: &'a str,
    backend_kind: &'a str,
    mutation_kind: ProvenanceMutationKind,
    session_id: &'a str,
    trace_id: &'a str,
    before_hash: &'a str,
    after_hash: &'a str,
}

#[derive(Clone, Copy)]
struct ToolPolicyHashInput<'a> {
    tool_name: &'a str,
    mutation_kind: &'a str,
    file_identity_fields: &'a [Text],
    backend_requirement: &'a str,
    receipt_redaction: &'a str,
}

fn checkpoint_receipt_hash(input: CheckpointReceiptHashInput<'_>) -> Text {
    let mut normalized = Text::from(PROVENANCE_GIT_AI_RECEIPT_SCHEMA);
    push_hash_part(&mut normalized, input.status);
    push_hash_part(&mut normalized, input.backend_kind);
    push_hash_part(&mut normalized, input.mutation_kind.as_str());
    push_hash_part(&mut normalized, input.session_id);
    push_hash_part(&mut normalized, input.trace_id);
    push_hash_part(&mut normalized, input.before_hash);
    push_hash_part(&mut normalized, input.after_hash);
    blake3_hex(normalized.as_bytes())
}

fn tool_policy_hash(input: ToolPolicyHashInput<'_>) -> Text {
    let mut normalized = Text::from(PROVENANCE_GIT_AI_POLICY_SCHEMA);
    push_hash_part(&mut normalized, input.tool_name);
    push_hash_part(&mut normalized, input.mutation_kind);
    for field in input.file_identity_fields {
        push_hash_part(&mut normalized, field.as_str());
    }
    push_hash_part(&mut normalized, input.backend_requirement);
    push_hash_part(&mut normalized, input.receipt_redaction);
    blake3_hex(normalized.as_bytes())
}
