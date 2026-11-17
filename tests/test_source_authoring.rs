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
    assert!(shell::preview_summary(Some(&original), candidate).unwrap().contains("Preview only"));
    assert_eq!(store.load_workflow("source-test").unwrap().unwrap().authoring, original.authoring);
    changed.program.parameters.insert("two".into(), serde_json::json!(false));
    assert!(store.admit_source_draft(&changed).is_err(), "mutated input cannot reuse old preview");
    store.preview_source_draft(&mut changed).unwrap();
    assert_eq!(changed.preview.as_ref().unwrap().candidate.graph.node_count(), 1);
    assert_eq!(store.admit_source_draft(&changed).unwrap().graph.node_count(), 1);
}

#[test]
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
fn authoring_source_artifact_and_compiler_identities_are_not_execution_or_semantic_equivalence() {
    let source = program();
    let before = shell::compile(&source).unwrap();
    let mut changed = source.clone();
    changed.files.insert("selected-helper.ncl".into(), b"fun value => value".to_vec());
    let after = shell::compile(&changed).unwrap();
    assert_ne!(source.identity().unwrap(), changed.identity().unwrap());
    assert_eq!(authoring::expanded_identity(&before).unwrap(), authoring::expanded_identity(&after).unwrap());
    assert_eq!(workflow_to_portable(&before), workflow_to_portable(&after));
    assert!(authoring::COMPLETE_MODULE_PROVENANCE_UNAVAILABLE.contains("unavailable"));
    assert!(authoring::ORIGIN_UNAVAILABLE.contains("unavailable"));
    // Labels remain in materialized identity: config/expression/runtime binding
    // semantics are not inferred away to manufacture a logical-equivalence claim.
    let mut relabeled = before.clone();
    relabeled.graph.node_mut(lattice::engine::workflow::NodeId(1)).unwrap().label =
        "diagnostic-or-binding-not-inferred".into();
    assert_ne!(authoring::expanded_identity(&before).unwrap(), authoring::expanded_identity(&relabeled).unwrap());
}

#[test]
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
    store.save_workflow_cas(&candidate, None).unwrap();
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
        std::process::Command::new(env!("CARGO_BIN_EXE_lattice"))
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

#[test]
fn authoring_catalog_delete_recreate_does_not_allow_revision_aba() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let original = admitted(&store);
    let mut stale = draft(&store, "before-delete");
    store.preview_source_draft(&mut stale).unwrap();
    store.delete_workflow("source-test").unwrap();
    let mut recreated = original.clone();
    recreated.catalog_revision = None;
    let recreated = store.save_workflow_cas(&recreated, None).unwrap();
    assert_ne!(recreated.catalog_revision, original.catalog_revision);
    assert!(store.admit_source_draft(&stale).is_err());
}

#[test]
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
fn authoring_native_import_admission_rejects_unused_imports_but_not_comments_or_strings() {
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
    let source = format!("let never_called = fun value => import \"/must-not-be-opened.ncl\" in ({flat})");
    let program = CapturedProgram {
        entry: "workflow.ncl".into(),
        files: BTreeMap::from([("workflow.ncl".into(), source.into_bytes())]),
        kind: ProgramKind::Workflow,
        parameters: BTreeMap::new(),
    };
    assert_eq!(
        shell::compile(&program).unwrap_err(),
        "captured module import evaluation is unsupported; source preview requires import-free entry"
    );
}

#[test]
fn authoring_ordinary_ambient_import_remains_compatible_but_captured_module_edit_cannot_preview() {
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
    assert!(draft.diagnostic.as_ref().unwrap().contains("unsupported"));
    assert!(draft.preview.is_none());
    assert_eq!(store.load_workflow("source-test").unwrap().unwrap().catalog_revision, saved.catalog_revision);
    assert_eq!(store.load_workflow("source-test").unwrap().unwrap().graph.node_count(), 1);
}

#[test]
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
    let imported = store.admit_imported_workflow(&candidate, newer.catalog_revision.as_ref()).unwrap();
    assert_eq!(workflow_to_portable(&imported), workflow_to_portable(&candidate));
    assert_eq!(imported.authoring.unwrap().state, AssociationState::RetainedEntrySnapshot);
}
