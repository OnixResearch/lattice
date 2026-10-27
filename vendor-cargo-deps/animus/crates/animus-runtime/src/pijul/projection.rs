use animus_adapters::pijul::PijulRequest;
use animus_adapters::{
    CheckpointDraft, CheckpointRecord, SnapshotBackendKind, SnapshotIntentKind,
    SnapshotRecordDraft, SpeculativeWorkspaceDescriptor, SpeculativeWorkspaceDescriptorDraft,
    validate_snapshot_record,
};

use crate::provenance::{ProvenanceFact, ProvenanceOpaqueFact, ProvenanceSafeField};

use super::{
    CHECKPOINT_BEFORE_DOMAIN, PROJECTION_DOMAIN, PROJECTION_TARGETS, PROVENANCE_SCHEMA,
    PijulProfile, PijulProjectionBundle, PijulReceipt, PijulReceiptContext, PijulValidationError,
    PijulValidationIssue, RECEIPT_DOMAIN, REQUIRED_NON_CLAIMS, ValidatedPijulChange,
    ValidatedPijulOutput, WORKSPACE_DOMAIN, hash_parts, validate_blake3_hex,
};

const CHECKPOINT_STATUS: &str = "inspected";
const CHECKPOINT_BACKEND: &str = "other";
const WORKSPACE_FILE_INCLUSION: &str = "tracked_only";
const WORKSPACE_OPERATION: &str = "create";
const WORKSPACE_COMMIT_TARGET: &str = "parent";
const WORKSPACE_ABORT_POLICY: &str = "preserve_on_failure";
const WORKSPACE_CONFLICT_POLICY: &str = "fail";
const WORKSPACE_CLEANUP_POLICY: &str = "required";
const WORKSPACE_CAPABILITY: &str = "vcs_worktree";
const WORKSPACE_ROOT_LABEL: &str = "repository-root";
const SAFE_RECEIPT_STATUS: &str = "validated";
const SAFE_REPOSITORY_CLASS: &str = "explicit-local";
const SAFE_REDACTION_CLASS: &str = "metadata-only";
const SAFE_OMISSION_CLASS: &str = "content-omitted";
const SAFE_TRUNCATION_CLASS: &str = "not-truncated";
const REDACTED_FILE_COUNT: u32 = 0;
const EMPTY_CHANNEL_LABEL: &str = "channel-unreported";
const AUTHOR_REPORTED_LABEL: &str = "author-descriptive";
const AUTHOR_UNREPORTED_LABEL: &str = "author-unreported";
const SESSION_LABEL: &str = "pijul-adapter";
const INITIAL_FRESHNESS_EPOCH: u32 = 1;
const PROVENANCE_FIELD_COUNT: usize = 5;

/// Projects a validated log observation through existing VCS-neutral owners.
///
/// # Errors
///
/// Returns an error when no change is available or an existing owner rejects
/// the metadata-only projection.
pub fn project_pijul_log(
    profile: &PijulProfile,
    request: &PijulRequest,
    output: &ValidatedPijulOutput,
) -> Result<PijulProjectionBundle, PijulValidationError> {
    let ValidatedPijulOutput::Log(changes) = output else {
        return Err(PijulValidationError::new(
            PijulValidationIssue::ProjectionRejected,
            "projection_output",
        ));
    };
    let change = changes.first().ok_or_else(|| {
        PijulValidationError::new(PijulValidationIssue::ProjectionRejected, "empty_log")
    })?;
    let channel = request.channel.as_deref().unwrap_or(EMPTY_CHANNEL_LABEL);
    let projection_identity = projection_identity(profile, request, change, channel);
    let workspace_hash = hash_parts(
        WORKSPACE_DOMAIN,
        &[request.repository_identity.as_str(), channel],
    );
    let before_hash = hash_parts(
        CHECKPOINT_BEFORE_DOMAIN,
        &[request.repository_identity.as_str(), channel],
    );
    let changed_count = u32::try_from(changes.len()).unwrap_or(u32::MAX);
    let checkpoint_id = format!("pijul-change-{}", change.hash);
    let checkpoint = CheckpointRecord::new(CheckpointDraft {
        backend_kind: CHECKPOINT_BACKEND,
        checkpoint_id: checkpoint_id.as_str(),
        workspace_label: channel,
        workspace_hash: workspace_hash.as_str(),
        before_hash: before_hash.as_str(),
        after_hash: projection_identity.as_str(),
        changed_file_count: changed_count,
        session_hash: None,
        tool_label: Some("pijul-log"),
        status: CHECKPOINT_STATUS,
    })
    .map_err(|_error| projection_error("checkpoint"))?;
    let content_hashes = [projection_identity.as_str()];
    let receipt_hash = hash_parts(RECEIPT_DOMAIN, &[projection_identity.as_str(), "snapshot"]);
    let actor_label = if change.author_count > 0 {
        AUTHOR_REPORTED_LABEL
    } else {
        AUTHOR_UNREPORTED_LABEL
    };
    let snapshot = validate_snapshot_record(&SnapshotRecordDraft {
        intent: SnapshotIntentKind::Manual,
        backend: SnapshotBackendKind::Other,
        snapshot_id: projection_identity.as_str(),
        parent_snapshot_id: None,
        workspace_label: channel,
        workspace_hash: workspace_hash.as_str(),
        content_hashes: &content_hashes,
        changed_artifact_count: changed_count,
        actor_label,
        session_label: SESSION_LABEL,
        freshness_epoch: INITIAL_FRESHNESS_EPOCH,
        receipt_hash: receipt_hash.as_str(),
    })
    .map_err(|_error| projection_error("snapshot"))?;
    let provenance = project_provenance(request, output, projection_identity.as_str())?;
    let roots = [WORKSPACE_ROOT_LABEL];
    let capabilities = [WORKSPACE_CAPABILITY];
    let workspace = SpeculativeWorkspaceDescriptor::new(SpeculativeWorkspaceDescriptorDraft {
        source_id: request.repository_identity.as_str(),
        workspace_id: workspace_hash.as_str(),
        parent_workspace_id: None,
        writable_roots: &roots,
        file_inclusion: WORKSPACE_FILE_INCLUSION,
        operation: WORKSPACE_OPERATION,
        commit_target: WORKSPACE_COMMIT_TARGET,
        abort_policy: WORKSPACE_ABORT_POLICY,
        conflict_policy: WORKSPACE_CONFLICT_POLICY,
        cleanup_policy: WORKSPACE_CLEANUP_POLICY,
        required_capabilities: &capabilities,
    })
    .map_err(|_error| projection_error("workspace"))?;
    Ok(PijulProjectionBundle {
        checkpoint,
        snapshot,
        provenance,
        workspace,
        projection_identity,
    })
}

/// Creates a metadata-only receipt for a validated observation.
///
/// # Errors
///
/// Returns an error when caller-supplied policy or executor identities are not
/// lowercase BLAKE3 hex identities.
pub fn project_pijul_receipt(
    profile: &PijulProfile,
    request: &PijulRequest,
    output: &ValidatedPijulOutput,
    context: PijulReceiptContext<'_>,
) -> Result<PijulReceipt, PijulValidationError> {
    validate_blake3_hex(profile.policy_identity(), "policy_identity")?;
    validate_blake3_hex(context.executor_identity, "executor_identity")?;
    let record_count = output.record_count();
    let conflict_count = output.conflict_count();
    let record_count_text = record_count.to_string();
    let conflict_count_text = conflict_count.to_string();
    let file_count_text = REDACTED_FILE_COUNT.to_string();
    let receipt_identity = hash_parts(
        RECEIPT_DOMAIN,
        &[
            profile.cohort_identity(),
            profile.policy_identity(),
            context.executor_identity,
            SAFE_REPOSITORY_CLASS,
            request.operation.as_str(),
            record_count_text.as_str(),
            conflict_count_text.as_str(),
            file_count_text.as_str(),
            SAFE_REDACTION_CLASS,
            SAFE_OMISSION_CLASS,
            SAFE_TRUNCATION_CLASS,
        ],
    );
    Ok(PijulReceipt {
        cohort_identity: profile.cohort_identity().to_owned(),
        policy_identity: profile.policy_identity().to_owned(),
        executor_identity: context.executor_identity.to_owned(),
        repository_class: SAFE_REPOSITORY_CLASS.to_owned(),
        operation: request.operation,
        status: SAFE_RECEIPT_STATUS.to_owned(),
        record_count,
        conflict_count,
        file_count: REDACTED_FILE_COUNT,
        confirmation: request.confirmation,
        redaction_class: SAFE_REDACTION_CLASS.to_owned(),
        omission_class: SAFE_OMISSION_CLASS.to_owned(),
        truncation_class: SAFE_TRUNCATION_CLASS.to_owned(),
        projection_targets: PROJECTION_TARGETS
            .iter()
            .map(|target| (*target).to_owned())
            .collect(),
        non_claims: REQUIRED_NON_CLAIMS
            .iter()
            .map(|non_claim| (*non_claim).to_owned())
            .collect(),
        receipt_identity,
    })
}

fn projection_identity(
    profile: &PijulProfile,
    request: &PijulRequest,
    change: &ValidatedPijulChange,
    channel: &str,
) -> String {
    hash_parts(
        PROJECTION_DOMAIN,
        &[
            profile.cohort_identity(),
            request.repository_identity.as_str(),
            channel,
            change.hash.as_str(),
            change.state.as_deref().unwrap_or("state-unreported"),
        ],
    )
}

fn project_provenance(
    request: &PijulRequest,
    output: &ValidatedPijulOutput,
    fact_id: &str,
) -> Result<ProvenanceFact, PijulValidationError> {
    let record_count = output.record_count().to_string();
    let conflict_count = output.conflict_count().to_string();
    let channel_class = if request.channel.is_some() {
        "selected"
    } else {
        "unreported"
    };
    let metadata = Vec::with_capacity(PROVENANCE_FIELD_COUNT);
    let metadata = append_field(metadata, "operation", request.operation.as_str());
    let metadata = append_field(metadata, "record-count", record_count.as_str());
    let metadata = append_field(metadata, "conflict-count", conflict_count.as_str());
    let metadata = append_field(metadata, "channel-class", channel_class);
    let metadata = append_field(metadata, "redaction", SAFE_REDACTION_CLASS);
    ProvenanceOpaqueFact::new(fact_id.to_owned(), PROVENANCE_SCHEMA.to_owned(), metadata)
        .map(ProvenanceFact::Opaque)
        .map_err(|_error| projection_error("provenance"))
}

fn append_field(
    mut fields: Vec<ProvenanceSafeField>,
    key: &str,
    value: &str,
) -> Vec<ProvenanceSafeField> {
    fields.push(ProvenanceSafeField {
        key: key.to_owned(),
        value: value.to_owned(),
    });
    fields
}

fn projection_error(field: &'static str) -> PijulValidationError {
    PijulValidationError::new(PijulValidationIssue::ProjectionRejected, field)
}
