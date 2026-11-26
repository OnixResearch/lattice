//! Source revisions are input snapshots, not closure, logical-equivalence or execution receipts.
use std::collections::BTreeMap;
use std::collections::HashMap;

use lattice::authoring::AssociationState;
use lattice::authoring::CapturedProgram;
use lattice::authoring::ProgramKind;
use lattice::authoring::SourceDraft;
use lattice::authoring::shell;
use lattice::authoring::{self};
use lattice::engine::workflow::Workflow;
use lattice::engine::workflow::WorkflowTimestamp;
use lattice::store::Store;
use lattice::workflow_format::workflow_to_nickel;
use lattice::workflow_format::workflow_to_portable;

fn lattice_binary() -> std::path::PathBuf {
    std::env::var_os("LATTICE_QUALIFIED_BINARY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_BIN_EXE_lattice")))
}

fn flat(name: &str, two: bool) -> Workflow {
    let mut workflow = Workflow::new(name, WorkflowTimestamp::epoch());
    let first = workflow.add_shell_node("first", "printf must-not-run");
    if two {
        let second = workflow.add_shell_node("second", "printf must-not-run");
        let output = workflow.graph.node(first).unwrap().output_ports[0].id;
        let input = workflow.graph.node(second).unwrap().input_ports[0].id;
        workflow.graph.add_edge(output, input).unwrap();
    }
    workflow
}

fn program() -> CapturedProgram {
    let one = nickel_eval::to_nickel(&workflow_to_portable(&flat("source-test", false))).unwrap();
    let two = nickel_eval::to_nickel(&workflow_to_portable(&flat("source-test", true))).unwrap();
    let source = format!(
        r#"
let selected = fun two => if two then ({two}) else ({one}) in
let helper = fun prefix node => {{
  id = node.id, label = prefix ++ node.label, x = node.x, y = node.y,
  input_ports = node.input_ports, output_ports = node.output_ports,
}} in
{{
  parameters = {{ two = true, first = "one-", second = "two-" }},
  build = fun p => let workflow = selected p.two in {{
    meta = workflow.meta,
    nodes = std.array.map (fun node => helper (if node.id == 0 then p.first else p.second) node) workflow.nodes,
    edges = workflow.edges, configs = workflow.configs, triggers = workflow.triggers,
  }},
}}
"#
    );
    CapturedProgram {
        entry: "workflow.ncl".into(),
        files: BTreeMap::from([("workflow.ncl".into(), source.into_bytes())]),
        kind: ProgramKind::Parameterized,
        parameters: BTreeMap::new(),
    }
}

fn draft(store: &Store, id: &str) -> SourceDraft {
    let current = store.load_workflow("source-test").unwrap();
    SourceDraft {
        id: id.into(),
        workflow_name: "source-test".into(),
        expected_revision: current.as_ref().and_then(|workflow| workflow.catalog_revision.clone()),
        program: current
            .and_then(|workflow| workflow.authoring.map(|source| std::sync::Arc::unwrap_or_clone(source.program)))
            .unwrap_or_else(program),
        diagnostic: None,
        preview: None,
    }
}

fn admitted(store: &Store) -> Workflow {
    let mut draft = draft(store, "initial");
    store.preview_source_draft(&mut draft).unwrap();
    store.admit_source_draft(&draft).unwrap()
}

// r[verify lattice.source_authoring.capture]
// r[verify lattice.source_authoring.capture.reopen]
#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_import_store_reopen_and_exact_source_archive_preserve_helpers_and_parameters() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("state.redb");
    let source = program();
    let before;
    {
        let store = Store::new(&path).unwrap();
        before = admitted(&store);
        assert_eq!(before.graph.node_count(), 2);
        assert_eq!(before.authoring.as_ref().unwrap().program.as_ref(), &source);
    }
    let store = Store::new(&path).unwrap();
    let reopened = store.load_workflow("source-test").unwrap().unwrap();
    assert_eq!(reopened.authoring, before.authoring);
    assert_eq!(reopened.catalog_revision, before.catalog_revision);
    let archive = shell::default_export(&reopened, false).unwrap();
    let archive_path = directory.path().join("source.author.json");
    std::fs::write(&archive_path, &archive).unwrap();
    assert_eq!(shell::read_source_archive(&archive_path).unwrap(), source);
    assert!(shell::validate_export_path(&reopened, false, &directory.path().join("misleading.ncl")).is_err());
    assert!(shell::validate_export_path(&reopened, false, &archive_path).is_ok());
    let expanded = shell::default_export(&reopened, true).unwrap();
    assert!(std::str::from_utf8(&expanded).unwrap().starts_with("# Artifact: lattice.expanded-workflow.v1"));
    assert_ne!(archive, expanded);
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_parameter_preview_is_inspectable_and_changes_only_source_defined_instance() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let original = admitted(&store);
    let mut changed = draft(&store, "parameters");
    changed.program.parameters.insert("first".into(), serde_json::json!("changed-"));
    store.preview_source_draft(&mut changed).unwrap();
    let candidate = &changed.preview.as_ref().unwrap().candidate;
    let before = workflow_to_portable(&original);
    let after = workflow_to_portable(candidate);
    assert_ne!(before.nodes[0].label, after.nodes[0].label);
    assert_eq!(before.nodes[1].label, after.nodes[1].label);
    assert_eq!(store.load_workflow("source-test").unwrap().unwrap().authoring, original.authoring);
    changed.program.parameters.insert("two".into(), serde_json::json!(false));
    assert!(store.admit_source_draft(&changed).is_err(), "mutated input cannot reuse old preview");
    store.preview_source_draft(&mut changed).unwrap();
    assert_eq!(changed.preview.as_ref().unwrap().candidate.graph.node_count(), 1);
    assert_eq!(store.admit_source_draft(&changed).unwrap().graph.node_count(), 1);
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_template_source_edit_recompiles_all_source_defined_instances() {
    let mut source = program();
    let before = shell::compile(&source).unwrap();
    let entry = source.files.get_mut("workflow.ncl").unwrap();
    *entry = String::from_utf8(entry.clone())
        .unwrap()
        .replace("label = prefix ++ node.label", "label = \"shared-\" ++ prefix ++ node.label")
        .into_bytes();
    let after = shell::compile(&source).unwrap();
    for (before, after) in workflow_to_portable(&before).nodes.iter().zip(workflow_to_portable(&after).nodes.iter()) {
        assert_eq!(after.label, format!("shared-{}", before.label));
    }
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_failed_draft_is_bounded_editable_and_does_not_replace_admission() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("state.redb");
    let before;
    {
        let store = Store::new(&path).unwrap();
        before = admitted(&store);
        let mut failed = draft(&store, "failed");
        failed.program.files.insert("workflow.ncl".into(), b"let private_secret_source =".to_vec());
        assert!(store.preview_source_draft(&mut failed).is_err());
        assert!(failed.preview.is_none());
        assert!(failed.diagnostic.as_ref().unwrap().len() <= 128);
        assert!(!failed.diagnostic.as_ref().unwrap().contains("private_secret"));
        assert_eq!(store.load_workflow("source-test").unwrap().unwrap().catalog_revision, before.catalog_revision);
    }
    let reopened = Store::new(&path).unwrap();
    let failed = reopened.load_source_draft("failed").unwrap().unwrap();
    assert_eq!(failed.program.files["workflow.ncl"], b"let private_secret_source =");
    assert!(failed.diagnostic.is_some());
    assert_eq!(reopened.load_workflow("source-test").unwrap().unwrap().authoring, before.authoring);
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_undeclared_parameters_and_drifted_compiler_are_not_admitted() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let before = admitted(&store);
    let mut candidate = draft(&store, "undeclared");
    candidate.program.parameters.insert("inferred_instance_parameter".into(), serde_json::json!(7));
    assert!(store.preview_source_draft(&mut candidate).is_err());
    candidate.program.parameters.clear();
    store.preview_source_draft(&mut candidate).unwrap();
    candidate.preview.as_mut().unwrap().compiler.profile = "not-selected".into();
    store.save_source_draft(&candidate).unwrap();
    assert!(store.admit_source_draft(&candidate).is_err());
    assert!(store.load_source_draft("undeclared").unwrap().unwrap().diagnostic.is_some());
    assert_eq!(store.load_workflow("source-test").unwrap().unwrap().catalog_revision, before.catalog_revision);
}

// r[verify lattice.source_authoring.admission]
// r[verify lattice.source_authoring.admission.concurrent]
#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_stale_and_concurrent_drafts_cannot_replace_newer_revision() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    admitted(&store);
    let mut left = draft(&store, "left");
    let mut right = draft(&store, "right");
    left.program.parameters.insert("first".into(), serde_json::json!("left-"));
    right.program.parameters.insert("first".into(), serde_json::json!("right-"));
    store.preview_source_draft(&mut left).unwrap();
    store.preview_source_draft(&mut right).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let handles = [left, right]
        .into_iter()
        .map(|draft| {
            let store = store.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                store.admit_source_draft(&draft).is_ok()
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(handles.into_iter().map(|handle| handle.join().unwrap()).filter(|success| *success).count(), 1);
    let revisions = [
        store.load_source_draft("left").unwrap().unwrap(),
        store.load_source_draft("right").unwrap().unwrap(),
    ];
    assert_eq!(revisions.iter().filter(|draft| draft.diagnostic.is_some()).count(), 1);
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_durable_preview_replacement_and_candidate_mutation_are_stale() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    admitted(&store);
    let mut old = draft(&store, "same-draft");
    store.preview_source_draft(&mut old).unwrap();
    let mut replacement = old.clone();
    replacement.program.parameters.insert("first".into(), serde_json::json!("new-"));
    store.preview_source_draft(&mut replacement).unwrap();
    assert!(store.admit_source_draft(&old).is_err());
    assert!(
        store.load_source_draft("same-draft").unwrap().unwrap().diagnostic.is_none(),
        "stale actor cannot overwrite newer draft diagnostics"
    );
    replacement.preview.as_mut().unwrap().candidate.add_shell_node("not-previewed", "false");
    store.save_source_draft(&replacement).unwrap();
    assert!(store.admit_source_draft(&replacement).is_err());
}

// r[verify lattice.source_authoring.fork]
// r[verify lattice.source_authoring.fork.snapshot]
#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_materialized_edit_paths_fork_and_old_snapshot_retains_exact_association() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let old_snapshot = admitted(&store);
    let old_source = old_snapshot.authoring.clone();
    let mut typed = old_snapshot.clone();
    lattice::tui::authoring::apply(&mut typed, Some(0), "graph.label", "materialized").unwrap();
    assert_eq!(typed.authoring.as_ref().unwrap().state, AssociationState::MaterializedFork);
    assert_eq!(typed.authoring.as_ref().unwrap().source_revision, old_source.as_ref().unwrap().source_revision);
    let saved = store.save_workflow_cas(&typed, typed.catalog_revision.as_ref()).unwrap();
    assert_ne!(saved.catalog_revision, old_snapshot.catalog_revision);
    assert_eq!(old_snapshot.authoring, old_source);
    assert_eq!(workflow_to_portable(&old_snapshot).nodes[0].label, "one-first");
    // Saving a direct public-field mutation uses the same rule, even without a UI hook.
    let mut raw = saved.clone();
    raw.meta.description = "raw mutation".into();
    store.save_workflow(&raw).unwrap();
    assert_eq!(
        store.load_workflow("source-test").unwrap().unwrap().authoring.unwrap().state,
        AssociationState::MaterializedFork
    );
    assert!(store.save_workflow(&old_snapshot).is_err());
    assert_eq!(shell::source_export(&saved).unwrap(), shell::source_export(&typed).unwrap());
}

#[test]
fn authoring_legacy_and_explicit_expanded_roundtrips_have_no_fabricated_source() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let legacy = flat("legacy", true);
    let mut json = serde_json::to_value(&legacy).unwrap();
    json.as_object_mut().unwrap().remove("authoring");
    json.as_object_mut().unwrap().remove("catalog_revision");
    let legacy: Workflow = serde_json::from_value(json).unwrap();
    store.save_workflow(&legacy).unwrap();
    let reopened = store.load_workflow("legacy").unwrap().unwrap();
    assert!(reopened.authoring.is_none());
    assert!(shell::source_export(&reopened).is_err());
    let expanded = shell::default_export(&reopened, false).unwrap();
    let path = directory.path().join("expanded.ncl");
    std::fs::write(&path, &expanded).unwrap();
    let imported = lattice::workflow_format::load_workflow_nickel_file(&path).unwrap();
    assert_eq!(workflow_to_portable(&imported), workflow_to_portable(&legacy));
    assert_eq!(imported.authoring.as_ref().unwrap().state, AssociationState::RetainedEntrySnapshot);
}

#[test]
fn saved_six_field_v1_association_survives_reopen_export_and_cas_save() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("legacy.redb");
    let input = directory.path().join("legacy.ncl");
    std::fs::write(&input, workflow_to_nickel(&flat("legacy-source", true))).unwrap();
    let mut legacy = lattice::workflow_format::load_workflow_nickel_file(&input).unwrap();
    // Reproduce the published writer's six-field identity, independently of the
    // current expanded_identity implementation. Persist its old JSON shape
    // directly, so no current save/reconciliation can repair the fixture first.
    let prior_identity = authoring::identity(
        "lattice.materialized-workflow.v1",
        &(&legacy.meta, &legacy.graph, &legacy.configs, &legacy.credentials, &legacy.runner, &legacy.triggers),
    )
    .unwrap();
    legacy.authoring.as_mut().unwrap().expanded_artifact = prior_identity.clone();
    let expected = legacy.authoring.clone();
    let mut record = serde_json::to_value(&legacy).unwrap();
    for field in [
        "control_dependencies",
        "construction_profile",
        "construction_runtime",
        "catalog_revision",
    ] {
        record.as_object_mut().unwrap().remove(field);
    }
    let association = record["authoring"].as_object_mut().unwrap();
    association.remove("observation");
    association.remove("construction");
    let bytes = serde_json::to_vec(&record).unwrap();
    drop(Store::new(&path).unwrap());
    {
        let database = redb::Database::create(&path).unwrap();
        let transaction = database.begin_write().unwrap();
        {
            let workflows: redb::TableDefinition<&str, &[u8]> = redb::TableDefinition::new("workflows");
            transaction.open_table(workflows).unwrap().insert("legacy-source", bytes.as_slice()).unwrap();
        }
        transaction.commit().unwrap();
    }
    {
        let store = Store::new(&path).unwrap();
        let reopened = store.load_workflow("legacy-source").unwrap().unwrap();
        assert_eq!(reopened.authoring, expected);
        assert_eq!(authoring::expanded_identity(&reopened).unwrap(), prior_identity);
        let archive: serde_json::Value =
            serde_json::from_slice(&shell::default_export(&reopened, false).unwrap()).unwrap();
        assert_eq!(archive["association"], serde_json::to_value(expected.as_ref().unwrap()).unwrap());
        let saved = store.save_workflow_cas(&reopened, reopened.catalog_revision.as_ref()).unwrap();
        assert_eq!(saved.authoring, expected);
        assert_ne!(saved.catalog_revision, reopened.catalog_revision);
    }
    let store = Store::new(&path).unwrap();
    let reopened = store.load_workflow("legacy-source").unwrap().unwrap();
    assert_eq!(reopened.authoring, expected);
    for graph_edit in [true, false] {
        let mut edited = reopened.clone();
        if graph_edit {
            edited.graph.node_mut(lattice::engine::workflow::NodeId(0)).unwrap().label = "changed label".into();
        } else {
            edited
                .configs
                .insert(0, lattice::engine::workflow::NodeConfig::shell("printf changed-but-never-run"));
        }
        let archive: serde_json::Value =
            serde_json::from_slice(&shell::default_export(&edited, false).unwrap()).unwrap();
        assert_eq!(archive["association"]["state"], "MaterializedFork");
        authoring::reconcile_materialized(&mut edited).unwrap();
        assert_eq!(edited.authoring.as_ref().unwrap().state, AssociationState::MaterializedFork);
        assert_eq!(edited.authoring.as_ref().unwrap().expanded_artifact, prior_identity);
    }
    let mut mismatched = reopened;
    mismatched.authoring.as_mut().unwrap().expanded_artifact =
        authoring::ContentIdentity::from_bytes(b"wrong v1 identity");
    authoring::reconcile_materialized(&mut mismatched).unwrap();
    assert_eq!(mismatched.authoring.unwrap().state, AssociationState::MaterializedFork);
}

#[test]
fn materialized_v2_binds_all_construction_facts_and_detects_each_drift() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("legacy.ncl");
    std::fs::write(&input, workflow_to_nickel(&flat("construction-facts", true))).unwrap();
    let legacy = lattice::workflow_format::load_workflow_nickel_file(&input).unwrap();
    let runtime = lattice::engine::workflow::ConstructionRuntimeBinding {
        runtime_artifact: "identity-only-runtime".into(),
        shell_path: "/never-executed/shell".into(),
        shell_artifact: "identity-only-shell".into(),
        shell_implementation: "identity-only-implementation".into(),
    };
    for fact in 0..3 {
        let mut changed = legacy.clone();
        match fact {
            0 => changed.control_dependencies.push((0, 1)),
            1 => changed.construction_profile = true,
            2 => changed.construction_runtime = Some(runtime.clone()),
            _ => unreachable!(),
        }
        assert_ne!(authoring::expanded_identity(&legacy).unwrap(), authoring::expanded_identity(&changed).unwrap());
        authoring::reconcile_materialized(&mut changed).unwrap();
        assert_eq!(changed.authoring.unwrap().state, AssociationState::MaterializedFork);
    }
    let mut construction = legacy;
    construction.control_dependencies.push((0, 1));
    construction.construction_profile = true;
    construction.construction_runtime = Some(runtime);
    let expected = authoring::identity(
        "lattice.materialized-workflow.v2",
        &(
            &construction.meta,
            &construction.graph,
            &construction.configs,
            &construction.credentials,
            &construction.runner,
            &construction.triggers,
            &construction.control_dependencies,
            &construction.construction_profile,
            &construction.construction_runtime,
        ),
    )
    .unwrap();
    assert_eq!(authoring::expanded_identity(&construction).unwrap(), expected);
    // Identity-only association for testing reconciliation, never runtime admission.
    construction.authoring.as_mut().unwrap().expanded_artifact = expected.clone();
    for fact in 0..9 {
        let mut changed = construction.clone();
        match fact {
            0 => changed.graph.node_mut(lattice::engine::workflow::NodeId(0)).unwrap().label.push_str(" changed"),
            1 => changed.configs.clear(),
            2 => changed.control_dependencies.clear(),
            3 => changed.construction_profile = false,
            4 => changed.construction_runtime = None,
            5 => changed.construction_runtime.as_mut().unwrap().runtime_artifact.push_str(" changed"),
            6 => changed.construction_runtime.as_mut().unwrap().shell_path.push_str(" changed"),
            7 => changed.construction_runtime.as_mut().unwrap().shell_artifact.push_str(" changed"),
            8 => changed.construction_runtime.as_mut().unwrap().shell_implementation.push_str(" changed"),
            _ => unreachable!(),
        }
        assert_ne!(authoring::expanded_identity(&changed).unwrap(), expected, "drift case {fact}");
        authoring::reconcile_materialized(&mut changed).unwrap();
        assert_eq!(changed.authoring.unwrap().state, AssociationState::MaterializedFork, "drift case {fact}");
    }
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_source_artifact_and_compiler_identities_are_not_execution_or_semantic_equivalence() {
    let source = program();
    let before = shell::compile(&source).unwrap();
    let mut changed = source.clone();
    changed.files.insert("selected-helper.ncl".into(), b"fun value => value".to_vec());
    let after = shell::compile(&changed).unwrap();
    assert_ne!(source.identity().unwrap(), changed.identity().unwrap());
    assert_eq!(authoring::expanded_identity(&before).unwrap(), authoring::expanded_identity(&after).unwrap());
    assert_eq!(workflow_to_portable(&before), workflow_to_portable(&after));
    // Labels remain in materialized identity: config/expression/runtime binding
    // semantics are not inferred away to manufacture a logical-equivalence claim.
    let mut relabeled = before.clone();
    relabeled.graph.node_mut(lattice::engine::workflow::NodeId(1)).unwrap().label =
        "diagnostic-or-binding-not-inferred".into();
    assert_ne!(authoring::expanded_identity(&before).unwrap(), authoring::expanded_identity(&relabeled).unwrap());
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_preview_export_and_store_do_not_execute_shell_or_deliver_credentials() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("must-not-exist");
    let mut workflow = flat("no-effects", false);
    workflow.set_node_config(
        lattice::engine::workflow::NodeId(0),
        lattice::engine::workflow::NodeConfig::shell(format!("touch {}", marker.display())),
    );
    workflow.set_node_credentials(lattice::engine::workflow::NodeId(0), lattice::credentials::NodeCredentialRefs {
        env: HashMap::from([(
            "TOKEN".into(),
            lattice::credentials::SecretRef::new("missing-provider", "never-resolved", None),
        )]),
    });
    let source = workflow_to_nickel(&workflow);
    let program = CapturedProgram {
        entry: "workflow.ncl".into(),
        files: BTreeMap::from([("workflow.ncl".into(), source.into_bytes())]),
        kind: ProgramKind::Workflow,
        parameters: BTreeMap::new(),
    };
    let candidate = shell::compile(&program).unwrap();
    shell::source_export(&candidate).unwrap();
    shell::expanded_export(&candidate);
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    assert!(
        store.save_workflow_cas(&candidate, None).is_err(),
        "a generic save cannot publish captured provenance"
    );
    let mut draft = SourceDraft {
        id: "no-effects".into(),
        workflow_name: "no-effects".into(),
        expected_revision: None,
        program,
        diagnostic: None,
        preview: None,
    };
    store.preview_source_draft(&mut draft).unwrap();
    store.admit_source_draft(&draft).unwrap();
    assert!(!marker.exists());
}

#[test]
fn authoring_capture_rejects_unsafe_or_oversized_members_without_fabricated_provenance() {
    let mut source = program();
    source.files.insert("../outside.ncl".into(), vec![]);
    assert!(source.validate().is_err());
    source.files.remove("../outside.ncl");
    source.files.insert("huge.ncl".into(), vec![0; lattice::workflow_template::MAX_FILE_BYTES + 1]);
    assert!(source.validate().is_err());
}

fn tui_source_edit(app: &mut lattice::tui::State, id: &str, text: &str) -> bool {
    app.open_authoring("source");
    let editor = app.authoring.as_mut().unwrap();
    editor.selected = editor.fields.iter().position(|field| field.id == id).unwrap();
    editor.begin(&app.workflow);
    editor.draft.as_mut().unwrap().text = text.into();
    app.confirm_authoring()
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_tui_source_preview_failure_reopen_admission_and_materialized_save_are_distinct() {
    let directory = tempfile::tempdir().unwrap();
    let mut app = lattice::tui::State::new(lattice::config::AppConfig {
        data_dir: directory.path().join("state"),
        ..Default::default()
    });
    let store = app.store.clone().unwrap();
    let original = admitted(&store);
    assert!(app.load_workflow_by_name("source-test"));
    assert!(!tui_source_edit(&mut app, "source.file/workflow.ncl", "let broken ="));
    assert!(app.authoring.as_ref().unwrap().error.is_some());
    assert_eq!(app.workflow.catalog_revision, original.catalog_revision);
    app.open_authoring("source");
    assert_eq!(
        app.authoring
            .as_ref()
            .unwrap()
            .fields
            .iter()
            .find(|field| field.id == "source.file/workflow.ncl")
            .unwrap()
            .value,
        "let broken ="
    );
    let source = String::from_utf8(program().files["workflow.ncl"].clone()).unwrap();
    assert!(!tui_source_edit(&mut app, "source.file/workflow.ncl", &source));
    assert!(!tui_source_edit(&mut app, "source.parameters", r#"{"first":"tui-"}"#));
    assert_eq!(app.workflow.catalog_revision, original.catalog_revision);
    assert!(tui_source_edit(&mut app, "source.admit", "admit"));
    assert!(!app.has_unsaved_changes());
    assert_ne!(app.workflow.catalog_revision, original.catalog_revision);
    assert_eq!(workflow_to_portable(&app.workflow).nodes[0].label, "tui-first");
    app.add_shell_node("materialized", "printf must-not-run");
    assert_eq!(app.workflow.authoring.as_ref().unwrap().state, AssociationState::MaterializedFork);
    app.save_workflow();
    assert!(!app.has_unsaved_changes());
    assert_eq!(
        store.load_workflow("source-test").unwrap().unwrap().authoring.unwrap().state,
        AssociationState::MaterializedFork
    );
}

// r[verify lattice.source_authoring.exports]
// r[verify lattice.source_authoring.exports.cli]
#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_cli_source_commands_are_registered_and_keep_artifact_kinds_explicit() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("config.ncl");
    let config_value = serde_json::json!({
        "data_dir": directory.path().join("state").to_string_lossy(),
        "default_timeout_secs": 60,
        "shell": "sh",
    });
    std::fs::write(&config, nickel_eval::to_nickel(&config_value).unwrap()).unwrap();
    let source = directory.path().join("source.ncl");
    std::fs::write(&source, &program().files["workflow.ncl"]).unwrap();
    let command = |args: &[&str]| {
        std::process::Command::new(lattice_binary())
            .arg("--config")
            .arg(&config)
            .args(args)
            .output()
            .unwrap()
    };
    let input = source.to_str().unwrap();
    let created = command(&[
        "authoring",
        "draft",
        "source-test",
        "cli-draft",
        "--input",
        input,
        "--parameterized",
    ]);
    assert!(created.status.success(), "{}", String::from_utf8_lossy(&created.stderr));
    let preview = command(&["authoring", "preview", "cli-draft"]);
    assert!(preview.status.success(), "{}", String::from_utf8_lossy(&preview.stderr));
    let admitted = command(&["authoring", "admit", "cli-draft"]);
    assert!(admitted.status.success(), "{}", String::from_utf8_lossy(&admitted.stderr));
    let ambiguous = directory.path().join("misleading.ncl");
    assert!(!command(&["export", "source-test", ambiguous.to_str().unwrap()]).status.success());
    assert!(!ambiguous.exists());
    let archive = directory.path().join("source.author.json");
    assert!(command(&["export", "source-test", archive.to_str().unwrap()]).status.success());
    let expected_source = CapturedProgram {
        entry: source.file_name().unwrap().to_str().unwrap().to_string(),
        files: BTreeMap::from([(
            source.file_name().unwrap().to_str().unwrap().to_string(),
            std::fs::read(&source).unwrap(),
        )]),
        kind: ProgramKind::Parameterized,
        parameters: BTreeMap::new(),
    };
    assert_eq!(shell::read_source_archive(&archive).unwrap(), expected_source);
    let expanded = directory.path().join("expanded.ncl");
    assert!(command(&["export", "source-test", expanded.to_str().unwrap(), "--expanded"]).status.success());
    let replacement = directory.path().join("replacement.ncl");
    std::fs::write(&replacement, workflow_to_nickel(&flat("source-test", false))).unwrap();
    let replaced = command(&["import", replacement.to_str().unwrap()]);
    assert!(replaced.status.success(), "{}", String::from_utf8_lossy(&replaced.stderr));
    let replaced_archive = directory.path().join("replacement.author.json");
    assert!(command(&["export", "source-test", replaced_archive.to_str().unwrap()]).status.success());
    let retained = shell::read_source_archive(&replaced_archive).unwrap();
    assert_eq!(retained.files[&retained.entry], std::fs::read(replacement).unwrap());
}

fn cli_output(directory: &std::path::Path, config: &std::path::Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(lattice_binary())
        .current_dir(directory)
        .arg("--config")
        .arg(config)
        .args(args)
        .output()
        .unwrap()
}

// r[verify lattice.graph_construction.profile]
// r[verify lattice.graph_construction.profile.cli]
// r[verify lattice.graph_construction.closure]
// r[verify lattice.graph_construction.origins.navigation]
#[test]
#[ignore = "requires exact captured native worker and immutable construction runtime; Nix qualification runs ignored tests"]
fn construction_cli_import_readmits_captured_source_without_weakening_file_import() {
    let directory = tempfile::tempdir().unwrap();
    let data = directory.path().join("state");
    let config = directory.path().join("config.ncl");
    std::fs::write(
        &config,
        nickel_eval::to_nickel(&serde_json::json!({
            "data_dir": data.to_string_lossy(),
            "default_timeout_secs": 60,
            "shell": "sh",
        }))
        .unwrap(),
    )
    .unwrap();
    let ordinary = directory.path().join("ordinary.ncl");
    std::fs::write(&ordinary, workflow_to_nickel(&flat("local-shell-demo", false))).unwrap();
    let source = directory.path().join("local-shell-construction.ncl");
    std::fs::write(&source, include_bytes!("../examples/construction/local-shell-construction.ncl")).unwrap();
    let relative_source = std::path::Path::new("local-shell-construction.ncl");
    let import = |path: &std::path::Path, construction: bool| {
        let mut command = std::process::Command::new(lattice_binary());
        command.current_dir(directory.path());
        command.arg("--config").arg(&config).arg("import").arg(path);
        if construction {
            command.arg("--construction");
        }
        command.output().unwrap()
    };
    let ordinary_result = import(&ordinary, false);
    assert!(ordinary_result.status.success(), "{}", String::from_utf8_lossy(&ordinary_result.stderr));
    let retained = {
        let store = Store::new(lattice::config::store_path(&data)).unwrap();
        let workflow = store.load_workflow("local-shell-demo").unwrap().unwrap();
        assert_eq!(workflow.authoring.as_ref().unwrap().state, AssociationState::RetainedEntrySnapshot);
        workflow
    };
    let unattributed = cli_output(directory.path(), &config, &["authoring", "origin", "local-shell-demo", "0"]);
    assert!(!unattributed.status.success(), "a retained snapshot has no construction origin to report");
    let captured_result = import(relative_source, true);
    assert!(captured_result.status.success(), "{}", String::from_utf8_lossy(&captured_result.stderr));
    let revision = {
        let store = Store::new(lattice::config::store_path(&data)).unwrap();
        let workflow = store.load_workflow("local-shell-demo").unwrap().unwrap();
        assert_ne!(workflow.catalog_revision, retained.catalog_revision);
        let captured = workflow.authoring.as_ref().unwrap();
        assert_eq!(captured.state, AssociationState::CapturedModules);
        assert_eq!(captured.program.files[&captured.program.entry], std::fs::read(&source).unwrap());
        assert!(captured.observation.is_some());
        assert!(captured.construction.is_some());
        assert!(
            authoring::construction::admit_runtime(&workflow).is_err(),
            "the test executable must not impersonate the importing CLI runtime"
        );
        assert!(store.admit_imported_workflow(&workflow, workflow.catalog_revision.as_ref()).is_err());
        for observation in [false, true] {
            let mut forged = retained.clone();
            let association = forged.authoring.as_mut().unwrap();
            association.construction = captured.construction.clone();
            if observation {
                association.observation = captured.observation.clone();
            }
            assert!(association.validate().is_err());
            assert!(store.admit_imported_workflow(&forged, workflow.catalog_revision.as_ref()).is_err());
        }
        assert_eq!(store.load_workflow("local-shell-demo").unwrap().unwrap().authoring, workflow.authoring);
        workflow.catalog_revision
    };
    let (left, left_stdout) = {
        let store = Store::new(lattice::config::store_path(&data)).unwrap();
        let workflow = store.load_workflow("local-shell-demo").unwrap().unwrap();
        let node = workflow.graph.nodes().find(|node| node.label.contains("left")).unwrap();
        (node.id.0.to_string(), node.output_ports[0].id.0.to_string())
    };
    for port in [None, Some(left_stdout.as_str())] {
        let mut args = vec!["authoring", "origin", "local-shell-demo", left.as_str()];
        args.extend(port.map(|port| ["--port", port]).into_iter().flatten());
        let origin = cli_output(directory.path(), &config, &args);
        assert!(origin.status.success(), "{}", String::from_utf8_lossy(&origin.stderr));
        let text = String::from_utf8(origin.stdout).unwrap();
        assert!(text.starts_with("template=worker instance="), "{text}");
        let instance = text.split("instance=").nth(1).and_then(|rest| rest.split(' ').next()).unwrap();
        assert!(instance.split('/').any(|part| part == "left"), "{text}");
        assert!(text.contains("location=local-shell-construction.ncl:"), "{text}");
    }
    let foreign_port = cli_output(directory.path(), &config, &[
        "authoring",
        "origin",
        "local-shell-demo",
        left.as_str(),
        "--port",
        "999999",
    ]);
    assert!(!foreign_port.status.success(), "a port outside the selected node is not attributed");

    // The registered runtime route runs the imported graph only with its exact grant.
    let denied = cli_output(directory.path(), &config, &["run", "local-shell-demo", "--output", "json"]);
    assert!(!denied.status.success(), "absent effect authority fails closed");
    let inspected = cli_output(directory.path(), &config, &["authority", "inspect", "local-shell-demo"]);
    assert!(inspected.status.success(), "{}", String::from_utf8_lossy(&inspected.stderr));
    let mut profile: serde_json::Value = serde_json::from_slice(&inspected.stdout).unwrap();
    profile["expires_at_unix_ms"] = serde_json::json!(4_102_444_800_000_u64);
    profile["roots"] = serde_json::json!({});
    std::fs::write(directory.path().join("grant.json"), serde_json::to_vec(&profile).unwrap()).unwrap();
    std::fs::write(directory.path().join("authority.ncl"), "import \"grant.json\"\n").unwrap();
    let authorized_config = directory.path().join("authorized.ncl");
    std::fs::write(
        &authorized_config,
        nickel_eval::to_nickel(&serde_json::json!({
            "data_dir": data.to_string_lossy(),
            "default_timeout_secs": 60,
            "shell": "sh",
            "effect_authority": directory.path().join("authority.ncl").to_string_lossy(),
        }))
        .unwrap(),
    )
    .unwrap();
    let ran = cli_output(directory.path(), &authorized_config, &["run", "local-shell-demo", "--output", "json"]);
    assert!(ran.status.success(), "{}", String::from_utf8_lossy(&ran.stderr));
    let summary: serde_json::Value = serde_json::from_slice(&ran.stdout).unwrap();
    assert_eq!(summary["completed"], 6);
    let repeated = import(relative_source, true);
    assert!(repeated.status.success(), "{}", String::from_utf8_lossy(&repeated.stderr));
    // A sibling import must resolve beneath the same normalized current-directory
    // root, rather than an empty parent or an ambient evaluator import path.
    std::fs::write(directory.path().join("wrapper.ncl"), "import \"./local-shell-construction.ncl\"").unwrap();
    let with_member = std::process::Command::new(lattice_binary())
        .current_dir(directory.path())
        .arg("--config")
        .arg(&config)
        .args([
            "import",
            "wrapper.ncl",
            "--construction",
            "--member",
            "local-shell-construction.ncl",
        ])
        .output()
        .unwrap();
    assert!(with_member.status.success(), "{}", String::from_utf8_lossy(&with_member.stderr));
    let newer = {
        let store = Store::new(lattice::config::store_path(&data)).unwrap();
        let workflow = store.load_workflow("local-shell-demo").unwrap().unwrap();
        let captured = workflow.authoring.as_ref().unwrap();
        assert_eq!(captured.state, AssociationState::CapturedModules);
        assert_eq!(captured.program.entry, "wrapper.ncl");
        assert_eq!(captured.program.files["local-shell-construction.ncl"], std::fs::read(&source).unwrap());
        assert!(captured.observation.as_ref().unwrap().modules.contains_key("local-shell-construction.ncl"));
        assert_ne!(workflow.catalog_revision, revision);
        workflow.catalog_revision
    };
    std::fs::write(&source, "let invalid =").unwrap();
    assert!(!import(relative_source, true).status.success());
    let store = Store::new(lattice::config::store_path(&data)).unwrap();
    assert_eq!(store.load_workflow("local-shell-demo").unwrap().unwrap().catalog_revision, newer);
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_catalog_delete_recreate_does_not_allow_revision_aba() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let original = admitted(&store);
    let mut stale = draft(&store, "before-delete");
    store.preview_source_draft(&mut stale).unwrap();
    store.delete_workflow("source-test").unwrap();
    let mut recreated_draft = draft(&store, "after-delete");
    store.preview_source_draft(&mut recreated_draft).unwrap();
    let recreated = store.admit_source_draft(&recreated_draft).unwrap();
    assert_ne!(recreated.catalog_revision, original.catalog_revision);
    assert!(store.admit_source_draft(&stale).is_err());
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_generic_save_cannot_erase_lineage_promote_a_fork_or_replace_source_without_admission() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let original = admitted(&store);
    let mut erased = original.clone();
    erased.authoring = None;
    erased.meta.description = "materialized edit with omitted association".into();
    let fork = store.save_workflow_cas(&erased, erased.catalog_revision.as_ref()).unwrap();
    assert_eq!(fork.authoring.as_ref().unwrap().state, AssociationState::MaterializedFork);
    assert_eq!(
        fork.authoring.as_ref().unwrap().source_revision,
        original.authoring.as_ref().unwrap().source_revision
    );
    let mut promoted = fork.clone();
    promoted.authoring.as_mut().unwrap().state = AssociationState::CapturedSnapshot;
    let retained = store.save_workflow_cas(&promoted, promoted.catalog_revision.as_ref()).unwrap();
    assert_eq!(retained.authoring.as_ref().unwrap().state, AssociationState::MaterializedFork);
    let mut unadmitted = retained.clone();
    let association = unadmitted.authoring.as_mut().unwrap();
    std::sync::Arc::make_mut(&mut association.program)
        .parameters
        .insert("first".into(), serde_json::json!("not-previewed-"));
    association.source_revision = association.program.identity().unwrap();
    assert!(store.save_workflow_cas(&unadmitted, unadmitted.catalog_revision.as_ref()).is_err());
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_state_only_downgrade_does_not_skip_replay_and_real_forks_remain_sticky() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let original = admitted(&store);
    let mut downgraded = original.clone();
    downgraded.authoring.as_mut().unwrap().state = AssociationState::MaterializedFork;
    let retained = store.save_workflow_cas(&downgraded, original.catalog_revision.as_ref()).unwrap();
    assert_eq!(retained.authoring.as_ref().unwrap().state, AssociationState::CapturedModules);
    assert_eq!(
        authoring::expanded_identity(&retained).unwrap(),
        retained.authoring.as_ref().unwrap().expanded_artifact
    );

    let mut edited = retained.clone();
    edited.meta.description = "a materialized edit".into();
    let fork = store.save_workflow_cas(&edited, retained.catalog_revision.as_ref()).unwrap();
    assert_eq!(fork.authoring.as_ref().unwrap().state, AssociationState::MaterializedFork);
    let mut reverted = fork.clone();
    reverted.meta.description = original.meta.description.clone();
    reverted.authoring.as_mut().unwrap().state = AssociationState::CapturedModules;
    assert_eq!(
        authoring::expanded_identity(&reverted).unwrap(),
        original.authoring.as_ref().unwrap().expanded_artifact
    );
    let sticky = store.save_workflow_cas(&reverted, fork.catalog_revision.as_ref()).unwrap();
    assert_eq!(sticky.authoring.as_ref().unwrap().state, AssociationState::MaterializedFork);
    assert_eq!(
        sticky.authoring.as_ref().unwrap().source_revision,
        original.authoring.as_ref().unwrap().source_revision
    );
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_captured_import_requires_observed_members_but_not_comments_or_strings() {
    let flat = workflow_to_nickel(&flat("source-test", false));
    for source in [
        format!("# import \"not-a-module\"\n{flat}"),
        format!("let text = \"import not-a-module\" in ({flat})"),
    ] {
        let program = CapturedProgram {
            entry: "workflow.ncl".into(),
            files: BTreeMap::from([("workflow.ncl".into(), source.into_bytes())]),
            kind: ProgramKind::Workflow,
            parameters: BTreeMap::new(),
        };
        shell::compile(&program).unwrap();
    }
    let source = "let selected = import \"./missing.ncl\" in selected".to_owned();
    let program = CapturedProgram {
        entry: "workflow.ncl".into(),
        files: BTreeMap::from([("workflow.ncl".into(), source.into_bytes())]),
        kind: ProgramKind::Workflow,
        parameters: BTreeMap::new(),
    };
    assert!(shell::compile(&program).is_err(), "an uncaptured selected module is unavailable to the worker");
}

#[test]
fn authoring_ordinary_ambient_import_remains_compatible_but_missing_captured_member_cannot_preview() {
    let directory = tempfile::tempdir().unwrap();
    let module = directory.path().join("module.ncl");
    std::fs::write(&module, workflow_to_nickel(&flat("source-test", false))).unwrap();
    let source = directory.path().join("entry.ncl");
    let import = format!("import {}", serde_json::to_string(module.to_str().unwrap()).unwrap());
    std::fs::write(&source, import).unwrap();
    let imported = lattice::workflow_format::load_workflow_nickel_file(&source).unwrap();
    assert_eq!(imported.authoring.as_ref().unwrap().state, AssociationState::RetainedEntrySnapshot);
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let saved = store.save_workflow_cas(&imported, None).unwrap();
    let mut draft = draft(&store, "module-edit");
    draft
        .program
        .files
        .insert("module.ncl".into(), workflow_to_nickel(&flat("source-test", true)).into_bytes());
    assert!(store.preview_source_draft(&mut draft).is_err());
    assert!(draft.diagnostic.is_some());
    assert!(draft.preview.is_none());
    assert_eq!(store.load_workflow("source-test").unwrap().unwrap().catalog_revision, saved.catalog_revision);
    assert_eq!(store.load_workflow("source-test").unwrap().unwrap().graph.node_count(), 1);
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_readmission_rechecks_current_native_validation_of_persisted_candidate() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let previous = admitted(&store);
    let mut draft = draft(&store, "native-validation");
    store.preview_source_draft(&mut draft).unwrap();
    let candidate = &mut draft.preview.as_mut().unwrap().candidate;
    candidate.configs.remove(&0);
    let changed_artifact = authoring::expanded_identity(candidate).unwrap();
    candidate.authoring.as_mut().unwrap().expanded_artifact = changed_artifact;
    store.save_source_draft(&draft).unwrap();
    assert!(store.admit_source_draft(&draft).is_err());
    assert_eq!(store.load_workflow("source-test").unwrap().unwrap().catalog_revision, previous.catalog_revision);
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_parameter_interpolation_text_is_data_in_preview_and_expanded_export() {
    let directory = tempfile::tempdir().unwrap();
    let mut source = program();
    let literal = "%{import \"/must-never-be-opened.ncl\"}";
    source.parameters.insert("first".into(), serde_json::json!(literal));
    let workflow = shell::compile(&source).unwrap();
    assert_eq!(workflow_to_portable(&workflow).nodes[0].label, format!("{literal}first"));
    let snapshot = workflow.clone();
    assert!(std::sync::Arc::ptr_eq(
        &workflow.authoring.as_ref().unwrap().program,
        &snapshot.authoring.as_ref().unwrap().program
    ));
    let path = directory.path().join("expanded.ncl");
    std::fs::write(&path, shell::expanded_export(&workflow)).unwrap();
    let reopened = lattice::workflow_format::load_workflow_nickel_file(&path).unwrap();
    assert_eq!(workflow_to_portable(&reopened), workflow_to_portable(&workflow));
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn authoring_file_import_cas_rejects_stale_or_changed_candidates() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let original = admitted(&store);
    let file = directory.path().join("replacement.ncl");
    std::fs::write(&file, workflow_to_nickel(&flat("source-test", false))).unwrap();
    let candidate = lattice::workflow_format::load_workflow_nickel_file(&file).unwrap();
    let mut edited = original.clone();
    edited.meta.description = "newer materialized edit".into();
    let newer = store.save_workflow_cas(&edited, original.catalog_revision.as_ref()).unwrap();
    assert!(store.admit_imported_workflow(&candidate, original.catalog_revision.as_ref()).is_err());
    let mut tampered = candidate.clone();
    tampered.configs.clear();
    assert!(store.admit_imported_workflow(&tampered, newer.catalog_revision.as_ref()).is_err());
    assert_eq!(store.load_workflow("source-test").unwrap().unwrap().catalog_revision, newer.catalog_revision);
    let mut forged = candidate.clone();
    let unrelated_observation = original.authoring.as_ref().unwrap().observation.clone().unwrap();
    unrelated_observation.validate().unwrap();
    forged.authoring.as_mut().unwrap().observation = Some(unrelated_observation);
    assert!(forged.authoring.as_ref().unwrap().validate().is_err());
    assert!(store.admit_imported_workflow(&forged, newer.catalog_revision.as_ref()).is_err());
    assert_eq!(store.load_workflow("source-test").unwrap().unwrap().catalog_revision, newer.catalog_revision);
    let imported = store.admit_imported_workflow(&candidate, newer.catalog_revision.as_ref()).unwrap();
    assert_eq!(workflow_to_portable(&imported), workflow_to_portable(&candidate));
    assert_eq!(imported.authoring.unwrap().state, AssociationState::RetainedEntrySnapshot);
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn self_consistent_observation_for_other_entry_cannot_publish_on_new_name() {
    let source = workflow_to_nickel(&flat("source-test", false));
    let first = CapturedProgram {
        entry: "first.ncl".into(),
        files: BTreeMap::from([
            ("first.ncl".into(), source.as_bytes().to_vec()),
            ("second.ncl".into(), source.into_bytes()),
        ]),
        kind: ProgramKind::Workflow,
        parameters: BTreeMap::new(),
    };
    let mut second = first.clone();
    second.entry = "second.ncl".into();
    let mut forged = shell::compile(&first).unwrap();
    let observed_elsewhere = shell::compile(&second).unwrap().authoring.unwrap().observation.unwrap();
    let association = forged.authoring.as_mut().unwrap();
    association.evaluated_output = authoring::ContentIdentity::from_bytes(observed_elsewhere.result.json.as_bytes());
    association.observation = Some(observed_elsewhere);
    association.validate().unwrap(); // Request/response correspondence is not source observation.
    assert!(shell::verify_observation(&first, association.observation.as_ref().unwrap()).is_err());
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    assert!(store.save_workflow_cas(&forged, None).is_err());
    assert!(store.save_workflow(&forged).is_err());
    assert!(store.load_workflow("source-test").unwrap().is_none());
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn parameter_application_receipt_is_rebuilt_from_declared_defaults() {
    let source = program();
    let mut changed = source.clone();
    changed.parameters.insert("first".into(), serde_json::json!("different-"));
    let mut original = shell::compile(&source).unwrap();
    let alternative = shell::compile(&changed).unwrap().authoring.unwrap().observation.unwrap();
    let association = original.authoring.as_mut().unwrap();
    assert_eq!(alternative.entry, association.observation.as_ref().unwrap().entry);
    assert_ne!(alternative.modules, association.observation.as_ref().unwrap().modules);
    association.evaluated_output = authoring::ContentIdentity::from_bytes(alternative.result.json.as_bytes());
    association.observation = Some(alternative);
    association.validate().unwrap(); // Matching retained files are insufficient for the generated application.
    assert!(shell::verify_observation(&source, association.observation.as_ref().unwrap()).is_err());
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn admitted_four_mebibyte_source_archive_roundtrips_even_with_decimal_byte_arrays() {
    const FILE: usize = lattice::workflow_template::MAX_FILE_BYTES;
    const BUNDLE: usize = lattice::workflow_template::MAX_BUNDLE_BYTES;
    let mut entry = workflow_to_nickel(&flat("archive-limit", false)).into_bytes();
    entry.extend_from_slice(b"\n#");
    entry.extend(std::iter::repeat_n(b'd', FILE - entry.len()));
    let mut files = BTreeMap::from([("entry.ncl".into(), entry)]);
    for index in 0..3 {
        let mut member = Vec::with_capacity(FILE);
        member.push(b'#');
        member.extend(std::iter::repeat_n(b'd', FILE - 1));
        files.insert(format!("extra-{index}.ncl"), member);
    }
    let source = CapturedProgram {
        entry: "entry.ncl".into(),
        files,
        kind: ProgramKind::Workflow,
        parameters: BTreeMap::new(),
    };
    assert_eq!(source.files.values().map(Vec::len).sum::<usize>(), BUNDLE);
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let mut draft = SourceDraft {
        id: "archive-limit".into(),
        workflow_name: "archive-limit".into(),
        expected_revision: None,
        program: source.clone(),
        diagnostic: None,
        preview: None,
    };
    store.preview_source_draft(&mut draft).unwrap();
    let admitted = store.admit_source_draft(&draft).unwrap();
    let archive = shell::source_export(&admitted).unwrap();
    assert!(archive.len() <= 64 * 1_048_576);
    let path = directory.path().join("limit.author.json");
    std::fs::write(&path, archive).unwrap();
    assert_eq!(shell::read_source_archive(&path).unwrap(), source);
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn source_archive_accepts_exact_limit_but_rejects_valid_json_with_oversized_whitespace() {
    use std::io::Write as _;

    const LIMIT: usize = 64 * 1_048_576;
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let archive = shell::source_export(&admitted(&store)).unwrap();
    assert!(archive.len() < LIMIT);
    let path = directory.path().join("bounded.author.json");
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(&archive).unwrap();
    let padding = [b' '; 64 * 1024];
    let mut remaining = LIMIT - archive.len();
    while remaining > 0 {
        let count = remaining.min(padding.len());
        file.write_all(&padding[..count]).unwrap();
        remaining -= count;
    }
    drop(file);
    assert_eq!(shell::read_source_archive(&path).unwrap(), program());

    let mut file = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
    file.write_all(b" ").unwrap();
    drop(file);
    assert!(shell::read_source_archive(&path).is_err());
}

#[test]
#[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
fn cas_cannot_replace_a_same_output_parameter_observation() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let mut initial = program();
    initial.parameters.insert("two".into(), serde_json::json!(false));
    let mut draft = SourceDraft {
        id: "same-output".into(),
        workflow_name: "source-test".into(),
        expected_revision: None,
        program: initial.clone(),
        diagnostic: None,
        preview: None,
    };
    store.preview_source_draft(&mut draft).unwrap();
    let admitted = store.admit_source_draft(&draft).unwrap();
    let mut other_program = initial;
    other_program.parameters.insert("second".into(), serde_json::json!("unused-"));
    let other = shell::compile(&other_program).unwrap();
    let original = admitted.authoring.as_ref().unwrap();
    let replacement = other.authoring.as_ref().unwrap();
    assert_eq!(original.evaluated_output, replacement.evaluated_output);
    assert_eq!(original.expanded_artifact, replacement.expanded_artifact);
    assert_ne!(original.observation, replacement.observation);
    let mut forged = admitted.clone();
    forged.authoring.as_mut().unwrap().observation = replacement.observation.clone();
    forged.authoring.as_ref().unwrap().validate().unwrap();
    assert!(store.save_workflow_cas(&forged, admitted.catalog_revision.as_ref()).is_err());
    assert_eq!(store.load_workflow("source-test").unwrap().unwrap().authoring, admitted.authoring);
}

/// Runs only in a child with every worker selection removed.
#[test]
fn captured_evaluation_without_selected_worker_child() {
    if std::env::var_os("LATTICE_TEST_NO_WORKER_CHILD").is_none() {
        return;
    }
    assert!(lattice::nickel::captured_worker().is_err(), "no worker may be selected in this child");
    assert_eq!(shell::compile(&program()).unwrap_err(), "captured evaluation failed");
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let mut pending = draft(&store, "no-worker");
    store.save_source_draft(&pending).unwrap();
    assert!(store.preview_source_draft(&mut pending).is_err());
    let retained = store.load_source_draft("no-worker").unwrap().unwrap();
    assert!(retained.preview.is_none());
    assert!(retained.diagnostic.is_some_and(|message| message.len() <= 128));
    assert!(store.load_workflow("source-test").unwrap().is_none(), "a failed preview admits nothing");
}

// Plain Cargo and Nix sandboxes cannot select the immutable worker; the absent
// selection must fail closed rather than evaluate through another path.
#[test]
fn captured_evaluation_fails_closed_without_a_selected_worker() {
    // r[verify lattice.graph_construction.profile]
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "captured_evaluation_without_selected_worker_child",
            "--nocapture",
        ])
        .env_remove("LATTICE_CLOSED_EVAL_WORKER")
        .env("LATTICE_TEST_NO_WORKER_CHILD", "1")
        .status()
        .unwrap();
    assert!(status.success(), "captured evaluation must fail closed with no selected worker");
}

mod source_edit_owner {
    use std::io::Write as _;
    use std::path::Path;
    use std::path::PathBuf;
    use std::process::Command;
    use std::process::Stdio;

    use serde_json::Value;
    use serde_json::json;

    use super::*;

    const SCHEMA: &str = "lattice.source-edit.v1";

    fn fixture(root: &Path) -> (PathBuf, PathBuf, Workflow, Vec<u8>) {
        let config = root.join("config.ncl");
        std::fs::write(
            &config,
            nickel_eval::to_nickel(&json!({"data_dir": root, "default_timeout_secs": 60, "shell": "sh"})).unwrap(),
        )
        .unwrap();
        let source = root.join("workflow.ncl");
        let bytes = nickel_eval::to_nickel(&workflow_to_portable(&flat("source-test", false))).unwrap().into_bytes();
        std::fs::write(&source, &bytes).unwrap();
        let db = lattice::config::store_path(root);
        let store = Store::new(&db).unwrap();
        let loaded = lattice::workflow_format::load_workflow_nickel_file(&source).unwrap();
        let workflow = store.admit_imported_workflow(&loaded, None).unwrap();
        (config, db, workflow, bytes)
    }

    fn call(config: &Path, request: &Value, expected: &str) -> Value {
        let mut child = Command::new(lattice_binary())
            .arg("--config")
            .arg(config)
            .args(["authoring", "source-edit"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&serde_json::to_vec(request).unwrap()).unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(
            output.status.code(),
            Some(if expected == "rejected" { 1 } else { 0 }),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let reply: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(reply["schema"], SCHEMA);
        assert_eq!(reply["status"], expected, "{reply}");
        reply
    }

    fn target(action: &str, head: &authoring::ContentIdentity) -> Value {
        json!({"schema": SCHEMA, "action": action, "workflow": "source-test", "expected_head": head, "draft_id": "owner-draft", "member": "workflow.ncl"})
    }

    // r[verify lattice.source_edit.view]
    #[test]
    fn exact_source_owner_inspects_and_views_without_initializing_or_writing() {
        let directory = tempfile::tempdir().unwrap();
        let (config, db, before, bytes) = fixture(directory.path());
        let db_bytes = std::fs::read(&db).unwrap();
        let head = before.catalog_revision.unwrap();
        let inspected =
            call(&config, &json!({"schema": SCHEMA, "action": "inspect", "workflow": "source-test"}), "inspected");
        assert_eq!(inspected["head"], json!(head));
        let association = before.authoring.unwrap();
        assert_eq!(
            inspected["source"],
            json!({"state": "RetainedEntrySnapshot", "source_revision": association.source_revision, "entry": "workflow.ncl", "members": ["workflow.ncl"]})
        );
        let mut request = target("view", &head);
        let reply = call(&config, &request, "viewed");
        assert_eq!(reply["head"], json!(head));
        assert_eq!(reply["source_revision"], json!(association.source_revision));
        assert_eq!(reply["bytes"], json!(bytes));
        assert_eq!(reply["draft_revision"], Value::Null);
        request["expected_head"] = json!(authoring::ContentIdentity::from_bytes(b"other head"));
        assert_eq!(call(&config, &request, "rejected")["code"], "head_conflict");
        request["expected_head"] = json!(head);
        request["member"] = json!("missing.ncl");
        assert_eq!(call(&config, &request, "rejected")["code"], "member_not_found");
        request["member"] = json!("../workflow.ncl");
        assert_eq!(call(&config, &request, "rejected")["code"], "invalid_request");
        assert_eq!(db_bytes, std::fs::read(&db).unwrap());

        let missing_dir = directory.path().join("missing");
        let config_value = json!({"data_dir": missing_dir, "default_timeout_secs": 60, "shell": "sh"});
        std::fs::write(&config, nickel_eval::to_nickel(&config_value).unwrap()).unwrap();
        request["member"] = json!("workflow.ncl");
        assert_eq!(call(&config, &request, "rejected")["code"], "store_error");
        assert!(!missing_dir.exists());
    }

    // r[verify lattice.source_edit.save]
    #[test]
    fn exact_source_owner_rejects_implicit_or_wrong_draft_cas_and_invalid_source() {
        let directory = tempfile::tempdir().unwrap();
        let (config, db, before, bytes) = fixture(directory.path());
        let head = before.catalog_revision.as_ref().unwrap();
        let mut request = target("save", head);
        request["bytes"] = json!(bytes);
        assert_eq!(call(&config, &request, "rejected")["code"], "invalid_request");
        request["expected_draft"] = json!(authoring::ContentIdentity::from_bytes(b"missing draft"));
        assert_eq!(call(&config, &request, "rejected")["code"], "draft_conflict");
        request["expected_draft"] = Value::Null;
        request["bytes"] = json!(b"let private_source =");
        assert_eq!(
            call(&config, &request, "rejected"),
            json!({"schema": SCHEMA, "status": "rejected", "code": "validation_failed"})
        );
        let store = Store::new(db).unwrap();
        assert!(store.load_source_draft("owner-draft").unwrap().is_none());
        assert_eq!(store.load_workflow("source-test").unwrap().unwrap().catalog_revision, before.catalog_revision);
    }

    // r[verify lattice.source_edit.receipt]
    #[test]
    #[ignore = "requires exact captured native worker; host qualification runs ignored tests"]
    fn exact_source_owner_native_receipt_binds_only_the_saved_draft() {
        let directory = tempfile::tempdir().unwrap();
        let (config, db, before, _) = fixture(directory.path());
        let head = before.catalog_revision.as_ref().unwrap();
        let mut save = target("save", head);
        let bytes = nickel_eval::to_nickel(&workflow_to_portable(&flat("source-test", true))).unwrap().into_bytes();
        save["bytes"] = json!(bytes);
        save["expected_draft"] = Value::Null;
        let saved = call(&config, &save, "saved");
        assert_eq!(saved["head"], json!(head));
        assert!(saved.get("bytes").is_none());
        let read = call(&config, &target("view", head), "viewed");
        assert_eq!(read["bytes"], json!(bytes));
        assert_eq!(read["source_revision"], saved["source_revision"]);
        assert_eq!(read["draft_revision"], saved["draft_revision"]);
        assert_eq!(call(&config, &save, "rejected")["code"], "draft_conflict");
        save["expected_draft"] = saved["draft_revision"].clone();
        save["bytes"] = json!(b"let private_source =");
        assert_eq!(call(&config, &save, "rejected")["code"], "validation_failed");
        {
            let store = Store::new(&db).unwrap();
            let draft = store.load_source_draft("owner-draft").unwrap().unwrap();
            assert_eq!(draft.program.files["workflow.ncl"], bytes);
            assert!(draft.diagnostic.is_none());
            assert_eq!(saved["draft_revision"], json!(authoring::identity("lattice.source-draft.v1", &draft).unwrap()));
            let preview = draft.preview.unwrap();
            let source = preview.candidate.authoring.unwrap();
            assert_eq!(preview.candidate.graph.node_count(), 2);
            assert!(preview.compiler.worker_artifact.is_some());
            assert_eq!(
                saved["receipt"],
                json!({"compiler": preview.compiler, "evaluated_output": source.evaluated_output, "expanded_artifact": source.expanded_artifact})
            );
            assert_eq!(saved["source_revision"], json!(preview.input_identity));
            let admitted = store.load_workflow("source-test").unwrap().unwrap();
            assert_eq!(admitted.catalog_revision, before.catalog_revision);
            assert_eq!(admitted.authoring, before.authoring);
            assert_eq!(workflow_to_portable(&admitted), workflow_to_portable(&before));
        }
        let replacement = format!("# second successful edit\n{}", String::from_utf8(bytes).unwrap());
        save["bytes"] = json!(replacement.as_bytes());
        let replaced = call(&config, &save, "saved");
        assert_ne!(saved["draft_revision"], replaced["draft_revision"]);
        assert_eq!(call(&config, &save, "rejected")["code"], "draft_conflict");
        assert_eq!(call(&config, &target("view", head), "viewed")["bytes"], json!(replacement.as_bytes()));
    }
}
