const RAW_DIFF_MARKERS: &[&str] = &["RAW_DIFF:", "diff --git"];
const RAW_FILE_MARKERS: &[&str] = &["RAW_FILE:", "file body"];
const CREDENTIAL_MARKERS: &[&str] = &[
    "BEGIN OPENSSH PRIVATE KEY",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "AWS_SECRET_ACCESS_KEY",
    "sk-",
];
const ENVIRONMENT_DUMP_MARKERS: &[&str] = &["ENV_DUMP:", "RAW_ENVIRONMENT:"];
const FULL_PATH_MARKERS: &[&str] = &["/home/", "C:\\Users\\", "\\Users\\"];

/// Borrowed safe checkpoint receipt input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckpointReceiptDraft<'a> {
    /// Receipt operation.
    pub operation: super::CheckpointOperation,
    /// Receipt status.
    pub status: super::CheckpointStatus,
    /// Backend kind.
    pub backend_kind: super::CheckpointBackendKind,
    /// Opaque checkpoint ID.
    pub checkpoint_id: &'a str,
    /// Safe workspace identity hash.
    pub workspace_hash: &'a super::CheckpointHash,
    /// Optional before content hash.
    pub before_hash: Option<&'a super::CheckpointHash>,
    /// Optional after content hash.
    pub after_hash: Option<&'a super::CheckpointHash>,
    /// Bounded changed-file count.
    pub changed_file_count: u32,
    /// Confirmation status.
    pub confirmation: super::RollbackConfirmationStatus,
    /// Failure class.
    pub failure_class: super::CheckpointFailureClass,
}

/// Safe checkpoint or rollback receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointReceipt {
    operation: super::CheckpointOperation,
    status: super::CheckpointStatus,
    backend_kind: super::CheckpointBackendKind,
    checkpoint_id: alloc::string::String,
    workspace_hash: super::CheckpointHash,
    before_hash: Option<super::CheckpointHash>,
    after_hash: Option<super::CheckpointHash>,
    changed_file_count: u32,
    confirmation: super::RollbackConfirmationStatus,
    failure_class: super::CheckpointFailureClass,
    receipt_hash: super::CheckpointHash,
}

impl CheckpointReceipt {
    /// Builds a safe checkpoint receipt and computes its BLAKE3 receipt hash.
    ///
    /// # Errors
    ///
    /// Returns [`super::CheckpointError`] when safe metadata fields or bounded counts are invalid.
    pub fn new(draft: CheckpointReceiptDraft<'_>) -> Result<Self, super::CheckpointError> {
        let checkpoint_id = super::core::required_string(super::CHECKPOINT_ID_FIELD, draft.checkpoint_id)?;
        super::core::validate_changed_file_count(draft.changed_file_count)?;
        let receipt_hash = build_hash(&checkpoint_id, draft);
        Ok(Self {
            operation: draft.operation,
            status: draft.status,
            backend_kind: draft.backend_kind,
            checkpoint_id,
            workspace_hash: draft.workspace_hash.clone(),
            before_hash: draft.before_hash.cloned(),
            after_hash: draft.after_hash.cloned(),
            changed_file_count: draft.changed_file_count,
            confirmation: draft.confirmation,
            failure_class: draft.failure_class,
            receipt_hash,
        })
    }

    /// Returns the receipt operation.
    #[must_use]
    pub const fn operation(&self) -> super::CheckpointOperation {
        self.operation
    }
    /// Returns the receipt status.
    #[must_use]
    pub const fn status(&self) -> super::CheckpointStatus {
        self.status
    }
    /// Returns the backend kind.
    #[must_use]
    pub const fn backend_kind(&self) -> super::CheckpointBackendKind {
        self.backend_kind
    }
    /// Returns the opaque checkpoint ID.
    #[must_use]
    pub fn checkpoint_id(&self) -> &str {
        self.checkpoint_id.as_str()
    }
    /// Returns the safe workspace hash.
    #[must_use]
    pub const fn workspace_hash(&self) -> &super::CheckpointHash {
        &self.workspace_hash
    }
    /// Returns the optional before content hash.
    #[must_use]
    pub const fn before_hash(&self) -> Option<&super::CheckpointHash> {
        self.before_hash.as_ref()
    }
    /// Returns the optional after content hash.
    #[must_use]
    pub const fn after_hash(&self) -> Option<&super::CheckpointHash> {
        self.after_hash.as_ref()
    }
    /// Returns the bounded changed-file count.
    #[must_use]
    pub const fn changed_file_count(&self) -> u32 {
        self.changed_file_count
    }
    /// Returns rollback confirmation status.
    #[must_use]
    pub const fn confirmation(&self) -> super::RollbackConfirmationStatus {
        self.confirmation
    }
    /// Returns the failure class.
    #[must_use]
    pub const fn failure_class(&self) -> super::CheckpointFailureClass {
        self.failure_class
    }
    /// Returns the BLAKE3 receipt hash.
    #[must_use]
    pub const fn receipt_hash(&self) -> &super::CheckpointHash {
        &self.receipt_hash
    }
}

/// Leak class detected in candidate checkpoint receipt text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointReceiptLeak {
    /// Raw diff text appeared in receipt evidence.
    RawDiff,
    /// Raw file body appeared in receipt evidence.
    RawFileBody,
    /// Credential-looking material appeared in receipt evidence.
    Credential,
    /// Raw environment dump appeared in receipt evidence.
    EnvironmentDump,
    /// Full private or absolute path list appeared in receipt evidence.
    FullPathList,
}
impl CheckpointReceiptLeak {
    /// Returns the stable leak label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RawDiff => "raw_diff",
            Self::RawFileBody => "raw_file_body",
            Self::Credential => "credential",
            Self::EnvironmentDump => "environment_dump",
            Self::FullPathList => "full_path_list",
        }
    }
}

const LEAK_MARKERS: &[(CheckpointReceiptLeak, &[&str])] = &[
    (CheckpointReceiptLeak::RawDiff, RAW_DIFF_MARKERS),
    (CheckpointReceiptLeak::RawFileBody, RAW_FILE_MARKERS),
    (CheckpointReceiptLeak::Credential, CREDENTIAL_MARKERS),
    (CheckpointReceiptLeak::EnvironmentDump, ENVIRONMENT_DUMP_MARKERS),
    (CheckpointReceiptLeak::FullPathList, FULL_PATH_MARKERS),
];

/// Validates candidate checkpoint receipt text for unsafe sentinel material.
///
/// # Errors
///
/// Returns [`CheckpointReceiptLeak`] when receipt text contains unsafe material.
pub fn validate_checkpoint_receipt_text(text: &str) -> Result<(), CheckpointReceiptLeak> {
    first_leak(text).map_or(Ok(()), Err)
}

struct CanonicalText {
    value: alloc::string::String,
}
impl CanonicalText {
    fn new() -> Self {
        Self {
            value: alloc::string::String::new(),
        }
    }
    fn push(&mut self, pair: CanonicalPair<'_>) {
        if !self.value.is_empty() {
            self.value.push_str(super::FIELD_SEPARATOR);
        }
        self.value.push_str(pair.key);
        self.value.push_str(super::KEY_VALUE_SEPARATOR);
        self.value.push_str(pair.value);
    }
    fn finish(self) -> alloc::string::String {
        self.value
    }
}

#[derive(Clone, Copy)]
struct CanonicalPair<'a> {
    key: &'a str,
    value: &'a str,
}

fn build_hash(checkpoint_id: &str, draft: CheckpointReceiptDraft<'_>) -> super::CheckpointHash {
    let canonical = canonical_text(checkpoint_id, draft);
    super::CheckpointHash::from_blake3_bytes(canonical.as_bytes())
}

fn canonical_text(checkpoint_id: &str, draft: CheckpointReceiptDraft<'_>) -> alloc::string::String {
    assert!(!checkpoint_id.is_empty(), "checkpoint id must be validated");
    assert!(
        draft.changed_file_count <= super::MAX_CHANGED_FILE_COUNT,
        "changed-file count must be bounded before receipt hashing"
    );
    let mut canonical = CanonicalText::new();
    canonical.push(CanonicalPair {
        key: "schema",
        value: super::RECEIPT_SCHEMA,
    });
    canonical.push(CanonicalPair {
        key: "operation",
        value: draft.operation.as_str(),
    });
    canonical.push(CanonicalPair {
        key: "status",
        value: draft.status.as_str(),
    });
    canonical.push(CanonicalPair {
        key: "backend",
        value: draft.backend_kind.as_str(),
    });
    canonical.push(CanonicalPair {
        key: "checkpoint",
        value: checkpoint_id,
    });
    canonical.push(CanonicalPair {
        key: "workspace",
        value: draft.workspace_hash.as_str(),
    });
    canonical.push(CanonicalPair {
        key: "before",
        value: draft.before_hash.map_or(super::NO_HASH_SENTINEL, super::CheckpointHash::as_str),
    });
    canonical.push(CanonicalPair {
        key: "after",
        value: draft.after_hash.map_or(super::NO_HASH_SENTINEL, super::CheckpointHash::as_str),
    });
    canonical.push(CanonicalPair {
        key: "changed_files",
        value: &alloc::format!("{}", draft.changed_file_count),
    });
    canonical.push(CanonicalPair {
        key: "confirmation",
        value: draft.confirmation.as_str(),
    });
    canonical.push(CanonicalPair {
        key: "failure",
        value: draft.failure_class.as_str(),
    });
    canonical.finish()
}

fn first_leak(text: &str) -> Option<CheckpointReceiptLeak> {
    LEAK_MARKERS.iter().find_map(|(leak, markers)| contains_marker(text, markers).then_some(*leak))
}
fn contains_marker(text: &str, markers: &[&str]) -> bool {
    markers.iter().any(|marker| text.contains(marker))
}
