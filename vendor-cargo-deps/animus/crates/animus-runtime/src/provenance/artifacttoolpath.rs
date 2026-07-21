/// Artifact kind for provenance identity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProvenanceArtifactKind {
    /// Model request or response boundary artifact.
    Model,
    /// Tool request or response boundary artifact.
    Tool,
    /// Runtime metadata artifact.
    Runtime,
    /// Documentation or review artifact.
    Documentation,
    /// Interchange projection artifact.
    Interchange,
}

impl ProvenanceArtifactKind {
    /// Returns the stable artifact-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Tool => "tool",
            Self::Runtime => "runtime",
            Self::Documentation => "documentation",
            Self::Interchange => "interchange",
        }
    }
}

/// Artifact redaction class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProvenanceRedactionClass {
    /// Payload may be displayed by default.
    Public,
    /// Payload is omitted and only metadata is displayed.
    MetadataOnly,
    /// Payload exists but is replaced by a redaction marker.
    RedactedPayload,
    /// Payload is secret and must not be displayed.
    Secret,
}

impl ProvenanceRedactionClass {
    /// Returns the stable redaction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => PUBLIC_REDACTION_MARKER,
            Self::MetadataOnly => METADATA_ONLY_REDACTION_MARKER,
            Self::RedactedPayload => REDACTED_PAYLOAD_MARKER,
            Self::Secret => SECRET_REDACTION_MARKER,
        }
    }

    /// Returns true when payload bodies may be displayed by default.
    #[must_use]
    pub const fn allows_payload_display(self) -> bool {
        matches!(self, Self::Public)
    }
}

/// Borrowed artifact identity draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceArtifactDraft<'a> {
    /// Artifact kind.
    pub kind: ProvenanceArtifactKind,
    /// Redaction class.
    pub redaction: ProvenanceRedactionClass,
    /// Normalized payload used for hashing.
    pub normalized_payload: &'a str,
    /// Dependency artifact hashes.
    pub dependencies: &'a [&'a str],
}

/// Canonical BLAKE3 artifact identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceArtifact {
    /// Artifact kind.
    pub kind: ProvenanceArtifactKind,
    /// Redaction class.
    pub redaction: ProvenanceRedactionClass,
    /// Canonical envelope schema version.
    pub schema_version: u16,
    /// Sorted and deduplicated dependency hashes.
    pub dependencies: Collection<Text>,
    /// Deterministic BLAKE3 artifact hash.
    pub artifact_hash: Text,
    payload: Text,
}

impl ProvenanceArtifact {
    /// Builds a canonical artifact identity.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when any dependency hash is invalid.
    pub fn from_draft(draft: &ProvenanceArtifactDraft<'_>) -> Result<Self, ProvenanceError> {
        let mut dependencies = Collection::new();
        for dependency in draft.dependencies {
            dependencies.push(validate_hash("artifact_dependency", dependency)?.to_owned());
        }
        dependencies.sort();
        dependencies.dedup();
        let artifact_hash = artifact_hash(
            draft.kind,
            draft.redaction,
            draft.normalized_payload,
            dependencies.as_slice(),
        );
        Ok(Self {
            kind: draft.kind,
            redaction: draft.redaction,
            schema_version: PROVENANCE_ARTIFACT_SCHEMA_VERSION,
            dependencies,
            artifact_hash,
            payload: draft.normalized_payload.to_owned(),
        })
    }

    /// Returns the payload when the redaction class allows display.
    #[must_use]
    pub fn review_payload(&self) -> Option<&str> {
        if self.redaction.allows_payload_display() {
            Some(self.payload.as_str())
        } else {
            None
        }
    }

    /// Returns the redaction marker for receipt or review output.
    #[must_use]
    pub const fn redaction_marker(&self) -> &'static str {
        self.redaction.as_str()
    }

    /// Parses and validates a BLAKE3 artifact hash.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] if the value is not a 64-character
    /// lowercase hexadecimal digest.
    pub fn parse_hash(value: &str) -> Result<Text, ProvenanceError> {
        Ok(validate_hash("artifact_hash", value)?.to_owned())
    }
}

/// Toolpath projection policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolpathExportPolicy {
    /// Default redacted export; full-fidelity raw payload export is unavailable.
    RedactedDefault,
}

/// Redacted Toolpath-compatible step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolpathStep {
    /// Step identity.
    pub id: Text,
    /// Parent step identities.
    pub parents: Collection<Text>,
    /// Toolpath-compatible actor label.
    pub actor: Text,
    /// Safe change kind.
    pub change_kind: Text,
    /// BLAKE3 artifact hashes referenced by this step.
    pub artifact_hashes: Collection<Text>,
    /// Safe typed refs and correlation labels.
    pub refs: Collection<Text>,
    /// Redaction markers proving raw material was omitted.
    pub redaction_markers: Collection<Text>,
}

/// Redacted Toolpath Graph/Path/Step projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolpathProjection {
    /// Projection schema label.
    pub schema: Text,
    /// Active path head.
    pub path_head: Text,
    /// Redacted steps.
    pub steps: Collection<ToolpathStep>,
    /// Dead-end step ids.
    pub dead_end_steps: Collection<Text>,
}

/// Exports redacted Animus provenance into a Toolpath-compatible projection.
///
/// # Errors
///
/// Returns [`ProvenanceError`] on malformed graph data or unsupported policy.
pub fn export_toolpath_projection(
    entries: &[ProvenanceLedgerEntry],
    active_head: &str,
    policy: ToolpathExportPolicy,
) -> Result<ToolpathProjection, ProvenanceError> {
    match policy {
        ToolpathExportPolicy::RedactedDefault => {
            let graph_entries: Collection<ProvenanceGraphEntry> = entries
                .iter()
                .map(ProvenanceLedgerEntry::graph_entry)
                .collect();
            let branch_report = validate_provenance_graph(graph_entries.as_slice(), active_head)?;
            let mut steps = Collection::new();
            for entry in entries {
                steps.push(project_toolpath_step(entry)?);
            }
            Ok(ToolpathProjection {
                schema: TOOLPATH_PROJECTION_SCHEMA.to_owned(),
                path_head: active_head.to_owned(),
                steps,
                dead_end_steps: branch_report.dead_end_entry_ids,
            })
        }
    }
}

fn project_toolpath_step(entry: &ProvenanceLedgerEntry) -> Result<ToolpathStep, ProvenanceError> {
    let parents = entry.parents().to_vec();
    match entry {
        ProvenanceLedgerEntry::Message(message) => Ok(ToolpathStep {
            id: message.id.clone(),
            parents,
            actor: message.actor.toolpath_label(),
            change_kind: message.role.as_str().to_owned(),
            artifact_hashes: vec![
                validate_hash("content_hash", message.content_hash.as_str())?.to_owned(),
            ],
            refs: Collection::new(),
            redaction_markers: if contains_unsafe_material(message.content.as_str()).is_some() {
                vec![TOOLPATH_REDACTED_PROMPT_MARKER.to_owned()]
            } else {
                vec![REDACTED_PAYLOAD_MARKER.to_owned()]
            },
        }),
        ProvenanceLedgerEntry::Summary(summary) => Ok(ToolpathStep {
            id: summary.id.clone(),
            parents,
            actor: "agent:summary".to_owned(),
            change_kind: ProvenanceReplayRole::Summary.as_str().to_owned(),
            artifact_hashes: vec![
                validate_hash("summary_hash", summary.summary_hash.as_str())?.to_owned(),
            ],
            refs: Collection::new(),
            redaction_markers: vec![TOOLPATH_REDACTED_PROMPT_MARKER.to_owned()],
        }),
        ProvenanceLedgerEntry::Usage(usage) => Ok(ToolpathStep {
            id: usage.id.clone(),
            parents,
            actor: "ci:usage".to_owned(),
            change_kind: "usage".to_owned(),
            artifact_hashes: Collection::new(),
            refs: vec![usage.model_label.clone()],
            redaction_markers: vec![METADATA_ONLY_REDACTION_MARKER.to_owned()],
        }),
        ProvenanceLedgerEntry::Receipt(receipt) => Ok(ToolpathStep {
            id: receipt.id.clone(),
            parents,
            actor: receipt.actor_label.clone(),
            change_kind: receipt.status.clone(),
            artifact_hashes: receipt.artifact_hashes.clone(),
            refs: receipt
                .issue_code
                .iter()
                .chain(receipt.backend_label.iter())
                .chain(receipt.interchange_label.iter())
                .cloned()
                .collect(),
            redaction_markers: receipt.redaction_markers.clone(),
        }),
        ProvenanceLedgerEntry::Unsupported(unsupported) => Ok(ToolpathStep {
            id: unsupported.id.clone(),
            parents,
            actor: format!("agent:{}", unsupported.adapter_label),
            change_kind: "unsupported".to_owned(),
            artifact_hashes: vec![
                validate_hash("payload_hash", unsupported.payload_hash.as_str())?.to_owned(),
            ],
            refs: Collection::new(),
            redaction_markers: vec![TOOLPATH_REDACTED_ARTIFACT_MARKER.to_owned()],
        }),
    }
}
