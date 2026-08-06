// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven
// validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(compound_condition, excessive_file_length, function_length, non_trait_imports)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::BLAKE3_ALGORITHM;
use super::EMPTY_COUNT;
use super::FIELD_BRANCH;
use super::FIELD_CONTEXT_ESTIMATE;
use super::FIELD_CUT_POINT;
use super::FIELD_POLICY;
use super::FIELD_RESOURCE_ID;
use super::FIELD_SUMMARIZER;
use super::FIELD_SUMMARY_TEXT;
use super::FIELD_TOOL_BINDING;
use super::HASH_PART_SEPARATOR;
use super::PI_COMPACTION_HASH_DOMAIN;
use super::REQUIRED_COUNT;
use super::REQUIRED_USIZE;
use super::data::PiBranchHandoffPlan;
use super::data::PiBranchHandoffRequest;
use super::data::PiBranchSummaryMode;
use super::data::PiCompactionDecision;
use super::data::PiCompactionDiagnostic;
use super::data::PiCompactionDiagnosticKind;
use super::data::PiCompactionFingerprint;
use super::data::PiCompactionPlan;
use super::data::PiCompactionPolicy;
use super::data::PiCompactionRedactionBehavior;
use super::data::PiCompactionReplayEntry;
use super::data::PiCompactionReplayEntryKind;
use super::data::PiCompactionSummarizerRequest;
use super::data::PiCompactionSummarizerState;
use super::data::PiCompactionSummaryEntry;
use super::data::PiCompactionTrigger;
use super::data::PiCompactionValidationResult;

const HASH_RESOURCE_PREFIX: &str = "hash:";
const MAX_SUMMARY_TEXT_CHARS: usize = 4096;
const RAW_CREDENTIAL_MARKER: &str = "API_KEY";
const RAW_HOST_PATH_MARKER: &str = "/home/";
const RAW_PROVIDER_MARKER: &str = "provider_payload";
const RAW_TOOL_MARKER: &str = "tool_output:";
const WINDOWS_PATH_MARKER: &str = ":\\";

/// Computes a domain-separated BLAKE3 fingerprint for Pi compaction data.
#[must_use]
pub fn blake3_pi_compaction_fingerprint(label: &str, bytes: &[u8]) -> PiCompactionFingerprint {
    let mut hasher = blake3::Hasher::new();
    hasher.update(PI_COMPACTION_HASH_DOMAIN.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(bytes);
    PiCompactionFingerprint::new(BLAKE3_ALGORITHM, hasher.finalize().to_hex().to_string())
}

/// Evaluates whether policy and estimate require compaction.
#[must_use]
pub fn evaluate_pi_compaction_trigger(
    policy: &PiCompactionPolicy,
    context_estimate_units: Option<u32>,
) -> PiCompactionTrigger {
    if !policy.enabled {
        return PiCompactionTrigger::Disabled;
    }
    if invalid_budget(policy) {
        return PiCompactionTrigger::InvalidPolicy;
    }
    let Some(estimate) = context_estimate_units else {
        return PiCompactionTrigger::MissingEstimate;
    };
    let usable_budget = policy.context_budget_units - policy.response_reserve_units;
    if estimate > usable_budget {
        PiCompactionTrigger::Required
    } else {
        PiCompactionTrigger::UnderBudget
    }
}

/// Prepares a deterministic compaction plan over already-loaded entries.
///
/// # Errors
///
/// This function returns diagnostics inside [`PiCompactionPlan`] instead of a
/// `Result`; invalid entries or policies produce an invalid trigger plan and no
/// host dispatch.
#[must_use]
pub fn prepare_pi_compaction(
    policy: &PiCompactionPolicy,
    entries: &[PiCompactionReplayEntry],
    context_estimate_units: Option<u32>,
) -> PiCompactionPlan {
    let mut diagnostics = validate_policy_and_entries(policy, entries);
    let trigger = evaluate_pi_compaction_trigger(policy, context_estimate_units);
    if trigger == PiCompactionTrigger::MissingEstimate {
        diagnostics.push(PiCompactionDiagnostic::new(
            PiCompactionDiagnosticKind::MissingEstimate,
            FIELD_CONTEXT_ESTIMATE,
            "host context estimate is required before compaction",
        ));
    }
    if trigger == PiCompactionTrigger::InvalidPolicy {
        diagnostics.push(PiCompactionDiagnostic::new(
            PiCompactionDiagnosticKind::InvalidBudget,
            FIELD_POLICY,
            "context budget must exceed response reserve and kept-recent budget must be positive",
        ));
    }
    if trigger != PiCompactionTrigger::Required || !diagnostics.is_empty() {
        return PiCompactionPlan {
            trigger,
            cut_index: None,
            summarized_entry_ids: Vec::new(),
            kept_entry_ids: entry_ids(entries),
            split_turn_prefix_entry_ids: Vec::new(),
            split_turn: false,
            diagnostics,
        };
    }

    let cut_index = select_valid_cut_index(entries, policy.kept_recent_units);
    let summarized_entry_ids = entry_ids(&entries[..cut_index]);
    let kept_entry_ids = entry_ids(&entries[cut_index..]);
    let split_turn_prefix_entry_ids = split_turn_prefix(entries, cut_index);
    let split_turn = !split_turn_prefix_entry_ids.is_empty();

    PiCompactionPlan {
        trigger,
        cut_index: Some(cut_index),
        summarized_entry_ids,
        kept_entry_ids,
        split_turn_prefix_entry_ids,
        split_turn,
        diagnostics,
    }
}

/// Evaluates whether a required compaction may dispatch to an injected summarizer.
#[must_use]
pub fn evaluate_required_pi_summarizer(
    policy: &PiCompactionPolicy,
    plan: &PiCompactionPlan,
    state: PiCompactionSummarizerState,
) -> PiCompactionDecision {
    if plan.trigger != PiCompactionTrigger::Required || plan.cut_index.is_none() {
        return PiCompactionDecision {
            dispatch_summarizer: false,
            request: None,
            diagnostics: plan.diagnostics.clone(),
        };
    }
    if state != PiCompactionSummarizerState::Ready {
        return PiCompactionDecision {
            dispatch_summarizer: false,
            request: None,
            diagnostics: vec![PiCompactionDiagnostic::new(
                PiCompactionDiagnosticKind::SummarizerUnavailable,
                FIELD_SUMMARIZER,
                summarizer_state_detail(state),
            )],
        };
    }
    PiCompactionDecision {
        dispatch_summarizer: true,
        request: Some(PiCompactionSummarizerRequest {
            summary_format: policy.summary_format,
            summarizer_selection: policy.summarizer_selection,
            summarized_entry_ids: plan.summarized_entry_ids.clone(),
            kept_entry_ids: plan.kept_entry_ids.clone(),
        }),
        diagnostics: Vec::new(),
    }
}

/// Prepares a branch handoff summary plan for host-owned navigation.
#[must_use]
pub fn prepare_pi_branch_handoff_summary(request: &PiBranchHandoffRequest) -> PiBranchHandoffPlan {
    if request.mode == PiBranchSummaryMode::NotRequested {
        return PiBranchHandoffPlan {
            summary_required: false,
            common_ancestor_id: None,
            abandoned_entry_ids: Vec::new(),
            target_entry_ids: Vec::new(),
            diagnostics: Vec::new(),
        };
    }

    let graph = branch_graph(&request.entries);
    let mut diagnostics = validate_branch_request(request, &graph);
    if !diagnostics.is_empty() {
        return PiBranchHandoffPlan {
            summary_required: false,
            common_ancestor_id: None,
            abandoned_entry_ids: Vec::new(),
            target_entry_ids: Vec::new(),
            diagnostics,
        };
    }

    let old_path = ancestry_path(&graph, &request.old_leaf_id);
    let target_path = ancestry_path(&graph, &request.target_leaf_id);
    let target_set: BTreeSet<String> = target_path.iter().cloned().collect();
    let common_ancestor_id = old_path.iter().find(|entry_id| target_set.contains(*entry_id)).cloned();
    let Some(common_ancestor_id) = common_ancestor_id else {
        diagnostics.push(PiCompactionDiagnostic::new(
            PiCompactionDiagnosticKind::InvalidBranch,
            FIELD_BRANCH,
            "old and target leaves do not share a common ancestor",
        ));
        return PiBranchHandoffPlan {
            summary_required: false,
            common_ancestor_id: None,
            abandoned_entry_ids: Vec::new(),
            target_entry_ids: Vec::new(),
            diagnostics,
        };
    };
    let abandoned_entry_ids = entries_before_ancestor(&old_path, &common_ancestor_id);
    let target_entry_ids = entries_before_ancestor(&target_path, &common_ancestor_id);
    PiBranchHandoffPlan {
        summary_required: !abandoned_entry_ids.is_empty(),
        common_ancestor_id: Some(common_ancestor_id),
        abandoned_entry_ids,
        target_entry_ids,
        diagnostics,
    }
}

/// Validates safe summary metadata before ledger append.
///
/// # Errors
///
/// Returns typed diagnostics when the summary is empty or overlong, counts are
/// zero, hashes are invalid, resource identifiers are unsafe, or raw prompt/tool,
/// provider, credential, or host path markers appear.
pub fn validate_pi_summary_entry(entry: &PiCompactionSummaryEntry) -> PiCompactionValidationResult {
    let mut diagnostics = Vec::new();
    let summary_chars = entry.summary_text.chars().count();
    if summary_chars == EMPTY_COUNT || summary_chars > MAX_SUMMARY_TEXT_CHARS {
        diagnostics.push(PiCompactionDiagnostic::new(
            PiCompactionDiagnosticKind::InvalidSummaryText,
            FIELD_SUMMARY_TEXT,
            "summary text must be non-empty and bounded",
        ));
    }
    if contains_unsafe_marker(&entry.summary_text) {
        diagnostics.push(PiCompactionDiagnostic::new(
            PiCompactionDiagnosticKind::UnsafeMetadata,
            FIELD_SUMMARY_TEXT,
            "summary text contains unsafe raw material marker",
        ));
    }
    if entry.source_entry_count < REQUIRED_COUNT || entry.source_turn_count < REQUIRED_COUNT {
        diagnostics.push(PiCompactionDiagnostic::new(
            PiCompactionDiagnosticKind::UnsafeMetadata,
            FIELD_SUMMARY_TEXT,
            "summary source counts must be positive",
        ));
    }
    require_hash(&entry.source_hash, FIELD_SUMMARY_TEXT, &mut diagnostics);
    require_hash(&entry.summary_hash, FIELD_SUMMARY_TEXT, &mut diagnostics);
    validate_resource_ids(entry, &mut diagnostics);
    to_validation_result(diagnostics)
}

fn ancestry_path(graph: &BTreeMap<String, Vec<String>>, leaf_id: &str) -> Vec<String> {
    let mut path = Vec::new();
    let mut current = Some(leaf_id.to_string());
    while let Some(entry_id) = current {
        path.push(entry_id.clone());
        current = graph.get(&entry_id).and_then(|parents| parents.first()).cloned();
    }
    path
}

fn blank(text: &str) -> bool {
    text.trim().is_empty()
}

fn branch_graph(entries: &[PiCompactionReplayEntry]) -> BTreeMap<String, Vec<String>> {
    let mut graph = BTreeMap::new();
    for entry in entries {
        graph.insert(entry.id.clone(), entry.parent_ids.clone());
    }
    graph
}

fn contains_unsafe_marker(text: &str) -> bool {
    text.contains(RAW_TOOL_MARKER)
        || text.contains(RAW_PROVIDER_MARKER)
        || text.contains(RAW_CREDENTIAL_MARKER)
        || text.contains(RAW_HOST_PATH_MARKER)
        || text.contains(WINDOWS_PATH_MARKER)
}

fn entries_before_ancestor(path: &[String], ancestor: &str) -> Vec<String> {
    let mut selected = Vec::new();
    for entry_id in path {
        if entry_id == ancestor {
            break;
        }
        selected.push(entry_id.clone());
    }
    selected.reverse();
    selected
}

fn entry_ids(entries: &[PiCompactionReplayEntry]) -> Vec<String> {
    entries.iter().map(|entry| entry.id.clone()).collect()
}

fn invalid_budget(policy: &PiCompactionPolicy) -> bool {
    policy.context_budget_units <= policy.response_reserve_units || policy.kept_recent_units < REQUIRED_COUNT
}

fn require_hash(fingerprint: &PiCompactionFingerprint, field: &str, diagnostics: &mut Vec<PiCompactionDiagnostic>) {
    if fingerprint.algorithm != BLAKE3_ALGORITHM || blank(&fingerprint.value_hex) {
        diagnostics.push(PiCompactionDiagnostic::new(
            PiCompactionDiagnosticKind::InvalidHash,
            field,
            "compaction hashes must use non-empty BLAKE3 fingerprints",
        ));
    }
}

fn select_valid_cut_index(entries: &[PiCompactionReplayEntry], kept_recent_units: u32) -> usize {
    let mut cut_index = suffix_cut_index(entries, kept_recent_units);
    loop {
        let adjusted = adjust_cut_for_tool_bindings(entries, cut_index);
        if adjusted == cut_index {
            return cut_index;
        }
        cut_index = adjusted;
    }
}

fn suffix_cut_index(entries: &[PiCompactionReplayEntry], kept_recent_units: u32) -> usize {
    let mut suffix_units = 0_u32;
    let mut cut_index = entries.len();
    for (index, entry) in entries.iter().enumerate().rev() {
        let next_units = suffix_units.saturating_add(entry.estimate_units);
        if next_units > kept_recent_units && cut_index != entries.len() {
            return cut_index;
        }
        suffix_units = next_units;
        cut_index = index;
    }
    cut_index
}

fn adjust_cut_for_tool_bindings(entries: &[PiCompactionReplayEntry], cut_index: usize) -> usize {
    let mut adjusted = cut_index;
    for kept_index in cut_index..entries.len() {
        let entry = &entries[kept_index];
        if entry.kind != PiCompactionReplayEntryKind::ToolResult {
            continue;
        }
        let Some(binding) = &entry.tool_binding else {
            adjusted = adjusted.min(kept_index);
            continue;
        };
        if let Some(call_index) = tool_call_index(entries, &binding.call_id) {
            adjusted = adjusted.min(call_index);
        } else {
            adjusted = adjusted.min(kept_index);
        }
    }
    adjusted
}

fn split_turn_prefix(entries: &[PiCompactionReplayEntry], cut_index: usize) -> Vec<String> {
    if cut_index == EMPTY_COUNT || cut_index >= entries.len() {
        return Vec::new();
    }
    let kept_turn_id = &entries[cut_index].turn_id;
    let mut prefix = Vec::new();
    let mut index = cut_index;
    while index > EMPTY_COUNT {
        let previous_index = index - REQUIRED_USIZE;
        let previous = &entries[previous_index];
        if &previous.turn_id != kept_turn_id {
            break;
        }
        prefix.push(previous.id.clone());
        index = previous_index;
    }
    prefix.reverse();
    prefix
}

fn summarizer_state_detail(state: PiCompactionSummarizerState) -> &'static str {
    match state {
        PiCompactionSummarizerState::Ready => "summarizer is ready",
        PiCompactionSummarizerState::Missing => "required summarizer service is missing",
        PiCompactionSummarizerState::Denied => "required summarizer service denied the request",
        PiCompactionSummarizerState::Cancelled => "required summarizer service was cancelled",
        PiCompactionSummarizerState::MalformedOutput => "required summarizer service returned malformed output",
    }
}

fn tool_call_index(entries: &[PiCompactionReplayEntry], call_id: &str) -> Option<usize> {
    entries.iter().position(|entry| {
        entry.kind == PiCompactionReplayEntryKind::ToolCall
            && entry.tool_binding.as_ref().is_some_and(|binding| binding.call_id == call_id)
    })
}

fn to_validation_result(diagnostics: Vec<PiCompactionDiagnostic>) -> PiCompactionValidationResult {
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn validate_branch_request(
    request: &PiBranchHandoffRequest,
    graph: &BTreeMap<String, Vec<String>>,
) -> Vec<PiCompactionDiagnostic> {
    let mut diagnostics = Vec::new();
    if !graph.contains_key(&request.old_leaf_id) || !graph.contains_key(&request.target_leaf_id) {
        diagnostics.push(PiCompactionDiagnostic::new(
            PiCompactionDiagnosticKind::InvalidBranch,
            FIELD_BRANCH,
            "old and target leaves must both exist in the branch graph",
        ));
    }
    for (entry_id, parents) in graph {
        for parent in parents {
            if !graph.contains_key(parent) {
                diagnostics.push(PiCompactionDiagnostic::new(
                    PiCompactionDiagnosticKind::InvalidBranch,
                    FIELD_BRANCH,
                    format!("branch entry {entry_id} references a missing parent"),
                ));
            }
        }
    }
    diagnostics
}

fn validate_policy_and_entries(
    policy: &PiCompactionPolicy,
    entries: &[PiCompactionReplayEntry],
) -> Vec<PiCompactionDiagnostic> {
    let mut diagnostics = Vec::new();
    if invalid_budget(policy) {
        diagnostics.push(PiCompactionDiagnostic::new(
            PiCompactionDiagnosticKind::InvalidBudget,
            FIELD_POLICY,
            "context budget must exceed response reserve and kept-recent budget must be positive",
        ));
    }
    if entries.len() < REQUIRED_USIZE {
        diagnostics.push(PiCompactionDiagnostic::new(
            PiCompactionDiagnosticKind::MissingField,
            FIELD_CUT_POINT,
            "compaction requires at least one replay entry",
        ));
    }
    let mut ids = BTreeSet::new();
    for entry in entries {
        if blank(&entry.id) || blank(&entry.turn_id) || !ids.insert(entry.id.clone()) {
            diagnostics.push(PiCompactionDiagnostic::new(
                PiCompactionDiagnosticKind::MissingField,
                FIELD_CUT_POINT,
                "entry IDs and turn IDs must be present and unique",
            ));
        }
        require_hash(&entry.content_hash, FIELD_CUT_POINT, &mut diagnostics);
        match entry.kind {
            PiCompactionReplayEntryKind::ToolCall | PiCompactionReplayEntryKind::ToolResult => {
                if entry.tool_binding.is_none() {
                    diagnostics.push(PiCompactionDiagnostic::new(
                        PiCompactionDiagnosticKind::OrphanToolResult,
                        FIELD_TOOL_BINDING,
                        "tool calls and tool results require an explicit binding",
                    ));
                }
            }
            PiCompactionReplayEntryKind::User
            | PiCompactionReplayEntryKind::Assistant
            | PiCompactionReplayEntryKind::Summary => {}
        }
    }
    diagnostics
}

fn validate_resource_id(
    id: &str,
    redaction_behavior: PiCompactionRedactionBehavior,
    diagnostics: &mut Vec<PiCompactionDiagnostic>,
) {
    let hash_only_mismatch =
        redaction_behavior == PiCompactionRedactionBehavior::HashOnly && !id.starts_with(HASH_RESOURCE_PREFIX);
    if blank(id) || contains_unsafe_marker(id) || hash_only_mismatch {
        diagnostics.push(PiCompactionDiagnostic::new(
            PiCompactionDiagnosticKind::UnsafeMetadata,
            FIELD_RESOURCE_ID,
            "resource metadata must use redacted labels or hashes only",
        ));
    }
}

fn validate_resource_ids(entry: &PiCompactionSummaryEntry, diagnostics: &mut Vec<PiCompactionDiagnostic>) {
    for id in entry
        .redacted_resource_ids
        .iter()
        .chain(entry.cumulative_read_resource_ids.iter())
        .chain(entry.cumulative_modified_resource_ids.iter())
    {
        validate_resource_id(id, entry.redaction_behavior, diagnostics);
    }
}
