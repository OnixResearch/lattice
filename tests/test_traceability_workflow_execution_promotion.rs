#[path = "traceability_promotion/core.rs"]
mod core;
#[path = "traceability_workflow_execution_ids.rs"]
mod workflow_execution_ids;

use core::CoverageReceipt;
use std::collections::BTreeSet;

use workflow_execution_ids::IDS;

const BASELINE: &str = include_str!("fixtures/traceability-workflow-execution-completion-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-workflow-execution-promotion-current.json");
const POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const AGENT: &str = include_str!("test_agent_nodes.rs");
const OPENAI: &str = include_str!("openai/execution.rs");
const RUNNER: &str = include_str!("test_runner_core.rs");
const EXPRESSION: &str = include_str!("test_expression_mapping.rs");
const FILES: &str = include_str!("test_file_nodes.rs");
const HEADLESS: &str = include_str!("test_headless.rs");
const HTTP: &str = include_str!("test_http_node.rs");
const IMAGES: &str = include_str!("test_image_nodes.rs");
const VALUES: &str = include_str!("test_value.rs");
const LLM_GRAPH: &str = include_str!("test_llm_operating_graph.rs");
const EXECUTOR: &str = include_str!("test_executor.rs");
const RATS: &str = include_str!("test_rats_source_closure.rs");
const REPLAY: &str = include_str!("test_replay.rs");
const SUBWORKFLOW: &str = include_str!("test_subworkflow_execution.rs");
const BASELINE_COUNT: usize = 181;
const CURRENT_COUNT: usize = 98;
const PROMOTED_COUNT: usize = 83;
const UNRELATED: &str = concat!("n8n", "clone.tui_ux.add_node.cancel");
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

type Binding = (&'static str, &'static str);
type BindingGroup = (&'static str, &'static str, &'static [Binding]);

const AGENT_BINDINGS: &[Binding] = &[
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.cache_key.image_edit"),
        "agent_cache_key_changes_for_prompt_target_and_image_edits",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.cache_key.prompt_edit"),
        "agent_cache_key_changes_for_prompt_target_and_image_edits",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.cache_key.target_edit"),
        "agent_cache_key_changes_for_prompt_target_and_image_edits",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.clankers_adapter.image_context"),
        "clankers_agent_node_executes_image_request_through_adapter",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.clankers_adapter.session_failure"),
        "session_failure_marks_node_failed",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.command_runtimes"),
        "default_agent_runtime_config_contains_command_presets",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.command_runtimes.compatibility"),
        "default_agent_runtime_config_contains_command_presets",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.config.clankers"),
        "clankers_agent_node_executes_image_request_through_adapter",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.config.invalid"),
        "empty_agent_prompt_is_validation_diagnostic",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.config.pi"),
        "add_agent_node_records_config_and_ports",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.errors.adapter"),
        "adapter_failure_marks_node_failed",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.errors.credential"),
        "credential_failure_skips_agent_adapter_call",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.errors.decode"),
        "decode_agent_command_output_rejects_invalid_json",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.execute.image"),
        "clankers_agent_node_executes_image_request_through_adapter",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.execute.text_json"),
        "pi_agent_node_executes_text_request_through_adapter",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.execute.type_mismatch"),
        "output_mode_mismatch_marks_node_failed",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.output_file_handoff"),
        "production_image_handoff_loads_typed_image",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.output_file_handoff.capability_read.scenario.rejected"),
        "handoff_rejects_output_path_outside_working_directory",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.output_file_handoff.capability_read.scenario.success"),
        "production_image_handoff_loads_typed_image",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.pi_adapter.image_prompt"),
        "production_command_plan_uses_runtime_args_and_prompt_inputs",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.pi_adapter.preflight_reject"),
        "preflight_rejection_marks_node_failed",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.production_adapters"),
        "production_pi_text_adapter_executes_command",
    ),
];

const OPENAI_BINDINGS: &[(&str, &str)] = &[
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.openai_compatible"),
        "production_adapter_decodes_text_and_json_outputs",
    ),
    (
        concat!("n8n", "clone.workflow_execution.agent_nodes.openai_compatible.scenario.real-graph"),
        "production_transport_calls_local_compatible_endpoint",
    ),
];

const RUNNER_BINDINGS: &[(&str, &str)] = &[
    (
        concat!("n8n", "clone.workflow_execution.artifact_references"),
        "artifact_reference_carries_hash_type_length_and_backend",
    ),
    (
        concat!("n8n", "clone.workflow_execution.durable_runner"),
        "durable_runner_decision_skips_completed_and_blocks_failed_dependencies",
    ),
    (
        concat!("n8n", "clone.workflow_execution.generic_pipeline_runner"),
        "runner_plan_preserves_typed_nodes_edges_and_runtime_bindings",
    ),
    (concat!("n8n", "clone.workflow_execution.runner_events"), "runner_events_assign_ordered_sequences"),
    (
        concat!("n8n", "clone.workflow_execution.runtime_registry"),
        "builtin_runtime_registry_accepts_default_shell_binding",
    ),
    (
        concat!("n8n", "clone.workflow_execution.seaglass_graph_core.scenario.compatibility"),
        "shared_graph_core_reports_critical_path_and_bottlenecks",
    ),
    (
        concat!("n8n", "clone.workflow_execution.seaglass_graph_core.scenario.provenance"),
        "runner_plan_preserves_typed_nodes_edges_and_runtime_bindings",
    ),
    (
        concat!("n8n", "clone.workflow_execution.trait_boundaries"),
        "node_runtime_trait_exposes_binding_and_node_scoped_failure",
    ),
];

const EXPRESSION_BINDINGS: &[(&str, &str)] = &[(
    concat!("n8n", "clone.workflow_execution.expression_mapping.config"),
    "shell_command_resolves_upstream_stdout_expression",
)];

const FILE_BINDINGS: &[(&str, &str)] = &[
    (
        concat!("n8n", "clone.workflow_execution.file_nodes.config.read"),
        "add_file_node_records_config_and_ports",
    ),
    (
        concat!("n8n", "clone.workflow_execution.file_nodes.config.write"),
        "write_file_node_writes_content_and_reports_bytes",
    ),
    (
        concat!("n8n", "clone.workflow_execution.file_nodes.interface.memory"),
        "read_file_node_reads_content_through_interface",
    ),
    (
        concat!("n8n", "clone.workflow_execution.file_nodes.local_capabilities.scenario.nested-io"),
        "local_filesystem_writes_and_appends_within_allowed_root",
    ),
    (
        concat!("n8n", "clone.workflow_execution.file_nodes.read.content"),
        "read_file_node_reads_content_through_interface",
    ),
    (concat!("n8n", "clone.workflow_execution.file_nodes.read.missing"), "missing_file_marks_node_failed"),
    (
        concat!("n8n", "clone.workflow_execution.file_nodes.safety.outside_root"),
        "path_outside_allowed_roots_is_validation_diagnostic",
    ),
    (
        concat!("n8n", "clone.workflow_execution.file_nodes.safety.overwrite"),
        "write_without_overwrite_fails_without_modifying_existing_file",
    ),
    (
        concat!("n8n", "clone.workflow_execution.file_nodes.write_append.append"),
        "append_file_node_appends_input_content",
    ),
    (
        concat!("n8n", "clone.workflow_execution.file_nodes.write_append.write"),
        "write_file_node_writes_content_and_reports_bytes",
    ),
];

const HEADLESS_BINDINGS: &[(&str, &str)] = &[
    (
        concat!("n8n", "clone.workflow_execution.headless.errors.load_failure"),
        "headless_run_missing_workflow_fails_without_history_record",
    ),
    (
        concat!("n8n", "clone.workflow_execution.headless.exit_status.failure"),
        "headless_run_failed_node_returns_failure_summary",
    ),
    (
        concat!("n8n", "clone.workflow_execution.headless.exit_status.success"),
        "headless_run_saved_workflow_succeeds_and_persists_history",
    ),
    (
        concat!("n8n", "clone.workflow_execution.headless.history.persist"),
        "headless_run_saved_workflow_succeeds_and_persists_history",
    ),
    (
        concat!("n8n", "clone.workflow_execution.headless.run.missing"),
        "headless_run_missing_workflow_fails_without_history_record",
    ),
    (
        concat!("n8n", "clone.workflow_execution.headless.run.saved"),
        "headless_run_saved_workflow_succeeds_and_persists_history",
    ),
    (
        concat!("n8n", "clone.workflow_execution.headless.summary.json"),
        "cli_run_saved_workflow_emits_json_summary",
    ),
    (
        concat!("n8n", "clone.workflow_execution.headless.summary.text"),
        "headless_run_saved_workflow_succeeds_and_persists_history",
    ),
];

const HTTP_BINDINGS: &[(&str, &str)] = &[
    (
        concat!("n8n", "clone.workflow_execution.http_node.cache_key.url_edit"),
        "http_cache_key_changes_when_request_config_changes",
    ),
    (
        concat!("n8n", "clone.workflow_execution.http_node.config.create"),
        "add_http_node_records_config_and_ports",
    ),
    (
        concat!("n8n", "clone.workflow_execution.http_node.config.invalid"),
        "invalid_http_method_and_url_are_validation_diagnostics",
    ),
    (
        concat!("n8n", "clone.workflow_execution.http_node.errors.timeout"),
        "http_timeout_marks_node_failed",
    ),
    (
        concat!("n8n", "clone.workflow_execution.http_node.execute.failure"),
        "http_transport_failure_marks_node_failed",
    ),
    (
        concat!("n8n", "clone.workflow_execution.http_node.execute.success"),
        "http_node_executes_through_client_and_maps_outputs",
    ),
    (
        concat!("n8n", "clone.workflow_execution.http_node.outputs.body"),
        "http_node_executes_through_client_and_maps_outputs",
    ),
];

const IMAGE_BINDINGS: &[(&str, &str)] = &[
    (
        concat!("n8n", "clone.workflow_execution.image_input_nodes.capability_read.scenario.success"),
        "local_image_io_loads_png_and_jpeg_files",
    ),
    (
        concat!("n8n", "clone.workflow_execution.image_input_nodes.load.local"),
        "default_executor_loads_local_image_file",
    ),
    (
        concat!("n8n", "clone.workflow_execution.image_input_nodes.load.missing"),
        "missing_image_marks_node_failed",
    ),
    (
        concat!("n8n", "clone.workflow_execution.image_input_nodes.load.oversized"),
        "oversized_image_marks_node_failed",
    ),
    (
        concat!("n8n", "clone.workflow_execution.image_input_nodes.load.unsupported"),
        "unsupported_image_format_marks_node_failed",
    ),
    (
        concat!("n8n", "clone.workflow_execution.image_input_nodes.production_io"),
        "local_image_io_loads_png_and_jpeg_files",
    ),
];

const VALUE_BINDINGS: &[(&str, &str)] = &[
    (concat!("n8n", "clone.workflow_execution.image_values.model.incompatible"), "type_tag_mismatches"),
    (
        concat!("n8n", "clone.workflow_execution.image_values.model.route"),
        "value_from_image_is_typed_and_display_safe",
    ),
    (
        concat!("n8n", "clone.workflow_execution.image_values.model.store"),
        "image_hash_includes_bytes_and_media_type",
    ),
];

const LLM_BINDINGS: &[(&str, &str)] = &[(
    concat!("n8n", "clone.workflow_execution.llm_operating_graph.scenario.feedback-boundary"),
    "failed_role_blocks_aggregation_and_effect_nodes",
)];

const EXECUTOR_BINDINGS: &[(&str, &str)] = &[
    (
        concat!("n8n", "clone.workflow_execution.pipeline_stdin.cache_real_ports"),
        "run_uses_cache_when_hash_matches",
    ),
    (
        concat!("n8n", "clone.workflow_execution.pipeline_stdin.cache_real_ports.routable"),
        "regression_connected_stdout_reaches_downstream_stdin",
    ),
    (
        concat!("n8n", "clone.workflow_execution.pipeline_stdin.real_port_outputs"),
        "execute_echo_produces_stdout",
    ),
    (
        concat!("n8n", "clone.workflow_execution.pipeline_stdin.real_port_outputs.completed"),
        "execute_echo_produces_stdout",
    ),
    (
        concat!("n8n", "clone.workflow_execution.pipeline_stdin.stdout_to_stdin"),
        "regression_connected_stdout_reaches_downstream_stdin",
    ),
    (
        concat!("n8n", "clone.workflow_execution.pipeline_stdin.stdout_to_stdin.connected"),
        "regression_connected_stdout_reaches_downstream_stdin",
    ),
];

const RATS_BINDINGS: &[(&str, &str)] = &[
    (
        concat!("n8n", "clone.workflow_execution.rats_dependency"),
        "pinned_rats_snapshot_is_source_complete_and_has_a_focused_consumer_compile_rail",
    ),
    (
        concat!("n8n", "clone.workflow_execution.rats_dependency.scenario.source-completeness"),
        "pinned_rats_snapshot_is_source_complete_and_has_a_focused_consumer_compile_rail",
    ),
];

const REPLAY_BINDINGS: &[(&str, &str)] = &[
    (
        concat!("n8n", "clone.workflow_execution.replay.artifact_placeholders.missing"),
        "missing_artifact_payload_renders_placeholder_without_validation_failure",
    ),
    (
        concat!("n8n", "clone.workflow_execution.replay.core.fold"),
        "replay_timeline_folds_events_and_moves_cursor",
    ),
    (
        concat!("n8n", "clone.workflow_execution.replay.core.no_side_effects"),
        "replay_timeline_folds_events_and_moves_cursor",
    ),
    (
        concat!("n8n", "clone.workflow_execution.replay.event_store.completed"),
        "store_persists_replay_events_for_run",
    ),
    (
        concat!("n8n", "clone.workflow_execution.replay.event_store.partial"),
        "store_persists_replay_events_for_run",
    ),
];

const SUBWORKFLOW_BINDINGS: &[(&str, &str)] = &[
    (
        concat!("n8n", "clone.workflow_execution.subworkflows.events.correlation"),
        "child_events_bind_parent_run_child_run_and_subworkflow_node_identity",
    ),
    (
        concat!("n8n", "clone.workflow_execution.subworkflows.invocation.outputs"),
        "saved_child_executes_and_routes_declared_output_to_the_real_parent_port",
    ),
];

const BINDING_GROUPS: &[BindingGroup] = &[
    ("agent", AGENT, AGENT_BINDINGS),
    ("openai", OPENAI, OPENAI_BINDINGS),
    ("runner", RUNNER, RUNNER_BINDINGS),
    ("expression", EXPRESSION, EXPRESSION_BINDINGS),
    ("files", FILES, FILE_BINDINGS),
    ("headless", HEADLESS, HEADLESS_BINDINGS),
    ("http", HTTP, HTTP_BINDINGS),
    ("images", IMAGES, IMAGE_BINDINGS),
    ("values", VALUES, VALUE_BINDINGS),
    ("llm-graph", LLM_GRAPH, LLM_BINDINGS),
    ("executor", EXECUTOR, EXECUTOR_BINDINGS),
    ("rats", RATS, RATS_BINDINGS),
    ("replay", REPLAY, REPLAY_BINDINGS),
    ("subworkflow", SUBWORKFLOW, SUBWORKFLOW_BINDINGS),
];

fn receipt(source: &str) -> CoverageReceipt {
    serde_json::from_str(source).expect("coverage decodes")
}

fn codes(issues: &[core::PromotionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

fn bound_ids() -> Vec<&'static str> {
    BINDING_GROUPS
        .iter()
        .flat_map(|(_, _, bindings)| bindings.iter().map(|(requirement_id, _)| *requirement_id))
        .collect()
}

// r[verify lattice.repository_validation.workflow_execution_promotion]
// r[verify lattice.repository_validation.workflow_execution_promotion.scenario.delta]
#[test]
fn exact_workflow_execution_promotion_preserves_unrelated_gaps() {
    let baseline = receipt(BASELINE);
    let current = receipt(CURRENT);
    let issues = core::validate_promotion(&baseline, &current, IDS);
    assert!(issues.is_empty(), "issues: {issues:?}");
    assert_eq!(baseline.missing.len(), BASELINE_COUNT);
    assert_eq!(current.missing.len(), CURRENT_COUNT);
    assert_eq!(IDS.len(), PROMOTED_COUNT);
    assert!(baseline.dangling.is_empty());
    assert!(current.dangling.is_empty());
    assert!(!baseline.valid);
    assert!(!current.valid);
}

// r[verify lattice.repository_validation.workflow_execution_promotion.scenario.bindings]
#[test]
fn workflow_execution_markers_are_locally_bound_and_leave_policy() {
    let bound = bound_ids();
    assert_eq!(bound.len(), PROMOTED_COUNT);
    assert_eq!(bound.iter().copied().collect::<BTreeSet<_>>(), IDS.iter().copied().collect());
    let binding_issues = BINDING_GROUPS
        .iter()
        .flat_map(|(name, source, bindings)| {
            core::validate_marker_bindings(source, bindings).into_iter().map(|issue| (*name, issue))
        })
        .collect::<Vec<_>>();
    assert!(binding_issues.is_empty(), "binding issues: {binding_issues:?}");
    let policy_issues = core::validate_policy_excludes_promoted_ids(POLICY, IDS);
    assert!(policy_issues.is_empty(), "policy: {policy_issues:?}");
}

// r[verify lattice.repository_validation.workflow_execution_promotion.scenario.fail_closed]
#[test]
fn workflow_execution_promotion_drift_fails_closed() {
    let mut baseline = receipt(BASELINE);
    baseline.missing.push(baseline.missing[0].clone());
    let mut current = receipt(CURRENT);
    current.missing.retain(|identifier| identifier != UNRELATED);
    current.missing.push(IDS[0].to_string());
    current.missing.push(IDS[0].to_string());
    current.missing.push(ADDED.to_string());
    current.dangling.push(DANGLING.to_string());
    current.valid = true;
    let mut allowlist = IDS.to_vec();
    allowlist.push(IDS[0]);

    let promotion_issues = core::validate_promotion(&baseline, &current, &allowlist);
    let promotion_codes = codes(&promotion_issues);
    let binding_issues = core::validate_marker_bindings(AGENT, &[(IDS[0], "missing_test")]);
    let binding_codes = codes(&binding_issues);
    let stale_policy = format!("requirement_id = \"{}\"", IDS[0]);
    let policy_issues = core::validate_policy_excludes_promoted_ids(&stale_policy, IDS);
    let policy_codes = codes(&policy_issues);

    for expected in [
        "baseline.duplicate_missing",
        "baseline.unsorted",
        "current.duplicate_missing",
        "current.unsorted",
        "current.dangling",
        "current.expected_invalid",
        "allowlist.duplicate",
        "transition.expected_removal_missing",
        "transition.unexpected_removed",
        "transition.unexpected_added",
    ] {
        assert!(promotion_codes.contains(expected));
    }
    assert!(binding_codes.contains("binding.missing"));
    assert!(policy_codes.contains("policy.promoted_present"));
}
