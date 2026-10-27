//! Pure coding-session state machine and durable message wire helpers.

#![allow(clippy::too_many_lines, missing_docs)]

const FIELD_CONFIG: &str = "config";
const FIELD_INPUT: &str = "input";
const FIELD_ID: &str = "identity";
const FIELD_STATE: &str = "state";
const FIELD_RUN: &str = "run";
const FIELD_QUEUE: &str = "queue";
const FIELD_EFFECT: &str = "effect";
const FIELD_RECOVERY: &str = "recovery";
const FIELD_MESSAGE: &str = "message";
const FIELD_WIRE: &str = "message_wire";
const QUEUE_SOURCE: &str = "coding-session-host";
const QUEUE_TEXT_IDENTITY_DOMAIN: &str = "animus.coding-session.queue-text.v1";
const QUEUE_STATE_IDENTITY_DOMAIN: &str = "animus.coding-session.queue-state.v1";
const QUEUE_REDACTION_MARKER: &str = "private_queue_text_omitted";
const PROMPT_ROLE_SEPARATOR: &str = ": ";
const PROMPT_MESSAGE_SEPARATOR: &str = "\n";
const GENERATED_ENTRY_PREFIX: &str = "entry.";
const GENERATED_REPAIR_PREFIX: &str = "repair.";
const GENERATED_MAINTENANCE_PREFIX: &str = "maintenance.";
const GENERATED_AUTHORITY_PREFIX: &str = "authority.";
const GENERATED_ID_HEX_CHARS: usize = 24;
const COUNTER_INCREMENT: u32 = 1;
const WIRE_U16_BYTES: usize = 2;
const WIRE_RUN_LENGTH_OFFSET: usize = 2;
const WIRE_CALL_LENGTH_OFFSET: usize = 4;
const WIRE_TOOL_LENGTH_OFFSET: usize = 6;
const SYNTHETIC_REPAIR_BODY: &str = "interrupted tool result unavailable after recovery";
const MAX_AUTHORITY_BINDINGS_PER_COMMIT: usize = 1;
const INITIAL_PROMPT_MESSAGE_CAPACITY: usize = 2;
const COMMIT_EVENTS_PER_MESSAGE: usize = 2;

#[derive(Clone, Copy)]
struct PromptDraft<'a> {
    run_id: &'a str,
    entry_id: &'a str,
    input: &'a super::data::CodingSessionInput,
}

#[derive(Clone, Copy)]
struct QueueAppendDraft<'a> {
    item_id: &'a str,
    entry_id: &'a str,
    input: &'a super::data::CodingSessionInput,
    behavior: super::data::CodingSessionQueueBehavior,
}

#[derive(Clone, Copy)]
struct QueueEditDraft<'a> {
    item_id: &'a str,
    precondition: &'a super::data::CodingSessionQueueMutationPrecondition,
    input: &'a super::data::CodingSessionInput,
}

#[derive(Clone, Copy)]
struct QueueConflictDraft<'a> {
    item_id: &'a str,
    entry_id: &'a str,
    operation: super::data::CodingSessionQueueMutationOperation,
}

#[derive(Clone, Copy)]
struct SummarizerSuccess<'a> {
    entry_id: &'a str,
    summary: &'a str,
}

#[derive(Clone, Copy)]
struct PrefixedIdDraft<'a> {
    prefix: &'a str,
    hash: &'a str,
}

#[derive(Clone, Copy)]
struct CommitSuccess<'a> {
    new_head_hash: &'a str,
    active_leaf_id: &'a str,
    durable_cursor: u64,
    synchronized: bool,
}

struct StagedCommit {
    messages: Vec<super::data::CodingSessionMessage>,
    queued_item_id: Option<String>,
    after_commit: super::data::CodingSessionAfterCommit,
    events: Vec<crate::telemetry::RuntimeEvent>,
}

#[derive(Clone, Copy)]
struct GeneratedMessageDraft<'a> {
    run_id: &'a str,
    kind: super::data::CodingSessionMessageKind,
    body: &'a str,
    tool_correlation_id: Option<&'a str>,
    tool_name: Option<&'a str>,
    parent_id: Option<&'a str>,
}

#[derive(Clone, Copy)]
pub(super) struct CodingSessionMessageDraft<'a> {
    pub(super) entry_id: &'a str,
    pub(super) parent_id: Option<&'a str>,
    pub(super) run_id: &'a str,
    pub(super) kind: super::data::CodingSessionMessageKind,
    pub(super) body: &'a str,
    pub(super) tool_correlation_id: Option<&'a str>,
    pub(super) tool_name: Option<&'a str>,
}

// r[impl agent_harness_runtime.coding_session_host.functional_core]
/// Creates a validated coding-session state with no ambient services.
///
/// # Errors
///
/// Returns [`super::data::CodingSessionError`] when an identity, service posture, or queue policy is invalid.
pub fn new_coding_session_state(
    config: super::data::CodingSessionConfig,
    queue_policy: crate::mailbox::MessageQueuePolicy,
) -> Result<super::data::CodingSessionState, super::data::CodingSessionError> {
    validate_id(config.session_id.as_str())?;
    validate_id(config.branch_id.as_str())?;
    validate_services(config.services)?;
    crate::session_authority_profile::validate_session_authority_profile(&config.authority_profile)
        .map_err(|_error| error(super::data::CodingSessionIssue::InvalidInput, FIELD_CONFIG))?;
    crate::session_authority_profile::build_session_authority_binding(
        crate::session_authority_profile::SessionAuthorityBindingDraft {
            profile: &config.authority_profile,
            authorization_identity: config.authority_authorization_identity.as_str(),
            branch_id: config.branch_id.as_str(),
            prior_head_hash: super::data::CODING_SESSION_EMPTY_HEAD_HASH,
            prior_profile_identity: None,
        },
    )
    .map_err(|_error| error(super::data::CodingSessionIssue::InvalidInput, FIELD_CONFIG))?;
    if !queue_policy.admits_source(QUEUE_SOURCE) {
        return Err(error(
            super::data::CodingSessionIssue::InvalidInput,
            FIELD_CONFIG,
        ));
    }
    let recovery = if config.resume_required {
        super::data::CodingSessionRecoveryPosture::ResumeRequired
    } else {
        super::data::CodingSessionRecoveryPosture::NewStoreReady
    };
    let state = super::data::CodingSessionState {
        config,
        queue_policy,
        run: None,
        queue: None,
        queued_bodies: Vec::new(),
        queue_generation: super::data::CodingSessionQueueGeneration::default(),
        messages: Vec::new(),
        active_leaf_id: None,
        durable_head_hash: String::from(super::data::CODING_SESSION_EMPTY_HEAD_HASH),
        durable_cursor: 0,
        committed_authority_profile_identity: None,
        authority_binding_identity: None,
        pending_effect: None,
        recovery,
        terminal: None,
        last_issue: None,
        commit_count: 0,
        provider_dispatch_count: 0,
        tool_dispatch_count: 0,
        emitted_event_count: 0,
    };
    debug_assert!(state.run.is_none() && state.queue.is_none());
    debug_assert!(state.pending_effect.is_none());
    Ok(state)
}

/// Plans an explicit abandon, fork, or rebind outcome from durable session authority metadata.
///
/// # Errors
///
/// Returns [`super::data::CodingSessionError`] when committed binding metadata is malformed or the
/// requested transition cannot satisfy the authority-profile transition contract.
pub fn plan_coding_session_authority_transition(
    state: &super::data::CodingSessionState,
    requested_profile: &crate::session_authority_profile::SessionRuntimeAuthorityProfile,
    action: crate::session_authority_profile::SessionAuthorityTransitionAction<'_>,
) -> Result<
    crate::session_authority_profile::SessionAuthorityTransitionPlan,
    super::data::CodingSessionError,
> {
    let committed_binding = latest_authority_binding(state.messages())?;
    let comparison = crate::session_authority_profile::compare_session_authority_profiles(
        committed_binding
            .as_ref()
            .map(crate::session_authority_profile::SessionAuthorityBinding::profile),
        requested_profile,
    )
    .map_err(|_error| {
        error(
            super::data::CodingSessionIssue::AuthorityProfileMismatch,
            FIELD_CONFIG,
        )
    })?;
    debug_assert_eq!(
        comparison.current_profile_identity(),
        requested_profile.profile_identity()
    );
    debug_assert!(
        comparison.changed_components().len()
            <= crate::session_authority_profile::SessionAuthorityComponentClass::ALL.len()
    );
    crate::session_authority_profile::plan_session_authority_transition(
        crate::session_authority_profile::SessionAuthorityTransitionRequest {
            comparison: &comparison,
            requested_profile,
            prior_head_hash: state.durable_head_hash(),
            action,
        },
    )
    .map_err(|_error| {
        error(
            super::data::CodingSessionIssue::AuthorityReauthorizationRequired,
            FIELD_CONFIG,
        )
    })
}

/// Reduces one host command without performing effects.
///
/// # Errors
///
/// Returns [`super::data::CodingSessionError`] when the command is stale, invalid, or forbidden by state.
pub fn reduce_coding_session_command(
    state: &super::data::CodingSessionState,
    command: &super::data::CodingSessionCommand,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    match command {
        super::data::CodingSessionCommand::Prompt {
            run_id,
            entry_id,
            input,
        } => reduce_prompt(
            state,
            PromptDraft {
                run_id,
                entry_id,
                input,
            },
        ),
        super::data::CodingSessionCommand::Continue { run_id } => reduce_continue(state, run_id),
        super::data::CodingSessionCommand::Steer {
            item_id,
            entry_id,
            input,
        } => reduce_queue_append(
            state,
            QueueAppendDraft {
                item_id,
                entry_id,
                input,
                behavior: super::data::CodingSessionQueueBehavior::Steering,
            },
        ),
        super::data::CodingSessionCommand::FollowUp {
            item_id,
            entry_id,
            input,
        } => reduce_queue_append(
            state,
            QueueAppendDraft {
                item_id,
                entry_id,
                input,
                behavior: super::data::CodingSessionQueueBehavior::FollowUp,
            },
        ),
        super::data::CodingSessionCommand::EditQueued {
            item_id,
            precondition,
            input,
        } => reduce_queue_edit(
            state,
            QueueEditDraft {
                item_id,
                precondition,
                input,
            },
        ),
        super::data::CodingSessionCommand::RemoveQueued {
            item_id,
            precondition,
        } => reduce_queue_remove(state, item_id, precondition),
        super::data::CodingSessionCommand::Cancel { run_id } => reduce_cancel(state, run_id),
        super::data::CodingSessionCommand::Resume => reduce_resume(state),
        super::data::CodingSessionCommand::Compact { entry_id } => reduce_compact(state, entry_id),
        super::data::CodingSessionCommand::RebindAuthority {
            entry_id,
            profile,
            authorization_identity,
        } => reduce_rebind_authority(state, entry_id, profile, authorization_identity),
    }
}

/// Reduces one correlated host effect outcome without performing further effects.
///
/// # Errors
///
/// Returns [`super::data::CodingSessionError`] when the outcome is stale, malformed, or inconsistent.
pub fn reduce_coding_session_outcome(
    state: &super::data::CodingSessionState,
    outcome: &super::data::CodingSessionEffectOutcome,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    let pending = state
        .pending_effect
        .as_ref()
        .ok_or_else(|| error(super::data::CodingSessionIssue::StaleEffect, FIELD_EFFECT))?;
    if pending.effect_id() != outcome.effect_id() {
        return Err(error(
            super::data::CodingSessionIssue::StaleEffect,
            FIELD_EFFECT,
        ));
    }
    match (pending, outcome) {
        (
            super::data::CodingSessionEffectRequest::Commit(request),
            super::data::CodingSessionEffectOutcome::CommitSucceeded {
                new_head_hash,
                active_leaf_id,
                durable_cursor,
                synchronized,
                ..
            },
        ) => reduce_commit_success(
            state,
            request,
            CommitSuccess {
                new_head_hash,
                active_leaf_id,
                durable_cursor: *durable_cursor,
                synchronized: *synchronized,
            },
        ),
        (
            super::data::CodingSessionEffectRequest::Provider(request),
            super::data::CodingSessionEffectOutcome::ProviderSucceeded { reply, .. },
        ) => reduce_provider_success(state, request, reply),
        (
            super::data::CodingSessionEffectRequest::Tool(request),
            super::data::CodingSessionEffectOutcome::ToolSucceeded { response, .. },
        ) => reduce_tool_success(state, request, response),
        (
            super::data::CodingSessionEffectRequest::Cancellation(request),
            super::data::CodingSessionEffectOutcome::CancellationObserved {
                run_id, observed, ..
            },
        ) => reduce_cancellation_observed(state, request, run_id, *observed),
        (
            super::data::CodingSessionEffectRequest::Recovery(_request),
            super::data::CodingSessionEffectOutcome::RecoverySucceeded { snapshot, .. },
        ) => reduce_recovery_success(state, snapshot),
        (
            super::data::CodingSessionEffectRequest::Summarizer(request),
            super::data::CodingSessionEffectOutcome::SummarizerSucceeded {
                entry_id, summary, ..
            },
        ) => reduce_summarizer_success(state, request, SummarizerSuccess { entry_id, summary }),
        (_, super::data::CodingSessionEffectOutcome::Failed { service, issue, .. }) => {
            reduce_effect_failure(state, *service, *issue)
        }
        _mismatched => Err(error(
            super::data::CodingSessionIssue::StaleEffect,
            FIELD_EFFECT,
        )),
    }
}

// r[impl agent_harness_runtime.coding_session_host.events]
/// Projects one provider-neutral runtime event into shared frontend state.
#[must_use]
pub fn reduce_coding_session_frontend_event(
    state: &super::data::CodingSessionFrontendState,
    event: &crate::telemetry::RuntimeEvent,
) -> super::data::CodingSessionFrontendState {
    let mut next = state.clone();
    match event {
        crate::telemetry::RuntimeEvent::SessionStateChanged {
            status,
            queue_count,
            ..
        } => {
            next.lifecycle_status.clone_from(status);
            next.queue_count = *queue_count;
        }
        crate::telemetry::RuntimeEvent::SessionQueueMutationObserved { mutation } => {
            next.queue_generation = mutation.generation;
            next.queue_identity = Some(mutation.queue_identity.clone());
            next.queue_mutation_status = Some(String::from(mutation.status));
            next.queue_conflict = mutation.conflict;
        }
        crate::telemetry::RuntimeEvent::SessionCommitObserved { .. } => {
            next.committed_message_count = next.committed_message_count.saturating_add(1);
        }
        crate::telemetry::RuntimeEvent::SessionAuthorityProfileObserved { status, .. } => {
            next.lifecycle_status.clone_from(status);
        }
        crate::telemetry::RuntimeEvent::ToolRequested { request } => {
            next.active_tool_correlation = Some(String::from(request.correlation_id()));
        }
        crate::telemetry::RuntimeEvent::ToolResponded { .. }
        | crate::telemetry::RuntimeEvent::ToolDenied { .. } => {
            next.active_tool_correlation = None;
        }
        crate::telemetry::RuntimeEvent::Cancelled { .. } => {
            next.cancellation_observed = true;
        }
        crate::telemetry::RuntimeEvent::Shutdown { status } => {
            next.terminal_status = Some(String::from(status.as_str()));
        }
        _other => {}
    }
    debug_assert!(next.committed_message_count >= state.committed_message_count);
    debug_assert!(
        next.committed_message_count
            <= state
                .committed_message_count
                .saturating_add(COUNTER_INCREMENT)
    );
    next
}

/// Encodes a validated durable message into the deterministic private wire format.
///
/// # Errors
///
/// Returns [`super::data::CodingSessionError`] when the message is invalid or exceeds a wire bound.
pub fn encode_coding_session_message(
    message: &super::data::CodingSessionMessage,
) -> Result<Vec<u8>, super::data::CodingSessionError> {
    validate_message(message)?;
    debug_assert!(!message.entry_id().is_empty());
    debug_assert!(!message.run_id().is_empty());
    let run_bytes = message.run_id.as_bytes();
    let call_bytes = message
        .tool_correlation_id
        .as_deref()
        .unwrap_or_default()
        .as_bytes();
    let tool_bytes = message.tool_name.as_deref().unwrap_or_default().as_bytes();
    let run_len = u16::try_from(run_bytes.len())
        .map_err(|_overflow| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_WIRE))?;
    let call_len = u16::try_from(call_bytes.len())
        .map_err(|_overflow| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_WIRE))?;
    let tool_len = u16::try_from(tool_bytes.len())
        .map_err(|_overflow| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_WIRE))?;
    let capacity_bytes = super::data::MESSAGE_WIRE_PREFIX_BYTES
        .checked_add(run_bytes.len())
        .and_then(|value| value.checked_add(call_bytes.len()))
        .and_then(|value| value.checked_add(tool_bytes.len()))
        .and_then(|value| value.checked_add(message.body.len()))
        .ok_or_else(|| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_WIRE))?;
    let mut bytes = Vec::with_capacity(capacity_bytes);
    bytes.push(super::data::MESSAGE_WIRE_VERSION);
    bytes.push(message.kind.wire_code());
    bytes.extend_from_slice(run_len.to_le_bytes().as_slice());
    bytes.extend_from_slice(call_len.to_le_bytes().as_slice());
    bytes.extend_from_slice(tool_len.to_le_bytes().as_slice());
    bytes.extend_from_slice(run_bytes);
    bytes.extend_from_slice(call_bytes);
    bytes.extend_from_slice(tool_bytes);
    bytes.extend_from_slice(message.body.as_bytes());
    Ok(bytes)
}

/// Decodes and validates one deterministic private durable-message payload.
///
/// # Errors
///
/// Returns [`super::data::CodingSessionError`] when framing, text, identity, or message data is invalid.
pub fn decode_coding_session_message(
    entry_id: &str,
    parent_id: Option<&str>,
    payload: &[u8],
) -> Result<super::data::CodingSessionMessage, super::data::CodingSessionError> {
    if payload.len() < super::data::MESSAGE_WIRE_PREFIX_BYTES {
        return Err(error(
            super::data::CodingSessionIssue::InvalidInput,
            FIELD_WIRE,
        ));
    }
    if payload[0] != super::data::MESSAGE_WIRE_VERSION {
        return Err(error(
            super::data::CodingSessionIssue::InvalidInput,
            FIELD_WIRE,
        ));
    }
    debug_assert!(payload.len() >= super::data::MESSAGE_WIRE_PREFIX_BYTES);
    debug_assert_eq!(payload[0], super::data::MESSAGE_WIRE_VERSION);
    let kind = super::data::CodingSessionMessageKind::from_wire_code(payload[1])
        .ok_or_else(|| error(super::data::CodingSessionIssue::InvalidInput, FIELD_WIRE))?;
    let run_len = read_wire_length(payload, WIRE_RUN_LENGTH_OFFSET)?;
    let call_len = read_wire_length(payload, WIRE_CALL_LENGTH_OFFSET)?;
    let tool_len = read_wire_length(payload, WIRE_TOOL_LENGTH_OFFSET)?;
    let run_end = super::data::MESSAGE_WIRE_PREFIX_BYTES
        .checked_add(run_len)
        .ok_or_else(|| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_WIRE))?;
    let call_end = run_end
        .checked_add(call_len)
        .ok_or_else(|| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_WIRE))?;
    let tool_end = call_end
        .checked_add(tool_len)
        .ok_or_else(|| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_WIRE))?;
    if tool_end > payload.len() {
        return Err(error(
            super::data::CodingSessionIssue::InvalidInput,
            FIELD_WIRE,
        ));
    }
    let run_id = decode_utf8(&payload[super::data::MESSAGE_WIRE_PREFIX_BYTES..run_end])?;
    let tool_correlation_id = decode_optional_utf8(&payload[run_end..call_end])?;
    let tool_name = decode_optional_utf8(&payload[call_end..tool_end])?;
    let body = decode_utf8(&payload[tool_end..])?;
    build_message(CodingSessionMessageDraft {
        entry_id,
        parent_id,
        run_id: run_id.as_str(),
        kind,
        body: body.as_str(),
        tool_correlation_id: tool_correlation_id.as_deref(),
        tool_name: tool_name.as_deref(),
    })
}

/// Returns the hard effect-step bound enforced by the host shell.
#[must_use]
pub const fn coding_session_effect_step_bound() -> u32 {
    super::data::MAX_EFFECT_STEPS_PER_COMMAND
}

// r[impl agent_harness_runtime.coding_session_host.single_run]
fn reduce_prompt(
    state: &super::data::CodingSessionState,
    draft: PromptDraft<'_>,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    ensure_dispatch_ready(state)?;
    if state.run.is_some() {
        return Err(error(
            super::data::CodingSessionIssue::AlreadyRunning,
            FIELD_RUN,
        ));
    }
    ensure_no_pending_effect(state)?;
    validate_input(draft.input)?;
    validate_id(draft.run_id)?;
    let messages = build_prompt_messages(state, draft)?;
    let mut next = state.clone();
    next.run = Some(super::data::CodingSessionRun {
        run_id: String::from(draft.run_id),
        status: super::data::CodingSessionRunStatus::Active,
    });
    next.queue = Some(
        crate::mailbox::MessageQueueState::new(draft.run_id).map_err(|_queue_error| {
            error(super::data::CodingSessionIssue::QueueRejected, FIELD_QUEUE)
        })?,
    );
    next.queued_bodies.clear();
    next.terminal = None;
    next.last_issue = None;
    let events = vec![session_state_event(&next, "active")?];
    stage_commit(
        next,
        StagedCommit {
            messages,
            queued_item_id: None,
            after_commit: super::data::CodingSessionAfterCommit::DispatchProvider,
            events,
        },
    )
}

fn reduce_continue(
    state: &super::data::CodingSessionState,
    run_id: &str,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    ensure_no_pending_effect(state)?;
    ensure_dispatch_ready(state)?;
    if state.run.is_some() {
        return Err(error(
            super::data::CodingSessionIssue::AlreadyRunning,
            FIELD_RUN,
        ));
    }
    if state.messages.is_empty() {
        return Err(error(
            super::data::CodingSessionIssue::InvalidState,
            FIELD_STATE,
        ));
    }
    ensure_authority_bound(state)?;
    validate_id(run_id)?;
    let mut next = state.clone();
    next.run = Some(super::data::CodingSessionRun {
        run_id: String::from(run_id),
        status: super::data::CodingSessionRunStatus::Active,
    });
    next.queue = Some(
        crate::mailbox::MessageQueueState::new(run_id).map_err(|_queue_error| {
            error(super::data::CodingSessionIssue::QueueRejected, FIELD_QUEUE)
        })?,
    );
    next.queued_bodies.clear();
    next.terminal = None;
    next.last_issue = None;
    let events = vec![session_state_event(&next, "active")?];
    stage_provider_dispatch(next, events)
}

// r[impl agent_harness_runtime.coding_session_host.queues]
fn reduce_queue_append(
    state: &super::data::CodingSessionState,
    draft: QueueAppendDraft<'_>,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    validate_input(draft.input)?;
    validate_id(draft.item_id)?;
    validate_id(draft.entry_id)?;
    let run = state
        .run
        .as_ref()
        .ok_or_else(|| error(super::data::CodingSessionIssue::NotRunning, FIELD_RUN))?;
    if run.status != super::data::CodingSessionRunStatus::Active {
        return Err(error(
            super::data::CodingSessionIssue::InvalidState,
            FIELD_RUN,
        ));
    }
    let queue = state
        .queue
        .as_ref()
        .ok_or_else(|| error(super::data::CodingSessionIssue::InvalidState, FIELD_QUEUE))?;
    let body_hash = hash_parts(
        super::data::CODING_SESSION_MESSAGE_HASH_DOMAIN,
        &[draft.input.body.as_str()],
    );
    let transition = crate::mailbox::apply_message_queue_operation(
        queue,
        crate::mailbox::MessageQueueOperation::Append(crate::mailbox::QueuedMessageDraft {
            item_id: draft.item_id,
            active_work_id: run.run_id(),
            source: QUEUE_SOURCE,
            body_hash: body_hash.as_str(),
            safety: crate::mailbox::MessageSafetyClass::Safe,
            budget: crate::mailbox::MessageBudgetClass::WithinBudget,
            policy_hash: state.queue_policy.policy_hash(),
        }),
        &state.queue_policy,
    )
    .map_err(|_queue_error| error(super::data::CodingSessionIssue::QueueRejected, FIELD_QUEUE))?;
    let next_generation = next_queue_generation(state)?;
    let mut next = state.clone();
    next.queue = Some(transition.state);
    next.queue_generation = next_generation;
    next.queued_bodies
        .push(super::data::CodingSessionQueuedBody {
            item_id: String::from(draft.item_id),
            entry_id: String::from(draft.entry_id),
            behavior: draft.behavior,
            body: draft.input.body.clone(),
            body_hash,
        });
    let events = vec![session_state_event(&next, draft.behavior.as_str())?];
    finish_transition(next, events, state.pending_effect.clone())
}

// r[impl agent_harness_runtime.coding_session_queue_cas.commands]
#[must_use]
pub fn coding_session_queue_text_identity(text: &str) -> String {
    hash_parts(QUEUE_TEXT_IDENTITY_DOMAIN, &[text])
}

/// Returns the current private compare-and-swap precondition for one queue item.
///
/// This API exposes a private text identity and is only for an authorized owner
/// of the current session snapshot.
///
/// # Errors
///
/// Returns [`super::data::CodingSessionError`] when the item is absent or its
/// position exceeds the bounded command representation.
pub fn coding_session_queue_precondition(
    state: &super::data::CodingSessionState,
    item_id: &str,
) -> Result<super::data::CodingSessionQueueMutationPrecondition, super::data::CodingSessionError> {
    validate_id(item_id)?;
    let (position, queued) = state
        .queued_bodies
        .iter()
        .enumerate()
        .find(|(_position, queued)| queued.item_id == item_id)
        .ok_or_else(|| error(super::data::CodingSessionIssue::QueueRejected, FIELD_QUEUE))?;
    let position = u32::try_from(position)
        .map_err(|_overflow| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_QUEUE))?;
    Ok(super::data::CodingSessionQueueMutationPrecondition::new(
        queued.entry_id.clone(),
        state.queue_generation,
        position,
        coding_session_queue_text_identity(queued.body.as_str()),
    ))
}

/// Projects bounded queue metadata without queued text or private text identities.
///
/// # Errors
///
/// Returns [`super::data::CodingSessionError`] when an entry position exceeds
/// the bounded snapshot representation.
pub fn project_coding_session_queue_safe_snapshot(
    state: &super::data::CodingSessionState,
) -> Result<super::data::CodingSessionQueueSafeSnapshot, super::data::CodingSessionError> {
    let mut entries = Vec::with_capacity(state.queued_bodies.len());
    for (position, queued) in state.queued_bodies.iter().enumerate() {
        entries.push(super::data::CodingSessionQueueSafeEntry {
            item_id: queued.item_id.clone(),
            entry_id: queued.entry_id.clone(),
            position: u32::try_from(position).map_err(|_overflow| {
                error(super::data::CodingSessionIssue::BoundExceeded, FIELD_QUEUE)
            })?,
            behavior: queued.behavior,
        });
    }
    Ok(super::data::CodingSessionQueueSafeSnapshot {
        generation: state.queue_generation,
        queue_identity: queue_state_identity(state),
        entries,
    })
}

/// Projects metadata-only evidence for an accepted queue mutation.
///
/// # Errors
///
/// Returns [`super::data::CodingSessionError`] when safe snapshot projection fails.
pub fn project_coding_session_queue_success_event(
    state: &super::data::CodingSessionState,
    operation: super::data::CodingSessionQueueMutationOperation,
    item_id: &str,
    precondition: &super::data::CodingSessionQueueMutationPrecondition,
) -> Result<super::data::CodingSessionQueueMutationEvent, super::data::CodingSessionError> {
    validate_id(item_id)?;
    validate_id(precondition.entry_id())?;
    let snapshot = project_coding_session_queue_safe_snapshot(state)?;
    Ok(super::data::CodingSessionQueueMutationEvent {
        operation,
        status: "accepted",
        item_id: String::from(item_id),
        entry_id: String::from(precondition.entry_id()),
        position: Some(precondition.expected_position()),
        generation: snapshot.generation,
        queue_identity: snapshot.queue_identity,
        conflict: None,
        redaction_marker: QUEUE_REDACTION_MARKER,
    })
}

#[must_use]
pub fn project_coding_session_queue_conflict_event(
    conflict: &super::data::CodingSessionQueueConflict,
) -> super::data::CodingSessionQueueMutationEvent {
    let position = conflict
        .snapshot
        .entries
        .iter()
        .find(|entry| entry.item_id == conflict.item_id)
        .map(|entry| entry.position);
    super::data::CodingSessionQueueMutationEvent {
        operation: conflict.operation,
        status: "conflict",
        item_id: conflict.item_id.clone(),
        entry_id: conflict.entry_id.clone(),
        position,
        generation: conflict.snapshot.generation,
        queue_identity: conflict.snapshot.queue_identity.clone(),
        conflict: Some(conflict.kind),
        redaction_marker: QUEUE_REDACTION_MARKER,
    }
}

fn queue_state_identity(state: &super::data::CodingSessionState) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(QUEUE_STATE_IDENTITY_DOMAIN.as_bytes());
    hasher.update(super::data::HASH_SEPARATOR);
    hasher.update(&state.queue_generation.value().to_le_bytes());
    for queued in &state.queued_bodies {
        for part in [
            queued.item_id.as_str(),
            queued.entry_id.as_str(),
            queued.behavior.as_str(),
            queued.body_hash.as_str(),
        ] {
            hasher.update(super::data::HASH_SEPARATOR);
            hasher.update(part.as_bytes());
        }
    }
    hasher.finalize().to_hex().to_string()
}

fn next_queue_generation(
    state: &super::data::CodingSessionState,
) -> Result<super::data::CodingSessionQueueGeneration, super::data::CodingSessionError> {
    state
        .queue_generation
        .checked_next()
        .ok_or_else(|| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_QUEUE))
}

fn validate_queue_precondition(
    precondition: &super::data::CodingSessionQueueMutationPrecondition,
) -> Result<(), super::data::CodingSessionError> {
    validate_id(precondition.entry_id())?;
    validate_hash(precondition.expected_text_identity())
}

// r[impl agent_harness_runtime.coding_session_queue_cas.reducer]
fn compare_queue_precondition(
    state: &super::data::CodingSessionState,
    item_id: &str,
    precondition: &super::data::CodingSessionQueueMutationPrecondition,
    operation: super::data::CodingSessionQueueMutationOperation,
) -> Result<usize, super::data::CodingSessionError> {
    validate_id(item_id)?;
    let Some((position, queued)) = state
        .queued_bodies
        .iter()
        .enumerate()
        .find(|(_position, queued)| queued.item_id == item_id)
    else {
        return queue_conflict(
            state,
            QueueConflictDraft {
                item_id,
                entry_id: precondition.entry_id(),
                operation,
            },
            super::data::CodingSessionQueueConflictKind::MissingItem,
        );
    };
    if state.queue_generation != precondition.expected_generation() {
        return queue_conflict(
            state,
            QueueConflictDraft {
                item_id,
                entry_id: precondition.entry_id(),
                operation,
            },
            super::data::CodingSessionQueueConflictKind::StaleGeneration,
        );
    }
    if queued.entry_id != precondition.entry_id() {
        return queue_conflict(
            state,
            QueueConflictDraft {
                item_id,
                entry_id: precondition.entry_id(),
                operation,
            },
            super::data::CodingSessionQueueConflictKind::WrongEntry,
        );
    }
    let current_position = u32::try_from(position)
        .map_err(|_overflow| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_QUEUE))?;
    if current_position != precondition.expected_position() {
        return queue_conflict(
            state,
            QueueConflictDraft {
                item_id,
                entry_id: precondition.entry_id(),
                operation,
            },
            super::data::CodingSessionQueueConflictKind::MovedPosition,
        );
    }
    let current_text_identity = coding_session_queue_text_identity(queued.body.as_str());
    if current_text_identity != precondition.expected_text_identity() {
        return queue_conflict(
            state,
            QueueConflictDraft {
                item_id,
                entry_id: precondition.entry_id(),
                operation,
            },
            super::data::CodingSessionQueueConflictKind::StaleText,
        );
    }
    Ok(position)
}

// r[impl agent_harness_runtime.coding_session_queue_cas.conflicts]
fn queue_conflict<T>(
    state: &super::data::CodingSessionState,
    draft: QueueConflictDraft<'_>,
    kind: super::data::CodingSessionQueueConflictKind,
) -> Result<T, super::data::CodingSessionError> {
    let snapshot = project_coding_session_queue_safe_snapshot(state)?;
    Err(super::data::CodingSessionError::queue_conflict(
        super::data::CodingSessionQueueConflict {
            operation: draft.operation,
            item_id: String::from(draft.item_id),
            entry_id: String::from(draft.entry_id),
            kind,
            snapshot,
            redaction_marker: QUEUE_REDACTION_MARKER,
        },
    ))
}

fn reduce_queue_edit(
    state: &super::data::CodingSessionState,
    draft: QueueEditDraft<'_>,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    validate_input(draft.input)?;
    validate_queue_precondition(draft.precondition)?;
    let position = compare_queue_precondition(
        state,
        draft.item_id,
        draft.precondition,
        super::data::CodingSessionQueueMutationOperation::Edit,
    )?;
    let next_generation = next_queue_generation(state)?;
    let run = state
        .run
        .as_ref()
        .ok_or_else(|| error(super::data::CodingSessionIssue::NotRunning, FIELD_RUN))?;
    let queue = state
        .queue
        .as_ref()
        .ok_or_else(|| error(super::data::CodingSessionIssue::InvalidState, FIELD_QUEUE))?;
    let replacement_hash = hash_parts(
        super::data::CODING_SESSION_MESSAGE_HASH_DOMAIN,
        &[draft.input.body.as_str()],
    );
    let transition = crate::mailbox::apply_message_queue_operation(
        queue,
        crate::mailbox::MessageQueueOperation::Edit(crate::mailbox::MessageEditDraft {
            item_id: draft.item_id,
            active_work_id: run.run_id(),
            source: QUEUE_SOURCE,
            replacement_body_hash: replacement_hash.as_str(),
            safety: crate::mailbox::MessageSafetyClass::Safe,
            budget: crate::mailbox::MessageBudgetClass::WithinBudget,
        }),
        &state.queue_policy,
    )
    .map_err(|_queue_error| error(super::data::CodingSessionIssue::QueueRejected, FIELD_QUEUE))?;
    let mut next = state.clone();
    let queued = next
        .queued_bodies
        .get_mut(position)
        .ok_or_else(|| error(super::data::CodingSessionIssue::InvalidState, FIELD_QUEUE))?;
    queued.body.clone_from(&draft.input.body);
    queued.body_hash = replacement_hash;
    next.queue = Some(transition.state);
    next.queue_generation = next_generation;
    let mutation = project_coding_session_queue_success_event(
        &next,
        super::data::CodingSessionQueueMutationOperation::Edit,
        draft.item_id,
        draft.precondition,
    )?;
    let events = vec![
        session_state_event(&next, "queue_cas_edited")?,
        crate::telemetry::RuntimeEvent::SessionQueueMutationObserved { mutation },
    ];
    finish_transition(next, events, state.pending_effect.clone())
}

fn reduce_queue_remove(
    state: &super::data::CodingSessionState,
    item_id: &str,
    precondition: &super::data::CodingSessionQueueMutationPrecondition,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    validate_queue_precondition(precondition)?;
    let _position = compare_queue_precondition(
        state,
        item_id,
        precondition,
        super::data::CodingSessionQueueMutationOperation::Remove,
    )?;
    let next_generation = next_queue_generation(state)?;
    let run = state
        .run
        .as_ref()
        .ok_or_else(|| error(super::data::CodingSessionIssue::NotRunning, FIELD_RUN))?;
    let queue = state
        .queue
        .as_ref()
        .ok_or_else(|| error(super::data::CodingSessionIssue::InvalidState, FIELD_QUEUE))?;
    let transition = crate::mailbox::apply_message_queue_operation(
        queue,
        crate::mailbox::MessageQueueOperation::Delete(crate::mailbox::MessageItemOperationDraft {
            item_id,
            active_work_id: run.run_id(),
            source: QUEUE_SOURCE,
        }),
        &state.queue_policy,
    )
    .map_err(|_queue_error| error(super::data::CodingSessionIssue::QueueRejected, FIELD_QUEUE))?;
    let mut next = state.clone();
    next.queue = Some(transition.state);
    next.queue_generation = next_generation;
    next.queued_bodies
        .retain(|queued| queued.item_id != item_id);
    let mutation = project_coding_session_queue_success_event(
        &next,
        super::data::CodingSessionQueueMutationOperation::Remove,
        item_id,
        precondition,
    )?;
    let events = vec![
        session_state_event(&next, "queue_cas_removed")?,
        crate::telemetry::RuntimeEvent::SessionQueueMutationObserved { mutation },
    ];
    finish_transition(next, events, state.pending_effect.clone())
}

fn reduce_cancel(
    state: &super::data::CodingSessionState,
    run_id: &str,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    validate_id(run_id)?;
    let run = state
        .run
        .as_ref()
        .ok_or_else(|| error(super::data::CodingSessionIssue::NotRunning, FIELD_RUN))?;
    if run.run_id() != run_id {
        return Err(error(super::data::CodingSessionIssue::StaleRun, FIELD_RUN));
    }
    if matches!(
        state.pending_effect,
        Some(super::data::CodingSessionEffectRequest::Commit(_))
    ) {
        return Err(error(
            super::data::CodingSessionIssue::InvalidState,
            FIELD_EFFECT,
        ));
    }
    let mut next = state.clone();
    if let Some(active_run) = next.run.as_mut() {
        active_run.status = super::data::CodingSessionRunStatus::Cancelling;
    }
    let effect_id = effect_id(
        "cancellation",
        &[state.session_id(), run_id, state.durable_head_hash()],
    );
    let effect = super::data::CodingSessionEffectRequest::Cancellation(
        super::data::CodingSessionCancellationRequest {
            effect_id,
            run_id: String::from(run_id),
        },
    );
    next.pending_effect = Some(effect.clone());
    let events = vec![session_state_event(&next, "cancelling")?];
    finish_transition(next, events, Some(effect))
}

fn reduce_resume(
    state: &super::data::CodingSessionState,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    ensure_no_pending_effect(state)?;
    if state.run.is_some()
        || state.recovery != super::data::CodingSessionRecoveryPosture::ResumeRequired
    {
        return Err(error(
            super::data::CodingSessionIssue::InvalidState,
            FIELD_RECOVERY,
        ));
    }
    let effect = super::data::CodingSessionEffectRequest::Recovery(
        super::data::CodingSessionRecoveryRequest {
            effect: effect_id(
                "recovery",
                &[
                    state.session_id(),
                    state.branch_id(),
                    state.durable_head_hash(),
                ],
            ),
            session: String::from(state.session_id()),
            branch: String::from(state.branch_id()),
        },
    );
    let mut next = state.clone();
    next.pending_effect = Some(effect.clone());
    finish_transition(next, Vec::new(), Some(effect))
}

fn reduce_compact(
    state: &super::data::CodingSessionState,
    entry_id: &str,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    ensure_no_pending_effect(state)?;
    ensure_dispatch_ready(state)?;
    validate_id(entry_id)?;
    if state.messages.is_empty() {
        return Err(error(
            super::data::CodingSessionIssue::InvalidState,
            FIELD_MESSAGE,
        ));
    }
    let run_id = state.run.as_ref().map_or_else(
        || maintenance_run_id(state),
        |run| String::from(run.run_id()),
    );
    let effect = super::data::CodingSessionEffectRequest::Summarizer(
        super::data::CodingSessionSummarizerRequest {
            effect_id: effect_id(
                "summarizer",
                &[state.session_id(), entry_id, state.durable_head_hash()],
            ),
            run_id,
            entry_id: String::from(entry_id),
            messages: state
                .messages
                .iter()
                .filter(|message| {
                    message.kind() != super::data::CodingSessionMessageKind::AuthorityProfileBinding
                })
                .cloned()
                .collect(),
        },
    );
    let mut next = state.clone();
    next.pending_effect = Some(effect.clone());
    finish_transition(next, Vec::new(), Some(effect))
}

// r[impl agent_harness_runtime.session_authority_profile.durable_binding]
fn reduce_rebind_authority(
    state: &super::data::CodingSessionState,
    entry_id: &str,
    profile: &crate::session_authority_profile::SessionRuntimeAuthorityProfile,
    authorization_identity: &str,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    ensure_no_pending_effect(state)?;
    ensure_dispatch_ready(state)?;
    ensure_authority_bound(state)?;
    if state.run.is_some() {
        return Err(error(
            super::data::CodingSessionIssue::AlreadyRunning,
            FIELD_RUN,
        ));
    }
    validate_id(entry_id)?;
    let comparison = crate::session_authority_profile::compare_session_authority_profiles(
        Some(state.authority_profile()),
        profile,
    )
    .map_err(|_error| {
        error(
            super::data::CodingSessionIssue::AuthorityProfileMismatch,
            FIELD_CONFIG,
        )
    })?;
    let plan = crate::session_authority_profile::plan_session_authority_transition(
        crate::session_authority_profile::SessionAuthorityTransitionRequest {
            comparison: &comparison,
            requested_profile: profile,
            prior_head_hash: state.durable_head_hash(),
            action: crate::session_authority_profile::SessionAuthorityTransitionAction::Rebind {
                branch_id: state.branch_id(),
                authorization_identity,
            },
        },
    )
    .map_err(|_error| {
        error(
            super::data::CodingSessionIssue::AuthorityProfileMismatch,
            FIELD_CONFIG,
        )
    })?;
    let binding = plan.binding().ok_or_else(|| {
        error(
            super::data::CodingSessionIssue::AuthorityProfileMismatch,
            FIELD_CONFIG,
        )
    })?;
    let payload = crate::session_authority_profile::encode_session_authority_binding(binding)
        .map_err(|_error| error(super::data::CodingSessionIssue::InvalidInput, FIELD_CONFIG))?;
    let message = build_message(CodingSessionMessageDraft {
        entry_id,
        parent_id: state.active_leaf_id(),
        run_id: maintenance_run_id(state).as_str(),
        kind: super::data::CodingSessionMessageKind::AuthorityProfileBinding,
        body: payload.as_str(),
        tool_correlation_id: None,
        tool_name: None,
    })?;
    debug_assert_eq!(binding.profile(), profile);
    debug_assert_eq!(binding.prior_head_hash(), state.durable_head_hash());
    stage_commit(
        state.clone(),
        StagedCommit {
            messages: vec![message],
            queued_item_id: None,
            after_commit: super::data::CodingSessionAfterCommit::AuthorityRebound,
            events: Vec::new(),
        },
    )
}

fn reduce_commit_success(
    state: &super::data::CodingSessionState,
    request: &super::data::CodingSessionCommitRequest,
    outcome: CommitSuccess<'_>,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    if !outcome.synchronized || request.messages.is_empty() {
        return reduce_effect_failure(
            state,
            super::data::CodingSessionEffectService::Persistence,
            super::data::CodingSessionServiceIssue::Failed,
        );
    }
    validate_commit_success(state, request, outcome)?;
    let committed_binding = authority_binding_from_messages(request.messages.as_slice())?;
    if let Some(binding) = committed_binding.as_ref() {
        validate_committed_authority_binding(state, binding)?;
    }
    let mut next = state.clone();
    next.pending_effect = None;
    if let Some(binding) = committed_binding.as_ref() {
        apply_committed_authority_binding(&mut next, binding);
    }
    next.messages.extend(request.messages.iter().cloned());
    next.active_leaf_id = Some(String::from(outcome.active_leaf_id));
    next.durable_head_hash = String::from(outcome.new_head_hash);
    next.durable_cursor = outcome.durable_cursor;
    next.commit_count = increment(next.commit_count)?;
    acknowledge_queued_commit(&mut next, request.queued_item_id.as_deref())?;
    let mut events = Vec::with_capacity(
        request
            .messages
            .len()
            .saturating_mul(COMMIT_EVENTS_PER_MESSAGE),
    );
    for message in &request.messages {
        events.push(commit_event(&next, message)?);
        if message.kind() == super::data::CodingSessionMessageKind::AuthorityProfileBinding {
            let binding =
                crate::session_authority_profile::decode_session_authority_binding(message.body())
                    .map_err(|_error| {
                        error(
                            super::data::CodingSessionIssue::AuthorityProfileMismatch,
                            FIELD_MESSAGE,
                        )
                    })?;
            events.push(authority_profile_event(&next, &binding, "committed")?);
        }
    }
    handle_after_commit(next, &request.after_commit, events)
}

fn validate_commit_success(
    state: &super::data::CodingSessionState,
    request: &super::data::CodingSessionCommitRequest,
    outcome: CommitSuccess<'_>,
) -> Result<(), super::data::CodingSessionError> {
    validate_hash(outcome.new_head_hash)?;
    let has_expected_leaf = outcome.active_leaf_id == request.proposed_active_leaf_id();
    let has_expected_session = request.session_id() == state.session_id();
    let has_expected_branch = request.branch_id() == state.branch_id();
    let has_expected_head = request.prior_head_hash == state.durable_head_hash;
    let has_expected_parent = request.prior_active_leaf_id.as_deref() == state.active_leaf_id();
    if !has_expected_leaf || !has_expected_session {
        return Err(error(
            super::data::CodingSessionIssue::StaleEffect,
            FIELD_EFFECT,
        ));
    }
    if !has_expected_branch || !has_expected_head {
        return Err(error(
            super::data::CodingSessionIssue::StaleEffect,
            FIELD_EFFECT,
        ));
    }
    if !has_expected_parent {
        return Err(error(
            super::data::CodingSessionIssue::StaleEffect,
            FIELD_EFFECT,
        ));
    }
    let message_count = u64::try_from(request.messages.len()).map_err(|_overflow| {
        error(
            super::data::CodingSessionIssue::BoundExceeded,
            FIELD_MESSAGE,
        )
    })?;
    let expected_cursor = state
        .durable_cursor
        .checked_add(message_count)
        .ok_or_else(|| {
            error(
                super::data::CodingSessionIssue::BoundExceeded,
                FIELD_MESSAGE,
            )
        })?;
    if outcome.durable_cursor != expected_cursor {
        return Err(error(
            super::data::CodingSessionIssue::StaleEffect,
            FIELD_EFFECT,
        ));
    }
    debug_assert!(message_count > 0);
    debug_assert_eq!(outcome.durable_cursor, expected_cursor);
    Ok(())
}

fn reduce_provider_success(
    state: &super::data::CodingSessionState,
    request: &super::data::CodingSessionProviderRequest,
    reply: &super::data::CodingSessionProviderReply,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    ensure_current_run(state, request.run_id())?;
    let parent_id = state.active_leaf_id();
    match reply {
        super::data::CodingSessionProviderReply::Final(response) => {
            let message = build_generated_message(
                state,
                GeneratedMessageDraft {
                    run_id: request.run_id(),
                    kind: super::data::CodingSessionMessageKind::Assistant,
                    body: response.message(),
                    tool_correlation_id: None,
                    tool_name: None,
                    parent_id,
                },
            )?;
            let mut next = state.clone();
            next.pending_effect = None;
            stage_commit(
                next,
                StagedCommit {
                    messages: vec![message],
                    queued_item_id: None,
                    after_commit: super::data::CodingSessionAfterCommit::Complete {
                        response: response.clone(),
                    },
                    events: Vec::new(),
                },
            )
        }
        super::data::CodingSessionProviderReply::ToolCall {
            assistant_response,
            tool_request,
        } => {
            let message = build_generated_message(
                state,
                GeneratedMessageDraft {
                    run_id: request.run_id(),
                    kind: super::data::CodingSessionMessageKind::AssistantToolCall,
                    body: assistant_response.message(),
                    tool_correlation_id: Some(tool_request.correlation_id()),
                    tool_name: Some(tool_request.name()),
                    parent_id,
                },
            )?;
            let mut next = state.clone();
            next.pending_effect = None;
            stage_commit(
                next,
                StagedCommit {
                    messages: vec![message],
                    queued_item_id: None,
                    after_commit: super::data::CodingSessionAfterCommit::DispatchTool {
                        assistant_response: assistant_response.clone(),
                        tool_request: tool_request.clone(),
                    },
                    events: Vec::new(),
                },
            )
        }
    }
}

fn reduce_tool_success(
    state: &super::data::CodingSessionState,
    request: &super::data::CodingSessionToolRequest,
    response: &animus_adapters::ToolResponse,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    ensure_current_run(state, request.run_id())?;
    let message = build_generated_message(
        state,
        GeneratedMessageDraft {
            run_id: request.run_id(),
            kind: super::data::CodingSessionMessageKind::ToolResult,
            body: response.model_visible_text(),
            tool_correlation_id: Some(request.request().correlation_id()),
            tool_name: Some(request.request().name()),
            parent_id: state.active_leaf_id(),
        },
    )?;
    let mut next = state.clone();
    next.pending_effect = None;
    stage_commit(
        next,
        StagedCommit {
            messages: vec![message],
            queued_item_id: None,
            after_commit: super::data::CodingSessionAfterCommit::ContinueProvider {
                tool_response: response.clone(),
            },
            events: Vec::new(),
        },
    )
}

fn reduce_cancellation_observed(
    state: &super::data::CodingSessionState,
    request: &super::data::CodingSessionCancellationRequest,
    run_id: &str,
    observed: bool,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    ensure_current_run(state, request.run_id())?;
    if request.run_id() != run_id {
        return Err(error(super::data::CodingSessionIssue::StaleRun, FIELD_RUN));
    }
    let mut next = state.clone();
    next.pending_effect = None;
    if !observed {
        next.recovery = super::data::CodingSessionRecoveryPosture::Blocked;
        next.terminal = Some(super::data::CodingSessionTerminalKind::Blocked);
        next.last_issue = Some(super::data::CodingSessionIssue::CancellationFailure);
        let events = vec![session_state_event(&next, "cancellation_unobserved")?];
        return finish_transition(next, events, None);
    }
    next.run = None;
    next.queue = None;
    next.queued_bodies.clear();
    next.terminal = Some(super::data::CodingSessionTerminalKind::Cancelled);
    let events = vec![
        crate::telemetry::RuntimeEvent::Cancelled {
            reason: String::from("correlated cancellation observed"),
        },
        session_state_event(&next, "cancelled")?,
        crate::telemetry::RuntimeEvent::Shutdown {
            status: crate::RuntimeTerminalKind::Stopped,
        },
    ];
    finish_transition(next, events, None)
}

// r[impl agent_harness_runtime.coding_session_host.recovery]
fn reduce_recovery_success(
    state: &super::data::CodingSessionState,
    snapshot: &super::data::CodingSessionRecoverySnapshot,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    validate_recovery_snapshot(state, snapshot)?;
    if is_blocked_recovery_status(snapshot.status) {
        return block_recovery(state, snapshot.status);
    }
    let recovered_binding = latest_authority_binding(snapshot.messages.as_slice())?;
    let comparison = crate::session_authority_profile::compare_session_authority_profiles(
        recovered_binding
            .as_ref()
            .map(crate::session_authority_profile::SessionAuthorityBinding::profile),
        state.authority_profile(),
    )
    .map_err(|_error| {
        error(
            super::data::CodingSessionIssue::AuthorityProfileMismatch,
            FIELD_RECOVERY,
        )
    })?;
    let (mut next, mut events) = recovered_state(state, snapshot)?;
    if comparison.status()
        != crate::session_authority_profile::SessionAuthorityResumeStatus::ExactMatch
    {
        next.recovery = super::data::CodingSessionRecoveryPosture::Blocked;
        next.terminal = Some(super::data::CodingSessionTerminalKind::Blocked);
        next.last_issue = Some(match comparison.status() {
            crate::session_authority_profile::SessionAuthorityResumeStatus::LegacyUnbound => {
                super::data::CodingSessionIssue::AuthorityProfileUnbound
            }
            crate::session_authority_profile::SessionAuthorityResumeStatus::DriftBlocked
            | crate::session_authority_profile::SessionAuthorityResumeStatus::ExactMatch => {
                super::data::CodingSessionIssue::AuthorityProfileMismatch
            }
        });
        events.push(authority_comparison_event(&next, &comparison)?);
        return finish_transition(next, events, None);
    }
    let binding = recovered_binding.as_ref().ok_or_else(|| {
        error(
            super::data::CodingSessionIssue::AuthorityProfileUnbound,
            FIELD_RECOVERY,
        )
    })?;
    validate_recovered_authority_binding(&next, binding)?;
    next.committed_authority_profile_identity =
        Some(String::from(binding.profile().profile_identity()));
    next.authority_binding_identity = Some(String::from(binding.binding_identity()));
    events.push(authority_comparison_event(&next, &comparison)?);
    let repairs = build_recovery_repairs(&next)?;
    if repairs.is_empty() {
        next.recovery = super::data::CodingSessionRecoveryPosture::Ready;
        return finish_transition(next, events, None);
    }
    next.recovery = super::data::CodingSessionRecoveryPosture::RepairRequired;
    stage_commit(
        next,
        StagedCommit {
            messages: repairs,
            queued_item_id: None,
            after_commit: super::data::CodingSessionAfterCommit::RecoveryReady,
            events,
        },
    )
}

const fn is_blocked_recovery_status(status: super::data::CodingSessionRecoveryStatus) -> bool {
    matches!(
        status,
        super::data::CodingSessionRecoveryStatus::IncompleteTail
            | super::data::CodingSessionRecoveryStatus::Corrupt
    )
}

fn block_recovery(
    state: &super::data::CodingSessionState,
    status: super::data::CodingSessionRecoveryStatus,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(is_blocked_recovery_status(status));
    let mut next = state.clone();
    next.pending_effect = None;
    next.recovery = super::data::CodingSessionRecoveryPosture::Blocked;
    next.terminal = Some(super::data::CodingSessionTerminalKind::Blocked);
    next.last_issue = Some(super::data::CodingSessionIssue::RecoveryBlocked);
    let events = vec![recovery_event(&next, status)?];
    debug_assert_eq!(
        next.last_issue,
        Some(super::data::CodingSessionIssue::RecoveryBlocked)
    );
    finish_transition(next, events, None)
}

fn recovered_state(
    state: &super::data::CodingSessionState,
    snapshot: &super::data::CodingSessionRecoverySnapshot,
) -> Result<
    (
        super::data::CodingSessionState,
        Vec<crate::telemetry::RuntimeEvent>,
    ),
    super::data::CodingSessionError,
> {
    let mut next = state.clone();
    next.pending_effect = None;
    next.messages.clone_from(&snapshot.messages);
    next.active_leaf_id.clone_from(&snapshot.active_leaf_id);
    next.durable_head_hash
        .clone_from(&snapshot.committed_head_hash);
    next.durable_cursor = snapshot.durable_cursor;
    next.terminal = None;
    next.last_issue = None;
    let events = vec![recovery_event(&next, snapshot.status)?];
    debug_assert_eq!(next.messages, snapshot.messages);
    debug_assert_eq!(next.durable_cursor, snapshot.durable_cursor);
    Ok((next, events))
}

fn reduce_summarizer_success(
    state: &super::data::CodingSessionState,
    request: &super::data::CodingSessionSummarizerRequest,
    outcome: SummarizerSuccess<'_>,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    if request.entry_id() != outcome.entry_id {
        return Err(error(
            super::data::CodingSessionIssue::StaleEffect,
            FIELD_EFFECT,
        ));
    }
    validate_body(outcome.summary)?;
    let message = build_message(CodingSessionMessageDraft {
        entry_id: outcome.entry_id,
        parent_id: state.active_leaf_id(),
        run_id: request.run_id(),
        kind: super::data::CodingSessionMessageKind::Summary,
        body: outcome.summary,
        tool_correlation_id: None,
        tool_name: None,
    })?;
    let mut next = state.clone();
    next.pending_effect = None;
    stage_commit(
        next,
        StagedCommit {
            messages: vec![message],
            queued_item_id: None,
            after_commit: super::data::CodingSessionAfterCommit::CompactionReady,
            events: Vec::new(),
        },
    )
}

fn reduce_effect_failure(
    state: &super::data::CodingSessionState,
    service: super::data::CodingSessionEffectService,
    _issue: super::data::CodingSessionServiceIssue,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    let issue = match service {
        super::data::CodingSessionEffectService::Persistence
        | super::data::CodingSessionEffectService::Recovery => {
            super::data::CodingSessionIssue::PersistenceFailure
        }
        super::data::CodingSessionEffectService::Provider => {
            super::data::CodingSessionIssue::ProviderFailure
        }
        super::data::CodingSessionEffectService::Tool => {
            super::data::CodingSessionIssue::ToolFailure
        }
        super::data::CodingSessionEffectService::Cancellation => {
            super::data::CodingSessionIssue::CancellationFailure
        }
        super::data::CodingSessionEffectService::Summarizer => {
            super::data::CodingSessionIssue::SummarizerFailure
        }
        super::data::CodingSessionEffectService::Authority => {
            super::data::CodingSessionIssue::AuthorityEnforcementUnavailable
        }
        super::data::CodingSessionEffectService::Timer
        | super::data::CodingSessionEffectService::Policy => {
            super::data::CodingSessionIssue::UnavailableService
        }
    };
    let mut next = state.clone();
    next.pending_effect = None;
    next.recovery = super::data::CodingSessionRecoveryPosture::Blocked;
    next.terminal = Some(super::data::CodingSessionTerminalKind::Blocked);
    next.last_issue = Some(issue);
    let events = vec![session_state_event(&next, service.as_str())?];
    let transition = finish_transition(next, events, None)?;
    debug_assert_eq!(
        transition.state.terminal,
        Some(super::data::CodingSessionTerminalKind::Blocked)
    );
    Ok(transition)
}

fn handle_after_commit(
    mut state: super::data::CodingSessionState,
    after_commit: &super::data::CodingSessionAfterCommit,
    mut events: Vec<crate::telemetry::RuntimeEvent>,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    match after_commit {
        super::data::CodingSessionAfterCommit::DispatchProvider => {
            stage_provider_dispatch(state, events)
        }
        super::data::CodingSessionAfterCommit::DispatchTool {
            assistant_response,
            tool_request,
        } => {
            events.push(crate::telemetry::RuntimeEvent::ModelResponded {
                response: assistant_response.clone(),
            });
            stage_tool_dispatch(state, tool_request.clone(), events)
        }
        super::data::CodingSessionAfterCommit::ContinueProvider { tool_response } => {
            events.push(crate::telemetry::RuntimeEvent::ToolResponded {
                response: tool_response.clone(),
            });
            stage_provider_dispatch(state, events)
        }
        super::data::CodingSessionAfterCommit::Complete { response } => {
            if state.queued_bodies.is_empty() {
                events.push(crate::telemetry::RuntimeEvent::FinalResponse {
                    response: response.clone(),
                });
                state.run = None;
                state.queue = None;
                state.terminal = Some(super::data::CodingSessionTerminalKind::Completed);
                events.push(session_state_event(&state, "completed")?);
                events.push(crate::telemetry::RuntimeEvent::Shutdown {
                    status: crate::RuntimeTerminalKind::FinalResponse,
                });
                finish_transition(state, events, None)
            } else {
                events.push(crate::telemetry::RuntimeEvent::ModelResponded {
                    response: response.clone(),
                });
                stage_next_queued_commit(state, events)
            }
        }
        super::data::CodingSessionAfterCommit::InjectNextOrProvider => {
            let should_inject_all =
                state.config.queue_profile == super::data::CodingSessionQueueProfile::AllAtOnce;
            if should_inject_all && !state.queued_bodies.is_empty() {
                stage_next_queued_commit(state, events)
            } else {
                stage_provider_dispatch(state, events)
            }
        }
        super::data::CodingSessionAfterCommit::RecoveryReady => {
            state.recovery = super::data::CodingSessionRecoveryPosture::Ready;
            events.push(session_state_event(&state, "recovered")?);
            finish_transition(state, events, None)
        }
        super::data::CodingSessionAfterCommit::CompactionReady => {
            events.push(crate::telemetry::RuntimeEvent::CompactionRequested {
                summary: String::from("durable compaction boundary committed"),
            });
            finish_transition(state, events, None)
        }
        super::data::CodingSessionAfterCommit::AuthorityRebound => {
            events.push(session_state_event(&state, "authority_rebound")?);
            finish_transition(state, events, None)
        }
    }
}

// r[impl agent_harness_runtime.coding_session_host.durable_boundaries]
fn stage_commit(
    mut state: super::data::CodingSessionState,
    draft: StagedCommit,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    if draft.messages.is_empty() {
        return Err(error(
            super::data::CodingSessionIssue::InvalidInput,
            FIELD_MESSAGE,
        ));
    }
    validate_commit_messages(&state, draft.messages.as_slice())?;
    let last_hash = draft
        .messages
        .last()
        .map(super::data::CodingSessionMessage::message_hash)
        .ok_or_else(|| error(super::data::CodingSessionIssue::InvalidInput, FIELD_MESSAGE))?;
    let effect =
        super::data::CodingSessionEffectRequest::Commit(super::data::CodingSessionCommitRequest {
            effect_id: effect_id(
                "commit",
                &[state.session_id(), state.durable_head_hash(), last_hash],
            ),
            run_id: state.run.as_ref().map(|run| String::from(run.run_id())),
            session_id: String::from(state.session_id()),
            branch_id: String::from(state.branch_id()),
            prior_head_hash: state.durable_head_hash.clone(),
            prior_active_leaf_id: state.active_leaf_id.clone(),
            messages: draft.messages,
            queued_item_id: draft.queued_item_id,
            after_commit: draft.after_commit,
        });
    state.pending_effect = Some(effect.clone());
    finish_transition(state, draft.events, Some(effect))
}

fn stage_provider_dispatch(
    mut state: super::data::CodingSessionState,
    mut events: Vec<crate::telemetry::RuntimeEvent>,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    let run_id = current_run_id(&state)?.to_string();
    let request = build_provider_request(&state)?;
    let active_leaf_id = state
        .active_leaf_id()
        .ok_or_else(|| error(super::data::CodingSessionIssue::InvalidState, FIELD_STATE))?
        .to_string();
    let request_sequence = u64::from(state.provider_dispatch_count)
        .checked_add(u64::from(COUNTER_INCREMENT))
        .ok_or_else(|| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_STATE))?;
    let effect = super::data::CodingSessionEffectRequest::Provider(
        super::data::CodingSessionProviderRequest {
            effect_id: effect_id(
                "provider",
                &[
                    state.session_id(),
                    run_id.as_str(),
                    state.durable_head_hash(),
                ],
            ),
            run_id,
            session_id: String::from(state.session_id()),
            branch_id: String::from(state.branch_id()),
            active_leaf_id,
            durable_head_hash: String::from(state.durable_head_hash()),
            request_sequence,
            request: request.clone(),
        },
    );
    state.provider_dispatch_count = increment(state.provider_dispatch_count)?;
    state.pending_effect = Some(effect.clone());
    events.push(session_state_event(&state, "provider_dispatched")?);
    finish_transition(state, events, Some(effect))
}

fn stage_tool_dispatch(
    mut state: super::data::CodingSessionState,
    tool_request: animus_adapters::ToolRequest,
    mut events: Vec<crate::telemetry::RuntimeEvent>,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    let run_id = current_run_id(&state)?.to_string();
    let effect =
        super::data::CodingSessionEffectRequest::Tool(super::data::CodingSessionToolRequest {
            effect_id: effect_id(
                "tool",
                &[
                    state.session_id(),
                    run_id.as_str(),
                    tool_request.correlation_id(),
                    state.durable_head_hash(),
                ],
            ),
            run_id,
            request: tool_request.clone(),
        });
    state.tool_dispatch_count = increment(state.tool_dispatch_count)?;
    state.pending_effect = Some(effect.clone());
    events.push(crate::telemetry::RuntimeEvent::ToolRequested {
        request: tool_request,
    });
    finish_transition(state, events, Some(effect))
}

fn stage_next_queued_commit(
    mut state: super::data::CodingSessionState,
    events: Vec<crate::telemetry::RuntimeEvent>,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    let run_id = current_run_id(&state)?.to_string();
    let queue = state
        .queue
        .as_ref()
        .ok_or_else(|| error(super::data::CodingSessionIssue::InvalidState, FIELD_QUEUE))?;
    let transition = crate::mailbox::apply_message_queue_operation(
        queue,
        crate::mailbox::MessageQueueOperation::DispatchNext(
            crate::mailbox::MessageScopeOperationDraft {
                active_work_id: run_id.as_str(),
                source: QUEUE_SOURCE,
            },
        ),
        &state.queue_policy,
    )
    .map_err(|_queue_error| error(super::data::CodingSessionIssue::QueueRejected, FIELD_QUEUE))?;
    let dispatch = transition
        .state
        .pending_dispatch()
        .ok_or_else(|| error(super::data::CodingSessionIssue::InvalidState, FIELD_QUEUE))?;
    let queued = state
        .queued_bodies
        .iter()
        .find(|queued| queued.item_id == dispatch.item_id())
        .cloned()
        .ok_or_else(|| error(super::data::CodingSessionIssue::QueueRejected, FIELD_QUEUE))?;
    let message = build_message(CodingSessionMessageDraft {
        entry_id: queued.entry_id.as_str(),
        parent_id: state.active_leaf_id(),
        run_id: run_id.as_str(),
        kind: super::data::CodingSessionMessageKind::User,
        body: queued.body.as_str(),
        tool_correlation_id: None,
        tool_name: None,
    })?;
    state.queue_generation = next_queue_generation(&state)?;
    state.queue = Some(transition.state);
    stage_commit(
        state,
        StagedCommit {
            messages: vec![message],
            queued_item_id: Some(queued.item_id),
            after_commit: super::data::CodingSessionAfterCommit::InjectNextOrProvider,
            events,
        },
    )
}

fn acknowledge_queued_commit(
    state: &mut super::data::CodingSessionState,
    queued_item_id: Option<&str>,
) -> Result<(), super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    let Some(item_id) = queued_item_id else {
        return Ok(());
    };
    let run_id = current_run_id(state)?.to_string();
    let queue = state
        .queue
        .as_ref()
        .ok_or_else(|| error(super::data::CodingSessionIssue::InvalidState, FIELD_QUEUE))?;
    let transition = crate::mailbox::apply_message_queue_operation(
        queue,
        crate::mailbox::MessageQueueOperation::Acknowledge(
            crate::mailbox::DispatchAcknowledgementDraft {
                item_id,
                active_work_id: run_id.as_str(),
                outcome: crate::mailbox::MessageDispatchAcknowledgementOutcome::Accepted,
            },
        ),
        &state.queue_policy,
    )
    .map_err(|_queue_error| error(super::data::CodingSessionIssue::QueueRejected, FIELD_QUEUE))?;
    let next_generation = next_queue_generation(state)?;
    state.queue = Some(transition.state);
    state.queue_generation = next_generation;
    state
        .queued_bodies
        .retain(|queued| queued.item_id != item_id);
    Ok(())
}

fn build_provider_request(
    state: &super::data::CodingSessionState,
) -> Result<animus_adapters::ModelRequest, super::data::CodingSessionError> {
    let mut prompt = String::new();
    for message in &state.messages {
        if message.kind == super::data::CodingSessionMessageKind::AuthorityProfileBinding {
            continue;
        }
        prompt.push_str(message.kind.as_str());
        prompt.push_str(PROMPT_ROLE_SEPARATOR);
        prompt.push_str(message.body.as_str());
        prompt.push_str(PROMPT_MESSAGE_SEPARATOR);
    }
    animus_adapters::ModelRequest::new(prompt.as_str()).map_err(|_error| {
        error(
            super::data::CodingSessionIssue::BoundExceeded,
            FIELD_MESSAGE,
        )
    })
}

fn validate_commit_messages(
    state: &super::data::CodingSessionState,
    messages: &[super::data::CodingSessionMessage],
) -> Result<(), super::data::CodingSessionError> {
    let mut expected_parent = state.active_leaf_id();
    for message in messages {
        validate_message(message)?;
        if message.parent_id() != expected_parent {
            return Err(error(
                super::data::CodingSessionIssue::InvalidInput,
                FIELD_MESSAGE,
            ));
        }
        expected_parent = Some(message.entry_id());
    }
    Ok(())
}

fn validate_recovery_snapshot(
    state: &super::data::CodingSessionState,
    snapshot: &super::data::CodingSessionRecoverySnapshot,
) -> Result<(), super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    if snapshot.messages.len() > super::data::MAX_RECOVERY_MESSAGES {
        return Err(error(
            super::data::CodingSessionIssue::BoundExceeded,
            FIELD_RECOVERY,
        ));
    }
    validate_hash(snapshot.committed_head_hash.as_str())?;
    let message_count = u64::try_from(snapshot.messages.len()).map_err(|_overflow| {
        error(
            super::data::CodingSessionIssue::BoundExceeded,
            FIELD_RECOVERY,
        )
    })?;
    if snapshot.durable_cursor != message_count {
        return Err(error(
            super::data::CodingSessionIssue::RecoveryBlocked,
            FIELD_RECOVERY,
        ));
    }
    let mut expected_parent = None;
    for message in &snapshot.messages {
        validate_message(message)?;
        if message.parent_id() != expected_parent {
            return Err(error(
                super::data::CodingSessionIssue::RecoveryBlocked,
                FIELD_RECOVERY,
            ));
        }
        expected_parent = Some(message.entry_id());
    }
    if snapshot.active_leaf_id.as_deref() != expected_parent {
        return Err(error(
            super::data::CodingSessionIssue::RecoveryBlocked,
            FIELD_RECOVERY,
        ));
    }
    if snapshot.messages.is_empty()
        && snapshot.committed_head_hash != super::data::CODING_SESSION_EMPTY_HEAD_HASH
    {
        return Err(error(
            super::data::CodingSessionIssue::RecoveryBlocked,
            FIELD_RECOVERY,
        ));
    }
    if state.recovery != super::data::CodingSessionRecoveryPosture::ResumeRequired {
        return Err(error(
            super::data::CodingSessionIssue::InvalidState,
            FIELD_RECOVERY,
        ));
    }
    Ok(())
}

fn build_recovery_repairs(
    state: &super::data::CodingSessionState,
) -> Result<Vec<super::data::CodingSessionMessage>, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    let mut unmatched = Vec::<(&str, &str, &str)>::with_capacity(state.messages.len());
    for message in &state.messages {
        if message.kind == super::data::CodingSessionMessageKind::AssistantToolCall {
            let call_id = message.tool_correlation_id().ok_or_else(|| {
                error(
                    super::data::CodingSessionIssue::RecoveryBlocked,
                    FIELD_RECOVERY,
                )
            })?;
            let tool_name = message.tool_name().ok_or_else(|| {
                error(
                    super::data::CodingSessionIssue::RecoveryBlocked,
                    FIELD_RECOVERY,
                )
            })?;
            unmatched.push((call_id, tool_name, message.run_id()));
        } else if message.kind == super::data::CodingSessionMessageKind::ToolResult {
            let call_id = message.tool_correlation_id().ok_or_else(|| {
                error(
                    super::data::CodingSessionIssue::RecoveryBlocked,
                    FIELD_RECOVERY,
                )
            })?;
            unmatched.retain(|(pending, _, _)| *pending != call_id);
        }
    }
    let mut repairs = Vec::with_capacity(unmatched.len());
    let mut parent = state.active_leaf_id.clone();
    for (call_id, tool_name, run_id) in unmatched {
        let repair_hash = hash_parts(
            super::data::CODING_SESSION_MESSAGE_HASH_DOMAIN,
            &[state.session_id(), call_id, state.durable_head_hash()],
        );
        let entry_id = prefixed_id(PrefixedIdDraft {
            prefix: GENERATED_REPAIR_PREFIX,
            hash: repair_hash.as_str(),
        });
        let message = build_message(CodingSessionMessageDraft {
            entry_id: entry_id.as_str(),
            parent_id: parent.as_deref(),
            run_id,
            kind: super::data::CodingSessionMessageKind::Repair,
            body: SYNTHETIC_REPAIR_BODY,
            tool_correlation_id: Some(call_id),
            tool_name: Some(tool_name),
        })?;
        parent = Some(String::from(message.entry_id()));
        repairs.push(message);
    }
    Ok(repairs)
}

fn build_prompt_messages(
    state: &super::data::CodingSessionState,
    draft: PromptDraft<'_>,
) -> Result<Vec<super::data::CodingSessionMessage>, super::data::CodingSessionError> {
    let mut messages = Vec::with_capacity(INITIAL_PROMPT_MESSAGE_CAPACITY);
    let mut parent_id = state.active_leaf_id().map(String::from);
    if let Some(binding_message) =
        initial_authority_binding_message(state, draft.run_id, parent_id.as_deref())?
    {
        parent_id = Some(String::from(binding_message.entry_id()));
        messages.push(binding_message);
    }
    let user_message = build_message(CodingSessionMessageDraft {
        entry_id: draft.entry_id,
        parent_id: parent_id.as_deref(),
        run_id: draft.run_id,
        kind: super::data::CodingSessionMessageKind::User,
        body: draft.input.body.as_str(),
        tool_correlation_id: None,
        tool_name: None,
    })?;
    messages.push(user_message);
    debug_assert!(!messages.is_empty());
    debug_assert!(messages.len() <= INITIAL_PROMPT_MESSAGE_CAPACITY);
    Ok(messages)
}

fn initial_authority_binding_message(
    state: &super::data::CodingSessionState,
    run_id: &str,
    parent_id: Option<&str>,
) -> Result<Option<super::data::CodingSessionMessage>, super::data::CodingSessionError> {
    match state.committed_authority_profile_identity() {
        Some(committed_identity)
            if committed_identity == state.authority_profile().profile_identity() =>
        {
            Ok(None)
        }
        Some(_mismatched_identity) => Err(error(
            super::data::CodingSessionIssue::AuthorityProfileMismatch,
            FIELD_CONFIG,
        )),
        None => {
            let binding = crate::session_authority_profile::build_session_authority_binding(
                crate::session_authority_profile::SessionAuthorityBindingDraft {
                    profile: state.authority_profile(),
                    authorization_identity: state.authority_authorization_identity(),
                    branch_id: state.branch_id(),
                    prior_head_hash: state.durable_head_hash(),
                    prior_profile_identity: None,
                },
            )
            .map_err(|_error| {
                error(
                    super::data::CodingSessionIssue::AuthorityProfileMismatch,
                    FIELD_CONFIG,
                )
            })?;
            let payload =
                crate::session_authority_profile::encode_session_authority_binding(&binding)
                    .map_err(|_error| {
                        error(
                            super::data::CodingSessionIssue::AuthorityProfileMismatch,
                            FIELD_CONFIG,
                        )
                    })?;
            let entry_hash = hash_parts(
                super::data::CODING_SESSION_MESSAGE_HASH_DOMAIN,
                &[
                    state.session_id(),
                    state.branch_id(),
                    binding.binding_identity(),
                    state.durable_head_hash(),
                ],
            );
            let entry_id = prefixed_id(PrefixedIdDraft {
                prefix: GENERATED_AUTHORITY_PREFIX,
                hash: entry_hash.as_str(),
            });
            let message = build_message(CodingSessionMessageDraft {
                entry_id: entry_id.as_str(),
                parent_id,
                run_id,
                kind: super::data::CodingSessionMessageKind::AuthorityProfileBinding,
                body: payload.as_str(),
                tool_correlation_id: None,
                tool_name: None,
            })?;
            debug_assert_eq!(
                message.kind(),
                super::data::CodingSessionMessageKind::AuthorityProfileBinding
            );
            debug_assert_eq!(message.parent_id(), parent_id);
            Ok(Some(message))
        }
    }
}

fn authority_binding_from_messages(
    messages: &[super::data::CodingSessionMessage],
) -> Result<
    Option<crate::session_authority_profile::SessionAuthorityBinding>,
    super::data::CodingSessionError,
> {
    let mut binding = None;
    let mut binding_count_items = 0_usize;
    for message in messages {
        if message.kind() != super::data::CodingSessionMessageKind::AuthorityProfileBinding {
            continue;
        }
        binding_count_items = binding_count_items.saturating_add(1);
        if binding_count_items > MAX_AUTHORITY_BINDINGS_PER_COMMIT {
            return Err(error(
                super::data::CodingSessionIssue::AuthorityProfileMismatch,
                FIELD_MESSAGE,
            ));
        }
        binding = Some(
            crate::session_authority_profile::decode_session_authority_binding(message.body())
                .map_err(|_error| {
                    error(
                        super::data::CodingSessionIssue::AuthorityProfileMismatch,
                        FIELD_MESSAGE,
                    )
                })?,
        );
    }
    debug_assert!(binding_count_items <= MAX_AUTHORITY_BINDINGS_PER_COMMIT);
    debug_assert_eq!(
        binding.is_some(),
        binding_count_items == MAX_AUTHORITY_BINDINGS_PER_COMMIT
    );
    Ok(binding)
}

fn latest_authority_binding(
    messages: &[super::data::CodingSessionMessage],
) -> Result<
    Option<crate::session_authority_profile::SessionAuthorityBinding>,
    super::data::CodingSessionError,
> {
    let mut latest = None;
    for message in messages {
        if message.kind() == super::data::CodingSessionMessageKind::AuthorityProfileBinding {
            latest = Some(
                crate::session_authority_profile::decode_session_authority_binding(message.body())
                    .map_err(|_error| {
                        error(
                            super::data::CodingSessionIssue::AuthorityProfileMismatch,
                            FIELD_RECOVERY,
                        )
                    })?,
            );
        }
    }
    Ok(latest)
}

fn validate_committed_authority_binding(
    state: &super::data::CodingSessionState,
    binding: &crate::session_authority_profile::SessionAuthorityBinding,
) -> Result<(), super::data::CodingSessionError> {
    if binding.branch_id() != state.branch_id()
        || binding.prior_head_hash() != state.durable_head_hash()
    {
        return Err(error(
            super::data::CodingSessionIssue::AuthorityProfileMismatch,
            FIELD_MESSAGE,
        ));
    }
    if binding.prior_profile_identity() != state.committed_authority_profile_identity() {
        return Err(error(
            super::data::CodingSessionIssue::AuthorityProfileMismatch,
            FIELD_MESSAGE,
        ));
    }
    if state.committed_authority_profile_identity().is_none()
        && binding.profile() != state.authority_profile()
    {
        return Err(error(
            super::data::CodingSessionIssue::AuthorityProfileMismatch,
            FIELD_MESSAGE,
        ));
    }
    debug_assert_eq!(binding.branch_id(), state.branch_id());
    debug_assert_eq!(binding.prior_head_hash(), state.durable_head_hash());
    Ok(())
}

fn validate_recovered_authority_binding(
    state: &super::data::CodingSessionState,
    binding: &crate::session_authority_profile::SessionAuthorityBinding,
) -> Result<(), super::data::CodingSessionError> {
    if binding.branch_id() != state.branch_id() || binding.profile() != state.authority_profile() {
        return Err(error(
            super::data::CodingSessionIssue::AuthorityProfileMismatch,
            FIELD_RECOVERY,
        ));
    }
    debug_assert_eq!(binding.branch_id(), state.branch_id());
    debug_assert_eq!(binding.profile(), state.authority_profile());
    Ok(())
}

fn apply_committed_authority_binding(
    state: &mut super::data::CodingSessionState,
    binding: &crate::session_authority_profile::SessionAuthorityBinding,
) {
    state.config.authority_profile = binding.profile().clone();
    state.config.authority_authorization_identity = String::from(binding.authorization_identity());
    state.committed_authority_profile_identity =
        Some(String::from(binding.profile().profile_identity()));
    state.authority_binding_identity = Some(String::from(binding.binding_identity()));
    debug_assert_eq!(
        state.committed_authority_profile_identity(),
        Some(binding.profile().profile_identity())
    );
    debug_assert_eq!(
        state.authority_binding_identity(),
        Some(binding.binding_identity())
    );
}

fn ensure_authority_bound(
    state: &super::data::CodingSessionState,
) -> Result<(), super::data::CodingSessionError> {
    if state.committed_authority_profile_identity()
        != Some(state.authority_profile().profile_identity())
    {
        return Err(error(
            super::data::CodingSessionIssue::AuthorityProfileUnbound,
            FIELD_STATE,
        ));
    }
    if state.authority_binding_identity().is_none() {
        return Err(error(
            super::data::CodingSessionIssue::AuthorityProfileUnbound,
            FIELD_STATE,
        ));
    }
    Ok(())
}

fn build_generated_message(
    state: &super::data::CodingSessionState,
    draft: GeneratedMessageDraft<'_>,
) -> Result<super::data::CodingSessionMessage, super::data::CodingSessionError> {
    debug_assert!(state.run.is_some() || state.queue.is_none());
    debug_assert!(state.queue.is_some() || state.queued_bodies.is_empty());
    let source_hash = hash_parts(
        super::data::CODING_SESSION_MESSAGE_HASH_DOMAIN,
        &[
            state.session_id(),
            draft.run_id,
            draft.kind.as_str(),
            draft.body,
            draft.tool_correlation_id.unwrap_or_default(),
            state.durable_head_hash(),
        ],
    );
    let entry_id = prefixed_id(PrefixedIdDraft {
        prefix: GENERATED_ENTRY_PREFIX,
        hash: source_hash.as_str(),
    });
    build_message(CodingSessionMessageDraft {
        entry_id: entry_id.as_str(),
        parent_id: draft.parent_id,
        run_id: draft.run_id,
        kind: draft.kind,
        body: draft.body,
        tool_correlation_id: draft.tool_correlation_id,
        tool_name: draft.tool_name,
    })
}

fn build_message(
    draft: CodingSessionMessageDraft<'_>,
) -> Result<super::data::CodingSessionMessage, super::data::CodingSessionError> {
    validate_id(draft.entry_id)?;
    if let Some(parent) = draft.parent_id {
        validate_id(parent)?;
    }
    validate_id(draft.run_id)?;
    validate_body(draft.body)?;
    validate_tool_fields(draft.kind, draft.tool_correlation_id, draft.tool_name)?;
    if draft.kind == super::data::CodingSessionMessageKind::AuthorityProfileBinding {
        crate::session_authority_profile::decode_session_authority_binding(draft.body).map_err(
            |_error| {
                error(
                    super::data::CodingSessionIssue::AuthorityProfileMismatch,
                    FIELD_MESSAGE,
                )
            },
        )?;
    }
    let message_hash = hash_parts(
        super::data::CODING_SESSION_MESSAGE_HASH_DOMAIN,
        &[
            draft.entry_id,
            draft.parent_id.unwrap_or_default(),
            draft.run_id,
            draft.kind.as_str(),
            draft.body,
            draft.tool_correlation_id.unwrap_or_default(),
            draft.tool_name.unwrap_or_default(),
        ],
    );
    let message = super::data::CodingSessionMessage {
        entry_id: String::from(draft.entry_id),
        parent_id: draft.parent_id.map(String::from),
        run_id: String::from(draft.run_id),
        kind: draft.kind,
        body: String::from(draft.body),
        tool_correlation_id: draft.tool_correlation_id.map(String::from),
        tool_name: draft.tool_name.map(String::from),
        message_hash,
    };
    debug_assert_eq!(message.entry_id(), draft.entry_id);
    debug_assert_eq!(message.run_id(), draft.run_id);
    Ok(message)
}

fn validate_message(
    message: &super::data::CodingSessionMessage,
) -> Result<(), super::data::CodingSessionError> {
    let rebuilt = build_message(CodingSessionMessageDraft {
        entry_id: message.entry_id(),
        parent_id: message.parent_id(),
        run_id: message.run_id(),
        kind: message.kind(),
        body: message.body(),
        tool_correlation_id: message.tool_correlation_id(),
        tool_name: message.tool_name(),
    })?;
    if rebuilt.message_hash != message.message_hash {
        return Err(error(
            super::data::CodingSessionIssue::InvalidInput,
            FIELD_MESSAGE,
        ));
    }
    Ok(())
}

fn validate_tool_fields(
    kind: super::data::CodingSessionMessageKind,
    tool_correlation_id: Option<&str>,
    tool_name: Option<&str>,
) -> Result<(), super::data::CodingSessionError> {
    let needs_tool_binding = matches!(
        kind,
        super::data::CodingSessionMessageKind::AssistantToolCall
            | super::data::CodingSessionMessageKind::ToolResult
            | super::data::CodingSessionMessageKind::Repair
    );
    if needs_tool_binding != tool_correlation_id.is_some()
        || needs_tool_binding != tool_name.is_some()
    {
        return Err(error(
            super::data::CodingSessionIssue::InvalidInput,
            FIELD_MESSAGE,
        ));
    }
    debug_assert_eq!(tool_correlation_id.is_some(), needs_tool_binding);
    debug_assert_eq!(tool_name.is_some(), needs_tool_binding);
    if let Some(value) = tool_correlation_id {
        validate_id(value)?;
    }
    if let Some(value) = tool_name {
        validate_id(value)?;
    }
    Ok(())
}

fn validate_input(
    input: &super::data::CodingSessionInput,
) -> Result<(), super::data::CodingSessionError> {
    match input.posture {
        super::data::CodingSessionInputPosture::ExpandedAndAdmitted => {
            validate_body(input.body.as_str())
        }
        super::data::CodingSessionInputPosture::UnexpandedWorkflowCommand
        | super::data::CodingSessionInputPosture::Denied => Err(error(
            super::data::CodingSessionIssue::InputNotAdmitted,
            FIELD_INPUT,
        )),
    }
}

fn validate_body(value: &str) -> Result<(), super::data::CodingSessionError> {
    let is_valid = !value.is_empty()
        && value.len() <= super::data::MAX_MESSAGE_BYTES
        && !value.chars().any(|character| character == '\0');
    if !is_valid {
        return Err(error(
            super::data::CodingSessionIssue::InvalidInput,
            FIELD_INPUT,
        ));
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<(), super::data::CodingSessionError> {
    let has_valid_length_bytes = !value.is_empty() && value.len() <= super::data::MAX_SAFE_ID_BYTES;
    let has_valid_chars = value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'));
    if !has_valid_length_bytes || !has_valid_chars {
        return Err(error(
            super::data::CodingSessionIssue::InvalidInput,
            FIELD_ID,
        ));
    }
    Ok(())
}

fn validate_hash(value: &str) -> Result<(), super::data::CodingSessionError> {
    let is_valid = value.len()
        == crate::durable_session_store::DURABLE_SESSION_EMPTY_HEAD_HASH.len()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !is_valid {
        return Err(error(
            super::data::CodingSessionIssue::InvalidInput,
            FIELD_ID,
        ));
    }
    Ok(())
}

fn validate_services(
    services: super::data::CodingSessionServiceSet,
) -> Result<(), super::data::CodingSessionError> {
    let postures = [
        services.provider,
        services.tool,
        services.persistence,
        services.policy,
        services.cancellation,
        services.timer,
        services.summarizer,
    ];
    if postures
        .iter()
        .any(|posture| *posture != super::data::CodingSessionServicePosture::Available)
    {
        return Err(error(
            super::data::CodingSessionIssue::UnavailableService,
            FIELD_CONFIG,
        ));
    }
    debug_assert!(
        postures
            .iter()
            .all(|posture| *posture == super::data::CodingSessionServicePosture::Available)
    );
    debug_assert_eq!(
        services.provider,
        super::data::CodingSessionServicePosture::Available
    );
    Ok(())
}

fn ensure_no_pending_effect(
    state: &super::data::CodingSessionState,
) -> Result<(), super::data::CodingSessionError> {
    if state.pending_effect.is_some() {
        return Err(error(
            super::data::CodingSessionIssue::InvalidState,
            FIELD_EFFECT,
        ));
    }
    Ok(())
}

fn ensure_dispatch_ready(
    state: &super::data::CodingSessionState,
) -> Result<(), super::data::CodingSessionError> {
    if !state.recovery.permits_dispatch() {
        return Err(error(
            super::data::CodingSessionIssue::RecoveryBlocked,
            FIELD_RECOVERY,
        ));
    }
    Ok(())
}

fn ensure_current_run(
    state: &super::data::CodingSessionState,
    run_id: &str,
) -> Result<(), super::data::CodingSessionError> {
    let run = state
        .run
        .as_ref()
        .ok_or_else(|| error(super::data::CodingSessionIssue::NotRunning, FIELD_RUN))?;
    if run.run_id() != run_id {
        return Err(error(super::data::CodingSessionIssue::StaleRun, FIELD_RUN));
    }
    Ok(())
}

fn current_run_id(
    state: &super::data::CodingSessionState,
) -> Result<&str, super::data::CodingSessionError> {
    state
        .run
        .as_ref()
        .map(super::data::CodingSessionRun::run_id)
        .ok_or_else(|| error(super::data::CodingSessionIssue::NotRunning, FIELD_RUN))
}

fn session_state_event(
    state: &super::data::CodingSessionState,
    status: &str,
) -> Result<crate::telemetry::RuntimeEvent, super::data::CodingSessionError> {
    validate_id_or_status(status)?;
    let queue_count = u32::try_from(state.queued_bodies.len())
        .map_err(|_overflow| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_QUEUE))?;
    Ok(crate::telemetry::RuntimeEvent::SessionStateChanged {
        session_id: String::from(state.session_id()),
        run_id: state.run.as_ref().map(|run| String::from(run.run_id())),
        status: String::from(status),
        queue_count,
    })
}

fn commit_event(
    state: &super::data::CodingSessionState,
    message: &super::data::CodingSessionMessage,
) -> Result<crate::telemetry::RuntimeEvent, super::data::CodingSessionError> {
    validate_hash(state.durable_head_hash())?;
    Ok(crate::telemetry::RuntimeEvent::SessionCommitObserved {
        session_id: String::from(state.session_id()),
        run_id: String::from(message.run_id()),
        entry_id: String::from(message.entry_id()),
        message_kind: String::from(message.kind().as_str()),
        head_hash: String::from(state.durable_head_hash()),
        durable_cursor: state.durable_cursor,
    })
}

fn recovery_event(
    state: &super::data::CodingSessionState,
    status: super::data::CodingSessionRecoveryStatus,
) -> Result<crate::telemetry::RuntimeEvent, super::data::CodingSessionError> {
    validate_hash(state.durable_head_hash())?;
    Ok(crate::telemetry::RuntimeEvent::SessionRecoveryObserved {
        session_id: String::from(state.session_id()),
        status: String::from(status.as_str()),
        head_hash: String::from(state.durable_head_hash()),
        active_leaf_id: state.active_leaf_id.clone(),
    })
}

fn authority_profile_event(
    state: &super::data::CodingSessionState,
    binding: &crate::session_authority_profile::SessionAuthorityBinding,
    status: &str,
) -> Result<crate::telemetry::RuntimeEvent, super::data::CodingSessionError> {
    validate_id(status)?;
    validate_hash(binding.profile().profile_identity())?;
    validate_hash(binding.binding_identity())?;
    Ok(
        crate::telemetry::RuntimeEvent::SessionAuthorityProfileObserved {
            session_id: String::from(state.session_id()),
            branch_id: String::from(state.branch_id()),
            status: String::from(status),
            profile_identity: String::from(binding.profile().profile_identity()),
            binding_identity: Some(String::from(binding.binding_identity())),
            authorization_identity: Some(String::from(binding.authorization_identity())),
            enforcement_observation_identity: None,
            changed_components: Vec::new(),
        },
    )
}

fn authority_comparison_event(
    state: &super::data::CodingSessionState,
    comparison: &crate::session_authority_profile::SessionAuthorityComparison,
) -> Result<crate::telemetry::RuntimeEvent, super::data::CodingSessionError> {
    validate_hash(comparison.current_profile_identity())?;
    let changed_components = comparison
        .changed_components()
        .iter()
        .map(|class| String::from(class.as_str()))
        .collect::<Vec<_>>();
    debug_assert_eq!(
        comparison.current_profile_identity(),
        state.authority_profile().profile_identity()
    );
    debug_assert!(
        changed_components.len()
            <= crate::session_authority_profile::SessionAuthorityComponentClass::ALL.len()
    );
    Ok(
        crate::telemetry::RuntimeEvent::SessionAuthorityProfileObserved {
            session_id: String::from(state.session_id()),
            branch_id: String::from(state.branch_id()),
            status: String::from(comparison.status().as_str()),
            profile_identity: String::from(comparison.current_profile_identity()),
            binding_identity: state.authority_binding_identity.clone(),
            authorization_identity: Some(String::from(state.authority_authorization_identity())),
            enforcement_observation_identity: None,
            changed_components,
        },
    )
}

fn finish_transition(
    mut state: super::data::CodingSessionState,
    events: Vec<crate::telemetry::RuntimeEvent>,
    effect: Option<super::data::CodingSessionEffectRequest>,
) -> Result<super::data::CodingSessionTransition, super::data::CodingSessionError> {
    let event_count = u32::try_from(events.len())
        .map_err(|_overflow| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_STATE))?;
    state.emitted_event_count = state
        .emitted_event_count
        .checked_add(event_count)
        .ok_or_else(|| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_STATE))?;
    debug_assert_eq!(state.pending_effect.as_ref(), effect.as_ref());
    Ok(super::data::CodingSessionTransition {
        state,
        events,
        effect,
    })
}

fn increment(value: u32) -> Result<u32, super::data::CodingSessionError> {
    value
        .checked_add(COUNTER_INCREMENT)
        .ok_or_else(|| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_STATE))
}

fn effect_id(kind: &str, parts: &[&str]) -> String {
    hash_parts(
        super::data::CODING_SESSION_EFFECT_HASH_DOMAIN,
        &[&[kind], parts].concat(),
    )
}

fn hash_parts(domain: &str, parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(super::data::HASH_SEPARATOR);
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn prefixed_id(draft: PrefixedIdDraft<'_>) -> String {
    let bounded = draft
        .hash
        .get(..GENERATED_ID_HEX_CHARS)
        .unwrap_or(draft.hash);
    format!("{}{bounded}", draft.prefix)
}

fn maintenance_run_id(state: &super::data::CodingSessionState) -> String {
    let hash = hash_parts(
        super::data::CODING_SESSION_EFFECT_HASH_DOMAIN,
        &[state.session_id(), state.durable_head_hash()],
    );
    prefixed_id(PrefixedIdDraft {
        prefix: GENERATED_MAINTENANCE_PREFIX,
        hash: hash.as_str(),
    })
}

fn read_wire_length(
    payload: &[u8],
    offset: usize,
) -> Result<usize, super::data::CodingSessionError> {
    let end = offset
        .checked_add(WIRE_U16_BYTES)
        .ok_or_else(|| error(super::data::CodingSessionIssue::BoundExceeded, FIELD_WIRE))?;
    let bytes = payload
        .get(offset..end)
        .ok_or_else(|| error(super::data::CodingSessionIssue::InvalidInput, FIELD_WIRE))?;
    let array: [u8; WIRE_U16_BYTES] = bytes
        .try_into()
        .map_err(|_error| error(super::data::CodingSessionIssue::InvalidInput, FIELD_WIRE))?;
    Ok(usize::from(u16::from_le_bytes(array)))
}

fn decode_utf8(bytes: &[u8]) -> Result<String, super::data::CodingSessionError> {
    core::str::from_utf8(bytes)
        .map(String::from)
        .map_err(|_error| error(super::data::CodingSessionIssue::InvalidInput, FIELD_WIRE))
}

fn decode_optional_utf8(bytes: &[u8]) -> Result<Option<String>, super::data::CodingSessionError> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        decode_utf8(bytes).map(Some)
    }
}

fn validate_id_or_status(value: &str) -> Result<(), super::data::CodingSessionError> {
    validate_id(value)
}

pub(super) fn error(
    issue: super::data::CodingSessionIssue,
    field: &'static str,
) -> super::data::CodingSessionError {
    super::data::CodingSessionError::new(issue, field)
}

#[cfg(test)]
pub(super) fn test_message(
    draft: CodingSessionMessageDraft<'_>,
) -> Result<super::data::CodingSessionMessage, super::data::CodingSessionError> {
    build_message(draft)
}
