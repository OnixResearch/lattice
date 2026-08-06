//! VCS-neutral snapshot ledger contracts.
//!
//! Hosts may map these DTOs to jj, Git, `BranchFS`, in-memory, or custom recovery
//! backends. This module validates records and plans only; it never runs VCS
//! commands, creates branches or stashes, mutates worktrees, schedules snapshots,
//! opens UI, cleans up storage, or restores files.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::assertion_density,
        tigerstyle::compound_condition,
        tigerstyle::function_length,
        tigerstyle::non_trait_imports,
        tigerstyle::unbounded_collection_growth
    )
)]

use crate::artifact::Hash;
use crate::validation::AdapterContractError;
use crate::validation::FieldName;
use crate::validation::{self};

const SNAPSHOT_ID_FIELD: FieldName = FieldName::new("snapshot_id");
const WORKSPACE_LABEL_FIELD: FieldName = FieldName::new("snapshot_workspace_label");
const ACTOR_LABEL_FIELD: FieldName = FieldName::new("snapshot_actor_label");
const SESSION_LABEL_FIELD: FieldName = FieldName::new("snapshot_session_label");
const CAPABILITY_LABEL_FIELD: FieldName = FieldName::new("snapshot_capability_label");
const RECEIPT_STATUS_FIELD: FieldName = FieldName::new("snapshot_receipt_status");
const RECEIPT_ISSUE_FIELD: FieldName = FieldName::new("snapshot_receipt_issue");
const HASH_FIELD: FieldName = FieldName::new("snapshot_hash");

const MAX_CONTENT_HASHES: usize = 16;
const MAX_SNAPSHOT_RECORDS: usize = 64;
const MAX_CHANGED_ARTIFACT_COUNT: u32 = 512;
const STALE_FRESHNESS_EPOCH_DELTA: u32 = 2;
const UNSAFE_SNAPSHOT_MARKERS: &[&str] = &[
    "raw diff",
    "diff --git",
    "+++ ",
    "--- ",
    "raw file body",
    "command line",
    "git checkout",
    "git stash",
    "jj ",
    "refs/stash",
    "shadow branch",
    "provider payload",
    "credential",
    "secret",
    "token",
    "/home/",
    "/tmp/",
    "c:\\",
    "\\users\\",
    "raw tool output",
];

const _: () = assert!(MAX_CONTENT_HASHES > 0);
const _: () = assert!(MAX_SNAPSHOT_RECORDS > 0);
const _: () = assert!(MAX_CHANGED_ARTIFACT_COUNT > 0);
const _: () = assert!(STALE_FRESHNESS_EPOCH_DELTA > 0);

/// Product behaviors that snapshot ledger contracts explicitly do not perform.
pub const PROHIBITED_VCS_SNAPSHOT_EFFECTS: &[&str] = &[
    "VCS command execution",
    "shadow branch creation",
    "stash ref management",
    "jj operation mutation",
    "worktree write",
    "restore UI",
    "background snapshot scheduling",
    "automatic checkpoint creation",
    "autocommit",
];

/// Backend family that owns a snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotBackendKind {
    /// Jujutsu-style backend.
    Jj,
    /// Git-style backend.
    Git,
    /// BranchFS-style backend.
    BranchFs,
    /// In-memory host store.
    InMemory,
    /// Other host-owned backend.
    Other,
}

/// Snapshot intent supplied by host policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotIntentKind {
    /// Manual host-requested snapshot.
    Manual,
    /// Snapshot before a protected mutation.
    PreMutation,
    /// Turn-boundary snapshot requested by host policy.
    TurnBoundary,
    /// Recovery-oriented snapshot.
    Recovery,
}

/// Snapshot operation kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotOperationKind {
    /// Accept snapshot changes.
    Accept,
    /// Restore snapshot contents.
    Restore,
    /// Drop snapshot state.
    Drop,
    /// Squash snapshot lineage.
    Squash,
    /// Compare snapshots.
    Compare,
    /// Cleanup snapshot storage.
    Cleanup,
}

/// Protected-action class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotProtectionClass {
    /// Operation is protected and requires confirmation.
    Protected,
    /// Operation is read-only or otherwise unprotected.
    Unprotected,
}

/// Host confirmation status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotConfirmationStatus {
    /// Operator confirmed the operation.
    Confirmed,
    /// Operator denied the operation.
    Denied,
    /// Confirmation was missing.
    Missing,
}

/// Snapshot service availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotServiceStatus {
    /// Host injected a snapshot service.
    Available,
    /// Host did not inject a snapshot service.
    Missing,
    /// Host service is unavailable.
    Unavailable,
    /// Host service does not support the requested operation.
    Unsupported,
}

/// Snapshot plan status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotPlanStatus {
    /// Plan is ready for host admission.
    Ready,
    /// Service is missing or unavailable.
    ServiceUnavailable,
    /// Operation is unsupported by host service.
    Unsupported,
    /// Freshness proof is stale or mismatched.
    StaleSnapshot,
    /// Confirmation is required.
    ConfirmationRequired,
    /// Confirmation denied the operation.
    ConfirmationDenied,
    /// Metadata is malformed or unsafe.
    Rejected,
}

/// Snapshot policy planning status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotPolicyPlanStatus {
    /// Host may request snapshots under its policy.
    Ready,
    /// No automatic snapshot claim is made.
    NoSnapshotClaim,
}

/// Snapshot diagnostic code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotDiagnosticCode {
    /// Required field is empty.
    EmptyField,
    /// Field is too long.
    FieldTooLong,
    /// Field contains a NUL byte.
    NulCharacter,
    /// Collection is empty.
    EmptyCollection,
    /// Collection is too large.
    TooManyItems,
    /// Metadata leaks raw diff/path/command/provider data.
    UnsafeMetadata,
    /// Hash is malformed.
    InvalidHash,
    /// Changed artifact count is too large.
    ChangedCountTooLarge,
    /// Snapshot target was not present in the ledger.
    UnknownSnapshot,
    /// Freshness proof is stale or mismatched.
    StaleSnapshot,
    /// Service is missing or unavailable.
    ServiceUnavailable,
    /// Operation is unsupported.
    Unsupported,
    /// Confirmation is missing or denied.
    ConfirmationRequired,
}

/// Safe diagnostic emitted by snapshot ledger contracts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotDiagnostic {
    code: SnapshotDiagnosticCode,
    snapshot_id: Option<Text>,
    message: Text,
}

/// Snapshot record draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotRecordDraft<'a> {
    /// Snapshot intent.
    pub intent: SnapshotIntentKind,
    /// Backend kind.
    pub backend: SnapshotBackendKind,
    /// Opaque safe snapshot id.
    pub snapshot_id: &'a str,
    /// Optional parent snapshot id.
    pub parent_snapshot_id: Option<&'a str>,
    /// Safe workspace label or hash label.
    pub workspace_label: &'a str,
    /// Workspace identity hash.
    pub workspace_hash: &'a str,
    /// Content identities included in the snapshot.
    pub content_hashes: &'a [&'a str],
    /// Changed-artifact count.
    pub changed_artifact_count: u32,
    /// Safe actor label.
    pub actor_label: &'a str,
    /// Safe session label.
    pub session_label: &'a str,
    /// Host freshness epoch.
    pub freshness_epoch: u32,
    /// Safe receipt identity.
    pub receipt_hash: &'a str,
}

/// Validated snapshot record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotRecord {
    intent: SnapshotIntentKind,
    backend: SnapshotBackendKind,
    snapshot_id: Text,
    parent_snapshot_id: Option<Text>,
    workspace_label: Text,
    workspace_hash: Text,
    content_hashes: Collection<Text>,
    changed_artifact_count: u32,
    actor_label: Text,
    session_label: Text,
    freshness_epoch: u32,
    receipt_hash: Text,
}

/// Snapshot lineage summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotLineage {
    records: Collection<SnapshotRecord>,
    diagnostics: Collection<SnapshotDiagnostic>,
}

/// Freshness proof draft for a protected operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotFreshnessProofDraft<'a> {
    /// Target snapshot id.
    pub snapshot_id: &'a str,
    /// Expected current workspace identity.
    pub expected_workspace_hash: &'a str,
    /// Observed current workspace identity.
    pub observed_workspace_hash: &'a str,
    /// Proof epoch.
    pub proof_epoch: u32,
    /// Host current epoch.
    pub current_epoch: u32,
}

/// Validated freshness proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotFreshnessProof {
    snapshot_id: Text,
    expected_workspace_hash: Text,
    observed_workspace_hash: Text,
    proof_epoch: u32,
    current_epoch: u32,
}

/// Snapshot operation plan draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotOperationPlanDraft<'a> {
    /// Operation kind.
    pub operation: SnapshotOperationKind,
    /// Target snapshot id.
    pub target_snapshot_id: &'a str,
    /// Freshness proof.
    pub freshness: SnapshotFreshnessProofDraft<'a>,
    /// Required backend capability label.
    pub required_capability: &'a str,
    /// Protected-action class.
    pub protection: SnapshotProtectionClass,
    /// Confirmation status.
    pub confirmation: SnapshotConfirmationStatus,
}

/// Validated operation plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotOperationPlan {
    status: SnapshotPlanStatus,
    operation: SnapshotOperationKind,
    target_snapshot_id: Option<Text>,
    required_capability: Option<Text>,
    diagnostic: Option<SnapshotDiagnostic>,
}

/// Automatic snapshot policy plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotPolicyPlan {
    status: SnapshotPolicyPlanStatus,
    service_status: SnapshotServiceStatus,
    diagnostic: Option<SnapshotDiagnostic>,
}

/// Snapshot receipt draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotReceiptDraft<'a> {
    /// Operation kind.
    pub operation: SnapshotOperationKind,
    /// Receipt status label.
    pub status: &'a str,
    /// Backend kind.
    pub backend: SnapshotBackendKind,
    /// Snapshot id hash.
    pub snapshot_id_hash: &'a str,
    /// Workspace hash.
    pub workspace_hash: &'a str,
    /// Changed-artifact count.
    pub changed_artifact_count: u32,
    /// Freshness class label.
    pub freshness_class: &'a str,
    /// Confirmation status.
    pub confirmation: SnapshotConfirmationStatus,
    /// Policy hash.
    pub policy_hash: &'a str,
    /// Receipt hash.
    pub receipt_hash: &'a str,
    /// Optional safe issue code.
    pub issue_code: Option<&'a str>,
}

/// Validated safe receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotReceipt {
    operation: SnapshotOperationKind,
    status: Text,
    backend: SnapshotBackendKind,
    snapshot_id_hash: Text,
    workspace_hash: Text,
    changed_artifact_count: u32,
    freshness_class: Text,
    confirmation: SnapshotConfirmationStatus,
    policy_hash: Text,
    receipt_hash: Text,
    issue_code: Option<Text>,
}

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;
type SnapshotResult<T> = Result<T, SnapshotDiagnostic>;

impl SnapshotDiagnostic {
    /// Diagnostic code.
    #[must_use]
    pub const fn code(&self) -> SnapshotDiagnosticCode {
        self.code
    }

    /// Snapshot id when available.
    #[must_use]
    pub fn snapshot_id(&self) -> Option<&str> {
        self.snapshot_id.as_deref()
    }

    /// Safe message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl SnapshotRecord {
    /// Snapshot intent.
    #[must_use]
    pub const fn intent(&self) -> SnapshotIntentKind {
        self.intent
    }

    /// Backend kind.
    #[must_use]
    pub const fn backend(&self) -> SnapshotBackendKind {
        self.backend
    }

    /// Snapshot id.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Parent snapshot id.
    #[must_use]
    pub fn parent_snapshot_id(&self) -> Option<&str> {
        self.parent_snapshot_id.as_deref()
    }

    /// Workspace label.
    #[must_use]
    pub fn workspace_label(&self) -> &str {
        &self.workspace_label
    }

    /// Workspace hash.
    #[must_use]
    pub fn workspace_hash(&self) -> &str {
        &self.workspace_hash
    }

    /// Content hashes.
    #[must_use]
    pub fn content_hashes(&self) -> &[Text] {
        &self.content_hashes
    }

    /// Changed-artifact count.
    #[must_use]
    pub const fn changed_artifact_count(&self) -> u32 {
        self.changed_artifact_count
    }

    /// Actor label.
    #[must_use]
    pub fn actor_label(&self) -> &str {
        &self.actor_label
    }

    /// Session label.
    #[must_use]
    pub fn session_label(&self) -> &str {
        &self.session_label
    }

    /// Freshness epoch.
    #[must_use]
    pub const fn freshness_epoch(&self) -> u32 {
        self.freshness_epoch
    }

    /// Receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        &self.receipt_hash
    }
}

impl SnapshotLineage {
    /// Records in lineage order.
    #[must_use]
    pub fn records(&self) -> &[SnapshotRecord] {
        &self.records
    }

    /// Safe diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[SnapshotDiagnostic] {
        &self.diagnostics
    }
}

impl SnapshotFreshnessProof {
    /// Snapshot id.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Expected workspace hash.
    #[must_use]
    pub fn expected_workspace_hash(&self) -> &str {
        &self.expected_workspace_hash
    }

    /// Observed workspace hash.
    #[must_use]
    pub fn observed_workspace_hash(&self) -> &str {
        &self.observed_workspace_hash
    }

    /// Proof epoch.
    #[must_use]
    pub const fn proof_epoch(&self) -> u32 {
        self.proof_epoch
    }

    /// Current epoch.
    #[must_use]
    pub const fn current_epoch(&self) -> u32 {
        self.current_epoch
    }
}

impl SnapshotOperationPlan {
    /// Plan status.
    #[must_use]
    pub const fn status(&self) -> SnapshotPlanStatus {
        self.status
    }

    /// Operation kind.
    #[must_use]
    pub const fn operation(&self) -> SnapshotOperationKind {
        self.operation
    }

    /// Target snapshot id when accepted.
    #[must_use]
    pub fn target_snapshot_id(&self) -> Option<&str> {
        self.target_snapshot_id.as_deref()
    }

    /// Required capability.
    #[must_use]
    pub fn required_capability(&self) -> Option<&str> {
        self.required_capability.as_deref()
    }

    /// Diagnostic when rejected.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&SnapshotDiagnostic> {
        self.diagnostic.as_ref()
    }
}

impl SnapshotPolicyPlan {
    /// Policy plan status.
    #[must_use]
    pub const fn status(&self) -> SnapshotPolicyPlanStatus {
        self.status
    }

    /// Service status.
    #[must_use]
    pub const fn service_status(&self) -> SnapshotServiceStatus {
        self.service_status
    }

    /// Diagnostic.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&SnapshotDiagnostic> {
        self.diagnostic.as_ref()
    }
}

impl SnapshotReceipt {
    /// Operation kind.
    #[must_use]
    pub const fn operation(&self) -> SnapshotOperationKind {
        self.operation
    }

    /// Receipt status.
    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Backend kind.
    #[must_use]
    pub const fn backend(&self) -> SnapshotBackendKind {
        self.backend
    }

    /// Snapshot id hash.
    #[must_use]
    pub fn snapshot_id_hash(&self) -> &str {
        &self.snapshot_id_hash
    }

    /// Workspace hash.
    #[must_use]
    pub fn workspace_hash(&self) -> &str {
        &self.workspace_hash
    }

    /// Changed-artifact count.
    #[must_use]
    pub const fn changed_artifact_count(&self) -> u32 {
        self.changed_artifact_count
    }

    /// Freshness class.
    #[must_use]
    pub fn freshness_class(&self) -> &str {
        &self.freshness_class
    }

    /// Confirmation status.
    #[must_use]
    pub const fn confirmation(&self) -> SnapshotConfirmationStatus {
        self.confirmation
    }

    /// Policy hash.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        &self.policy_hash
    }

    /// Receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        &self.receipt_hash
    }

    /// Issue code.
    #[must_use]
    pub fn issue_code(&self) -> Option<&str> {
        self.issue_code.as_deref()
    }
}

/// Validates a backend-neutral snapshot record.
///
/// # Errors
///
/// Returns [`SnapshotDiagnostic`] for unsafe labels, malformed hashes, unbounded
/// content lists, or changed-count overflow.
pub fn validate_snapshot_record(draft: &SnapshotRecordDraft<'_>) -> SnapshotResult<SnapshotRecord> {
    validate_count(draft.changed_artifact_count)?;
    validate_non_empty_bound("snapshot_content_hashes", draft.content_hashes.len(), MAX_CONTENT_HASHES)?;
    let snapshot_id = safe_text(SNAPSHOT_ID_FIELD, draft.snapshot_id, None)?;
    let parent_snapshot_id = draft
        .parent_snapshot_id
        .map(|parent| safe_text(SNAPSHOT_ID_FIELD, parent, Some(snapshot_id.as_str())))
        .transpose()?;
    let mut content_hashes = Collection::with_capacity(draft.content_hashes.len());
    for hash in draft.content_hashes {
        content_hashes.push(safe_hash(hash)?);
    }
    Ok(SnapshotRecord {
        intent: draft.intent,
        backend: draft.backend,
        snapshot_id,
        parent_snapshot_id,
        workspace_label: safe_text(WORKSPACE_LABEL_FIELD, draft.workspace_label, None)?,
        workspace_hash: safe_hash(draft.workspace_hash)?,
        content_hashes,
        changed_artifact_count: draft.changed_artifact_count,
        actor_label: safe_text(ACTOR_LABEL_FIELD, draft.actor_label, None)?,
        session_label: safe_text(SESSION_LABEL_FIELD, draft.session_label, None)?,
        freshness_epoch: draft.freshness_epoch,
        receipt_hash: safe_hash(draft.receipt_hash)?,
    })
}

/// Builds a snapshot lineage and reports missing parents without reading VCS state.
///
/// # Errors
///
/// Returns [`SnapshotDiagnostic`] when the supplied record list is empty, exceeds bounds, or any
/// record fails snapshot validation.
pub fn validate_snapshot_lineage(drafts: &[SnapshotRecordDraft<'_>]) -> SnapshotResult<SnapshotLineage> {
    validate_non_empty_bound("snapshot_records", drafts.len(), MAX_SNAPSHOT_RECORDS)?;
    let mut records = Collection::with_capacity(drafts.len());
    for draft in drafts {
        records.push(validate_snapshot_record(draft)?);
    }
    let mut diagnostics = Collection::new();
    for record in &records {
        if let Some(parent) = record.parent_snapshot_id() {
            if !records.iter().any(|candidate| candidate.snapshot_id() == parent) {
                diagnostics.push(diagnostic(
                    SnapshotDiagnosticCode::UnknownSnapshot,
                    Some(record.snapshot_id()),
                    "snapshot parent is not present in supplied lineage",
                ));
            }
        }
    }
    Ok(SnapshotLineage { records, diagnostics })
}

/// Plans a snapshot operation without mutating the backend.
#[must_use]
pub fn plan_snapshot_operation(
    lineage: &SnapshotLineage,
    draft: &SnapshotOperationPlanDraft<'_>,
    service_status: SnapshotServiceStatus,
) -> SnapshotOperationPlan {
    if service_status == SnapshotServiceStatus::Missing || service_status == SnapshotServiceStatus::Unavailable {
        return rejected_plan(SnapshotRejection {
            operation: draft.operation,
            status: SnapshotPlanStatus::ServiceUnavailable,
            code: SnapshotDiagnosticCode::ServiceUnavailable,
            snapshot_id: None,
            message: "snapshot service unavailable",
        });
    }
    if service_status == SnapshotServiceStatus::Unsupported {
        return rejected_plan(SnapshotRejection {
            operation: draft.operation,
            status: SnapshotPlanStatus::Unsupported,
            code: SnapshotDiagnosticCode::Unsupported,
            snapshot_id: None,
            message: "snapshot operation unsupported",
        });
    }
    let target = match safe_text(SNAPSHOT_ID_FIELD, draft.target_snapshot_id, None) {
        Ok(value) => value,
        Err(error) => {
            return rejected_from_error(draft.operation, SnapshotPlanStatus::Rejected, error);
        }
    };
    let Some(record) = lineage.records().iter().find(|item| item.snapshot_id() == target) else {
        return rejected_plan(SnapshotRejection {
            operation: draft.operation,
            status: SnapshotPlanStatus::Rejected,
            code: SnapshotDiagnosticCode::UnknownSnapshot,
            snapshot_id: Some(target.as_str()),
            message: "target snapshot is missing",
        });
    };
    let proof = match validate_freshness_proof(&draft.freshness) {
        Ok(value) => value,
        Err(error) => {
            return rejected_from_error(draft.operation, SnapshotPlanStatus::Rejected, error);
        }
    };
    if proof.snapshot_id() != target
        || proof.expected_workspace_hash() != proof.observed_workspace_hash()
        || is_stale(proof.proof_epoch(), proof.current_epoch())
        || proof.expected_workspace_hash() != record.workspace_hash()
    {
        return rejected_plan(SnapshotRejection {
            operation: draft.operation,
            status: SnapshotPlanStatus::StaleSnapshot,
            code: SnapshotDiagnosticCode::StaleSnapshot,
            snapshot_id: Some(target.as_str()),
            message: "snapshot freshness proof is stale or mismatched",
        });
    }
    if draft.protection == SnapshotProtectionClass::Protected {
        match draft.confirmation {
            SnapshotConfirmationStatus::Confirmed => {}
            SnapshotConfirmationStatus::Denied => {
                return rejected_plan(SnapshotRejection {
                    operation: draft.operation,
                    status: SnapshotPlanStatus::ConfirmationDenied,
                    code: SnapshotDiagnosticCode::ConfirmationRequired,
                    snapshot_id: Some(target.as_str()),
                    message: "snapshot operation confirmation denied",
                });
            }
            SnapshotConfirmationStatus::Missing => {
                return rejected_plan(SnapshotRejection {
                    operation: draft.operation,
                    status: SnapshotPlanStatus::ConfirmationRequired,
                    code: SnapshotDiagnosticCode::ConfirmationRequired,
                    snapshot_id: Some(target.as_str()),
                    message: "snapshot operation requires confirmation",
                });
            }
        }
    }
    let capability = match safe_text(CAPABILITY_LABEL_FIELD, draft.required_capability, Some(target.as_str())) {
        Ok(value) => value,
        Err(error) => {
            return rejected_from_error(draft.operation, SnapshotPlanStatus::Rejected, error);
        }
    };
    SnapshotOperationPlan {
        status: SnapshotPlanStatus::Ready,
        operation: draft.operation,
        target_snapshot_id: Some(target),
        required_capability: Some(capability),
        diagnostic: None,
    }
}

/// Plans automatic snapshot policy and reports no claim when service or policy is absent.
#[must_use]
pub fn plan_snapshot_policy(service_status: SnapshotServiceStatus, policy_enabled: bool) -> SnapshotPolicyPlan {
    if !policy_enabled || service_status != SnapshotServiceStatus::Available {
        return SnapshotPolicyPlan {
            status: SnapshotPolicyPlanStatus::NoSnapshotClaim,
            service_status,
            diagnostic: Some(diagnostic(
                SnapshotDiagnosticCode::ServiceUnavailable,
                None,
                "no snapshot service or policy claim",
            )),
        };
    }
    SnapshotPolicyPlan {
        status: SnapshotPolicyPlanStatus::Ready,
        service_status,
        diagnostic: None,
    }
}

/// Validates a safe snapshot receipt.
///
/// # Errors
///
/// Returns [`SnapshotDiagnostic`] when receipt metadata leaks raw diffs, file
/// bodies, command lines, paths, provider payloads, credentials, or unsafe refs.
pub fn validate_snapshot_receipt(draft: &SnapshotReceiptDraft<'_>) -> SnapshotResult<SnapshotReceipt> {
    validate_count(draft.changed_artifact_count)?;
    let status = safe_text(RECEIPT_STATUS_FIELD, draft.status, None)?;
    let freshness_class = safe_text(RECEIPT_STATUS_FIELD, draft.freshness_class, None)?;
    let issue_code = draft.issue_code.map(|issue| safe_text(RECEIPT_ISSUE_FIELD, issue, None)).transpose()?;
    Ok(SnapshotReceipt {
        operation: draft.operation,
        status,
        backend: draft.backend,
        snapshot_id_hash: safe_hash(draft.snapshot_id_hash)?,
        workspace_hash: safe_hash(draft.workspace_hash)?,
        changed_artifact_count: draft.changed_artifact_count,
        freshness_class,
        confirmation: draft.confirmation,
        policy_hash: safe_hash(draft.policy_hash)?,
        receipt_hash: safe_hash(draft.receipt_hash)?,
        issue_code,
    })
}

fn validate_freshness_proof(draft: &SnapshotFreshnessProofDraft<'_>) -> SnapshotResult<SnapshotFreshnessProof> {
    let snapshot_id = safe_text(SNAPSHOT_ID_FIELD, draft.snapshot_id, None)?;
    Ok(SnapshotFreshnessProof {
        snapshot_id,
        expected_workspace_hash: safe_hash(draft.expected_workspace_hash)?,
        observed_workspace_hash: safe_hash(draft.observed_workspace_hash)?,
        proof_epoch: draft.proof_epoch,
        current_epoch: draft.current_epoch,
    })
}

fn validate_count(value: u32) -> SnapshotResult<()> {
    if value > MAX_CHANGED_ARTIFACT_COUNT {
        return Err(diagnostic(
            SnapshotDiagnosticCode::ChangedCountTooLarge,
            None,
            "snapshot changed-artifact count is too large",
        ));
    }
    Ok(())
}

fn validate_non_empty_bound(field: &'static str, len: usize, max: usize) -> SnapshotResult<()> {
    if len == 0 {
        return Err(SnapshotDiagnostic {
            code: SnapshotDiagnosticCode::EmptyCollection,
            snapshot_id: None,
            message: Text::from(field),
        });
    }
    if len > max {
        return Err(SnapshotDiagnostic {
            code: SnapshotDiagnosticCode::TooManyItems,
            snapshot_id: None,
            message: Text::from(field),
        });
    }
    Ok(())
}

fn safe_text(field: FieldName, value: &str, snapshot_id: Option<&str>) -> SnapshotResult<Text> {
    let normalized = validation::normalized_adapter_field(field, value).map_err(|error| map_adapter_error(&error))?;
    let lower = normalized.to_ascii_lowercase();
    if UNSAFE_SNAPSHOT_MARKERS.iter().any(|marker| lower.contains(marker)) {
        return Err(diagnostic(
            SnapshotDiagnosticCode::UnsafeMetadata,
            snapshot_id,
            "snapshot metadata contains unsafe marker",
        ));
    }
    Ok(normalized)
}

fn safe_hash(value: &str) -> SnapshotResult<Text> {
    let normalized =
        validation::normalized_adapter_field(HASH_FIELD, value).map_err(|error| map_adapter_error(&error))?;
    if Hash::new(&normalized).is_ok() {
        Ok(normalized)
    } else {
        Err(diagnostic(SnapshotDiagnosticCode::InvalidHash, None, HASH_FIELD.as_str()))
    }
}

fn is_stale(proof_epoch: u32, current_epoch: u32) -> bool {
    current_epoch.saturating_sub(proof_epoch) > STALE_FRESHNESS_EPOCH_DELTA
}

#[derive(Clone, Copy)]
struct SnapshotRejection<'a> {
    operation: SnapshotOperationKind,
    status: SnapshotPlanStatus,
    code: SnapshotDiagnosticCode,
    snapshot_id: Option<&'a str>,
    message: &'a str,
}

fn rejected_plan(rejection: SnapshotRejection<'_>) -> SnapshotOperationPlan {
    SnapshotOperationPlan {
        status: rejection.status,
        operation: rejection.operation,
        target_snapshot_id: None,
        required_capability: None,
        diagnostic: Some(diagnostic(rejection.code, rejection.snapshot_id, rejection.message)),
    }
}

fn rejected_from_error(
    operation: SnapshotOperationKind,
    status: SnapshotPlanStatus,
    error: SnapshotDiagnostic,
) -> SnapshotOperationPlan {
    SnapshotOperationPlan {
        status,
        operation,
        target_snapshot_id: None,
        required_capability: None,
        diagnostic: Some(error),
    }
}

fn map_adapter_error(error: &AdapterContractError) -> SnapshotDiagnostic {
    match error {
        AdapterContractError::EmptyField { field } => diagnostic(SnapshotDiagnosticCode::EmptyField, None, field),
        AdapterContractError::FieldTooLong { field, .. } => {
            diagnostic(SnapshotDiagnosticCode::FieldTooLong, None, field)
        }
        AdapterContractError::NulCharacter { field, .. } => {
            diagnostic(SnapshotDiagnosticCode::NulCharacter, None, field)
        }
        AdapterContractError::InvalidEvidenceHash { field, .. } => {
            diagnostic(SnapshotDiagnosticCode::InvalidHash, None, field)
        }
        AdapterContractError::EmptyCollection { field } => {
            diagnostic(SnapshotDiagnosticCode::EmptyCollection, None, field)
        }
        AdapterContractError::UnsupportedStatusState { field, .. }
        | AdapterContractError::UnsupportedStatusObservationKind { field, .. }
        | AdapterContractError::UnsupportedSessionReferenceKind { field, .. }
        | AdapterContractError::UnsupportedWorkspaceValue { field, .. }
        | AdapterContractError::UnsupportedDataspaceValue { field, .. }
        | AdapterContractError::UnsupportedRouterValue { field, .. } => {
            diagnostic(SnapshotDiagnosticCode::UnsafeMetadata, None, field)
        }
    }
}

fn diagnostic(code: SnapshotDiagnosticCode, snapshot_id: Option<&str>, message: &str) -> SnapshotDiagnostic {
    SnapshotDiagnostic {
        code,
        snapshot_id: snapshot_id.map(Text::from),
        message: Text::from(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SNAPSHOT_ONE: &str = "snapshot-one";
    const SNAPSHOT_TWO: &str = "snapshot-two";
    const WORKSPACE_LABEL: &str = "workspace-main";
    const ACTOR_LABEL: &str = "host-shell";
    const SESSION_LABEL: &str = "session-alpha";
    const CAPABILITY_RESTORE: &str = "snapshot-restore";
    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const CURRENT_EPOCH: u32 = 7;
    const FRESH_EPOCH: u32 = 7;
    const CHANGED_COUNT: u32 = 2;
    static CONTENT_HASHES: [&str; 1] = [HASH_A];

    fn record(
        backend: SnapshotBackendKind,
        snapshot_id: &'static str,
        parent: Option<&'static str>,
        workspace_hash: &'static str,
    ) -> SnapshotRecordDraft<'static> {
        SnapshotRecordDraft {
            intent: SnapshotIntentKind::Manual,
            backend,
            snapshot_id,
            parent_snapshot_id: parent,
            workspace_label: WORKSPACE_LABEL,
            workspace_hash,
            content_hashes: &CONTENT_HASHES,
            changed_artifact_count: CHANGED_COUNT,
            actor_label: ACTOR_LABEL,
            session_label: SESSION_LABEL,
            freshness_epoch: FRESH_EPOCH,
            receipt_hash: HASH_C,
        }
    }

    fn lineage() -> SnapshotLineage {
        validate_snapshot_lineage(&[
            record(SnapshotBackendKind::Jj, SNAPSHOT_ONE, None, HASH_A),
            record(SnapshotBackendKind::Git, SNAPSHOT_TWO, Some(SNAPSHOT_ONE), HASH_A),
        ])
        .unwrap()
    }

    fn freshness(snapshot_id: &'static str) -> SnapshotFreshnessProofDraft<'static> {
        SnapshotFreshnessProofDraft {
            snapshot_id,
            expected_workspace_hash: HASH_A,
            observed_workspace_hash: HASH_A,
            proof_epoch: FRESH_EPOCH,
            current_epoch: CURRENT_EPOCH,
        }
    }

    #[test]
    fn backend_neutral_records_and_lineage_validate_for_jj_and_git() {
        // r[verify agent_harness_runtime.hermes_vcs_snapshot_ledger.records]
        let lineage = lineage();

        assert_eq!(lineage.records().len(), 2);
        assert_eq!(lineage.records()[0].backend(), SnapshotBackendKind::Jj);
        assert_eq!(lineage.records()[1].backend(), SnapshotBackendKind::Git);
        assert_eq!(lineage.records()[1].parent_snapshot_id(), Some(SNAPSHOT_ONE));
        assert!(lineage.diagnostics().is_empty());
        assert!(PROHIBITED_VCS_SNAPSHOT_EFFECTS.contains(&"autocommit"));
    }

    #[test]
    fn restore_plans_require_fresh_workspace_proof_and_confirmation() {
        // r[verify agent_harness_runtime.hermes_vcs_snapshot_ledger.plans]
        let lineage = lineage();
        let confirmed = plan_snapshot_operation(
            &lineage,
            &SnapshotOperationPlanDraft {
                operation: SnapshotOperationKind::Restore,
                target_snapshot_id: SNAPSHOT_ONE,
                freshness: freshness(SNAPSHOT_ONE),
                required_capability: CAPABILITY_RESTORE,
                protection: SnapshotProtectionClass::Protected,
                confirmation: SnapshotConfirmationStatus::Confirmed,
            },
            SnapshotServiceStatus::Available,
        );
        let missing_confirmation = plan_snapshot_operation(
            &lineage,
            &SnapshotOperationPlanDraft {
                confirmation: SnapshotConfirmationStatus::Missing,
                ..SnapshotOperationPlanDraft {
                    operation: SnapshotOperationKind::Restore,
                    target_snapshot_id: SNAPSHOT_ONE,
                    freshness: freshness(SNAPSHOT_ONE),
                    required_capability: CAPABILITY_RESTORE,
                    protection: SnapshotProtectionClass::Protected,
                    confirmation: SnapshotConfirmationStatus::Confirmed,
                }
            },
            SnapshotServiceStatus::Available,
        );
        let stale = plan_snapshot_operation(
            &lineage,
            &SnapshotOperationPlanDraft {
                freshness: SnapshotFreshnessProofDraft {
                    observed_workspace_hash: HASH_B,
                    ..freshness(SNAPSHOT_ONE)
                },
                ..SnapshotOperationPlanDraft {
                    operation: SnapshotOperationKind::Restore,
                    target_snapshot_id: SNAPSHOT_ONE,
                    freshness: freshness(SNAPSHOT_ONE),
                    required_capability: CAPABILITY_RESTORE,
                    protection: SnapshotProtectionClass::Protected,
                    confirmation: SnapshotConfirmationStatus::Confirmed,
                }
            },
            SnapshotServiceStatus::Available,
        );

        assert_eq!(confirmed.status(), SnapshotPlanStatus::Ready);
        assert_eq!(missing_confirmation.status(), SnapshotPlanStatus::ConfirmationRequired);
        assert_eq!(stale.status(), SnapshotPlanStatus::StaleSnapshot);
    }

    #[test]
    fn missing_service_and_disabled_policy_make_no_snapshot_claim() {
        // r[verify agent_harness_runtime.hermes_vcs_snapshot_ledger.host_owned]
        let lineage = lineage();
        let missing = plan_snapshot_operation(
            &lineage,
            &SnapshotOperationPlanDraft {
                operation: SnapshotOperationKind::Compare,
                target_snapshot_id: SNAPSHOT_ONE,
                freshness: freshness(SNAPSHOT_ONE),
                required_capability: "snapshot-compare",
                protection: SnapshotProtectionClass::Unprotected,
                confirmation: SnapshotConfirmationStatus::Missing,
            },
            SnapshotServiceStatus::Missing,
        );
        let policy = plan_snapshot_policy(SnapshotServiceStatus::Missing, false);

        assert_eq!(missing.status(), SnapshotPlanStatus::ServiceUnavailable);
        assert_eq!(policy.status(), SnapshotPolicyPlanStatus::NoSnapshotClaim);
    }

    #[test]
    fn backend_command_capabilities_are_rejected_before_planning() {
        // r[verify agent_harness_runtime.hermes_vcs_snapshot_ledger.host_owned]
        let rejected = plan_snapshot_operation(
            &lineage(),
            &SnapshotOperationPlanDraft {
                operation: SnapshotOperationKind::Restore,
                target_snapshot_id: SNAPSHOT_ONE,
                freshness: freshness(SNAPSHOT_ONE),
                required_capability: "git checkout secret-ref",
                protection: SnapshotProtectionClass::Protected,
                confirmation: SnapshotConfirmationStatus::Confirmed,
            },
            SnapshotServiceStatus::Available,
        );

        assert_eq!(rejected.status(), SnapshotPlanStatus::Rejected);
        assert_eq!(rejected.diagnostic().unwrap().code(), SnapshotDiagnosticCode::UnsafeMetadata);
    }

    #[test]
    fn receipts_accept_hashes_and_reject_raw_diff_leaks() {
        // r[verify agent_harness_runtime.hermes_vcs_snapshot_ledger.receipts]
        let receipt = validate_snapshot_receipt(&SnapshotReceiptDraft {
            operation: SnapshotOperationKind::Restore,
            status: "planned",
            backend: SnapshotBackendKind::Jj,
            snapshot_id_hash: HASH_A,
            workspace_hash: HASH_A,
            changed_artifact_count: CHANGED_COUNT,
            freshness_class: "fresh",
            confirmation: SnapshotConfirmationStatus::Confirmed,
            policy_hash: HASH_B,
            receipt_hash: HASH_C,
            issue_code: Some("restore-planned"),
        })
        .unwrap();
        let leaking = validate_snapshot_receipt(&SnapshotReceiptDraft {
            operation: SnapshotOperationKind::Restore,
            status: "failed",
            backend: SnapshotBackendKind::Git,
            snapshot_id_hash: HASH_A,
            workspace_hash: HASH_A,
            changed_artifact_count: CHANGED_COUNT,
            freshness_class: "fresh",
            confirmation: SnapshotConfirmationStatus::Denied,
            policy_hash: HASH_B,
            receipt_hash: HASH_C,
            issue_code: Some("raw diff --git leaked"),
        })
        .expect_err("raw diff receipts must fail");

        assert_eq!(receipt.receipt_hash(), HASH_C);
        assert_eq!(leaking.code(), SnapshotDiagnosticCode::UnsafeMetadata);
    }
}
