/// Borrowed Git AI v3 file-attestation fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProvenanceGitAiFileAttestationDraft<'a> {
    /// Repository-relative file path from the compatibility log.
    pub relative_path: &'a str,
    /// Attributed line ranges from the compatibility log.
    pub line_ranges: &'a [ProvenanceLineRange],
    /// External Git AI/SHA-256-compatible file key.
    pub external_file_key: &'a str,
}

/// Git AI v3 file-attestation DTO at the optional adapter boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceGitAiFileAttestation {
    /// Repository-relative file path from the compatibility log.
    pub relative_path: Text,
    /// Attributed line ranges from the compatibility log.
    pub line_ranges: Collection<ProvenanceLineRange>,
    /// External Git AI/SHA-256-compatible file key.
    pub external_file_key: Text,
}

/// Borrowed Git AI metadata map entry before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProvenanceGitAiMetadataDraft<'a> {
    /// Metadata key.
    pub key: &'a str,
    /// Metadata value.
    pub value: &'a str,
}

/// Validated Git AI metadata map entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceGitAiMetadata {
    /// Metadata key.
    pub key: Text,
    /// Metadata value.
    pub value: Text,
}

/// Borrowed Git AI v3 authorship log fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProvenanceGitAiAuthorshipLogDraft<'a> {
    /// External schema version. Must be `authorship/3.0.0`.
    pub schema_version: &'a str,
    /// Git notes storage ref. Must be `refs/notes/ai`.
    pub storage_ref: &'a str,
    /// External Git AI/SHA-256-compatible session key.
    pub session_key: &'a str,
    /// External Git AI/SHA-256-compatible trace key.
    pub trace_key: &'a str,
    /// External Git AI/SHA-256-compatible known-human key.
    pub known_human_key: &'a str,
    /// External Git AI/SHA-256-compatible legacy prompt key.
    pub legacy_prompt_key: &'a str,
    /// File attestations.
    pub file_attestations: &'a [ProvenanceGitAiFileAttestationDraft<'a>],
    /// Metadata map entries.
    pub metadata: &'a [ProvenanceGitAiMetadataDraft<'a>],
}

/// Validated Git AI v3 authorship log at the optional adapter boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceGitAiAuthorshipLog {
    /// External schema version.
    pub schema_version: Text,
    /// Git notes storage ref.
    pub storage_ref: Text,
    /// External Git AI/SHA-256-compatible session key.
    pub session_key: Text,
    /// External Git AI/SHA-256-compatible trace key.
    pub trace_key: Text,
    /// External Git AI/SHA-256-compatible known-human key.
    pub known_human_key: Text,
    /// External Git AI/SHA-256-compatible legacy prompt key.
    pub legacy_prompt_key: Text,
    /// File attestations.
    pub file_attestations: Collection<ProvenanceGitAiFileAttestation>,
    /// Metadata map entries.
    pub metadata: Collection<ProvenanceGitAiMetadata>,
}

impl ProvenanceGitAiAuthorshipLog {
    /// Validates a Git AI v3 compatibility log.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when external schema, storage ref, external
    /// IDs, metadata, or file attestations are malformed.
    pub fn validate(
        draft: &ProvenanceGitAiAuthorshipLogDraft<'_>,
    ) -> Result<Self, ProvenanceError> {
        validate_exact_compat_label("schema_version", draft.schema_version, PROVENANCE_GIT_AI_SCHEMA_VERSION)?;
        validate_exact_compat_label("storage_ref", draft.storage_ref, PROVENANCE_GIT_AI_NOTES_REF)?;
        Ok(Self {
            schema_version: draft.schema_version.to_owned(),
            storage_ref: draft.storage_ref.to_owned(),
            session_key: validate_external_compat_id("session_key", draft.session_key)?,
            trace_key: validate_external_compat_id("trace_key", draft.trace_key)?,
            known_human_key: validate_external_compat_id("known_human_key", draft.known_human_key)?,
            legacy_prompt_key: validate_external_compat_id(
                "legacy_prompt_key",
                draft.legacy_prompt_key,
            )?,
            file_attestations: validate_git_ai_file_attestations(draft.file_attestations)?,
            metadata: validate_git_ai_metadata(draft.metadata)?,
        })
    }

    /// Returns the compatibility adapter label.
    #[must_use]
    pub const fn adapter_label() -> &'static str {
        PROVENANCE_GIT_AI_COMPAT_ADAPTER_LABEL
    }
}
fn validate_exact_compat_label(
    field: &'static str,
    value: &str,
    expected: &str,
) -> Result<(), ProvenanceError> {
    if value == expected {
        Ok(())
    } else {
        Err(ProvenanceError::new(
            ProvenanceErrorKind::InvalidCompatibility,
            field,
            value,
            format!("expected {expected}"),
        ))
    }
}

fn validate_external_compat_id(field: &'static str, value: &str) -> Result<Text, ProvenanceError> {
    if value.len() != EXTERNAL_COMPAT_HASH_HEX_LENGTH
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(ProvenanceError::new(
            ProvenanceErrorKind::InvalidCompatibility,
            field,
            UNKNOWN_SAFE_FIELD,
            "expected external compatibility hex key",
        ));
    }
    Ok(value.to_owned())
}

fn validate_git_ai_file_attestations(
    drafts: &[ProvenanceGitAiFileAttestationDraft<'_>],
) -> Result<Collection<ProvenanceGitAiFileAttestation>, ProvenanceError> {
    let mut attestations = Collection::with_capacity(drafts.len());
    for draft in drafts {
        validate_repository_relative_path(draft.relative_path)?;
        attestations.push(ProvenanceGitAiFileAttestation {
            relative_path: validate_safe_text("git_ai_relative_path", draft.relative_path.to_owned())?,
            line_ranges: compress_line_ranges(draft.line_ranges)?,
            external_file_key: validate_external_compat_id(
                "external_file_key",
                draft.external_file_key,
            )?,
        });
    }
    Ok(attestations)
}

fn validate_git_ai_metadata(
    drafts: &[ProvenanceGitAiMetadataDraft<'_>],
) -> Result<Collection<ProvenanceGitAiMetadata>, ProvenanceError> {
    let mut metadata = Collection::with_capacity(drafts.len());
    for draft in drafts {
        metadata.push(ProvenanceGitAiMetadata {
            key: validate_safe_text("git_ai_metadata_key", draft.key.to_owned())?,
            value: validate_safe_text("git_ai_metadata_value", draft.value.to_owned())?,
        });
    }
    Ok(metadata)
}
