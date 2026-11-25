use super::*;

// r[verify n8nclone.tui_ux.unsaved.mutation_marks_dirty]
#[test]
fn mutation_marks_workflow_dirty() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));

    assert!(!app.has_unsaved_changes());
    app.add_shell_node("Example", "echo example");

    assert!(app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.unsaved.save_clears]
#[test]
fn successful_save_clears_dirty_state() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Example", "echo example");

    assert!(app.has_unsaved_changes());
    app.save_workflow();

    assert!(!app.has_unsaved_changes());
    assert!(!app.is_confirming_quit());
}

#[test]
fn app_creates_missing_data_directory_and_persists_workflow() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let data_dir = temp_dir.path().join("nested").join("state");
    let mut app = State::new(AppConfig {
        data_dir: data_dir.clone(),
        ..AppConfig::default()
    });

    assert!(app.store.is_some(), "store opens in the created data directory");
    app.add_shell_node("Persist", "true");
    app.save_workflow();

    assert!(!app.has_unsaved_changes());
    assert!(crate::config::store_path(&data_dir).exists());
    assert!(app.store.as_ref().unwrap().load_workflow("untitled").unwrap().is_some());
}

// r[verify n8nclone.tui_ux.unsaved.failed_save]
#[test]
fn failed_save_keeps_dirty_state() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Example", "echo example");
    app.store = None;

    app.save_workflow();

    assert!(app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.unsaved.quit_confirm]
#[test]
fn dirty_quit_requests_confirmation() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Example", "echo example");

    assert!(!app.request_quit());
    assert!(app.is_confirming_quit());

    app.cancel_quit_confirmation();
    assert!(!app.is_confirming_quit());
}

#[test]
fn clean_quit_exits_without_confirmation() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));

    assert!(app.request_quit());
    assert!(!app.is_confirming_quit());
}

// r[verify n8nclone.tui_ux.workflow_browser.open]
#[test]
fn workflow_browser_lists_saved_workflows() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.workflow.meta.name = "saved".to_string();
    app.save_workflow();

    app.open_workflow_browser();

    assert!(app.is_workflow_browser_open());
    assert_eq!(app.workflow_browser.names, vec!["saved".to_string()]);
    assert_eq!(app.workflow_browser.selected, 0);
}

// r[verify n8nclone.tui_ux.workflow_browser.empty]
#[test]
fn workflow_browser_reports_empty_store() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));

    app.open_workflow_browser();

    assert!(app.is_workflow_browser_open());
    assert!(app.workflow_browser.names.is_empty());
    assert_eq!(app.workflow_browser.message.as_deref(), Some("No saved workflows available"),);
}

// r[verify n8nclone.tui_ux.workflow_browser.load]
#[test]
fn workflow_browser_loads_selected_workflow() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.workflow.meta.name = "saved".to_string();
    app.add_shell_node("Saved", "echo saved");
    app.save_workflow();
    app.workflow = Workflow::new("current", crate::engine::workflow::WorkflowTimestamp::epoch());
    app.sync_graph_state();

    app.open_workflow_browser();
    app.confirm_workflow_browser_selection();

    assert_eq!(app.workflow.meta.name, "saved");
    assert_eq!(app.workflow.graph.node_count(), 1);
    assert!(!app.is_workflow_browser_open());
    assert!(!app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.workflow_browser.load_failure]
#[test]
fn workflow_browser_load_failure_preserves_current_workflow() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.workflow = Workflow::new("current", crate::engine::workflow::WorkflowTimestamp::epoch());
    app.sync_graph_state();

    assert!(!app.load_workflow_by_name("missing"));

    assert_eq!(app.workflow.meta.name, "current");
    assert_eq!(app.workflow.graph.node_count(), 0);
}

#[test]
fn exhausted_run_identity_is_reported_before_starting_background_work() {
    let directory = tempfile::tempdir().expect("app directory");
    let mut app = State::new(test_config(&directory));
    app.next_tui_run_id = u64::MAX;
    let workflow = app.workflow.clone();
    assert!(!app.start_workflow_snapshot(workflow, "must not start"));
    assert!(!app.run_state.active);
    assert!(app.active_run.is_none());
    assert_eq!(app.next_tui_run_id, u64::MAX);
    assert!(output_text_lines(&app).iter().any(|line| line.contains("identity range exhausted")));
}

// r[verify lattice.tui_ux.aggregate_node_deletion]
// r[verify lattice.tui_ux.aggregate_node_deletion.scenario.missing]
#[test]
fn delete_selected_nodes_clears_active_editors_and_rejects_stale_selection() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Target", "echo target");
    let target = app.selected_node_id().expect("target selected");
    app.add_shell_node("Retained", "echo retained");
    let retained = app.selected_node_id().expect("retained selected");

    app.select_node(target);
    assert!(app.start_property_edit(PropertyEditField::Command));
    app.open_authoring("all");
    // Re-open property edit alongside authoring to verify both active editors clear.
    app.property_edit = Some(PropertyEditState {
        node_id: target,
        field: PropertyEditField::Command,
        original: "echo target".to_string(),
        draft: "echo pending".to_string(),
        workflow_snapshot: crate::workflow_format::workflow_to_portable(&app.workflow),
        authoring_snapshot: crate::authoring::identity("lattice.tui-authoring-snapshot.v1", &app.workflow).ok(),
    });
    assert_eq!(app.authoring.as_ref().and_then(|state| state.node_id), Some(target.0));
    app.clear_workflow_dirty();

    assert_eq!(app.delete_selected_nodes(), 1);
    assert!(app.property_edit.is_none(), "active property edit for deleted node must clear");
    assert!(app.authoring.is_none(), "active authoring browser for deleted node must clear");
    assert!(app.workflow.graph.node(target).is_none());
    assert!(app.workflow.node_config(target).is_none());
    assert!(app.workflow.graph.node(retained).is_some());
    assert!(app.has_unsaved_changes());

    app.clear_workflow_dirty();
    app.select_node(retained);
    app.open_authoring("all");
    let before = serde_json::to_value(&app.workflow).expect("workflow snapshot");
    app.graph_state
        .selected
        .insert(crate::tui::graph::subwayrat_node_id(crate::engine::workflow::NodeId(u64::MAX - 1)));
    assert_eq!(app.delete_selected_nodes(), 0);
    assert!(!app.has_unsaved_changes(), "stale deletion must not mark workflow dirty");
    assert_eq!(serde_json::to_value(&app.workflow).expect("workflow snapshot"), before);
    assert_eq!(app.authoring.as_ref().and_then(|state| state.node_id), Some(retained.0));
}

// r[verify lattice.credentials.animus_secret_providers.round_trip.scenario.delete]
#[test]
fn delete_selected_nodes_atomically_removes_node_credentials_and_keeps_round_trip_valid() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Target", "echo target");
    let target = app.selected_node_id().expect("target selected");
    app.workflow.set_node_credentials(target, NodeCredentialRefs {
        env: [("TARGET_TOKEN".to_string(), SecretRef::new("env", "TARGET_TOKEN", None))].into(),
    });
    app.add_shell_node("Retained", "echo retained");
    let retained = app.selected_node_id().expect("retained selected");
    app.workflow.set_node_credentials(retained, NodeCredentialRefs {
        env: [("KEPT_TOKEN".to_string(), SecretRef::new("env", "KEPT_TOKEN", None))].into(),
    });

    app.select_node(target);
    assert_eq!(app.delete_selected_nodes(), 1);
    assert!(app.workflow.node_credentials(target).is_none());
    assert!(!app.workflow.credentials.contains_key(&target.0));
    assert!(app.workflow.node_credentials(retained).is_some());
    assert!(app.validate_workflow_diagnostics().is_empty());

    app.save_workflow();
    let exported = crate::workflow_format::workflow_to_nickel(&app.workflow);
    let exported_path = temp_dir.path().join("exported.ncl");
    std::fs::write(&exported_path, &exported).expect("write exported workflow");
    let imported =
        crate::workflow_format::load_workflow_nickel_file(&exported_path).expect("exported workflow imports");
    let imported_retained =
        imported.graph.nodes().find(|node| node.label == "Retained").expect("retained node imported").id;
    assert_eq!(imported.graph.nodes().count(), 1);
    assert_eq!(imported.credentials.len(), 1);
    assert!(imported.node_credentials(imported_retained).is_some_and(|refs| refs.env.contains_key("KEPT_TOKEN")));
    assert!(!exported.contains("TARGET_TOKEN"));
    assert!(!exported.contains("Target"));
}

// r[verify lattice.credentials.animus_secret_providers.tui]
// r[verify lattice.credentials.animus_secret_providers.tui.scenario.edit]
// r[verify lattice.credentials.animus_secret_providers.tui.scenario.reachability]
// r[verify lattice.credentials.animus_secret_providers.round_trip]
#[cfg(feature = "animus-secretspec")]
#[test]
fn tui_secretspec_provider_selection_edit_recovery_and_safe_reachability() {
    use crate::credentials::config::CredentialProviderConfig;
    use crate::credentials::config::DeclarationMapping;
    use crate::credentials::config::HostInputConfig;
    use crate::credentials::config::ProviderScheme;
    use crate::credentials::config::RequestBounds;
    use crate::credentials::config::SecretSpecHostConfig;
    use crate::credentials::config::SecretSpecProviderConfig;

    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let manifest_path = temp_dir.path().join("secretspec.toml");
    let manifest_bytes = b"[project]\nname = 'lattice-tui-fixture'\nrevision = '1.0'\n";
    std::fs::write(&manifest_path, manifest_bytes).expect("manifest writes");
    let manifest_identity = blake3::hash(manifest_bytes).to_hex().to_string();

    let policy_json = serde_json::to_vec(&serde_json::json!({
        "schema_version": 1,
        "resolver_kind": animus_std_secretspec::RESOLVER_IDENTITY,
        "ambient": { "inherit_environment": false, "manifest_discovery": false },
        "audit": { "enabled": false, "authoritative": false },
        "write_operations_allowed": false,
        "non_claims": [
            "caller-authority",
            "credential-correctness",
            "credential-freshness",
            "provider-integrity",
            "audit-completeness",
            "production-readiness",
            "release-eligibility"
        ],
        "secretspec": {
            "version": animus_std_secretspec::SECRETSPEC_VERSION,
            "commit": animus_std_secretspec::SECRETSPEC_COMMIT,
            "crate_checksum": animus_std_secretspec::SECRETSPEC_CRATE_CHECKSUM,
            "license": "Apache-2.0",
            "features": ["sops"]
        },
        "bounded_exec": {
            "commit": animus_std_secretspec::BOUNDED_EXEC_COMMIT,
            "license": "AGPL-3.0-or-later"
        },
        "profiles": [{
            "profile": "service",
            "scope": "workflow",
            "declaration": "API_TOKEN",
            "manifest_identity": manifest_identity,
            "provider": "environment",
            "provider_selector": "env://",
            "purpose": "workflow-authentication",
            "destination": "provider_dispatch",
            "bounds": {
                "value_count_max": 1,
                "input_max_bytes": 4096,
                "value_max_bytes": 1024,
                "stdout_max_bytes": 1056,
                "stderr_max_bytes": 1024,
                "timeout_ms": 5000,
                "poll_interval_ms": 10,
                "teardown_timeout_ms": 1000
            }
        }]
    }))
    .expect("policy json serializes");
    let policy_identity = blake3::hash(&policy_json).to_hex().to_string();
    let policy_path = temp_dir.path().join("policy.json");
    std::fs::write(&policy_path, &policy_json).expect("policy writes");

    let provider_config = SecretSpecProviderConfig {
        id: "reviewed-secret".to_string(),
        cohort: "0.17.0".to_string(),
        manifest_identity,
        policy_identity,
        profile: "service".to_string(),
        scope: "workflow".to_string(),
        declarations: vec![DeclarationMapping {
            key: "token".to_string(),
            field: None,
            declaration: "API_TOKEN".to_string(),
            scheme: ProviderScheme::Environment,
        }],
        allowed_schemes: vec![ProviderScheme::Environment],
        reason: "workflow-authentication".to_string(),
        bounds: RequestBounds {
            input_max_bytes: 4096,
            value_max_bytes: 1024,
            stdout_max_bytes: 1056,
            stderr_max_bytes: 1024,
            timeout_ms: 5000,
            poll_interval_ms: 10,
            teardown_timeout_ms: 1000,
        },
    };
    let host_config = SecretSpecHostConfig {
        provider_id: "reviewed-secret".to_string(),
        policy_path: policy_path.clone(),
        manifest_path,
        worker_executable: temp_dir.path().join("absent-worker"),
        working_directory: temp_dir.path().to_path_buf(),
        input: HostInputConfig::Environment {
            variable: "LATTICE_TUI_SECRET_NEVER_LOADED".to_string(),
        },
    };
    let mut cfg = test_config(&temp_dir);
    cfg.credential_providers = vec![CredentialProviderConfig::AnimusSecretspec(provider_config)];
    cfg.credential_hosts = vec![host_config];

    let mut app = State::new(cfg);
    assert_eq!(app.configured_credential_providers(), vec!["env".to_string(), "reviewed-secret".to_string()]);

    app.add_shell_node("SecretConsumer", "true");
    let node_id = app.selected_node_id().expect("node selected");
    app.clear_workflow_dirty();

    assert!(app.start_property_edit(PropertyEditField::Credentials));
    app.property_edit.as_mut().expect("property edit").draft = "TOKEN=reviewed-secret:token".to_string();
    assert!(app.save_property_edit());
    assert_eq!(
        app.workflow.node_credentials(node_id).expect("credentials").env["TOKEN"],
        SecretRef::new("reviewed-secret", "token", None)
    );

    // Invalid edit preserves prior valid opaque reference.
    app.clear_workflow_dirty();
    assert!(app.start_property_edit(PropertyEditField::Credentials));
    app.property_edit.as_mut().expect("property edit").draft = "TOKEN=reviewed-secret:undeclared".to_string();
    assert!(!app.save_property_edit());
    assert!(app.property_edit.is_some());
    assert!(!app.has_unsaved_changes());
    assert_eq!(
        app.workflow.node_credentials(node_id).expect("credentials preserved").env["TOKEN"],
        SecretRef::new("reviewed-secret", "token", None)
    );
    app.cancel_property_edit();

    // Probe must render safe reachability without loading material.
    let probes = app.probe_selected_node_credentials();
    assert_eq!(probes.len(), 1);
    assert!(!probes[0].reachable);
    assert_eq!(probes[0].message, "configured; reachability not probed");
    assert!(output_text_lines(&app).iter().any(|line| line.contains("configured; reachability not probed")));

    // Properties panel renders provider list and safe reachability text.
    let area = ratatui::layout::Rect::new(0, 0, 140, 40);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    crate::tui::panels::render(&mut app, area, &mut buf);
    let rendered = buf.content().iter().map(|cell| cell.symbol()).collect::<String>();
    assert!(rendered.contains("reviewed-secret"));
    assert!(rendered.contains("configured; reachability not probed"));

    app.open_diagnostics();
    assert!(!crate::workflow_validation::has_fatal_diagnostics(&app.diagnostics.diagnostics));
    app.close_diagnostics();
    app.save_workflow();
    assert!(!app.has_unsaved_changes());

    // Broken host config reports fixed safe diagnostic class without host path.
    std::fs::remove_file(&policy_path).expect("remove policy");
    let broken_probe = app.probe_credential_reference(&SecretRef::new("reviewed-secret", "token", None));
    assert!(!broken_probe.reachable);
    assert_eq!(broken_probe.message, "credential_secretspec_unavailable");
}
