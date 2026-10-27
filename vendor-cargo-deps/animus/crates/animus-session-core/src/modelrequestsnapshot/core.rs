use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use blake3::Hasher;

use super::codec::{
    domain_hash, model_request_budget_identity, model_request_canonical_identity,
    model_request_output_control_identity, model_request_route_identity,
    model_request_tool_schema_set_identity, private_snapshot_payload, update_str, update_u32,
    update_u64,
};
use super::data::{
    BLAKE3_HEX_LENGTH, CommittedModelRequestSnapshot, MAX_SNAPSHOT_IDENTITIES,
    MAX_SNAPSHOT_LABEL_BYTES, MODEL_REQUEST_COMMIT_PLAN_IDENTITY_DOMAIN,
    MODEL_REQUEST_DISPATCH_PLAN_IDENTITY_DOMAIN, MODEL_REQUEST_SNAPSHOT_ENTRY_SCHEMA,
    MODEL_REQUEST_SNAPSHOT_IDENTITY_DOMAIN, MODEL_REQUEST_SNAPSHOT_RECORD_KIND,
    MODEL_REQUEST_SNAPSHOT_SCHEMA, MODEL_REQUEST_TERMINAL_IDENTITY_DOMAIN, ModelRequestAttemptKind,
    ModelRequestDispatchFacts, ModelRequestDispatchPlan, ModelRequestProviderTerminalDraft,
    ModelRequestProviderTerminalObservation, ModelRequestRecoveryDecision,
    ModelRequestRecoveryStatus, ModelRequestRetryOwnerDecision, ModelRequestSnapshot,
    ModelRequestSnapshotCommitObservation, ModelRequestSnapshotCommitPlan,
    ModelRequestSnapshotDraft, ModelRequestSnapshotError, ModelRequestSnapshotIssue,
};

const GENERATED_ENTRY_PREFIX: &str = "request-snapshot.";
const GENERATED_ENTRY_IDENTITY_CHARS: usize = 24;
const INITIAL_ATTEMPT_NUMBER: u32 = 1;
const FIRST_RETRY_ATTEMPT_NUMBER: u32 = 2;

// r[impl agent_harness_runtime.model_request_snapshot.projection]
// r[impl animus.dialect.binding]
/// Validates one complete provider-neutral private model-request snapshot.
///
/// # Errors
///
/// Returns a typed issue for missing sections, unsupported fields, unsafe labels,
/// duplicate identities, bounds, or any recomputed identity mismatch.
pub fn validate_model_request_snapshot(
    draft: &ModelRequestSnapshotDraft<'_>,
) -> Result<ModelRequestSnapshot, ModelRequestSnapshotError> {
    validate_snapshot_draft(draft)?;
    let canonical_request_identity = model_request_canonical_identity(draft.canonical_request)?;
    validate_exact_identity(
        draft.route.route_identity,
        model_request_route_identity(
            draft.route.provider_id,
            draft.route.provider_transport_id,
            draft.route.model_id,
            draft.route.model_dialect_id,
        )
        .as_str(),
        "model_route",
    )?;
    validate_exact_identity(
        draft.tool_schema_set_identity,
        model_request_tool_schema_set_identity(draft.canonical_request).as_str(),
        "tool_schema_set",
    )?;
    validate_exact_identity(
        draft.output_control_identity,
        model_request_output_control_identity(draft.canonical_request).as_str(),
        "output_controls",
    )?;
    validate_exact_identity(
        draft.budget_identity,
        model_request_budget_identity(draft.canonical_request).as_str(),
        "budgets",
    )?;
    let private_payload = private_snapshot_payload(draft, canonical_request_identity.as_str())?;
    let snapshot_identity = domain_hash(
        MODEL_REQUEST_SNAPSHOT_IDENTITY_DOMAIN,
        private_payload.as_slice(),
    );
    let snapshot = ModelRequestSnapshot {
        schema: MODEL_REQUEST_SNAPSHOT_SCHEMA,
        session_id: String::from(draft.session_id),
        branch_id: String::from(draft.branch_id),
        active_leaf_id: String::from(draft.active_leaf_id),
        turn_generation: draft.turn_generation,
        request_sequence: draft.request_sequence,
        canonical_request: draft.canonical_request.clone(),
        canonical_request_identity,
        provider_id: String::from(draft.route.provider_id),
        provider_transport_id: String::from(draft.route.provider_transport_id),
        model_id: String::from(draft.route.model_id),
        model_dialect_id: String::from(draft.route.model_dialect_id),
        route_identity: String::from(draft.route.route_identity),
        model_invocation_identity: String::from(draft.model_invocation_identity),
        request_context_plan_identity: String::from(draft.request_context_plan_identity),
        selected_context_contribution_identities: draft
            .selected_context_contribution_identities
            .iter()
            .map(|identity| String::from(*identity))
            .collect(),
        tool_schema_set_identity: String::from(draft.tool_schema_set_identity),
        output_control_identity: String::from(draft.output_control_identity),
        budget_identity: String::from(draft.budget_identity),
        authority_seal_identity: String::from(draft.authority_seal_identity),
        adapter_cohort_identity: String::from(draft.adapter_cohort_identity),
        private_payload,
        snapshot_identity,
    };
    debug_assert!(!snapshot.private_payload().is_empty());
    debug_assert_eq!(snapshot.schema(), MODEL_REQUEST_SNAPSHOT_SCHEMA);
    Ok(snapshot)
}

// r[impl agent_harness_runtime.model_request_snapshot.durability]
/// Plans one private durable snapshot append at the active session leaf.
///
/// # Errors
///
/// Returns a typed issue when the snapshot lineage or prior durable head is invalid.
pub fn plan_model_request_snapshot_commit(
    snapshot: ModelRequestSnapshot,
    prior_head_hash: &str,
) -> Result<ModelRequestSnapshotCommitPlan, ModelRequestSnapshotError> {
    validate_hash(prior_head_hash, "prior_head_hash")?;
    let identity_prefix = snapshot
        .snapshot_identity()
        .get(..GENERATED_ENTRY_IDENTITY_CHARS)
        .ok_or_else(|| {
            error(
                ModelRequestSnapshotIssue::InvalidIdentity,
                "snapshot_identity",
            )
        })?;
    let entry_id = format!("{GENERATED_ENTRY_PREFIX}{identity_prefix}");
    let mut hasher = Hasher::new();
    update_str(&mut hasher, MODEL_REQUEST_COMMIT_PLAN_IDENTITY_DOMAIN);
    update_str(&mut hasher, snapshot.snapshot_identity());
    update_str(&mut hasher, snapshot.session_id());
    update_str(&mut hasher, snapshot.branch_id());
    update_str(&mut hasher, snapshot.active_leaf_id());
    update_u64(&mut hasher, snapshot.turn_generation());
    update_u64(&mut hasher, snapshot.request_sequence());
    update_str(&mut hasher, prior_head_hash);
    update_str(&mut hasher, entry_id.as_str());
    let plan_identity = hasher.finalize().to_hex().to_string();
    Ok(ModelRequestSnapshotCommitPlan {
        entry_schema: MODEL_REQUEST_SNAPSHOT_ENTRY_SCHEMA,
        record_kind: MODEL_REQUEST_SNAPSHOT_RECORD_KIND,
        entry_id,
        parent_id: String::from(snapshot.active_leaf_id()),
        prior_head_hash: String::from(prior_head_hash),
        snapshot,
        plan_identity,
    })
}

// r[impl agent_harness_runtime.model_request_snapshot.durability]
/// Admits exact complete durable append evidence.
///
/// # Errors
///
/// Returns a typed issue for incomplete, unsynchronized, stale, substituted, or
/// differently projected commit observations.
pub fn observe_model_request_snapshot_commit(
    plan: ModelRequestSnapshotCommitPlan,
    observation: &ModelRequestSnapshotCommitObservation,
) -> Result<CommittedModelRequestSnapshot, ModelRequestSnapshotError> {
    validate_hash(observation.new_head_hash.as_str(), "new_head_hash")?;
    if !observation.complete || !observation.synchronized || observation.durable_cursor == 0 {
        return Err(error(
            ModelRequestSnapshotIssue::CommitMismatch,
            "snapshot_commit",
        ));
    }
    let snapshot = plan.snapshot();
    let exact = observation.plan_identity == plan.plan_identity()
        && observation.snapshot_identity == snapshot.snapshot_identity()
        && observation.session_id == snapshot.session_id()
        && observation.branch_id == snapshot.branch_id()
        && observation.active_leaf_id == snapshot.active_leaf_id()
        && observation.turn_generation == snapshot.turn_generation()
        && observation.request_sequence == snapshot.request_sequence();
    if !exact {
        return Err(error(
            ModelRequestSnapshotIssue::CommitMismatch,
            "snapshot_commit",
        ));
    }
    Ok(CommittedModelRequestSnapshot {
        plan,
        new_head_hash: observation.new_head_hash.clone(),
        durable_cursor: observation.durable_cursor,
    })
}

// r[impl agent_harness_runtime.model_request_snapshot.dispatch]
/// Plans an initial provider attempt from an exact committed snapshot.
///
/// # Errors
///
/// Returns a drift issue before provider contact when any current fact differs.
pub fn plan_initial_model_request_dispatch(
    committed: &CommittedModelRequestSnapshot,
    current: ModelRequestDispatchFacts<'_>,
) -> Result<ModelRequestDispatchPlan, ModelRequestSnapshotError> {
    plan_dispatch(
        committed,
        current,
        INITIAL_ATTEMPT_NUMBER,
        ModelRequestAttemptKind::Initial,
    )
}

// r[impl agent_harness_runtime.model_request_snapshot.dispatch]
/// Plans one exact same-request retry only after the existing retry owner admits it.
///
/// # Errors
///
/// Returns a retry or drift issue before provider contact when admission or any
/// bound request fact differs.
pub fn plan_same_model_request_retry(
    committed: &CommittedModelRequestSnapshot,
    current: ModelRequestDispatchFacts<'_>,
    owner_decision: ModelRequestRetryOwnerDecision,
    attempt_number: u32,
) -> Result<ModelRequestDispatchPlan, ModelRequestSnapshotError> {
    if owner_decision != ModelRequestRetryOwnerDecision::Admitted
        || attempt_number < FIRST_RETRY_ATTEMPT_NUMBER
    {
        return Err(error(
            ModelRequestSnapshotIssue::RetryDenied,
            "provider_retry_owner",
        ));
    }
    plan_dispatch(
        committed,
        current,
        attempt_number,
        ModelRequestAttemptKind::SameRequestRetry,
    )
}

/// Projects a correlated metadata-only terminal provider observation.
///
/// # Errors
///
/// Returns a mismatch when the snapshot, plan, cohort, or attempt is invalid.
pub fn observe_model_request_provider_terminal(
    dispatch: &ModelRequestDispatchPlan,
    draft: ModelRequestProviderTerminalDraft<'_>,
) -> Result<ModelRequestProviderTerminalObservation, ModelRequestSnapshotError> {
    validate_hash(draft.snapshot_identity, "snapshot_identity")?;
    validate_hash(draft.adapter_cohort_identity, "adapter_cohort_identity")?;
    if draft.snapshot_identity != dispatch.snapshot_identity()
        || draft.adapter_cohort_identity != dispatch.adapter_cohort_identity()
        || draft.attempt_number != dispatch.attempt_number()
    {
        return Err(error(
            ModelRequestSnapshotIssue::TerminalMismatch,
            "provider_terminal",
        ));
    }
    let mut hasher = Hasher::new();
    update_str(&mut hasher, MODEL_REQUEST_TERMINAL_IDENTITY_DOMAIN);
    update_str(&mut hasher, draft.snapshot_identity);
    update_str(&mut hasher, dispatch.plan_identity());
    update_str(&mut hasher, draft.adapter_cohort_identity);
    update_u32(&mut hasher, draft.attempt_number);
    update_str(&mut hasher, draft.status.as_str());
    Ok(ModelRequestProviderTerminalObservation {
        snapshot_identity: String::from(draft.snapshot_identity),
        dispatch_plan_identity: String::from(dispatch.plan_identity()),
        adapter_cohort_identity: String::from(draft.adapter_cohort_identity),
        attempt_number: draft.attempt_number,
        status: draft.status,
        observation_identity: hasher.finalize().to_hex().to_string(),
    })
}

/// Revalidates one decoded metadata-only terminal provider observation.
///
/// # Errors
///
/// Returns a terminal mismatch for malformed correlation values or identity drift.
pub fn validate_model_request_provider_terminal_observation(
    observation: &ModelRequestProviderTerminalObservation,
) -> Result<(), ModelRequestSnapshotError> {
    validate_hash(observation.snapshot_identity.as_str(), "snapshot_identity")?;
    validate_hash(
        observation.dispatch_plan_identity.as_str(),
        "dispatch_plan_identity",
    )?;
    validate_hash(
        observation.adapter_cohort_identity.as_str(),
        "adapter_cohort_identity",
    )?;
    validate_hash(
        observation.observation_identity.as_str(),
        "terminal_observation_identity",
    )?;
    if observation.attempt_number < INITIAL_ATTEMPT_NUMBER {
        return Err(error(
            ModelRequestSnapshotIssue::TerminalMismatch,
            "provider_terminal",
        ));
    }
    let mut hasher = Hasher::new();
    update_str(&mut hasher, MODEL_REQUEST_TERMINAL_IDENTITY_DOMAIN);
    update_str(&mut hasher, observation.snapshot_identity.as_str());
    update_str(&mut hasher, observation.dispatch_plan_identity.as_str());
    update_str(&mut hasher, observation.adapter_cohort_identity.as_str());
    update_u32(&mut hasher, observation.attempt_number);
    update_str(&mut hasher, observation.status.as_str());
    if hasher.finalize().to_hex().as_str() != observation.observation_identity {
        return Err(error(
            ModelRequestSnapshotIssue::TerminalMismatch,
            "provider_terminal",
        ));
    }
    Ok(())
}

// r[impl agent_harness_runtime.model_request_snapshot.recovery]
/// Classifies restart state without inferring provider delivery or replay safety.
///
/// # Errors
///
/// Returns a mismatch when supplied terminal evidence does not cite the exact snapshot.
pub fn classify_model_request_snapshot_recovery(
    committed: &CommittedModelRequestSnapshot,
    terminal: Option<&ModelRequestProviderTerminalObservation>,
) -> Result<ModelRequestRecoveryDecision, ModelRequestSnapshotError> {
    let snapshot_identity = committed.snapshot().snapshot_identity();
    match terminal {
        Some(observation) => {
            validate_model_request_provider_terminal_observation(observation)?;
            if observation.snapshot_identity != snapshot_identity
                || observation.adapter_cohort_identity
                    != committed.snapshot().adapter_cohort_identity()
            {
                return Err(error(
                    ModelRequestSnapshotIssue::TerminalMismatch,
                    "provider_terminal",
                ));
            }
            Ok(ModelRequestRecoveryDecision {
                snapshot_identity: String::from(snapshot_identity),
                status: ModelRequestRecoveryStatus::TerminalObserved,
                retry_owner_decision_required: false,
                automatic_replay: false,
            })
        }
        None => Ok(ModelRequestRecoveryDecision {
            snapshot_identity: String::from(snapshot_identity),
            status: ModelRequestRecoveryStatus::CommittedUnobserved,
            retry_owner_decision_required: true,
            automatic_replay: false,
        }),
    }
}

fn plan_dispatch(
    committed: &CommittedModelRequestSnapshot,
    current: ModelRequestDispatchFacts<'_>,
    attempt_number: u32,
    attempt_kind: ModelRequestAttemptKind,
) -> Result<ModelRequestDispatchPlan, ModelRequestSnapshotError> {
    validate_dispatch_facts(current)?;
    let snapshot = committed.snapshot();
    let exact = current.active_leaf_id == snapshot.active_leaf_id()
        && current.turn_generation == snapshot.turn_generation()
        && current.request_sequence == snapshot.request_sequence()
        && current.model_invocation_identity == snapshot.model_invocation_identity()
        && current.request_context_plan_identity == snapshot.request_context_plan_identity()
        && current.tool_schema_set_identity == snapshot.tool_schema_set_identity()
        && current.output_control_identity == snapshot.output_control_identity()
        && current.budget_identity == snapshot.budget_identity()
        && current.authority_seal_identity == snapshot.authority_seal_identity()
        && current.adapter_cohort_identity == snapshot.adapter_cohort_identity();
    if !exact {
        return Err(error(
            ModelRequestSnapshotIssue::DispatchDrift,
            "dispatch_facts",
        ));
    }
    let mut hasher = Hasher::new();
    update_str(&mut hasher, MODEL_REQUEST_DISPATCH_PLAN_IDENTITY_DOMAIN);
    update_str(&mut hasher, snapshot.snapshot_identity());
    update_str(&mut hasher, snapshot.canonical_request_identity());
    update_str(&mut hasher, snapshot.adapter_cohort_identity());
    update_str(&mut hasher, snapshot.active_leaf_id());
    update_u64(&mut hasher, snapshot.turn_generation());
    update_u64(&mut hasher, snapshot.request_sequence());
    update_u32(&mut hasher, attempt_number);
    update_str(&mut hasher, attempt_kind.as_str());
    Ok(ModelRequestDispatchPlan {
        snapshot_identity: String::from(snapshot.snapshot_identity()),
        canonical_request_identity: String::from(snapshot.canonical_request_identity()),
        adapter_cohort_identity: String::from(snapshot.adapter_cohort_identity()),
        active_leaf_id: String::from(snapshot.active_leaf_id()),
        turn_generation: snapshot.turn_generation(),
        request_sequence: snapshot.request_sequence(),
        attempt_number,
        attempt_kind,
        plan_identity: hasher.finalize().to_hex().to_string(),
    })
}

fn validate_snapshot_draft(
    draft: &ModelRequestSnapshotDraft<'_>,
) -> Result<(), ModelRequestSnapshotError> {
    if draft.schema != MODEL_REQUEST_SNAPSHOT_SCHEMA {
        return Err(error(
            ModelRequestSnapshotIssue::InvalidSchema,
            "snapshot_schema",
        ));
    }
    validate_label(draft.session_id, "session_id")?;
    validate_label(draft.branch_id, "branch_id")?;
    validate_label(draft.active_leaf_id, "active_leaf_id")?;
    validate_label(draft.route.provider_id, "provider_id")?;
    validate_label(draft.route.provider_transport_id, "provider_transport_id")?;
    validate_label(draft.route.model_id, "model_id")?;
    validate_label(draft.route.model_dialect_id, "model_dialect_id")?;
    if draft.turn_generation == 0 || draft.request_sequence == 0 {
        return Err(error(
            ModelRequestSnapshotIssue::MissingSection,
            "request_lineage",
        ));
    }
    if draft.canonical_request.messages.is_empty() {
        return Err(error(
            ModelRequestSnapshotIssue::MissingSection,
            "ordered_messages",
        ));
    }
    let identities = [
        (draft.route.route_identity, "model_route"),
        (draft.model_invocation_identity, "model_invocation"),
        (draft.request_context_plan_identity, "request_context_plan"),
        (draft.tool_schema_set_identity, "tool_schema_set"),
        (draft.output_control_identity, "output_controls"),
        (draft.budget_identity, "budgets"),
        (draft.authority_seal_identity, "authority_seal"),
        (draft.adapter_cohort_identity, "adapter_cohort"),
    ];
    for (identity, field) in identities {
        validate_hash(identity, field)?;
    }
    validate_context_identities(draft.selected_context_contribution_identities)?;
    if [
        draft.unsupported.credential,
        draft.unsupported.transport_header,
        draft.unsupported.provider_sdk_object,
        draft.unsupported.hidden_provider_default,
        draft.unsupported.provider_payload,
        draft.unsupported.private_path,
    ]
    .iter()
    .any(Option::is_some)
    {
        return Err(error(
            ModelRequestSnapshotIssue::UnsupportedField,
            "unsupported_request_field",
        ));
    }
    Ok(())
}

fn validate_context_identities(identities: &[&str]) -> Result<(), ModelRequestSnapshotError> {
    if identities.len() > MAX_SNAPSHOT_IDENTITIES {
        return Err(error(
            ModelRequestSnapshotIssue::BoundExceeded,
            "context_contributions",
        ));
    }
    let mut unique = BTreeSet::new();
    for identity in identities {
        validate_hash(identity, "context_contribution")?;
        if !unique.insert(*identity) {
            return Err(error(
                ModelRequestSnapshotIssue::DuplicateIdentity,
                "context_contribution",
            ));
        }
    }
    Ok(())
}

fn validate_dispatch_facts(
    current: ModelRequestDispatchFacts<'_>,
) -> Result<(), ModelRequestSnapshotError> {
    validate_label(current.active_leaf_id, "active_leaf_id")?;
    if current.turn_generation == 0 || current.request_sequence == 0 {
        return Err(error(
            ModelRequestSnapshotIssue::DispatchDrift,
            "dispatch_facts",
        ));
    }
    for (identity, field) in [
        (current.model_invocation_identity, "model_invocation"),
        (
            current.request_context_plan_identity,
            "request_context_plan",
        ),
        (current.tool_schema_set_identity, "tool_schema_set"),
        (current.output_control_identity, "output_controls"),
        (current.budget_identity, "budgets"),
        (current.authority_seal_identity, "authority_seal"),
        (current.adapter_cohort_identity, "adapter_cohort"),
    ] {
        validate_hash(identity, field)?;
    }
    Ok(())
}

fn validate_exact_identity(
    supplied: &str,
    expected: &str,
    field: &'static str,
) -> Result<(), ModelRequestSnapshotError> {
    if supplied == expected {
        Ok(())
    } else {
        Err(error(ModelRequestSnapshotIssue::IdentityMismatch, field))
    }
}

pub(crate) fn validate_hash(
    value: &str,
    field: &'static str,
) -> Result<(), ModelRequestSnapshotError> {
    if value.len() != BLAKE3_HEX_LENGTH
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(error(ModelRequestSnapshotIssue::InvalidIdentity, field));
    }
    Ok(())
}

pub(crate) fn validate_label(
    value: &str,
    field: &'static str,
) -> Result<(), ModelRequestSnapshotError> {
    if value.is_empty()
        || value.len() > MAX_SNAPSHOT_LABEL_BYTES
        || value.starts_with('/')
        || value.contains("://")
        || value.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(error(ModelRequestSnapshotIssue::InvalidLabel, field));
    }
    Ok(())
}

const fn error(issue: ModelRequestSnapshotIssue, field: &'static str) -> ModelRequestSnapshotError {
    ModelRequestSnapshotError::new(issue, field)
}
