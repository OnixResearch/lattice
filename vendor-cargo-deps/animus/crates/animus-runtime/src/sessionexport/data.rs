//! Typed redacted session-tree export values.

/// Generated policy schema identity.
pub const SESSION_TREE_EXPORT_POLICY_SCHEMA: &str = "animus.session-tree-export-policy.v1";
/// BLAKE3 domain for generated policy content.
pub const SESSION_TREE_EXPORT_POLICY_HASH_DOMAIN: &str = "animus.session-tree-export-policy-content.v1";
/// BLAKE3 domain for validated source identity.
pub const SESSION_TREE_EXPORT_SOURCE_HASH_DOMAIN: &str = "animus.session-tree-export-source.v1";
/// BLAKE3 domain for redacted tree projection identity.
pub const SESSION_TREE_EXPORT_PROJECTION_HASH_DOMAIN: &str = "animus.session-tree-export-projection.v1";
/// BLAKE3 domain for rendered artifact identity.
pub const SESSION_TREE_EXPORT_ARTIFACT_HASH_DOMAIN: &str = "animus.session-tree-export-artifact.v1";
/// BLAKE3 domain for export receipts.
pub const SESSION_TREE_EXPORT_RECEIPT_HASH_DOMAIN: &str = "animus.session-tree-export-receipt.v1";
/// Receipt schema identity.
pub const SESSION_TREE_EXPORT_RECEIPT_SCHEMA: &str = "animus.session-tree-export-receipt.v1";

pub(super) const BLAKE3_HEX_CHARS: usize = 64;
pub(super) const HASH_SEPARATOR: &str = "\u{1f}";
pub(super) const MAX_ENTRY_COUNT: u32 = 4_096;
pub(super) const MAX_DEPTH: u32 = 128;
pub(super) const MAX_SUMMARY_CHARS: u32 = 512;
pub(super) const MAX_ARTIFACT_BYTES: u32 = 8_388_608;
pub(super) const MAX_RECEIPT_REFERENCE_COUNT: u32 = 256;
pub(super) const MIN_COUNT: u32 = 1;
pub(super) const PRIVATE_FILE_MODE: u32 = 0o600;
pub(super) const REDACTION_MARKER: &str =
    "raw-prompts-tool-bodies-reasoning-provider-payloads-credentials-and-paths-omitted";
pub(super) const NON_CLAIM: &str = "export identity proves deterministic projection of supplied validated metadata, not session correctness, omitted-content completeness, behavioral correctness, release eligibility, or a training dataset";

/// Redacted entry kind admitted by export policy.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SessionTreeExportEntryKind {
    /// User body exists but is intentionally omitted.
    UserOmitted,
    /// Assistant body exists but is intentionally omitted.
    AssistantOmitted,
    /// Safe tool lifecycle metadata.
    ToolStatus,
    /// Compaction boundary metadata.
    Compaction,
    /// Interrupted-tool repair metadata.
    Repair,
    /// Safe branch summary metadata.
    BranchSummary,
    /// Safe system status metadata.
    SystemStatus,
}

impl SessionTreeExportEntryKind {
    /// Parses a stable policy label.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "user_omitted" => Some(Self::UserOmitted),
            "assistant_omitted" => Some(Self::AssistantOmitted),
            "tool_status" => Some(Self::ToolStatus),
            "compaction" => Some(Self::Compaction),
            "repair" => Some(Self::Repair),
            "branch_summary" => Some(Self::BranchSummary),
            "system_status" => Some(Self::SystemStatus),
            _other => None,
        }
    }

    /// Returns the stable policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserOmitted => "user_omitted",
            Self::AssistantOmitted => "assistant_omitted",
            Self::ToolStatus => "tool_status",
            Self::Compaction => "compaction",
            Self::Repair => "repair",
            Self::BranchSummary => "branch_summary",
            Self::SystemStatus => "system_status",
        }
    }
}

/// Supported artifact format.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SessionTreeExportFormat {
    /// Script-free self-contained HTML.
    Html,
    /// Safe line-oriented metadata.
    SafeLines,
}

impl SessionTreeExportFormat {
    /// Parses a stable policy label.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "html" => Some(Self::Html),
            "safe_lines" => Some(Self::SafeLines),
            _other => None,
        }
    }

    /// Returns the stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::SafeLines => "safe_lines",
        }
    }
}

/// Host-loaded generated policy input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionTreeExportPolicyInput {
    /// Schema identity.
    pub schema: String,
    /// Policy identity.
    pub policy_id: String,
    /// Maximum entry count.
    pub max_entries: u32,
    /// Maximum parent depth.
    pub max_depth: u32,
    /// Maximum safe-summary characters.
    pub max_summary_chars: u32,
    /// Maximum HTML artifact bytes.
    pub max_html_bytes: u32,
    /// Maximum safe-lines artifact bytes.
    pub max_safe_lines_bytes: u32,
    /// Allowed entry-kind labels.
    pub allowed_entry_kinds: Vec<String>,
    /// Allowed format labels.
    pub formats: Vec<String>,
    /// Required sibling-branch visibility posture.
    pub branch_visibility: String,
    /// Required tool metadata posture.
    pub tool_metadata_posture: String,
    /// Required custom-entry posture.
    pub custom_entry_posture: String,
    /// Exact safe receipt field classes.
    pub receipt_fields: Vec<String>,
    /// Required content posture.
    pub content_posture: String,
    /// Required path posture.
    pub path_posture: String,
    /// Required malformed-input posture.
    pub malformed_input: String,
    /// Required script posture.
    pub scripts: String,
    /// Required remote-resource posture.
    pub remote_resources: String,
    /// Canonical BLAKE3 policy identity.
    pub policy_hash: String,
}

/// Validated generated policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionTreeExportPolicy {
    pub(super) policy_id: String,
    pub(super) max_entries: u32,
    pub(super) max_depth: u32,
    pub(super) max_summary_chars: u32,
    pub(super) max_html_bytes: u32,
    pub(super) max_safe_lines_bytes: u32,
    pub(super) allowed_entry_kinds: Vec<SessionTreeExportEntryKind>,
    pub(super) formats: Vec<SessionTreeExportFormat>,
    pub(super) branch_visibility: String,
    pub(super) tool_metadata_posture: String,
    pub(super) custom_entry_posture: String,
    pub(super) receipt_fields: Vec<String>,
    pub(super) policy_hash: String,
}

impl SessionTreeExportPolicy {
    /// Returns the policy identity.
    #[must_use]
    pub fn policy_id(&self) -> &str {
        self.policy_id.as_str()
    }

    /// Returns the policy BLAKE3 identity.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns the maximum artifact bytes for a format.
    #[must_use]
    pub const fn artifact_byte_limit(&self, format: SessionTreeExportFormat) -> u32 {
        match format {
            SessionTreeExportFormat::Html => self.max_html_bytes,
            SessionTreeExportFormat::SafeLines => self.max_safe_lines_bytes,
        }
    }
}

/// Already-loaded safe session entry input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionTreeExportEntryInput {
    /// Stable entry identity.
    pub entry_id: String,
    /// Optional parent entry identity.
    pub parent_id: Option<String>,
    /// Stable storage order, starting at one.
    pub storage_sequence: u32,
    /// Redacted entry kind.
    pub kind: SessionTreeExportEntryKind,
    /// Exact prior entry repaired by a repair marker.
    pub repair_target_id: Option<String>,
    /// Safe bounded summary.
    pub safe_summary: String,
    /// BLAKE3 identity of the validated source entry.
    pub entry_hash: String,
    /// Safe status class.
    pub status: String,
    /// Count of intentionally omitted private fields.
    pub omitted_field_count: u32,
}

/// Validated session source input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionTreeExportSourceInput {
    /// Safe session identity.
    pub session_id: String,
    /// BLAKE3 snapshot identity.
    pub snapshot_hash: String,
    /// BLAKE3 replay-report identity.
    pub replay_hash: String,
    /// Selected active leaf identity.
    pub active_leaf_id: String,
    /// Entries in supplied storage order.
    pub entries: Vec<SessionTreeExportEntryInput>,
    /// Optional safe receipt identities.
    pub safe_receipt_hashes: Vec<String>,
}

/// One redacted projected entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionTreeExportEntry {
    pub(super) entry_id: String,
    pub(super) parent_id: Option<String>,
    pub(super) storage_sequence: u32,
    pub(super) depth: u32,
    pub(super) child_count: u32,
    pub(super) kind: SessionTreeExportEntryKind,
    pub(super) repair_target_id: Option<String>,
    pub(super) safe_summary: String,
    pub(super) entry_hash: String,
    pub(super) status: String,
    pub(super) omitted_field_count: u32,
    pub(super) is_active_path: bool,
}

impl SessionTreeExportEntry {
    /// Returns the entry identity.
    #[must_use]
    pub fn entry_id(&self) -> &str {
        self.entry_id.as_str()
    }

    /// Returns whether this entry is on the selected active path.
    #[must_use]
    pub const fn is_active_path(&self) -> bool {
        self.is_active_path
    }

    /// Returns the direct child count.
    #[must_use]
    pub const fn child_count(&self) -> u32 {
        self.child_count
    }
}

/// Validated redacted session-tree projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionTreeExportProjection {
    pub(super) session_id: String,
    pub(super) active_leaf_id: String,
    pub(super) source_hash: String,
    pub(super) policy_hash: String,
    pub(super) entries: Vec<SessionTreeExportEntry>,
    pub(super) active_path_count: u32,
    pub(super) branch_point_count: u32,
    pub(super) omission_count: u32,
    pub(super) projection_hash: String,
}

impl SessionTreeExportProjection {
    /// Returns projected entries in deterministic storage order.
    #[must_use]
    pub fn entries(&self) -> &[SessionTreeExportEntry] {
        self.entries.as_slice()
    }

    /// Returns the active-path entry count.
    #[must_use]
    pub const fn active_path_count(&self) -> u32 {
        self.active_path_count
    }

    /// Returns the branch-point count.
    #[must_use]
    pub const fn branch_point_count(&self) -> u32 {
        self.branch_point_count
    }

    /// Returns the projection BLAKE3 identity.
    #[must_use]
    pub fn projection_hash(&self) -> &str {
        self.projection_hash.as_str()
    }
}

/// Rendered redacted artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionTreeExportArtifact {
    pub(super) format: SessionTreeExportFormat,
    pub(super) bytes: Vec<u8>,
    pub(super) projection_hash: String,
    pub(super) byte_count: u32,
    pub(super) artifact_hash: String,
}

impl SessionTreeExportArtifact {
    /// Returns the artifact format.
    #[must_use]
    pub const fn format(&self) -> SessionTreeExportFormat {
        self.format
    }

    /// Returns artifact bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    /// Returns the artifact byte count.
    #[must_use]
    pub const fn byte_count(&self) -> u32 {
        self.byte_count
    }

    /// Returns the artifact BLAKE3 identity.
    #[must_use]
    pub fn artifact_hash(&self) -> &str {
        self.artifact_hash.as_str()
    }
}

/// Host-supplied sensitive values that receipts must reject.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SessionTreeExportSensitiveDraft<'a> {
    /// Raw prompt or assistant body.
    pub raw_content: Option<&'a str>,
    /// Raw tool body.
    pub tool_body: Option<&'a str>,
    /// Hidden reasoning.
    pub reasoning: Option<&'a str>,
    /// Credential material.
    pub credential: Option<&'a str>,
    /// Provider payload.
    pub provider_payload: Option<&'a str>,
    /// Source or destination path.
    pub private_path: Option<&'a str>,
    /// Environment or process value.
    pub environment: Option<&'a str>,
    /// Raw unbounded error body.
    pub raw_error: Option<&'a str>,
    /// Private custom-entry body.
    pub private_custom_entry: Option<&'a str>,
}

/// Exact safe host-write evidence required by receipt projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionTreeExportWriteEvidence<'a> {
    /// Safe destination-capability label, never a path.
    pub destination_capability: &'a str,
    /// Declared overwrite posture.
    pub allow_overwrite: bool,
    /// Requested private file mode.
    pub file_mode: u32,
    /// Admitted write byte budget.
    pub max_bytes: u32,
    /// Exact artifact identity observed by the host writer.
    pub artifact_hash: &'a str,
    /// Exact committed byte count.
    pub bytes_written: u32,
    /// Whether create or replacement committed.
    pub committed: bool,
    /// Whether requested synchronization completed.
    pub synchronized: bool,
    /// Safe bounded outcome class.
    pub status: &'a str,
}

/// Receipt projection input.
#[derive(Clone, Copy, Debug)]
pub struct SessionTreeExportReceiptInput<'a> {
    /// Validated policy.
    pub policy: &'a SessionTreeExportPolicy,
    /// Validated projection.
    pub projection: &'a SessionTreeExportProjection,
    /// Rendered artifact.
    pub artifact: &'a SessionTreeExportArtifact,
    /// Exact validated host-write evidence.
    pub write: SessionTreeExportWriteEvidence<'a>,
    /// Sensitive fields that must all be absent.
    pub sensitive: SessionTreeExportSensitiveDraft<'a>,
}

/// Metadata-only export receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionTreeExportReceipt {
    /// Receipt schema.
    pub schema: String,
    /// Policy identity.
    pub policy_id: String,
    /// Policy BLAKE3 identity.
    pub policy_hash: String,
    /// Validated source BLAKE3 identity.
    pub source_hash: String,
    /// Projection BLAKE3 identity.
    pub projection_hash: String,
    /// Admitted entry count.
    pub entry_count: u32,
    /// Selected active-path entry count.
    pub active_path_count: u32,
    /// Branch-point count.
    pub branch_point_count: u32,
    /// Intentionally omitted private-field count.
    pub omission_count: u32,
    /// Artifact format.
    pub format: SessionTreeExportFormat,
    /// Artifact BLAKE3 identity.
    pub artifact_hash: String,
    /// Artifact byte count.
    pub artifact_bytes: u32,
    /// Safe write status.
    pub write_status: String,
    /// Redaction marker.
    pub redaction_marker: String,
    /// Explicit non-claim.
    pub non_claim: String,
    /// Receipt BLAKE3 identity.
    pub receipt_hash: String,
}

/// Typed export issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionTreeExportIssue {
    /// Generated policy was malformed.
    InvalidPolicy,
    /// Generated policy identity was stale.
    StalePolicy,
    /// Source metadata was malformed or unsafe.
    InvalidSource,
    /// Tree contained duplicate identity or storage sequence.
    DuplicateEntry,
    /// Parent reference was absent.
    MissingParent,
    /// Parent graph contained a cycle.
    Cycle,
    /// Active leaf was absent.
    StaleActiveLeaf,
    /// Repair marker referenced an absent, self, or future entry.
    StaleRepairTarget,
    /// Tree or artifact exceeded a configured bound.
    BoundExceeded,
    /// Requested format was not admitted.
    UnsupportedFormat,
    /// Receipt input contained private material.
    ReceiptLeak,
    /// Host write request or outcome was invalid.
    InvalidWrite,
}

/// Safe export error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionTreeExportError {
    issue: SessionTreeExportIssue,
    field: &'static str,
}

impl SessionTreeExportError {
    pub(super) const fn new(issue: SessionTreeExportIssue, field: &'static str) -> Self {
        Self { issue, field }
    }

    /// Returns the issue class.
    #[must_use]
    pub const fn issue(&self) -> SessionTreeExportIssue {
        self.issue
    }

    /// Returns the safe field label.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }
}

impl core::fmt::Display for SessionTreeExportError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "session tree export rejected {}", self.field)
    }
}

impl std::error::Error for SessionTreeExportError {}
