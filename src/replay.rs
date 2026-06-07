//! Replay debugger core and redaction-safe bundle format.
//!
//! This module is a functional core: it folds immutable event records into
//! timelines, moves cursors, validates records, summarizes values, and encodes
//! replay bundles without performing store, clock, runtime, or UI I/O.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;

use rat_nodegraph::model::PortId;
use serde::Deserialize;
use serde::Serialize;

use crate::engine::Value;
use crate::engine::ValueData;
use crate::engine::Workflow;
use crate::engine::executor::NodeStatus;
use crate::engine::executor::RunId;
use crate::engine::executor::StepResult;
use crate::engine::runner::ArtifactReference;
use crate::engine::runner::RunnerEventKind;

const REPLAY_BUNDLE_SCHEMA_VERSION: u32 = 1;
const FIRST_EVENT_SEQUENCE: u64 = 0;
const EVENT_SEQUENCE_INCREMENT: u64 = 1;
const FIRST_CURSOR_INDEX: usize = 0;
const SUMMARY_MAX_CHARS: usize = 160;
const REDACTED_SUMMARY: &str = "[redacted]";
const MISSING_ARTIFACT_PAYLOAD_SUMMARY: &str = "artifact payload unavailable";
const REPLAY_REDACTION_POLICY: &str = "summaries_only_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEventRecord {
    pub run_id: RunId,
    pub sequence: u64,
    pub timestamp_epoch_ms: Option<i64>,
    pub node_id: Option<u64>,
    pub port_id: Option<u64>,
    pub parent_run_id: Option<RunId>,
    pub kind: RunnerEventKind,
    pub summary: String,
    pub redacted: bool,
    pub value: Option<ReplayValueSummary>,
    pub artifact: Option<ReplayArtifactReference>,
    #[serde(default)]
    pub contains_plaintext_secret_material: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayValueSummary {
    pub type_tag: String,
    pub hash: String,
    pub summary: String,
    pub redacted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayArtifactReference {
    pub content_hash: String,
    pub type_tag: String,
    pub media_type: Option<String>,
    pub byte_length: usize,
    pub backend_ref: String,
    pub sensitive: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayArtifactPayloadState {
    Available,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayArtifactView {
    pub reference: ReplayArtifactReference,
    pub payload_state: ReplayArtifactPayloadState,
    pub display_summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayNodeState {
    Pending,
    Ready,
    Running,
    Cached,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFrame {
    pub cursor: usize,
    pub event: ReplayEventRecord,
    pub node_states: BTreeMap<u64, ReplayNodeState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayTimeline {
    pub run_id: RunId,
    pub frames: Vec<ReplayFrame>,
    pub diagnostics: Vec<ReplayDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCursor {
    pub position: usize,
    pub total_frames: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayDiagnostic {
    pub code: ReplayDiagnosticCode,
    pub sequence: Option<u64>,
    pub node_id: Option<u64>,
    pub message: String,
    pub fatal: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayDiagnosticCode {
    EmptyEvents,
    OutOfOrderEvent,
    DuplicateSequence,
    TerminalStateMutation,
    UnknownNode,
    MalformedArtifactRef,
    MalformedParentCorrelation,
    PlaintextSecretMaterial,
    MalformedBundle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayBundle {
    pub schema_version: u32,
    pub run_id: RunId,
    pub workflow_name: String,
    pub workflow_hash: Option<String>,
    pub workflow_snapshot: Option<Workflow>,
    pub redaction_policy: String,
    pub events: Vec<ReplayEventRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayBundleError {
    MalformedJson(String),
    Validation(Vec<ReplayDiagnostic>),
    Serialize(String),
}

impl std::fmt::Display for ReplayBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedJson(message) => write!(f, "malformed replay bundle JSON: {message}"),
            Self::Validation(diagnostics) => {
                write!(f, "replay bundle validation failed with {} diagnostics", diagnostics.len())
            }
            Self::Serialize(message) => write!(f, "replay bundle serialization failed: {message}"),
        }
    }
}

impl std::error::Error for ReplayBundleError {}

// r[impl n8nclone.workflow_execution.replay.core]
// r[impl n8nclone.workflow_validation.replay_records]
pub fn build_replay_timeline(
    run_id: RunId,
    events: &[ReplayEventRecord],
    known_node_ids: Option<&BTreeSet<u64>>,
) -> ReplayTimeline {
    let diagnostics = validate_replay_events(events, known_node_ids);
    if diagnostics.iter().any(|diagnostic| diagnostic.fatal) {
        return ReplayTimeline {
            run_id,
            frames: Vec::new(),
            diagnostics,
        };
    }

    let mut node_states = BTreeMap::new();
    let mut frames = Vec::with_capacity(events.len());
    for event in events {
        apply_event_state(&mut node_states, event);
        frames.push(ReplayFrame {
            cursor: frames.len(),
            event: event.clone(),
            node_states: node_states.clone(),
        });
    }

    ReplayTimeline {
        run_id,
        frames,
        diagnostics,
    }
}

// r[impl n8nclone.workflow_validation.replay_records]
pub fn validate_replay_events(
    events: &[ReplayEventRecord],
    known_node_ids: Option<&BTreeSet<u64>>,
) -> Vec<ReplayDiagnostic> {
    if events.is_empty() {
        return vec![replay_diagnostic(
            ReplayDiagnosticCode::EmptyEvents,
            None,
            None,
            "replay event stream is empty",
            true,
        )];
    }

    let mut diagnostics = Vec::new();
    let mut previous_sequence = None;
    let mut terminal_nodes = BTreeSet::new();
    for event in events {
        validate_sequence(event, previous_sequence, &mut diagnostics);
        validate_known_node(event, known_node_ids, &mut diagnostics);
        validate_terminal_transition(event, &mut terminal_nodes, &mut diagnostics);
        validate_artifact_ref(event, &mut diagnostics);
        validate_parent_correlation(event, &mut diagnostics);
        validate_secret_material(event, &mut diagnostics);
        previous_sequence = Some(event.sequence);
    }
    diagnostics
}

fn validate_sequence(
    event: &ReplayEventRecord,
    previous_sequence: Option<u64>,
    diagnostics: &mut Vec<ReplayDiagnostic>,
) {
    if let Some(previous_sequence) = previous_sequence {
        if event.sequence == previous_sequence {
            diagnostics.push(replay_diagnostic(
                ReplayDiagnosticCode::DuplicateSequence,
                Some(event.sequence),
                event.node_id,
                "duplicate replay event sequence",
                true,
            ));
        }
        if event.sequence <= previous_sequence {
            diagnostics.push(replay_diagnostic(
                ReplayDiagnosticCode::OutOfOrderEvent,
                Some(event.sequence),
                event.node_id,
                "replay event sequence is out of order",
                true,
            ));
        }
    }
}

fn validate_known_node(
    event: &ReplayEventRecord,
    known_node_ids: Option<&BTreeSet<u64>>,
    diagnostics: &mut Vec<ReplayDiagnostic>,
) {
    let Some(node_id) = event.node_id else {
        return;
    };
    let Some(known_node_ids) = known_node_ids else {
        return;
    };
    if !known_node_ids.contains(&node_id) {
        diagnostics.push(replay_diagnostic(
            ReplayDiagnosticCode::UnknownNode,
            Some(event.sequence),
            Some(node_id),
            "replay event references a node outside the workflow snapshot",
            true,
        ));
    }
}

fn validate_terminal_transition(
    event: &ReplayEventRecord,
    terminal_nodes: &mut BTreeSet<u64>,
    diagnostics: &mut Vec<ReplayDiagnostic>,
) {
    let Some(node_id) = event.node_id else {
        return;
    };
    if terminal_nodes.contains(&node_id) && !is_terminal_event(event.kind) {
        diagnostics.push(replay_diagnostic(
            ReplayDiagnosticCode::TerminalStateMutation,
            Some(event.sequence),
            Some(node_id),
            "replay event mutates a terminal node state",
            true,
        ));
    }
    if is_terminal_event(event.kind) {
        terminal_nodes.insert(node_id);
    }
}

fn validate_artifact_ref(event: &ReplayEventRecord, diagnostics: &mut Vec<ReplayDiagnostic>) {
    let Some(artifact) = &event.artifact else {
        return;
    };
    if artifact.content_hash.trim().is_empty() || artifact.backend_ref.trim().is_empty() {
        diagnostics.push(replay_diagnostic(
            ReplayDiagnosticCode::MalformedArtifactRef,
            Some(event.sequence),
            event.node_id,
            "replay artifact reference is missing hash or backend ref",
            true,
        ));
    }
}

fn validate_parent_correlation(event: &ReplayEventRecord, diagnostics: &mut Vec<ReplayDiagnostic>) {
    if event.parent_run_id == Some(event.run_id) {
        diagnostics.push(replay_diagnostic(
            ReplayDiagnosticCode::MalformedParentCorrelation,
            Some(event.sequence),
            event.node_id,
            "replay event cannot use its own run id as parent",
            true,
        ));
    }
}

fn validate_secret_material(event: &ReplayEventRecord, diagnostics: &mut Vec<ReplayDiagnostic>) {
    if event.contains_plaintext_secret_material {
        diagnostics.push(replay_diagnostic(
            ReplayDiagnosticCode::PlaintextSecretMaterial,
            Some(event.sequence),
            event.node_id,
            "replay event is marked as plaintext secret material",
            true,
        ));
    }
}

fn apply_event_state(node_states: &mut BTreeMap<u64, ReplayNodeState>, event: &ReplayEventRecord) {
    let Some(node_id) = event.node_id else {
        return;
    };
    if let Some(state) = node_state_for_event(event.kind) {
        node_states.insert(node_id, state);
    }
}

fn node_state_for_event(kind: RunnerEventKind) -> Option<ReplayNodeState> {
    match kind {
        RunnerEventKind::NodeReady => Some(ReplayNodeState::Ready),
        RunnerEventKind::NodeStarted => Some(ReplayNodeState::Running),
        RunnerEventKind::CacheHit => Some(ReplayNodeState::Cached),
        RunnerEventKind::NodeCompleted => Some(ReplayNodeState::Completed),
        RunnerEventKind::NodeFailed => Some(ReplayNodeState::Failed),
        RunnerEventKind::NodeCancelled => Some(ReplayNodeState::Cancelled),
        RunnerEventKind::RunStarted
        | RunnerEventKind::RuntimeStdout
        | RunnerEventKind::RuntimeStderr
        | RunnerEventKind::ArtifactCreated
        | RunnerEventKind::RunCompleted => None,
    }
}

fn is_terminal_event(kind: RunnerEventKind) -> bool {
    matches!(
        kind,
        RunnerEventKind::CacheHit
            | RunnerEventKind::NodeCompleted
            | RunnerEventKind::NodeFailed
            | RunnerEventKind::NodeCancelled
    )
}

// r[impl n8nclone.tui_ux.replay_debugger.timeline]
pub fn replay_cursor(total_frames: usize) -> ReplayCursor {
    ReplayCursor {
        position: FIRST_CURSOR_INDEX,
        total_frames,
    }
}

// r[impl n8nclone.tui_ux.replay_debugger.timeline]
pub fn move_replay_cursor(cursor: ReplayCursor, delta: i32) -> ReplayCursor {
    if cursor.total_frames == 0 {
        return cursor;
    }
    let last_position = cursor.total_frames.saturating_sub(1);
    let position = if delta.is_negative() {
        cursor.position.saturating_sub(delta.unsigned_abs() as usize)
    } else {
        cursor.position.saturating_add(delta as usize).min(last_position)
    };
    ReplayCursor { position, ..cursor }
}

// r[impl n8nclone.workflow_execution.replay.event_store]
// r[impl n8nclone.tui_ux.replay_debugger.active_run]
pub fn cancelled_replay_events(run_id: RunId, workflow: &Workflow, timestamp_epoch_ms: i64) -> Vec<ReplayEventRecord> {
    let run_boundary_event_count = 2;
    let mut events = Vec::with_capacity(workflow.graph.node_count().saturating_add(run_boundary_event_count));
    events.push(replay_event_record(
        run_id,
        next_sequence(&events),
        Some(timestamp_epoch_ms),
        None,
        None,
        RunnerEventKind::RunStarted,
        format!("run {} started", run_id.0),
        None,
        None,
        false,
    ));
    for node in workflow.graph.nodes() {
        events.push(replay_event_record(
            run_id,
            next_sequence(&events),
            Some(timestamp_epoch_ms),
            Some(node.id.0),
            None,
            RunnerEventKind::NodeCancelled,
            format!("node {} cancelled", node.label),
            None,
            None,
            false,
        ));
    }
    events.push(replay_event_record(
        run_id,
        next_sequence(&events),
        Some(timestamp_epoch_ms),
        None,
        None,
        RunnerEventKind::RunCompleted,
        format!("run {} cancelled", run_id.0),
        None,
        None,
        false,
    ));
    events
}

// r[impl n8nclone.workflow_execution.replay.event_store]
pub fn replay_events_from_results(
    run_id: RunId,
    workflow: &Workflow,
    results: &[StepResult],
    started_epoch_ms: i64,
    finished_epoch_ms: Option<i64>,
) -> Vec<ReplayEventRecord> {
    let mut events = Vec::with_capacity(replay_event_capacity(results));
    events.push(replay_event_record(
        run_id,
        next_sequence(&events),
        Some(started_epoch_ms),
        None,
        None,
        RunnerEventKind::RunStarted,
        format!("run {} started", run_id.0),
        None,
        None,
        false,
    ));
    for step in results {
        append_step_events(&mut events, run_id, workflow, step, finished_epoch_ms);
    }
    events.push(replay_event_record(
        run_id,
        next_sequence(&events),
        finished_epoch_ms,
        None,
        None,
        RunnerEventKind::RunCompleted,
        format!("run {} completed", run_id.0),
        None,
        None,
        false,
    ));
    events
}

fn replay_event_capacity(results: &[StepResult]) -> usize {
    let per_step_event_count = 2;
    let run_boundary_event_count = 2;
    results.len().saturating_mul(per_step_event_count).saturating_add(run_boundary_event_count)
}

fn append_step_events(
    events: &mut Vec<ReplayEventRecord>,
    run_id: RunId,
    workflow: &Workflow,
    step: &StepResult,
    timestamp_epoch_ms: Option<i64>,
) {
    let label = workflow.graph.node(step.node_id).map(|node| node.label.clone()).unwrap_or_else(|| "?".to_string());
    events.push(replay_event_record(
        run_id,
        next_sequence(events),
        timestamp_epoch_ms,
        Some(step.node_id.0),
        None,
        RunnerEventKind::NodeStarted,
        format!("node {label} started"),
        None,
        None,
        false,
    ));
    match &step.status {
        NodeStatus::Completed { outputs, cached } => {
            append_completed_step_event(events, run_id, step, &label, outputs, *cached, timestamp_epoch_ms)
        }
        NodeStatus::Failed { error } => events.push(replay_event_record(
            run_id,
            next_sequence(events),
            timestamp_epoch_ms,
            Some(step.node_id.0),
            None,
            RunnerEventKind::NodeFailed,
            format!("node {label} failed: {error}"),
            None,
            None,
            false,
        )),
        NodeStatus::Skipped => events.push(replay_event_record(
            run_id,
            next_sequence(events),
            timestamp_epoch_ms,
            Some(step.node_id.0),
            None,
            RunnerEventKind::NodeCancelled,
            format!("node {label} skipped"),
            None,
            None,
            false,
        )),
        NodeStatus::Pending | NodeStatus::Running => {}
    }
}

fn append_completed_step_event(
    events: &mut Vec<ReplayEventRecord>,
    run_id: RunId,
    step: &StepResult,
    label: &str,
    outputs: &HashMap<PortId, Value>,
    cached: bool,
    timestamp_epoch_ms: Option<i64>,
) {
    let output_summary = outputs.values().next().map(summarize_value);
    let is_redacted = output_summary.as_ref().is_some_and(|summary| summary.redacted);
    let kind = if cached {
        RunnerEventKind::CacheHit
    } else {
        RunnerEventKind::NodeCompleted
    };
    let cache_label = if cached { " from cache" } else { "" };
    events.push(replay_event_record(
        run_id,
        next_sequence(events),
        timestamp_epoch_ms,
        Some(step.node_id.0),
        output_summary.as_ref().and_then(|summary| primary_output_port(outputs, summary)),
        kind,
        format!("node {label} completed{cache_label}"),
        output_summary,
        None,
        is_redacted,
    ));
}

fn primary_output_port(outputs: &HashMap<PortId, Value>, summary: &ReplayValueSummary) -> Option<u64> {
    outputs
        .iter()
        .find(|(_port_id, value)| value.hash.to_hex() == summary.hash)
        .map(|(port_id, _value)| port_id.0)
}

// r[impl n8nclone.workflow_execution.replay.event_store]
// r[impl n8nclone.tui_ux.replay_debugger.redaction]
pub fn summarize_value(value: &Value) -> ReplayValueSummary {
    let redacted = !value.sensitive_fragments.is_empty();
    let summary = if redacted {
        REDACTED_SUMMARY.to_string()
    } else {
        truncate_summary(value_display_summary(value))
    };
    ReplayValueSummary {
        type_tag: value.type_tag().as_str().to_string(),
        hash: value.hash.to_hex(),
        summary,
        redacted,
    }
}

fn value_display_summary(value: &Value) -> String {
    match &value.data {
        ValueData::String(value) => value.clone(),
        ValueData::Bytes(bytes) => format!("{} bytes", bytes.len()),
        ValueData::Json(value) => match serde_json::to_string(value) {
            Ok(serialized) => serialized,
            Err(error) => format!("json serialization failed: {error}"),
        },
        ValueData::Image(image) => image.summary(),
        ValueData::ExitCode(code) => format!("exit code: {code}"),
        ValueData::Error { message, .. } => format!("error: {message}"),
    }
}

fn truncate_summary(value: String) -> String {
    if value.chars().count() <= SUMMARY_MAX_CHARS {
        return value;
    }
    value.chars().take(SUMMARY_MAX_CHARS).collect()
}

fn replay_event_record(
    run_id: RunId,
    sequence: u64,
    timestamp_epoch_ms: Option<i64>,
    node_id: Option<u64>,
    port_id: Option<u64>,
    kind: RunnerEventKind,
    summary: String,
    value: Option<ReplayValueSummary>,
    artifact: Option<ReplayArtifactReference>,
    redacted: bool,
) -> ReplayEventRecord {
    ReplayEventRecord {
        run_id,
        sequence,
        timestamp_epoch_ms,
        node_id,
        port_id,
        parent_run_id: None,
        kind,
        summary,
        redacted,
        value,
        artifact,
        contains_plaintext_secret_material: false,
    }
}

fn next_sequence(events: &[ReplayEventRecord]) -> u64 {
    events
        .last()
        .map(|event| event.sequence.saturating_add(EVENT_SEQUENCE_INCREMENT))
        .unwrap_or(FIRST_EVENT_SEQUENCE)
}

impl From<&ArtifactReference> for ReplayArtifactReference {
    fn from(reference: &ArtifactReference) -> Self {
        Self {
            content_hash: reference.content_hash.clone(),
            type_tag: reference.type_tag.clone(),
            media_type: reference.media_type.clone(),
            byte_length: reference.byte_length,
            backend_ref: reference.backend_ref.clone(),
            sensitive: reference.sensitive,
            summary: reference.summary.clone(),
        }
    }
}

// r[impl n8nclone.workflow_execution.replay.artifact_placeholders]
pub fn artifact_view(reference: ReplayArtifactReference, payload_available: bool) -> ReplayArtifactView {
    let payload_state = if payload_available {
        ReplayArtifactPayloadState::Available
    } else {
        ReplayArtifactPayloadState::Missing
    };
    let display_summary = if payload_available {
        reference.summary.clone()
    } else {
        format!("{}: {} ({})", MISSING_ARTIFACT_PAYLOAD_SUMMARY, reference.content_hash, reference.type_tag)
    };
    ReplayArtifactView {
        reference,
        payload_state,
        display_summary,
    }
}

// r[impl n8nclone.workflow_format.replay_bundles]
pub fn export_replay_bundle(
    run_id: RunId,
    workflow_name: impl Into<String>,
    workflow_snapshot: Option<Workflow>,
    events: Vec<ReplayEventRecord>,
) -> ReplayBundle {
    let workflow_hash = workflow_snapshot.as_ref().map(workflow_snapshot_hash);
    ReplayBundle {
        schema_version: REPLAY_BUNDLE_SCHEMA_VERSION,
        run_id,
        workflow_name: workflow_name.into(),
        workflow_hash,
        workflow_snapshot,
        redaction_policy: REPLAY_REDACTION_POLICY.to_string(),
        events,
    }
}

// r[impl n8nclone.workflow_format.replay_bundles]
pub fn replay_bundle_to_pretty_json(bundle: &ReplayBundle) -> Result<String, ReplayBundleError> {
    serde_json::to_string_pretty(bundle).map_err(|error| ReplayBundleError::Serialize(error.to_string()))
}

// r[impl n8nclone.workflow_format.replay_bundles]
// r[impl n8nclone.workflow_validation.replay_records]
pub fn import_replay_bundle_json(input: &str) -> Result<ReplayBundle, ReplayBundleError> {
    let bundle = serde_json::from_str::<ReplayBundle>(input)
        .map_err(|error| ReplayBundleError::MalformedJson(error.to_string()))?;
    let diagnostics = validate_replay_bundle(&bundle);
    if diagnostics.iter().any(|diagnostic| diagnostic.fatal) {
        return Err(ReplayBundleError::Validation(diagnostics));
    }
    Ok(bundle)
}

// r[impl n8nclone.workflow_validation.replay_records]
pub fn validate_replay_bundle(bundle: &ReplayBundle) -> Vec<ReplayDiagnostic> {
    let mut diagnostics = Vec::new();
    if bundle.schema_version != REPLAY_BUNDLE_SCHEMA_VERSION {
        diagnostics.push(replay_diagnostic(
            ReplayDiagnosticCode::MalformedBundle,
            None,
            None,
            "unsupported replay bundle schema version",
            true,
        ));
    }
    if bundle.redaction_policy != REPLAY_REDACTION_POLICY {
        diagnostics.push(replay_diagnostic(
            ReplayDiagnosticCode::MalformedBundle,
            None,
            None,
            "unsupported replay bundle redaction policy",
            true,
        ));
    }
    let known_nodes = bundle.workflow_snapshot.as_ref().map(workflow_node_ids);
    diagnostics.extend(validate_replay_events(&bundle.events, known_nodes.as_ref()));
    diagnostics
}

fn workflow_node_ids(workflow: &Workflow) -> BTreeSet<u64> {
    workflow.graph.nodes().map(|node| node.id.0).collect()
}

fn workflow_snapshot_hash(workflow: &Workflow) -> String {
    match serde_json::to_vec(workflow) {
        Ok(bytes) => blake3::hash(&bytes).to_hex().to_string(),
        Err(error) => blake3::hash(format!("workflow-hash-error:{error}").as_bytes()).to_hex().to_string(),
    }
}

fn replay_diagnostic(
    code: ReplayDiagnosticCode,
    sequence: Option<u64>,
    node_id: Option<u64>,
    message: impl Into<String>,
    fatal: bool,
) -> ReplayDiagnostic {
    ReplayDiagnostic {
        code,
        sequence,
        node_id,
        message: message.into(),
        fatal,
    }
}
