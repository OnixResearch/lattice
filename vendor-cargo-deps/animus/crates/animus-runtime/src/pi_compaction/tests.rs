// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports
)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]

use super::*;

const ASSISTANT_ENTRY: &str = "assistant.entry";
const CALL_ENTRY: &str = "tool.call";
const COMMON_ENTRY: &str = "root.entry";
const DEFAULT_BUDGET_UNITS: u32 = 20;
const DEFAULT_CONTEXT_ESTIMATE: u32 = 24;
const DEFAULT_KEPT_UNITS: u32 = 6;
const DEFAULT_RESERVE_UNITS: u32 = 4;
const ENTRY_UNITS: u32 = 2;
const HASH_LABEL: &str = "hash";
const LARGE_ENTRY_UNITS: u32 = 10;
const MODIFIED_RESOURCE: &str = "modified:src-lib";
const OLD_BRANCH_ONE: &str = "old.one";
const OLD_BRANCH_TWO: &str = "old.two";
const READ_RESOURCE: &str = "read:readme";
const REDACTED_RESOURCE: &str = "resource:workspace";
const RESULT_ENTRY: &str = "tool.result";
const SUMMARY_TEXT: &str = "Summarized prior branch context with safe metadata.";
const TARGET_BRANCH_ONE: &str = "target.one";
const TOOL_CALL_ID: &str = "call.one";
const TURN_ONE: &str = "turn.one";
const TURN_TWO: &str = "turn.two";
const USER_ENTRY: &str = "user.entry";
const ZERO_UNITS: u32 = 0;

#[test]
fn policy_trigger_respects_enabled_budget_and_missing_estimates() {
    // r[agent_harness_runtime.pi_compaction.policy.trigger]
    let policy = compaction_policy(true, DEFAULT_KEPT_UNITS);
    assert_eq!(
        evaluate_pi_compaction_trigger(&policy, Some(DEFAULT_CONTEXT_ESTIMATE)),
        PiCompactionTrigger::Required
    );
    assert_eq!(
        evaluate_pi_compaction_trigger(&policy, Some(DEFAULT_BUDGET_UNITS - DEFAULT_RESERVE_UNITS)),
        PiCompactionTrigger::UnderBudget
    );
    assert_eq!(
        evaluate_pi_compaction_trigger(&policy, None),
        PiCompactionTrigger::MissingEstimate
    );

    let disabled = compaction_policy(false, DEFAULT_KEPT_UNITS);
    assert_eq!(
        evaluate_pi_compaction_trigger(&disabled, Some(DEFAULT_CONTEXT_ESTIMATE)),
        PiCompactionTrigger::Disabled
    );

    let invalid = compaction_policy(true, ZERO_UNITS);
    assert_eq!(
        evaluate_pi_compaction_trigger(&invalid, Some(DEFAULT_CONTEXT_ESTIMATE)),
        PiCompactionTrigger::InvalidPolicy
    );
}

#[test]
fn cut_point_moves_before_tool_call_to_avoid_orphaned_results() {
    // r[agent_harness_runtime.pi_compaction.cut_points.reject_tool_result]
    let entries = vec![
        entry(
            USER_ENTRY,
            TURN_ONE,
            PiCompactionReplayEntryKind::User,
            LARGE_ENTRY_UNITS,
        ),
        tool_entry(CALL_ENTRY, TURN_TWO, PiCompactionReplayEntryKind::ToolCall),
        tool_entry(
            RESULT_ENTRY,
            TURN_TWO,
            PiCompactionReplayEntryKind::ToolResult,
        ),
        entry(
            ASSISTANT_ENTRY,
            TURN_TWO,
            PiCompactionReplayEntryKind::Assistant,
            ENTRY_UNITS,
        ),
    ];
    let compaction_policy = compaction_policy(true, DEFAULT_KEPT_UNITS);
    let plan = prepare_pi_compaction(&compaction_policy, &entries, Some(DEFAULT_CONTEXT_ESTIMATE));

    assert_eq!(plan.trigger, PiCompactionTrigger::Required);
    assert_eq!(plan.cut_index, Some(1));
    assert_eq!(
        plan.kept_entry_ids,
        vec![
            CALL_ENTRY.to_string(),
            RESULT_ENTRY.to_string(),
            ASSISTANT_ENTRY.to_string(),
        ]
    );
    assert!(!plan.kept_entry_ids.contains(&USER_ENTRY.to_string()));
}

#[test]
fn split_turn_plan_records_prefix_and_kept_suffix_order() {
    // r[agent_harness_runtime.pi_compaction.split_turn.prefix]
    let entries = vec![
        entry(
            USER_ENTRY,
            TURN_ONE,
            PiCompactionReplayEntryKind::User,
            ENTRY_UNITS,
        ),
        entry(
            ASSISTANT_ENTRY,
            TURN_ONE,
            PiCompactionReplayEntryKind::Assistant,
            ENTRY_UNITS,
        ),
        entry(
            "summary.entry",
            TURN_ONE,
            PiCompactionReplayEntryKind::Summary,
            ENTRY_UNITS,
        ),
        entry(
            "user.two",
            TURN_TWO,
            PiCompactionReplayEntryKind::User,
            LARGE_ENTRY_UNITS,
        ),
        entry(
            "assistant.two",
            TURN_TWO,
            PiCompactionReplayEntryKind::Assistant,
            ENTRY_UNITS,
        ),
        entry(
            "assistant.three",
            TURN_TWO,
            PiCompactionReplayEntryKind::Assistant,
            ENTRY_UNITS,
        ),
    ];
    let compaction_policy = compaction_policy(true, DEFAULT_KEPT_UNITS);
    let plan = prepare_pi_compaction(&compaction_policy, &entries, Some(DEFAULT_CONTEXT_ESTIMATE));

    assert!(plan.split_turn);
    assert_eq!(
        plan.split_turn_prefix_entry_ids,
        vec!["user.two".to_string()]
    );
    assert_eq!(
        plan.kept_entry_ids,
        vec!["assistant.two".to_string(), "assistant.three".to_string()]
    );
}

#[test]
fn branch_handoff_selects_abandoned_span_and_honors_no_summary_request() {
    // r[agent_harness_runtime.pi_compaction.branch_summary.navigation]
    let entries = branch_entries();
    let request = PiBranchHandoffRequest {
        old_leaf_id: OLD_BRANCH_TWO.to_string(),
        target_leaf_id: TARGET_BRANCH_ONE.to_string(),
        mode: PiBranchSummaryMode::Requested,
        entries: entries.clone(),
    };
    let plan = prepare_pi_branch_handoff_summary(&request);

    assert!(plan.summary_required);
    assert_eq!(plan.common_ancestor_id, Some(COMMON_ENTRY.to_string()));
    assert_eq!(
        plan.abandoned_entry_ids,
        vec![OLD_BRANCH_ONE.to_string(), OLD_BRANCH_TWO.to_string()]
    );
    assert_eq!(plan.target_entry_ids, vec![TARGET_BRANCH_ONE.to_string()]);

    let no_summary = PiBranchHandoffRequest {
        mode: PiBranchSummaryMode::NotRequested,
        ..request
    };
    let plan = prepare_pi_branch_handoff_summary(&no_summary);
    assert!(!plan.summary_required);
    assert!(plan.abandoned_entry_ids.is_empty());
}

#[test]
fn missing_summarizer_halts_required_compaction_without_fallback() {
    // r[agent_harness_runtime.pi_compaction.injected_summarizer.missing]
    let entries = vec![
        entry(
            USER_ENTRY,
            TURN_ONE,
            PiCompactionReplayEntryKind::User,
            LARGE_ENTRY_UNITS,
        ),
        entry(
            ASSISTANT_ENTRY,
            TURN_TWO,
            PiCompactionReplayEntryKind::Assistant,
            ENTRY_UNITS,
        ),
    ];
    let compaction_policy = compaction_policy(true, DEFAULT_KEPT_UNITS);
    let plan = prepare_pi_compaction(&compaction_policy, &entries, Some(DEFAULT_CONTEXT_ESTIMATE));
    let missing = evaluate_required_pi_summarizer(
        &compaction_policy,
        &plan,
        PiCompactionSummarizerState::Missing,
    );

    assert!(!missing.dispatch_summarizer);
    assert!(missing.request.is_none());
    assert!(missing.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == PiCompactionDiagnosticKind::SummarizerUnavailable
    }));

    let ready = evaluate_required_pi_summarizer(
        &compaction_policy,
        &plan,
        PiCompactionSummarizerState::Ready,
    );
    assert!(ready.dispatch_summarizer);
    assert!(ready.request.is_some());
}

#[test]
fn summary_metadata_accepts_redacted_data_and_rejects_raw_material() {
    // r[agent_harness_runtime.pi_compaction.safe_metadata.reject_raw]
    let valid = summary_entry(SUMMARY_TEXT, vec![REDACTED_RESOURCE.to_string()]);
    assert!(validate_pi_summary_entry(&valid).is_ok());

    let invalid = PiCompactionSummaryEntry {
        summary_text: "tool_output: raw provider_payload API_KEY /home/user/file".to_string(),
        redacted_resource_ids: vec!["/home/user/secret".to_string()],
        source_hash: PiCompactionFingerprint::new("SHA256", "abc"),
        ..summary_entry(SUMMARY_TEXT, vec![REDACTED_RESOURCE.to_string()])
    };
    let diagnostics = validate_pi_summary_entry(&invalid).expect_err("unsafe metadata");

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == PiCompactionDiagnosticKind::UnsafeMetadata })
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == PiCompactionDiagnosticKind::InvalidHash })
    );
}

fn branch_entries() -> Vec<PiCompactionReplayEntry> {
    vec![
        entry_with_parents(
            COMMON_ENTRY,
            TURN_ONE,
            PiCompactionReplayEntryKind::User,
            Vec::new(),
        ),
        entry_with_parents(
            OLD_BRANCH_ONE,
            TURN_ONE,
            PiCompactionReplayEntryKind::Assistant,
            vec![COMMON_ENTRY.to_string()],
        ),
        entry_with_parents(
            OLD_BRANCH_TWO,
            TURN_TWO,
            PiCompactionReplayEntryKind::Assistant,
            vec![OLD_BRANCH_ONE.to_string()],
        ),
        entry_with_parents(
            TARGET_BRANCH_ONE,
            TURN_TWO,
            PiCompactionReplayEntryKind::Assistant,
            vec![COMMON_ENTRY.to_string()],
        ),
    ]
}

fn entry(
    id: &str,
    turn_id: &str,
    kind: PiCompactionReplayEntryKind,
    estimate_units: u32,
) -> PiCompactionReplayEntry {
    entry_with_parents(id, turn_id, kind, Vec::new()).with_estimate(estimate_units)
}

fn entry_with_parents(
    id: &str,
    turn_id: &str,
    kind: PiCompactionReplayEntryKind,
    parent_ids: Vec<String>,
) -> PiCompactionReplayEntry {
    PiCompactionReplayEntry {
        id: id.to_string(),
        turn_id: turn_id.to_string(),
        parent_ids,
        kind,
        estimate_units: ENTRY_UNITS,
        tool_binding: None,
        content_hash: fp(HASH_LABEL),
    }
}

fn fp(label: &str) -> PiCompactionFingerprint {
    blake3_pi_compaction_fingerprint(label, label.as_bytes())
}

fn compaction_policy(enabled: bool, kept_recent_units: u32) -> PiCompactionPolicy {
    PiCompactionPolicy {
        enabled,
        context_budget_units: DEFAULT_BUDGET_UNITS,
        response_reserve_units: DEFAULT_RESERVE_UNITS,
        kept_recent_units,
        summary_format: PiCompactionSummaryFormat::Markdown,
        summarizer_selection: PiCompactionSummarizerSelection::HostInjectedModel,
        redaction_behavior: PiCompactionRedactionBehavior::RequireRedactedResourceIds,
    }
}

fn summary_entry(
    summary_text: &str,
    redacted_resource_ids: Vec<String>,
) -> PiCompactionSummaryEntry {
    PiCompactionSummaryEntry {
        summary_text: summary_text.to_string(),
        source_entry_count: DEFAULT_RESERVE_UNITS,
        source_turn_count: ENTRY_UNITS,
        redacted_resource_ids,
        cumulative_read_resource_ids: vec![READ_RESOURCE.to_string()],
        cumulative_modified_resource_ids: vec![MODIFIED_RESOURCE.to_string()],
        source_hash: fp("source"),
        summary_hash: fp("summary"),
        split_turn: false,
        redaction_behavior: PiCompactionRedactionBehavior::RequireRedactedResourceIds,
    }
}

fn tool_entry(
    id: &str,
    turn_id: &str,
    kind: PiCompactionReplayEntryKind,
) -> PiCompactionReplayEntry {
    PiCompactionReplayEntry {
        tool_binding: Some(PiCompactionToolBinding {
            call_id: TOOL_CALL_ID.to_string(),
        }),
        ..entry(id, turn_id, kind, ENTRY_UNITS)
    }
}

trait WithEstimate {
    fn with_estimate(self, estimate_units: u32) -> Self;
}

impl WithEstimate for PiCompactionReplayEntry {
    fn with_estimate(mut self, estimate_units: u32) -> Self {
        self.estimate_units = estimate_units;
        self
    }
}
