#[path = "traceability_promotion/core.rs"]
mod core;
#[path = "traceability_tui_ux_ids.rs"]
mod tui_ux_ids;

use core::CoverageReceipt;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use tui_ux_ids::IDS;

const BASELINE: &str = include_str!("fixtures/traceability-tui-ux-completion-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-tui-ux-promotion-current.json");
const POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const APP: &str = include_str!("../src/tui/app.rs");
const INPUT: &str = include_str!("../src/tui/input.rs");
const PANELS: &str = include_str!("../src/tui/panels.rs");
const CONTROL: &str = include_str!("test_control_flow.rs");
const CREDENTIALS: &str = include_str!("test_credentials.rs");
const NODE_KIND: &str = include_str!("test_node_kind_provider.rs");
const REPLAY: &str = include_str!("test_replay.rs");
const RUN_HISTORY: &str = include_str!("test_run_history_details.rs");
const TRIGGERS: &str = include_str!("test_triggers.rs");
const WEBHOOKS: &str = include_str!("webhooks.rs");
const COMPLETION: &str = include_str!("test_tui_ux_completion.rs");
const BASELINE_COUNT: usize = 98;
const CURRENT_COUNT: usize = 0;
const PROMOTED_COUNT: usize = 98;
const BINDING_COUNT: usize = 108;
const DOUBLE_BINDING_COUNT: usize = 2;
const TRIPLE_BINDING_COUNT: usize = 3;
const WORKSPACE_BINDING_COUNT: usize = 4;
const REPEATED_BINDING_COUNTS: &[(&str, usize)] = &[
    (concat!("n8n", "clone.tui_ux.agent_nodes.add_flow"), DOUBLE_BINDING_COUNT),
    (concat!("n8n", "clone.tui_ux.agent_nodes.errors"), TRIPLE_BINDING_COUNT),
    (concat!("n8n", "clone.tui_ux.properties.discoverable_entrypoints"), DOUBLE_BINDING_COUNT),
    (concat!("n8n", "clone.tui_ux.unsaved.failed_save"), DOUBLE_BINDING_COUNT),
    (concat!("n8n", "clone.tui_ux.unsaved.mutation_marks_dirty"), DOUBLE_BINDING_COUNT),
    (concat!("n8n", "clone.tui_ux.unsaved.save_clears"), DOUBLE_BINDING_COUNT),
    (concat!("n8n", "clone.tui_ux.workspace"), WORKSPACE_BINDING_COUNT),
];
const UNRELATED: &str = "lattice.example.unaccepted.scenario.unrelated";
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

type Binding = (&'static str, &'static str);
type BindingGroup = (&'static str, &'static str, &'static [Binding]);

const APP_BINDINGS: &[Binding] = &[
    (concat!("n8n", "clone.tui_ux.add_node.cancel"), "add_node_flow_cancel_preserves_workflow"),
    (
        concat!("n8n", "clone.tui_ux.add_node.guided"),
        "add_node_flow_creates_shell_node_from_entered_fields",
    ),
    (
        concat!("n8n", "clone.tui_ux.add_node.shell_fields"),
        "add_node_flow_creates_shell_node_from_entered_fields",
    ),
    (concat!("n8n", "clone.tui_ux.workspace"), "add_node_flow_creates_shell_node_from_entered_fields"),
    (concat!("n8n", "clone.tui_ux.add_node.template"), "add_node_flow_creates_node_from_template"),
    (concat!("n8n", "clone.tui_ux.agent_nodes.add_flow"), "add_node_flow_creates_image_input_node"),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.add_flow"),
        "add_node_flow_creates_pi_and_clankers_agent_nodes",
    ),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.add_flow.cancel"),
        "add_node_flow_cancel_preserves_workflow",
    ),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.add_flow.clankers"),
        "add_node_flow_creates_pi_and_clankers_agent_nodes",
    ),
    (concat!("n8n", "clone.tui_ux.agent_nodes.add_flow.input"), "add_node_flow_creates_image_input_node"),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.add_flow.pi"),
        "add_node_flow_creates_pi_and_clankers_agent_nodes",
    ),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.errors"),
        "agent_error_feedback_is_visible_after_run_failure",
    ),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.errors"),
        "image_and_agent_property_edits_update_config_and_reject_invalid_values",
    ),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.errors"),
        "image_node_error_feedback_is_visible_after_run_failure",
    ),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.errors.adapter"),
        "agent_error_feedback_is_visible_after_run_failure",
    ),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.errors.invalid_target"),
        "image_and_agent_property_edits_update_config_and_reject_invalid_values",
    ),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.errors.missing_path"),
        "image_node_error_feedback_is_visible_after_run_failure",
    ),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.errors.unsupported_format"),
        "image_node_error_feedback_is_visible_after_run_failure",
    ),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.properties"),
        "image_and_agent_property_edits_update_config_and_reject_invalid_values",
    ),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.properties.image_path"),
        "image_and_agent_property_edits_update_config_and_reject_invalid_values",
    ),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.properties.invalid"),
        "image_and_agent_property_edits_update_config_and_reject_invalid_values",
    ),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.properties.prompt"),
        "image_and_agent_property_edits_update_config_and_reject_invalid_values",
    ),
    (
        concat!("n8n", "clone.tui_ux.agent_nodes.properties.target"),
        "image_and_agent_property_edits_update_config_and_reject_invalid_values",
    ),
    (
        concat!("n8n", "clone.tui_ux.background_runs.cancel"),
        "cancel_active_run_marks_pending_nodes_skipped",
    ),
    (concat!("n8n", "clone.tui_ux.background_runs.failure"), "failed_background_run_marks_node_failed"),
    (concat!("n8n", "clone.tui_ux.background_runs.status"), "background_run_starts_without_blocking"),
    (
        concat!("n8n", "clone.tui_ux.image_nodes.preview.no_raw_bytes"),
        "completed_image_outputs_render_bounded_preview_metadata",
    ),
    (
        concat!("n8n", "clone.tui_ux.image_nodes.preview.output_panel"),
        "completed_image_outputs_render_bounded_preview_metadata",
    ),
    (
        concat!("n8n", "clone.tui_ux.properties.tests"),
        "image_and_agent_property_edits_update_config_and_reject_invalid_values",
    ),
    (
        concat!("n8n", "clone.tui_ux.properties.validate_timeout"),
        "property_edit_rejects_invalid_timeout_without_mutating",
    ),
    (
        concat!("n8n", "clone.tui_ux.run_history.details.empty_error.storage"),
        "run_detail_storage_failure_keeps_history_open_and_reports_safe_error",
    ),
    (concat!("n8n", "clone.tui_ux.run_history.list"), "completed_background_run_is_persisted_to_history"),
    (
        concat!("n8n", "clone.tui_ux.run_history.persist"),
        "completed_background_run_is_persisted_to_history",
    ),
    (
        concat!("n8n", "clone.tui_ux.run_history.persist_record"),
        "completed_background_run_is_persisted_to_history",
    ),
    (concat!("n8n", "clone.tui_ux.workspace"), "completed_background_run_is_persisted_to_history"),
    (concat!("n8n", "clone.tui_ux.unsaved.failed_save"), "failed_save_keeps_dirty_state"),
    (concat!("n8n", "clone.tui_ux.unsaved.mutation_marks_dirty"), "mutation_marks_workflow_dirty"),
    (concat!("n8n", "clone.tui_ux.unsaved.quit_confirm"), "dirty_quit_requests_confirmation"),
    (concat!("n8n", "clone.tui_ux.unsaved.save_clears"), "successful_save_clears_dirty_state"),
    (concat!("n8n", "clone.tui_ux.workflow_browser.empty"), "workflow_browser_reports_empty_store"),
    (concat!("n8n", "clone.tui_ux.workflow_browser.load"), "workflow_browser_loads_selected_workflow"),
    (
        concat!("n8n", "clone.tui_ux.workflow_browser.load_failure"),
        "workflow_browser_load_failure_preserves_current_workflow",
    ),
    (concat!("n8n", "clone.tui_ux.workflow_browser.open"), "workflow_browser_lists_saved_workflows"),
];

const INPUT_BINDINGS: &[Binding] = &[
    (
        concat!("n8n", "clone.tui_ux.command_palette.escape"),
        "palette_escape_closes_without_changing_workflow",
    ),
    (
        concat!("n8n", "clone.tui_ux.command_palette.execute"),
        "palette_executes_filtered_action_and_closes",
    ),
    (
        concat!("n8n", "clone.tui_ux.command_palette.interactive"),
        "palette_navigation_executes_selected_action",
    ),
    (
        concat!("n8n", "clone.tui_ux.command_palette.no_match"),
        "palette_enter_with_no_match_keeps_recoverable_state",
    ),
    (concat!("n8n", "clone.tui_ux.credentials.providers.edit"), "property_key_edits_agent_credentials"),
    (
        concat!("n8n", "clone.tui_ux.mouse_wiring.drag_release"),
        "mouse_drag_release_creates_compatible_edge",
    ),
    (
        concat!("n8n", "clone.tui_ux.mouse_wiring.drag_release.compatible"),
        "mouse_drag_release_creates_compatible_edge",
    ),
    (
        concat!("n8n", "clone.tui_ux.mouse_wiring.invalid_release"),
        "mouse_drag_release_on_non_port_cancels_wiring_without_edge",
    ),
    (
        concat!("n8n", "clone.tui_ux.mouse_wiring.snap_near_ports"),
        "mouse_drag_release_near_visible_input_snaps_to_edge",
    ),
    (
        concat!("n8n", "clone.tui_ux.mouse_wiring.snap_near_ports.far_release"),
        "mouse_drag_release_on_non_port_cancels_wiring_without_edge",
    ),
    (
        concat!("n8n", "clone.tui_ux.mouse_wiring.snap_near_ports.release"),
        "mouse_drag_release_near_visible_input_snaps_to_edge",
    ),
    (
        concat!("n8n", "clone.tui_ux.mouse_wiring.visible_port_clicks"),
        "mouse_click_release_keeps_click_to_click_wiring_available",
    ),
    (
        concat!("n8n", "clone.tui_ux.mouse_wiring.visible_port_clicks.complete"),
        "mouse_click_release_keeps_click_to_click_wiring_available",
    ),
    (
        concat!("n8n", "clone.tui_ux.mouse_wiring.visible_port_clicks.start"),
        "mouse_click_release_keeps_click_to_click_wiring_available",
    ),
    (
        concat!("n8n", "clone.tui_ux.properties.cancel"),
        "canvas_key_cancels_property_edit_without_mutating_command",
    ),
    (
        concat!("n8n", "clone.tui_ux.properties.discoverable_entrypoints"),
        "canvas_key_edits_shell_command_without_focus_change",
    ),
    (
        concat!("n8n", "clone.tui_ux.properties.discoverable_entrypoints"),
        "palette_executes_filtered_action_and_closes",
    ),
    (
        concat!("n8n", "clone.tui_ux.properties.discoverable_entrypoints.canvas_command"),
        "canvas_key_edits_shell_command_without_focus_change",
    ),
    (
        concat!("n8n", "clone.tui_ux.properties.discoverable_entrypoints.no_selection"),
        "edit_shortcut_without_selection_does_not_mutate_workflow",
    ),
    (
        concat!("n8n", "clone.tui_ux.properties.discoverable_entrypoints.palette_command"),
        "palette_filters_to_edit_command_action",
    ),
    (
        concat!("n8n", "clone.tui_ux.properties.edit_command"),
        "canvas_key_edits_shell_command_without_focus_change",
    ),
    (concat!("n8n", "clone.tui_ux.properties.edit_shell"), "property_keys_edit_shell_command"),
    (concat!("n8n", "clone.tui_ux.properties.modal_text_entry"), "property_keys_edit_shell_command"),
    (
        concat!("n8n", "clone.tui_ux.properties.modal_text_entry.cancel"),
        "canvas_key_cancels_property_edit_without_mutating_command",
    ),
    (
        concat!("n8n", "clone.tui_ux.properties.modal_text_entry.save_command"),
        "property_keys_edit_shell_command",
    ),
];

const PANEL_BINDINGS: &[Binding] = &[
    (
        concat!("n8n", "clone.tui_ux.empty_state.next_actions"),
        "empty_state_copy_lists_next_actions_for_full_canvas",
    ),
    (
        concat!("n8n", "clone.tui_ux.empty_state.tests"),
        "empty_state_visibility_requires_empty_workflow_without_modal",
    ),
    (concat!("n8n", "clone.tui_ux.help_footer.canvas"), "contextual_help_matches_canvas_and_wiring_modes"),
    (
        concat!("n8n", "clone.tui_ux.help_footer.properties_output"),
        "contextual_help_matches_properties_and_output_focus",
    ),
    (
        concat!("n8n", "clone.tui_ux.help_footer.truncate"),
        "contextual_help_uses_compact_copy_for_narrow_width",
    ),
    (concat!("n8n", "clone.tui_ux.help_footer.wiring"), "contextual_help_matches_canvas_and_wiring_modes"),
    (
        concat!("n8n", "clone.tui_ux.keyboard_first.discoverable"),
        "contextual_help_matches_canvas_and_wiring_modes",
    ),
    (
        concat!("n8n", "clone.tui_ux.properties.discoverable_help"),
        "contextual_help_matches_canvas_and_wiring_modes",
    ),
    (
        concat!("n8n", "clone.tui_ux.properties.discoverable_help.footer"),
        "contextual_help_matches_properties_and_output_focus",
    ),
    (
        concat!("n8n", "clone.tui_ux.properties.discoverable_help.palette"),
        "contextual_help_reports_palette_modal",
    ),
    (concat!("n8n", "clone.tui_ux.unsaved.dirty_indicator"), "unsaved_indicator_marks_dirty_workflows"),
    (concat!("n8n", "clone.tui_ux.unsaved.failed_save"), "unsaved_indicator_marks_dirty_workflows"),
    (
        concat!("n8n", "clone.tui_ux.unsaved.mutation_marks_dirty"),
        "unsaved_indicator_marks_dirty_workflows",
    ),
    (concat!("n8n", "clone.tui_ux.unsaved.save_clears"), "unsaved_indicator_marks_dirty_workflows"),
];

const CONTROL_BINDINGS: &[Binding] = &[(
    (concat!("n8n", "clone.tui_ux.control_flow.properties")),
    "tui_adds_condition_node_and_preserves_previous_predicate_on_invalid_edit",
)];

const CREDENTIAL_BINDINGS: &[Binding] = &[(
    (concat!("n8n", "clone.tui_ux.credentials.providers.test")),
    "provider_probe_hides_resolved_secret_and_redacts_error_key",
)];

const NODE_KIND_BINDINGS: &[Binding] = &[(
    (concat!("n8n", "clone.tui_ux.node_kind_provider")),
    "provider_property_metadata_lists_shell_type_and_command",
)];

const REPLAY_BINDINGS: &[Binding] = &[
    (
        concat!("n8n", "clone.tui_ux.replay_debugger.redaction.secret"),
        "sensitive_values_are_redacted_in_replay_summaries",
    ),
    (
        concat!("n8n", "clone.tui_ux.replay_debugger.timeline.step"),
        "replay_timeline_folds_events_and_moves_cursor",
    ),
];

const RUN_HISTORY_BINDINGS: &[Binding] = &[
    (
        concat!("n8n", "clone.tui_ux.replay_debugger.timeline.open"),
        "app_opens_historical_replay_and_renders_timeline_and_node_outputs",
    ),
    (
        concat!("n8n", "clone.tui_ux.run_history.details.persist.nodes"),
        "run_detail_record_captures_node_outputs",
    ),
    (concat!("n8n", "clone.tui_ux.run_history.empty_error"), "app_reports_missing_run_history_details"),
    (concat!("n8n", "clone.tui_ux.run_history.inspect"), "app_inspects_run_history_details"),
    (concat!("n8n", "clone.tui_ux.workspace"), "app_inspects_run_history_details"),
    (
        concat!("n8n", "clone.tui_ux.run_history.node_outputs.inspect"),
        "app_opens_historical_replay_and_renders_timeline_and_node_outputs",
    ),
    (
        concat!("n8n", "clone.tui_ux.run_history.node_outputs.missing"),
        "app_renders_empty_node_output_and_unavailable_rerun_as_recoverable",
    ),
    (
        concat!("n8n", "clone.tui_ux.run_history.rerun.unavailable"),
        "app_renders_empty_node_output_and_unavailable_rerun_as_recoverable",
    ),
    (
        concat!("n8n", "clone.tui_ux.run_history.rerun.workflow"),
        "app_reruns_history_workflow_without_replacing_current_workflow",
    ),
    (
        concat!("n8n", "clone.tui_ux.run_history.timeline.show"),
        "app_opens_historical_replay_and_renders_timeline_and_node_outputs",
    ),
];

const TRIGGER_BINDINGS: &[Binding] = &[
    (
        concat!("n8n", "clone.tui_ux.triggers.manage.error"),
        "tui_trigger_error_summary_reports_status_without_raw_message_or_payload",
    ),
    (concat!("n8n", "clone.tui_ux.triggers.manage.list"), "tui_exposes_minimal_trigger_status_lines"),
];

const WEBHOOK_BINDINGS: &[Binding] = &[(
    (concat!("n8n", "clone.tui_ux.webhook_listener.cli.start")),
    "operator_listener_loads_persisted_trigger_and_accepts_loopback_request",
)];

const COMPLETION_BINDINGS: &[Binding] = &[
    (
        concat!("n8n", "clone.tui_ux.background_runs.long_running"),
        "active_run_keeps_rendering_and_input_live_while_replay_events_stream",
    ),
    (
        concat!("n8n", "clone.tui_ux.background_runs.responsive"),
        "active_run_keeps_rendering_and_input_live_while_replay_events_stream",
    ),
    (
        concat!("n8n", "clone.tui_ux.image_nodes.preview"),
        "canvas_renders_bounded_image_metadata_fallback_without_raw_bytes",
    ),
    (
        concat!("n8n", "clone.tui_ux.image_nodes.preview.canvas"),
        "canvas_renders_bounded_image_metadata_fallback_without_raw_bytes",
    ),
    (
        concat!("n8n", "clone.tui_ux.keyboard_first"),
        "workspace_render_exposes_all_panels_and_keyboard_authoring_actions",
    ),
    (
        concat!("n8n", "clone.tui_ux.replay_debugger.active_run.stream"),
        "active_run_keeps_rendering_and_input_live_while_replay_events_stream",
    ),
    (
        concat!("n8n", "clone.tui_ux.workspace"),
        "workspace_render_exposes_all_panels_and_keyboard_authoring_actions",
    ),
    (
        concat!("n8n", "clone.tui_ux.workspace.starts"),
        "workspace_render_exposes_all_panels_and_keyboard_authoring_actions",
    ),
];

const BINDING_GROUPS: &[BindingGroup] = &[
    ("app", APP, APP_BINDINGS),
    ("input", INPUT, INPUT_BINDINGS),
    ("panels", PANELS, PANEL_BINDINGS),
    ("control", CONTROL, CONTROL_BINDINGS),
    ("credentials", CREDENTIALS, CREDENTIAL_BINDINGS),
    ("node-kind", NODE_KIND, NODE_KIND_BINDINGS),
    ("replay", REPLAY, REPLAY_BINDINGS),
    ("run-history", RUN_HISTORY, RUN_HISTORY_BINDINGS),
    ("triggers", TRIGGERS, TRIGGER_BINDINGS),
    ("webhooks", WEBHOOKS, WEBHOOK_BINDINGS),
    ("completion", COMPLETION, COMPLETION_BINDINGS),
];

fn receipt(source: &str) -> CoverageReceipt {
    serde_json::from_str(source).expect("coverage decodes")
}

fn codes(issues: &[core::PromotionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

fn bindings() -> Vec<Binding> {
    BINDING_GROUPS.iter().flat_map(|(_, _, bindings)| bindings.iter().copied()).collect()
}

fn binding_sources() -> Vec<&'static str> {
    BINDING_GROUPS.iter().map(|(_, source, _)| *source).collect()
}

fn bound_ids() -> Vec<&'static str> {
    bindings().into_iter().map(|(requirement_id, _)| requirement_id).collect()
}

fn expected_binding_count(requirement_id: &str) -> usize {
    REPEATED_BINDING_COUNTS
        .iter()
        .find(|(repeated_id, _)| *repeated_id == requirement_id)
        .map_or(1, |(_, count)| *count)
}

// r[verify lattice.repository_validation.tui_ux_promotion]
// r[verify lattice.repository_validation.tui_ux_promotion.scenario.delta]
#[test]
fn exact_final_tui_ux_promotion_reaches_successful_coverage() {
    let baseline = receipt(BASELINE);
    let current = receipt(CURRENT);
    let issues = core::validate_final_promotion(&baseline, &current, IDS);

    assert!(issues.is_empty(), "issues: {issues:?}");
    assert_eq!(baseline.missing.len(), BASELINE_COUNT);
    assert_eq!(current.missing.len(), CURRENT_COUNT);
    assert_eq!(IDS.len(), PROMOTED_COUNT);
    assert!(baseline.dangling.is_empty());
    assert!(current.dangling.is_empty());
    assert!(!baseline.valid);
    assert!(current.valid);
}

// r[verify lattice.repository_validation.tui_ux_promotion.scenario.bindings]
#[test]
fn tui_ux_markers_are_locally_bound_and_leave_policy() {
    let bound = bound_ids();
    assert_eq!(bound.len(), BINDING_COUNT);
    assert_eq!(bound.iter().copied().collect::<BTreeSet<_>>(), IDS.iter().copied().collect());
    let mut actual_counts = BTreeMap::new();
    for requirement_id in &bound {
        *actual_counts.entry(*requirement_id).or_insert(0_usize) += 1;
    }
    for requirement_id in IDS {
        assert_eq!(actual_counts.get(requirement_id).copied(), Some(expected_binding_count(requirement_id)));
    }
    let binding_issues = BINDING_GROUPS
        .iter()
        .flat_map(|(name, source, bindings)| {
            core::validate_marker_bindings(source, bindings).into_iter().map(|issue| (*name, issue))
        })
        .collect::<Vec<_>>();
    assert!(binding_issues.is_empty(), "binding issues: {binding_issues:?}");
    let occurrence_issues = core::validate_marker_occurrences(&binding_sources(), &bindings());
    assert!(occurrence_issues.is_empty(), "occurrence issues: {occurrence_issues:?}");
    let policy_issues = core::validate_policy_excludes_promoted_ids(POLICY, IDS);
    assert!(policy_issues.is_empty(), "policy: {policy_issues:?}");
}

// r[verify lattice.repository_validation.tui_ux_promotion.scenario.fail_closed]
#[test]
fn final_tui_ux_promotion_drift_fails_closed() {
    let mut baseline = receipt(BASELINE);
    baseline.missing.push(baseline.missing[0].clone());
    baseline.missing.push(UNRELATED.to_string());
    let mut current = receipt(CURRENT);
    current.missing.push(IDS[0].to_string());
    current.missing.push(IDS[0].to_string());
    current.missing.push(ADDED.to_string());
    current.dangling.push(DANGLING.to_string());
    current.referenced = current.referenced.saturating_sub(1);
    current.valid = false;
    let mut allowlist = IDS.to_vec();
    allowlist.push(IDS[0]);

    let promotion_issues = core::validate_final_promotion(&baseline, &current, &allowlist);
    let promotion_codes = codes(&promotion_issues);
    let binding_issues = core::validate_marker_bindings(APP, &[(IDS[0], "missing_test")]);
    let binding_codes = codes(&binding_issues);
    let extra_marker_source = format!("{APP}\n// r[verify {}]", IDS[0]);
    let occurrence_issues =
        core::validate_marker_occurrences(&[extra_marker_source.as_str()], &[(IDS[0], APP_BINDINGS[0].1)]);
    let occurrence_codes = codes(&occurrence_issues);
    let stale_policy = format!("requirement_id = \"{}\"", IDS[0]);
    let policy_issues = core::validate_policy_excludes_promoted_ids(&stale_policy, IDS);
    let policy_codes = codes(&policy_issues);

    for expected in [
        "baseline.duplicate_missing",
        "baseline.unsorted",
        "current.duplicate_missing",
        "current.unsorted",
        "current.dangling",
        "current.expected_complete",
        "current.expected_full_reference",
        "current.expected_valid",
        "allowlist.duplicate",
        "transition.expected_removal_missing",
        "transition.unexpected_removed",
        "transition.unexpected_added",
    ] {
        assert!(promotion_codes.contains(expected), "missing issue code {expected}: {promotion_issues:?}");
    }
    assert!(binding_codes.contains("binding.missing"));
    assert!(occurrence_codes.contains("binding.occurrence_count"));
    assert!(policy_codes.contains("policy.promoted_present"));
}
