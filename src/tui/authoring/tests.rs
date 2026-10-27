use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

use super::catalog;
use super::test_support::send;
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::NodeId;
use crate::engine::workflow::WorkflowTimestamp;
use crate::tui::App;
use crate::tui::app::AppAction;
use crate::tui::graph;
use crate::tui::input;
use crate::tui::panels;
use crate::workflow_format::load_workflow_nickel_file;
use crate::workflow_format::workflow_to_nickel;
use crate::workflow_format::workflow_to_portable;

fn app(root: &tempfile::TempDir) -> App {
    App::new(crate::config::AppConfig {
        data_dir: root.path().join("state"),
        ..Default::default()
    })
}
fn text(app: &mut App, value: &str) {
    for character in value.chars() {
        if character == '\n' {
            send(app, KeyModifiers::CONTROL, KeyCode::Char('j'));
        } else {
            send(app, KeyModifiers::NONE, KeyCode::Char(character));
        }
    }
}
fn select(app: &mut App, node: Option<u64>) {
    app.authoring = None;
    app.graph_state.selected.clear();
    if let Some(node) = node {
        app.graph_state.selected.insert(graph::subwayrat_node_id(NodeId(node)));
    }
    app.open_authoring("all");
}
fn draft(app: &mut App, id: &str, value: &str) {
    let state = app.authoring.as_mut().expect("typed browser");
    state.selected = state
        .fields
        .iter()
        .position(|field| field.id == id)
        .unwrap_or_else(|| panic!("missing authoring action {id}"));
    send(app, KeyModifiers::NONE, KeyCode::Enter);
    send(app, KeyModifiers::CONTROL, KeyCode::Char('u'));
    text(app, value);
}
fn edit(app: &mut App, id: &str, value: &str) {
    draft(app, id, value);
    send(app, KeyModifiers::NONE, KeyCode::Enter);
    let state = app.authoring.as_ref().expect("recoverable browser");
    assert!(state.error.is_none(), "{id}={value}: {:?}", state.error);
    assert!(state.draft.is_none(), "{id} did not commit");
}
fn create(app: &mut App, kind: char, label: &str, value: &str) -> u64 {
    app.authoring = None;
    app.open_add_node_flow();
    send(app, KeyModifiers::NONE, KeyCode::Char(kind));
    for _ in 0..app.add_node_flow.label.chars().count() {
        send(app, KeyModifiers::NONE, KeyCode::Backspace);
    }
    text(app, label);
    send(app, KeyModifiers::NONE, KeyCode::Enter);
    for _ in 0..app.add_node_flow.command.chars().count() {
        send(app, KeyModifiers::NONE, KeyCode::Backspace);
    }
    text(app, value);
    send(app, KeyModifiers::NONE, KeyCode::Enter);
    assert!(!app.is_add_node_flow_open(), "creation did not commit {label}");
    app.graph_state.focused.expect("new node is focused").0
}

// r[verify lattice.tui_ux.portable_authoring_parity.node_kinds]
// r[verify lattice.tui_ux.portable_authoring_parity.round_trip.scenario.complete]
#[test]
fn all_current_node_kinds_and_field_families_author_store_and_nickel_round_trip() {
    let root = tempfile::tempdir().unwrap();
    let mut app = app(&root);
    let shell = create(&mut app, 's', "shell", "printf original");
    let http = create(&mut app, 'h', "http", "GET https://example.test");
    let file = create(&mut app, 'f', "file", "read data.txt");
    let image = create(&mut app, 'i', "image", "/tmp/input.png");
    let agent = create(&mut app, 'p', "agent", "original prompt");
    let condition = create(&mut app, 'd', "condition", "yes=always,no=exists");
    let merge = create(&mut app, 'm', "merge", "all:left,right");
    let subworkflow = create(&mut app, 'w', "subworkflow", "child");
    for (node, prefix, edits) in [
        (shell, "shell", vec![
            ("command", "printf authored"),
            ("cwd", "/tmp"),
            ("env.add", "SAMPLE=value"),
            ("timeout_secs", "99"),
            ("pipe_stdin", "false"),
        ]),
        (http, "http", vec![
            ("method", "POST"),
            ("url", "https://example.test/authored"),
            ("headers.add", "X-Author=typed"),
            ("body", "payload"),
            ("timeout_secs", "21"),
        ]),
        (file, "file", vec![
            ("path", "/tmp/authored.txt"),
            ("allowed_roots.add", "/tmp"),
            ("content", "authored content"),
            ("operation", "write"),
            ("overwrite", "true"),
        ]),
        (image, "image_input", vec![
            ("path", "/tmp/authored.png"),
            ("allowed_roots.add", "/var/tmp"),
            ("max_bytes", "1024"),
        ]),
        (agent, "agent", vec![
            ("target", "clankers"),
            ("prompt", "authored prompt"),
            ("model_hint", "typed-model"),
            ("output_mode", "json"),
            ("tool_policy", "read-only"),
            ("context_refs.add", "opaque:context"),
            ("cwd", "/tmp"),
            ("session", "typed-session"),
            ("prompt_file", "prompt.md"),
        ]),
        (condition, "condition", vec![
            ("branches.0.predicate", "gt:2"),
            ("branches.1.label", "otherwise"),
            ("branches.add", "fallback=always"),
            ("branches.2.move", "0"),
        ]),
        (merge, "merge", vec![("policy", "collect"), ("required_inputs.add", "third")]),
        (subworkflow, "subworkflow", vec![
            ("workflow_name", "authored-child"),
            ("input_mappings.add", "request=child-request"),
            ("output_mappings.add", "child-result=result"),
        ]),
    ] {
        select(&mut app, Some(node));
        for (id, value) in edits {
            edit(&mut app, &format!("config.{prefix}.{id}"), value);
        }
    }
    select(&mut app, None);
    edit(&mut app, "config.approval.add", "approval|review-gate|operator");
    let approval =
        *app.workflow.configs.iter().find(|(_, config)| matches!(config, NodeConfig::Approval(_))).unwrap().0;
    select(&mut app, Some(approval));
    for (id, value) in [
        ("safe_summary", "Review typed authoring"),
        ("selected_input_ports.add", "request"),
        ("authority.local_actors.add", "reviewer"),
        ("authority.prohibit_self_approval", "false"),
        ("expiry.decision_ttl_millis", "60001"),
        ("expiry.resume_ttl_millis", "60002"),
        ("expiry.approval_must_be_fresh_at_resume", "false"),
        ("reason.required", "true"),
        ("reason.identity_bound", "false"),
        ("reason.max_bytes", "512"),
        ("resume.values.add", "answer=json"),
        ("resume.credential_bindings.add", "token"),
        ("terminal.on_rejection", "cancel_workflow"),
        ("terminal.on_expiry", "fail_workflow"),
    ] {
        edit(&mut app, &format!("config.approval.{id}"), value);
    }
    edit(&mut app, "config.approval.required_artifacts.add", &"a".repeat(64));

    // All derived OperationPlan fields come from a real admitted catalog/instance
    // and executable observation, not hand-edited content identities.
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let catalog_path = manifest.join("builtins/operations/catalog.ncl");
    let catalog_source = crate::operation::format::load_catalog(&catalog_path).unwrap();
    let revision = &catalog_source.types["cairn.repository"].definition.revision;
    let instances_path = root.path().join("instances.ncl");
    std::fs::write(&instances_path, format!(r#"{{schema = "lattice.operation-instances.v1", instances = [{{ id = "authoring-repository", display_alias = "Authoring repository", type_id = "cairn.repository", type_revision = "{revision}", target = {{ repository_root = {:?} }}, tags = {{}}, method_defaults = {{}}, credentials = {{}}, }}], }}"#, root.path().to_string_lossy())).unwrap();
    let executable = std::fs::canonicalize(manifest.join("builtins/operations/cairn-read-only-fixture")).unwrap();
    app.open_authoring("operation");
    for (id, value) in [
        ("operation.catalog_path", catalog_path.to_string_lossy().into_owned()),
        ("operation.instances_path", instances_path.to_string_lossy().into_owned()),
        ("operation.executable_path", executable.to_string_lossy().into_owned()),
        ("operation.label", "operation".into()),
        ("operation.load", "load".into()),
        ("operation.instance", "authoring-repository".into()),
        ("operation.method", "change_list".into()),
        ("operation.commit", "admit".into()),
    ] {
        edit(&mut app, id, &value);
    }
    let operation = app
        .workflow
        .configs
        .values()
        .find_map(|config| {
            if let NodeConfig::Operation(plan) = config {
                Some(plan.as_ref())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(operation.instance_id, "authoring-repository");
    assert!(crate::operation::planning::validate_plan_identity(operation).is_ok());
    let original_plan = operation.identity.clone();
    let operation_node = app
        .workflow
        .configs
        .iter()
        .find_map(|(id, config)| matches!(config, NodeConfig::Operation(_)).then_some(*id))
        .unwrap();
    select(&mut app, Some(operation_node));
    for (id, value) in [
        ("operation.catalog_path", catalog_path.to_string_lossy().into_owned()),
        ("operation.instances_path", instances_path.to_string_lossy().into_owned()),
        ("operation.executable_path", executable.to_string_lossy().into_owned()),
        ("operation.label", "operation".into()),
        ("operation.load", "load".into()),
        ("operation.instance", "authoring-repository".into()),
        ("operation.method", "validate".into()),
        ("operation.commit", "admit".into()),
    ] {
        edit(&mut app, id, &value);
    }
    let NodeConfig::Operation(replacement) = &app.workflow.configs[&operation_node] else {
        panic!("operation replacement lost its owner")
    };
    assert_eq!(replacement.method, "validate");
    assert_ne!(replacement.identity, original_plan);
    assert!(crate::operation::planning::validate_plan_identity(replacement).is_ok());

    select(&mut app, Some(shell));
    for (id, value) in [
        ("workflow.name", "authored-portable"),
        ("workflow.description", "All typed authoring families\nNo effects"),
        ("workflow.created", "2026-01-02T03:04:05Z"),
        ("workflow.modified", "2026-02-03T04:05:06Z"),
        ("graph.label", "authored shell"),
        ("graph.x", "-7"),
        ("graph.y", "19"),
        ("graph.port.add", "auto|output|custom-json|json"),
        ("credential.env.add", "API_TOKEN|env|AUTHORED_TOKEN|"),
        ("runner.binding", "shell|default"),
        ("runner.effect_policy", "process"),
        ("runner.cache_policy", "disabled"),
        ("runner.retry_policy", "none"),
        ("runner.artifact_policy", "metadata-only"),
        ("trigger.add", "manual|manual"),
        ("trigger.add", "scheduled|schedule|60|0|run_once"),
        ("trigger.add", "webhook|webhook|POST|/authored"),
        ("trigger.0.enabled", "true"),
        ("trigger.1.schedule_interval_secs", "120"),
        ("trigger.1.schedule_next_run_epoch_ms", "1000"),
        ("trigger.1.schedule_missed_run_policy", "skip"),
        ("trigger.2.auth", "blake3_signature|env|HOOK_KEY||X-Signature"),
        ("trigger.2.webhook_content_types.add", "application/json"),
        ("trigger.2.input_bindings.add", "event|json_pointer|/event|json|true"),
        ("trigger.2.overlap_policy", "allow"),
        ("trigger.2.idempotency_policy", "payload_json_pointer|/id"),
    ] {
        edit(&mut app, id, value);
    }
    let output = app
        .workflow
        .graph
        .node(NodeId(shell))
        .unwrap()
        .output_ports
        .iter()
        .find(|port| port.label == "custom-json")
        .unwrap()
        .id;
    select(&mut app, Some(http));
    edit(&mut app, "graph.port.add", "auto|input|request-json|json");
    let input = app
        .workflow
        .graph
        .node(NodeId(http))
        .unwrap()
        .input_ports
        .iter()
        .find(|port| port.label == "request-json")
        .unwrap()
        .id;
    edit(&mut app, "graph.edge.add", &format!("{}|{}", output.0, input.0));
    app.authoring = None;

    let expected = workflow_to_portable(&app.workflow);
    assert_eq!(expected.meta.name, "authored-portable");
    assert_eq!(expected.meta.modified, "2026-02-03T04:05:06+00:00");
    assert_eq!(
        expected
            .configs
            .iter()
            .map(|config| config.kind.as_str())
            .collect::<std::collections::BTreeSet<_>>(),
        [
            "shell",
            "http",
            "file",
            "image_input",
            "agent",
            "condition",
            "merge",
            "subworkflow",
            "approval",
            "operation"
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(app.workflow.graph.node(NodeId(shell)).unwrap().x, -7);
    assert_eq!(app.workflow.graph.node(NodeId(shell)).unwrap().y, 19);
    assert_eq!(app.workflow.credentials[&shell].env["API_TOKEN"].key, "AUTHORED_TOKEN");
    assert_eq!(app.workflow.triggers.len(), 3);
    let diagnostics = crate::workflow_validation::validate_workflow(&app.workflow);
    assert!(!crate::workflow_validation::has_fatal_diagnostics(&diagnostics), "{diagnostics:?}");
    catalog::check(&catalog::capabilities(), &serde_json::to_value(&expected).unwrap()).unwrap();
    app.save_workflow();
    assert!(!app.has_unsaved_changes());
    let stored = app.store.as_ref().unwrap().load_workflow("authored-portable").unwrap().unwrap();
    assert_eq!(workflow_to_portable(&stored), expected);
    std::fs::write(root.path().join("prompt.md"), "authored prompt").unwrap();
    let source = root.path().join("authored.ncl");
    std::fs::write(&source, workflow_to_nickel(&stored)).unwrap();
    let imported = load_workflow_nickel_file(&source).unwrap();
    assert_eq!(workflow_to_portable(&imported), expected);
    assert!(app.store.as_ref().unwrap().list_runs(10).unwrap().is_empty());
}

// r[verify lattice.tui_ux.portable_authoring_parity.catalog.scenario.missing]
#[test]
fn capability_matrix_rejects_missing_display_only_duplicate_stale_and_new_portable_fields() {
    let mut workflow = crate::engine::Workflow::new("catalog", WorkflowTimestamp::epoch());
    workflow.add_shell_node("shell", "true");
    let portable = serde_json::to_value(workflow_to_portable(&workflow)).unwrap();
    let rows = catalog::capabilities();
    catalog::check(&rows, &portable).unwrap();
    let mut omitted = rows.clone();
    omitted.retain(|row| row.id != "config.shell.command");
    assert!(
        catalog::check(&omitted, &portable)
            .unwrap_err()
            .iter()
            .any(|error| error.contains("config.shell.command"))
    );
    let mut display = rows.clone();
    display.iter_mut().find(|row| row.id == "config.shell.command").unwrap().mode = catalog::Mode::DisplayOnly;
    assert!(catalog::check(&display, &portable).unwrap_err().iter().any(|error| error.contains("display-only")));
    let mut duplicate = rows.clone();
    duplicate.push(rows[0].clone());
    assert!(catalog::check(&duplicate, &portable).unwrap_err().iter().any(|error| error.contains("duplicate")));
    let mut stale = rows.clone();
    stale[0].id = "removed.schema.field".into();
    assert!(
        catalog::check(&stale, &portable)
            .unwrap_err()
            .iter()
            .any(|error| error.contains("unknown capability"))
    );
    let mut expanded = portable;
    expanded["configs"][0]["new_portable_field"] = serde_json::json!(true);
    assert!(
        catalog::check(&rows, &expanded)
            .unwrap_err()
            .iter()
            .any(|error| error.contains("configs[].new_portable_field"))
    );
}

// r[verify lattice.tui_ux.portable_authoring_parity.round_trip.scenario.invalid]
// r[verify lattice.tui_ux.portable_authoring_parity.discoverability]
#[test]
fn invalid_cancelled_stale_drafts_and_save_rejection_preserve_workflow_and_store_bytes() {
    let root = tempfile::tempdir().unwrap();
    let mut app = app(&root);
    let shell = create(&mut app, 's', "shell", "printf no-effect");
    app.workflow.meta.name = "prior".into();
    app.save_workflow();
    let prior = serde_json::to_value(&app.workflow).unwrap();
    let saved_portable = workflow_to_portable(&app.workflow);
    let store_path = crate::config::store_path(&root.path().join("state"));
    let prior_store = std::fs::read(&store_path).unwrap();
    select(&mut app, Some(shell));
    for (id, value) in [
        ("config.shell.timeout_secs", "18446744073709551616"),
        ("credential.env.add", "TOKEN|env||"),
        ("runner.retry_policy", "unbounded"),
        ("graph.port.add", "0|output|duplicate|string"),
        ("trigger.add", "bad|schedule|0|0|skip"),
    ] {
        draft(&mut app, id, value);
        send(&mut app, KeyModifiers::NONE, KeyCode::Enter);
        assert!(app.authoring.as_ref().unwrap().error.is_some(), "{id} should reject");
        assert_eq!(serde_json::to_value(&app.workflow).unwrap(), prior);
        assert_eq!(std::fs::read(&store_path).unwrap(), prior_store);
        for (width, height) in [(32, 6), (80, 24), (12, 4)] {
            let area = ratatui::layout::Rect::new(0, 0, width, height);
            let mut buffer = ratatui::buffer::Buffer::empty(area);
            panels::render(&mut app, area, &mut buffer);
            let surface = buffer.content().iter().map(|cell| cell.symbol()).collect::<String>();
            assert!(surface.contains("Esc"), "recovery action hidden at {width}x{height}: {surface}");
        }
        send(&mut app, KeyModifiers::NONE, KeyCode::Esc);
    }
    draft(&mut app, "config.shell.command", "discard this command");
    send(&mut app, KeyModifiers::NONE, KeyCode::Esc);
    assert_eq!(serde_json::to_value(&app.workflow).unwrap(), prior);
    draft(&mut app, "config.shell.command", "stale command");
    app.workflow.meta.description = "concurrent edit".into();
    let concurrent = serde_json::to_value(&app.workflow).unwrap();
    send(&mut app, KeyModifiers::NONE, KeyCode::Enter);
    assert!(app.authoring.as_ref().unwrap().error.as_deref().unwrap().contains("changed"));
    assert_eq!(serde_json::to_value(&app.workflow).unwrap(), concurrent);
    send(&mut app, KeyModifiers::NONE, KeyCode::Esc);
    send(&mut app, KeyModifiers::NONE, KeyCode::Esc);
    app.workflow
        .runner
        .insert(u64::MAX, crate::engine::workflow::NodeRunnerSettings::new("shell", "default"));
    app.mark_workflow_dirty();
    app.save_workflow();
    assert!(app.has_unsaved_changes());
    assert_eq!(std::fs::read(&store_path).unwrap(), prior_store);
    let malformed = root.path().join("malformed.ncl");
    std::fs::write(&malformed, "{ not valid Nickel").unwrap();
    assert!(load_workflow_nickel_file(&malformed).is_err());
    assert_eq!(std::fs::read(&store_path).unwrap(), prior_store);
    let unavailable = app.store.take();
    app.save_workflow();
    assert!(app.has_unsaved_changes());
    assert_eq!(
        workflow_to_portable(&unavailable.as_ref().unwrap().load_workflow("prior").unwrap().unwrap()),
        saved_portable
    );
}

// r[verify lattice.tui_ux.portable_authoring_parity.node_kinds.scenario.cancel]
#[test]
fn every_guided_kind_rejects_invalid_or_cancelled_drafts_without_dirtying_prior_state() {
    let root = tempfile::tempdir().unwrap();
    let mut app = app(&root);
    for (kind, value) in [
        ('s', ""),
        ('h', "CONNECT invalid"),
        ('f', "read ../escape"),
        ('i', "../escape.png"),
        ('p', ""),
        ('c', ""),
        ('d', "duplicate=always,duplicate=exists"),
        ('m', "unknown:left,right"),
        ('w', ""),
    ] {
        let before = serde_json::to_value(&app.workflow).unwrap();
        let dirty = app.has_unsaved_changes();
        app.open_add_node_flow();
        send(&mut app, KeyModifiers::NONE, KeyCode::Char(kind));
        send(&mut app, KeyModifiers::NONE, KeyCode::Enter);
        for _ in 0..app.add_node_flow.command.chars().count() {
            send(&mut app, KeyModifiers::NONE, KeyCode::Backspace);
        }
        text(&mut app, value);
        send(&mut app, KeyModifiers::NONE, KeyCode::Enter);
        assert!(app.is_add_node_flow_open(), "invalid kind {kind} committed");
        assert_eq!(serde_json::to_value(&app.workflow).unwrap(), before);
        assert_eq!(app.has_unsaved_changes(), dirty);
        send(&mut app, KeyModifiers::NONE, KeyCode::Esc);
        assert_eq!(serde_json::to_value(&app.workflow).unwrap(), before);
        assert_eq!(app.has_unsaved_changes(), dirty);
    }
    assert_eq!(
        input::handle_event(&mut app, &Event::Key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL))),
        Some(AppAction::OpenAuthoring("all"))
    );
}
