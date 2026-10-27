mod safety;

pub use safety::{Leak, validate_text};

const RECEIPT_SCHEMA: &str = "animus.workspace-receipt.v1";
const CLEANUP_PENDING_LABEL: &str = "pending";
const MANIFEST_EMPTY_LABEL: &str = "empty_manifest";
const RECEIPT_HASH_PARTS: usize = 7;
const LABEL_DELIMITER_BYTES: usize = 1;

const _: () = assert!(RECEIPT_HASH_PARTS > 0);

/// Borrowed fields used to construct a safe workspace receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    /// Pure workspace plan.
    pub plan: &'a super::planning::Plan,
    /// Optional host-owned adapter outcome.
    pub outcome: Option<&'a super::result::Outcome>,
    /// Safe branch manifest labels or hashes.
    pub manifest_labels: &'a [&'a str],
}

/// Safe workspace receipt containing labels, counts, redactions, and BLAKE3 hashes only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    status: super::planning::Status,
    issue: super::planning::Issue,
    workspace_id: alloc::string::String,
    parent_workspace_id: Option<alloc::string::String>,
    descriptor_hash: alloc::string::String,
    source_hash: alloc::string::String,
    manifest_hash: alloc::string::String,
    operation_hash: alloc::string::String,
    cleanup_hash: alloc::string::String,
    receipt_hash: alloc::string::String,
    capability_labels: alloc::vec::Vec<alloc::string::String>,
    unsupported_capability_labels: alloc::vec::Vec<alloc::string::String>,
    manifest_count: u32,
    outcome_kind: Option<super::result::Kind>,
}

impl Record {
    /// Builds a safe workspace receipt from a pure plan and optional adapter outcome.
    #[must_use]
    pub fn new(draft: Draft<'_>) -> Self {
        let manifest_hash = manifest_hash(draft.manifest_labels);
        let cleanup_hash = cleanup_hash(draft.plan, draft.outcome);
        let outcome_kind = draft.outcome.map(super::result::Outcome::kind);
        let capability_labels =
            super::planning::hash::capability_labels(draft.plan.required_capabilities());
        let capability_gap_labels =
            super::planning::hash::capability_labels(draft.plan.unsupported_capabilities());
        let receipt_hash = identity(IdentityInput {
            plan: draft.plan,
            manifest_hash: manifest_hash.as_str(),
            cleanup_hash: cleanup_hash.as_str(),
            outcome_kind,
        });
        Self {
            status: draft.plan.status(),
            issue: draft.plan.issue(),
            workspace_id: alloc::string::String::from(draft.plan.selected_workspace_id()),
            parent_workspace_id: draft
                .plan
                .parent_workspace_id()
                .map(alloc::string::String::from),
            descriptor_hash: alloc::string::String::from(draft.plan.descriptor_hash()),
            source_hash: alloc::string::String::from(draft.plan.source_hash()),
            manifest_hash,
            operation_hash: alloc::string::String::from(draft.plan.operation_hash()),
            cleanup_hash,
            receipt_hash,
            capability_labels,
            unsupported_capability_labels: capability_gap_labels,
            manifest_count: super::planning::hash::count(draft.manifest_labels.len()),
            outcome_kind,
        }
    }

    /// Returns receipt status.
    #[must_use]
    pub const fn status(&self) -> super::planning::Status {
        self.status
    }

    /// Returns receipt issue.
    #[must_use]
    pub const fn issue(&self) -> super::planning::Issue {
        self.issue
    }

    /// Returns the selected workspace identity label.
    #[must_use]
    pub fn workspace_id(&self) -> &str {
        self.workspace_id.as_str()
    }

    /// Returns the parent workspace identity label, if known.
    #[must_use]
    pub fn parent_workspace_id(&self) -> Option<&str> {
        self.parent_workspace_id.as_deref()
    }

    /// Returns the BLAKE3 descriptor hash.
    #[must_use]
    pub fn descriptor_hash(&self) -> &str {
        self.descriptor_hash.as_str()
    }

    /// Returns the BLAKE3 source identity hash.
    #[must_use]
    pub fn source_hash(&self) -> &str {
        self.source_hash.as_str()
    }

    /// Returns the BLAKE3 manifest hash.
    #[must_use]
    pub fn manifest_hash(&self) -> &str {
        self.manifest_hash.as_str()
    }

    /// Returns the BLAKE3 operation hash.
    #[must_use]
    pub fn operation_hash(&self) -> &str {
        self.operation_hash.as_str()
    }

    /// Returns the BLAKE3 cleanup hash.
    #[must_use]
    pub fn cleanup_hash(&self) -> &str {
        self.cleanup_hash.as_str()
    }

    /// Returns the BLAKE3 receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }

    /// Returns bounded adapter capability labels.
    #[must_use]
    pub fn capability_labels(&self) -> &[alloc::string::String] {
        self.capability_labels.as_slice()
    }

    /// Returns unsupported adapter capability labels.
    #[must_use]
    pub fn unsupported_capability_labels(&self) -> &[alloc::string::String] {
        self.unsupported_capability_labels.as_slice()
    }

    /// Returns the bounded manifest label count.
    #[must_use]
    pub const fn manifest_count(&self) -> u32 {
        self.manifest_count
    }

    /// Returns the optional adapter outcome kind.
    #[must_use]
    pub const fn outcome_kind(&self) -> Option<super::result::Kind> {
        self.outcome_kind
    }
}

fn manifest_hash(manifest_labels: &[&str]) -> alloc::string::String {
    if manifest_labels.is_empty() {
        return super::planning::hash::parts(&[RECEIPT_SCHEMA, MANIFEST_EMPTY_LABEL]);
    }
    let labels_bytes = manifest_labels.iter().fold(0_usize, |total_bytes, label| {
        total_bytes.saturating_add(label.len().saturating_add(LABEL_DELIMITER_BYTES))
    });
    let capacity_bytes = RECEIPT_SCHEMA.len().saturating_add(labels_bytes);
    let mut normalized = alloc::string::String::with_capacity(capacity_bytes);
    normalized.push_str(RECEIPT_SCHEMA);
    for label in manifest_labels {
        normalized.push('|');
        normalized.push_str(label);
    }
    alloc::format!("{}", blake3::hash(normalized.as_bytes()).to_hex())
}

fn cleanup_hash(
    plan: &super::planning::Plan,
    outcome: Option<&super::result::Outcome>,
) -> alloc::string::String {
    let outcome_label = outcome.map_or(CLEANUP_PENDING_LABEL, |actual| actual.kind().as_str());
    super::planning::hash::parts(&[
        RECEIPT_SCHEMA,
        plan.selected_workspace_id(),
        plan.status().as_str(),
        outcome_label,
    ])
}

#[derive(Clone, Copy)]
struct IdentityInput<'a> {
    plan: &'a super::planning::Plan,
    manifest_hash: &'a str,
    cleanup_hash: &'a str,
    outcome_kind: Option<super::result::Kind>,
}

fn identity(input: IdentityInput<'_>) -> alloc::string::String {
    let outcome_label = input
        .outcome_kind
        .map_or(CLEANUP_PENDING_LABEL, super::result::Kind::as_str);
    let parts = [
        RECEIPT_SCHEMA,
        input.plan.status().as_str(),
        input.plan.issue().as_str(),
        input.plan.descriptor_hash(),
        input.manifest_hash,
        input.cleanup_hash,
        outcome_label,
    ];
    debug_assert_eq!(parts.len(), RECEIPT_HASH_PARTS);
    super::planning::hash::parts(&parts)
}
