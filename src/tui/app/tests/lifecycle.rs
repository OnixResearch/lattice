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

fn approval_gate_config(gate: &str) -> crate::workflow::gate::ApprovalNodeConfig {
    crate::workflow::gate::ApprovalNodeConfig {
        gate_id: crate::workflow::gate::GateId::try_from_text(gate).expect("gate id is valid"),
        safe_summary: format!("Review {gate}"),
        selected_input_ports: Vec::new(),
        required_artifacts: Vec::new(),
        authority: crate::workflow::gate::ApprovalAuthorityPolicy {
            profile: crate::workflow::gate::AuthorityProfile::LocalOperator,
            local_actors: vec![crate::workflow::gate::ActorId::try_from_text("reviewer-a").expect("actor id")],
            accepted_abilities: vec![
                crate::workflow::gate::DecisionAbility::Approve,
                crate::workflow::gate::DecisionAbility::Reject,
            ],
            prohibit_self_approval: true,
            delegated_policy_reference: None,
        },
        expiry: crate::workflow::gate::ApprovalExpiryPolicy {
            decision_ttl_millis: 60_000,
            resume_ttl_millis: 60_000,
            approval_must_be_fresh_at_resume: true,
            clock_source: crate::workflow::gate::ClockSourceClass::UtcWallClock,
        },
        reason: crate::workflow::gate::ApprovalReasonPolicy {
            required: false,
            identity_bound: true,
            max_bytes: 128,
        },
        resume: crate::workflow::gate::ApprovalResumeSchema {
            values: vec![crate::workflow::gate::ResumeValueField {
                name: crate::workflow::gate::ResumeBindingName::try_from_text("continue").expect("binding name"),
                type_tag: "string".to_string(),
            }],
            credential_bindings: Vec::new(),
        },
        terminal: crate::workflow::gate::ApprovalTerminalPolicy {
            on_rejection: crate::workflow::gate::TerminalOutcome::FailWorkflow,
            on_expiry: crate::workflow::gate::TerminalOutcome::CancelWorkflow,
        },
        required_non_claims: crate::workflow::gate::REQUIRED_APPROVAL_NON_CLAIMS.map(str::to_owned).to_vec(),
    }
}

fn approval_decision_command(
    request: &crate::workflow::gate::ApprovalRequest,
) -> crate::workflow::gate::DecisionCommand {
    crate::workflow::gate::DecisionCommand {
        request_id: request.id.clone(),
        expected_workflow_revision: request.workflow_revision.clone(),
        actor: crate::workflow::gate::ActorId::try_from_text("reviewer-a").expect("actor id"),
        decision: crate::workflow::gate::DecisionClass::Approve,
        reason: Some("reviewed".to_string()),
        observed_epoch_millis: request.requested_epoch_millis + 1,
        evidence: crate::workflow::gate::DecisionAuthorityEvidence::LocalOperator,
    }
}

fn approval_resume_command(request: &crate::workflow::gate::ApprovalRequest) -> crate::workflow::gate::ResumeCommand {
    crate::workflow::gate::ResumeCommand {
        request_id: request.id.clone(),
        expected_workflow_revision: request.workflow_revision.clone(),
        expected_event_head: request.events.last().expect("approval event head").id.clone(),
        owner: crate::workflow::gate::ResumeOwnerId::try_from_text("tui-operator").expect("owner id"),
        observed_epoch_millis: request.requested_epoch_millis + 1,
        values: vec![crate::workflow::gate::ResumeValue {
            name: crate::workflow::gate::ResumeBindingName::try_from_text("continue").expect("binding name"),
            value: crate::engine::value::Value::from_string("admitted".to_string()),
        }],
        credentials: Vec::new(),
    }
}

#[tokio::test]
async fn tui_resume_selected_approval_wires_workflow_env_credentials_and_authority_provider() {
    let env_key = if std::env::var("PATH").is_ok_and(|value| !value.is_empty()) {
        "PATH"
    } else {
        "HOME"
    };
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let effect_path = temp_dir.path().join("env-approval-delivered.txt");
    let mut workflow = Workflow::new("tui-approval-env-resume", crate::engine::workflow::WorkflowTimestamp::epoch());
    let approval_id = workflow.add_approval_node("approval", approval_gate_config("gate-env"));
    let shell_id = workflow.add_shell_node(
        "env-consumer",
        format!("[ -n \"$TOKEN\" ] && printf 'delivered' > '{}'", effect_path.display()),
    );
    workflow.set_node_credentials(shell_id, NodeCredentialRefs {
        env: [("TOKEN".to_string(), SecretRef::new("env", env_key, None))].into(),
    });
    let approval_out = workflow.graph.node(approval_id).expect("approval node").output_ports[0].id;
    let shell_in = workflow.graph.node(shell_id).expect("shell node").input_ports[0].id;
    workflow.graph.add_edge(approval_out, shell_in).expect("approval connects to consumer");

    let cfg = test_config(&temp_dir);
    let store_path = crate::config::store_path(&cfg.data_dir);
    let store = crate::store::Store::new(&store_path).expect("store opens");
    store.save_workflow(&workflow).expect("workflow saves");
    let run_id = crate::engine::executor::RunId(1);
    let summary = crate::headless::run_saved_workflow_with_app_config(&store, &workflow.meta.name, run_id, &cfg)
        .await
        .expect("workflow suspends at approval gate");
    assert_eq!(summary.status, crate::headless::HeadlessRunStatus::Suspended);
    assert!(!effect_path.exists());
    let initial_request =
        store.load_workflow_approval_for_run(run_id).expect("load approval").expect("approval exists");
    drop(store);

    let mut app = State::new(cfg).with_authority_provider(Arc::new(operator_authority(&workflow, temp_dir.path())));
    app.open_approvals();
    assert!(app.decide_selected_approval(&approval_decision_command(&initial_request)));
    let approved_request = app
        .store
        .as_ref()
        .expect("store open")
        .load_workflow_approval(&initial_request.id)
        .expect("reload approved request")
        .expect("approved request exists");
    assert!(app.resume_selected_approval(&approval_resume_command(&approved_request)).await);

    assert_eq!(std::fs::read_to_string(&effect_path).expect("effect file reads"), "delivered");
    let store = app.store.as_ref().expect("store open");
    let run = store.load_run(run_id).expect("load run").expect("run exists");
    assert!(run.finished_epoch_ms.is_some());
    assert_eq!(run.failures, 0);
    let details = store.load_run_details(run_id).expect("load details").expect("details exist");
    assert_eq!(details.nodes.len(), 2);
    assert!(details.nodes.iter().all(|node| node.status == crate::store::RunNodeDetailStatus::Completed));
}

// r[verify lattice.credentials.animus_secret_providers.tui]
#[cfg(all(unix, feature = "animus-secretspec"))]
#[tokio::test]
async fn tui_resume_selected_approval_delivers_and_redacts_secretspec_credential() {
    use std::os::unix::fs::PermissionsExt;

    use crate::credentials::config::CredentialProviderConfig;
    use crate::credentials::config::DeclarationMapping;
    use crate::credentials::config::HostInputConfig;
    use crate::credentials::config::ProviderScheme;
    use crate::credentials::config::RequestBounds;
    use crate::credentials::config::SecretSpecHostConfig;
    use crate::credentials::config::SecretSpecProviderConfig;

    const SENTINEL: &str = "hostile-secret-provider-value-not-public";

    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let manifest_path = temp_dir.path().join("secretspec.toml");
    let manifest_bytes = b"[project]\nname = 'lattice-tui-approval-resume'\nrevision = '1.0'\n";
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
            "provider": "systemd_credential",
            "provider_selector": "systemd-credential://",
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

    let creds_dir = temp_dir.path().join("systemd-creds");
    std::fs::create_dir_all(&creds_dir).expect("systemd creds dir creates");
    std::fs::write(creds_dir.join("API_TOKEN"), SENTINEL).expect("credential file writes");

    let worker_path = temp_dir.path().join("secretspec-worker.sh");
    std::fs::write(
        &worker_path,
        format!(
            "#!/bin/sh\nIFS= read -r _request || true\n[ -f \"$CREDENTIALS_DIRECTORY/API_TOKEN\" ] || exit 4\nprintf 'ANIMUS_SECRET_V1\\0\\050\\0\\0\\0\\0\\0\\0\\0{SENTINEL}'\n"
        ),
    )
    .expect("worker script writes");
    let mut perms = std::fs::metadata(&worker_path).expect("worker metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&worker_path, perms).expect("worker executable");

    let effect_path = temp_dir.path().join("secretspec-approval-delivered.txt");
    let mut workflow =
        Workflow::new("tui-approval-secretspec-resume", crate::engine::workflow::WorkflowTimestamp::epoch());
    let approval_id = workflow.add_approval_node("approval", approval_gate_config("gate-secretspec"));
    let shell_id = workflow.add_shell_node(
        "secretspec-consumer",
        format!(
            "printf '%s' \"$API_TOKEN\" && printf '%s' \"$API_TOKEN\" >&2 && printf '%s' \"$API_TOKEN\" > '{}'",
            effect_path.display()
        ),
    );
    workflow.set_node_credentials(shell_id, NodeCredentialRefs {
        env: [("API_TOKEN".to_string(), SecretRef::new("reviewed-secret", "token", None))].into(),
    });
    let approval_out = workflow.graph.node(approval_id).expect("approval node").output_ports[0].id;
    let shell_in = workflow.graph.node(shell_id).expect("shell node").input_ports[0].id;
    workflow.graph.add_edge(approval_out, shell_in).expect("approval connects to consumer");

    let _authority = operator_authority(&workflow, temp_dir.path());
    let mut cfg = test_config(&temp_dir);
    cfg.effect_authority = Some(temp_dir.path().join("authority.ncl"));
    cfg.credential_providers = vec![CredentialProviderConfig::AnimusSecretspec(SecretSpecProviderConfig {
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
            scheme: ProviderScheme::SystemdCredential,
        }],
        allowed_schemes: vec![ProviderScheme::SystemdCredential],
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
    })];
    cfg.credential_hosts = vec![SecretSpecHostConfig {
        provider_id: "reviewed-secret".to_string(),
        policy_path: policy_path.clone(),
        manifest_path,
        worker_executable: worker_path,
        working_directory: temp_dir.path().to_path_buf(),
        input: HostInputConfig::SystemdCredential { directory: creds_dir },
    }];

    let store_path = crate::config::store_path(&cfg.data_dir);
    let store = crate::store::Store::new(&store_path).expect("store opens");
    store.save_workflow(&workflow).expect("workflow saves");
    let run_id_1 = crate::engine::executor::RunId(1);
    let summary_1 = crate::headless::run_saved_workflow_with_app_config(&store, &workflow.meta.name, run_id_1, &cfg)
        .await
        .expect("run 1 suspends at approval gate");
    assert_eq!(summary_1.status, crate::headless::HeadlessRunStatus::Suspended);
    assert!(!effect_path.exists());
    let req_1 = store.load_workflow_approval_for_run(run_id_1).expect("load approval").expect("approval exists");

    let run_id_2 = crate::engine::executor::RunId(2);
    let summary_2 = crate::headless::run_saved_workflow_with_app_config(&store, &workflow.meta.name, run_id_2, &cfg)
        .await
        .expect("run 2 suspends at approval gate");
    assert_eq!(summary_2.status, crate::headless::HeadlessRunStatus::Suspended);
    let req_2 = store.load_workflow_approval_for_run(run_id_2).expect("load approval 2").expect("approval 2 exists");
    drop(store);

    let mut app = State::new(cfg.clone());
    app.open_approvals();
    app.approvals.selected = app
        .approvals
        .records
        .iter()
        .position(|rec| rec.request_id == req_1.id.as_str())
        .expect("run 1 approval present");
    assert!(app.decide_selected_approval(&approval_decision_command(&req_1)));
    app.approvals.selected = app
        .approvals
        .records
        .iter()
        .position(|rec| rec.request_id == req_1.id.as_str())
        .expect("run 1 approved record present");
    let approved_1 = app
        .store
        .as_ref()
        .expect("store open")
        .load_workflow_approval(&req_1.id)
        .expect("reload approved 1")
        .expect("approved 1 exists");
    assert!(app.resume_selected_approval(&approval_resume_command(&approved_1)).await);

    assert_eq!(std::fs::read_to_string(&effect_path).expect("effect file reads"), SENTINEL);
    let run_1 = app
        .store
        .as_ref()
        .expect("store open")
        .load_run(run_id_1)
        .expect("load run 1")
        .expect("run 1 exists");
    assert!(run_1.finished_epoch_ms.is_some());
    assert_eq!(run_1.failures, 0);
    let details_1 = app
        .store
        .as_ref()
        .expect("store open")
        .load_run_details(run_id_1)
        .expect("load details 1")
        .expect("details 1 exist");
    assert_eq!(details_1.nodes.len(), 2);
    assert!(details_1.nodes.iter().all(|node| node.status == crate::store::RunNodeDetailStatus::Completed));
    let consumer_detail = details_1.nodes.iter().find(|node| node.node_id == shell_id).expect("consumer detail");
    assert!(
        consumer_detail
            .outputs
            .iter()
            .any(|output| output.port_label == "stdout" && output.text == "<redacted>")
    );
    assert!(
        consumer_detail
            .outputs
            .iter()
            .any(|output| output.port_label == "stderr" && output.text == "<redacted>")
    );
    assert!(
        consumer_detail
            .outputs
            .iter()
            .filter(|output| output.port_label == "stdout" || output.port_label == "stderr")
            .all(|output| output.text == "<redacted>")
    );
    let replay_1 = app.store.as_ref().expect("store open").load_run_replay_events(run_id_1).expect("load replay 1");
    let raw_redb = std::fs::read(&store_path).expect("read raw redb");
    let raw_redb_text = String::from_utf8_lossy(&raw_redb);
    for surface in [
        output_text_lines(&app).join("\n"),
        serde_json::to_string(&details_1).expect("serialize details"),
        serde_json::to_string(&replay_1).expect("serialize replay"),
        raw_redb_text.into_owned(),
    ] {
        assert!(!surface.contains(SENTINEL), "sentinel leaked into surface");
    }

    // Fail-closed: deleting host policy before resuming Run 2 must fail resume without executing effect
    // or leaking.
    drop(app);
    std::fs::remove_file(&effect_path).expect("remove effect file before run 2");
    let mut app_2 = State::new(cfg);
    app_2.open_approvals();
    app_2.approvals.selected = app_2
        .approvals
        .records
        .iter()
        .position(|rec| rec.request_id == req_2.id.as_str())
        .expect("run 2 approval present");
    assert!(app_2.decide_selected_approval(&approval_decision_command(&req_2)));
    app_2.approvals.selected = app_2
        .approvals
        .records
        .iter()
        .position(|rec| rec.request_id == req_2.id.as_str())
        .expect("run 2 approved record present");
    let approved_2 = app_2
        .store
        .as_ref()
        .expect("store open")
        .load_workflow_approval(&req_2.id)
        .expect("reload approved 2")
        .expect("approved 2 exists");
    std::fs::remove_file(&policy_path).expect("remove policy before resume 2");
    assert!(!app_2.resume_selected_approval(&approval_resume_command(&approved_2)).await);
    assert!(!effect_path.exists());
    assert!(!output_text_lines(&app_2).join("\n").contains(SENTINEL));
}
