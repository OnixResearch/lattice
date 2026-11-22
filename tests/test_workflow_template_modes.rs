//! Workflow-template bundle modes, schema compatibility, and parameter admission.

use std::collections::BTreeMap;
use std::path::Path;

use lattice::engine::workflow::NodeConfig;
use lattice::file_node::FileOperationKind;
use lattice::workflow_template::ActivationBlock;
use lattice::workflow_template::ActivationPlan;
use lattice::workflow_template::ApplicationPlan;
use lattice::workflow_template::BundleMode;
use lattice::workflow_template::DiagnosticCode;
use lattice::workflow_template::classification::EffectClass;

const WORKFLOW_FILE: &str = "workflow.ncl";
const WORKFLOW_BYTES: &[u8] = b"captured workflow bytes";
const PREVIEW_ROOT: &str = "builtins/workflow-templates/shell-preview";
const REVIEW_ROOT: &str = "builtins/workflow-templates/agent-evaluation-catalog-review";

fn input(
    mode: Option<BundleMode>,
    workflow: lattice::engine::Workflow,
) -> lattice::workflow_template::BundleValidationInput {
    let kinds = vec!["shell".to_string(), "file".to_string(), "http".to_string()];
    lattice::workflow_template::BundleValidationInput {
        manifest_bytes: b"captured manifest bytes".to_vec(),
        manifest: lattice::workflow_template::BundleManifest {
            schema_version: lattice::workflow_template::CURRENT_BUNDLE_SCHEMA_VERSION,
            id: "mode-fixture".into(),
            display_name: "Mode fixture".into(),
            description: String::new(),
            category: "tests".into(),
            workflow: lattice::workflow_template::FileBinding {
                path: WORKFLOW_FILE.into(),
                blake3: lattice::workflow_template::ContentIdentity::from_bytes(WORKFLOW_BYTES),
            },
            orchestration: None,
            assets: Vec::new(),
            compatibility: lattice::workflow_template::CompatibilityRequirements {
                workflow_schema: lattice::workflow_template::CURRENT_WORKFLOW_SCHEMA_VERSION,
                orchestration_schema: None,
                node_kinds: kinds.clone(),
                runtime_kinds: kinds,
                feature_ids: vec!["lattice.workflow-template-bundle.v1".into()],
            },
            operator_notes: Vec::new(),
            non_claims: Vec::new(),
            mode,
            params: mode.map(|_| Vec::new()),
        },
        files: vec![lattice::workflow_template::BundleFile {
            path: WORKFLOW_FILE.into(),
            bytes: WORKFLOW_BYTES.to_vec(),
        }],
        workflow: Some(workflow),
        orchestration: None,
    }
}

fn workflow(build: impl FnOnce(&mut lattice::engine::Workflow)) -> lattice::engine::Workflow {
    let mut workflow =
        lattice::engine::Workflow::new("mode-fixture", lattice::engine::workflow::WorkflowTimestamp::epoch());
    build(&mut workflow);
    workflow
}

fn file_read(workflow: &mut lattice::engine::Workflow) {
    workflow.add_file_node("read", FileOperationKind::Read, "input.txt");
}

fn validate(input: lattice::workflow_template::BundleValidationInput) -> Result<BundleMode, DiagnosticCode> {
    lattice::workflow_template::validate_bundle(input)
        .map(|bundle| bundle.manifest().mode())
        .map_err(|diagnostic| diagnostic.code)
}

// r[verify lattice.workflow_templates.mode.scenario.classification]
#[test]
fn effects_are_classified_conservatively() {
    let classes = [
        (NodeConfig::file(FileOperationKind::Read, "a"), EffectClass::ReadOnly),
        (NodeConfig::file(FileOperationKind::Write, "a"), EffectClass::Mutating),
        (NodeConfig::file(FileOperationKind::Append, "a"), EffectClass::Mutating),
        (NodeConfig::image_input("a.png"), EffectClass::ReadOnly),
        (NodeConfig::shell("cat a"), EffectClass::Mutating),
        (NodeConfig::http("GET", "https://example.invalid"), EffectClass::Mutating),
    ];
    for (config, expected) in classes {
        let actual = lattice::workflow_template::classification::classify_node(&config);
        assert_eq!(actual, expected, "{}", config.kind_label());
    }
}

// r[verify lattice.workflow_templates.mode.scenario.enforced]
#[test]
fn read_only_bundles_admit_only_provably_read_only_effects() {
    assert_eq!(validate(input(Some(BundleMode::ReadOnly), workflow(file_read))), Ok(BundleMode::ReadOnly));
    assert_eq!(validate(input(Some(BundleMode::Mutating), workflow(file_read))), Ok(BundleMode::Mutating));
    for mutating in [
        workflow(|workflow| {
            workflow.add_shell_node("cat", "cat input.txt");
        }),
        workflow(|workflow| {
            workflow.add_file_node("write", FileOperationKind::Write, "out.txt");
        }),
        workflow(|workflow| {
            workflow.add_http_node("get", "GET", "https://example.invalid");
        }),
        workflow(|workflow| {
            file_read(workflow);
            workflow.add_shell_node("cat", "true");
        }),
    ] {
        assert_eq!(validate(input(Some(BundleMode::ReadOnly), mutating.clone())), Err(DiagnosticCode::ModeViolation));
        assert_eq!(validate(input(Some(BundleMode::Mutating), mutating)), Ok(BundleMode::Mutating));
    }
}

// r[verify lattice.workflow_templates.mode.scenario.compatibility]
#[test]
fn schema_one_loads_as_mutating_and_mixed_schemas_fail_clearly() {
    let shell = || {
        workflow(|workflow| {
            workflow.add_shell_node("cat", "true");
        })
    };
    let mut legacy = input(None, shell());
    legacy.manifest.schema_version = lattice::workflow_template::LEGACY_BUNDLE_SCHEMA_VERSION;
    let legacy = lattice::workflow_template::validate_bundle(legacy).expect("schema 1 manifests load");
    assert_eq!((legacy.manifest().mode(), legacy.manifest().params().len()), (BundleMode::Mutating, 0));
    assert!(lattice::workflow_template::bind(&legacy, &BTreeMap::new()).is_ok());

    let mut legacy_with_mode = input(Some(BundleMode::Mutating), shell());
    legacy_with_mode.manifest.schema_version = lattice::workflow_template::LEGACY_BUNDLE_SCHEMA_VERSION;
    assert_eq!(validate(legacy_with_mode), Err(DiagnosticCode::UnsupportedSchema));
    assert_eq!(validate(input(None, shell())), Err(DiagnosticCode::InvalidManifest));
    let mut missing_params = input(Some(BundleMode::Mutating), shell());
    missing_params.manifest.params = None;
    assert_eq!(validate(missing_params), Err(DiagnosticCode::InvalidManifest));
    let mut future = input(Some(BundleMode::Mutating), shell());
    future.manifest.schema_version = lattice::workflow_template::CURRENT_BUNDLE_SCHEMA_VERSION + 1;
    assert_eq!(validate(future), Err(DiagnosticCode::UnsupportedSchema));

    let mut manifest = serde_json::to_value(legacy.manifest()).expect("manifest serializes");
    manifest["schema_version"] = serde_json::json!(2);
    manifest["params"] = serde_json::json!([]);
    manifest["mode"] = serde_json::json!("readonly");
    assert!(serde_json::from_value::<lattice::workflow_template::BundleManifest>(manifest).is_err());
}

// r[verify lattice.workflow_templates.mode.scenario.enforced]
// r[verify lattice.workflow_templates.activation.scenario.blocked]
#[test]
fn save_and_start_require_bound_values_and_an_admitted_mode() {
    let template = lattice::workflow_template::validate_bundle(input(
        Some(BundleMode::Mutating),
        workflow(|workflow| {
            workflow.add_shell_node("cat", "true");
        }),
    ))
    .expect("fixture validates");
    let facts = lattice::workflow_template::ActivationFacts {
        explicit_save_requested: true,
        explicit_start_requested: true,
        saved_workflow_revision: Some(template.workflow_revision().clone()),
        current_orchestration_revision: None,
        runtime_admitted: true,
        effects_admitted: true,
        credentials_admitted: true,
        triggers_admitted: true,
        positive_budgets_admitted: true,
        actor_authority_admitted: true,
    };
    let missing = lattice::workflow_template::ApplicationFacts {
        workflow: lattice::workflow_template::ExistingContent::Missing,
        orchestration: None,
    };
    let unbound = ActivationPlan::Blocked(ActivationBlock::ParamsUnbound);
    assert_eq!(lattice::workflow_template::plan_activation(&template, &facts), unbound);
    let conflict = ApplicationPlan::Conflict(DiagnosticCode::ParamsUnbound);
    assert_eq!(lattice::workflow_template::plan_application(&template, &missing), conflict);
    let directory = tempfile::tempdir().expect("store directory");
    let store = lattice::store::Store::new(directory.path().join("modes.redb")).expect("store opens");
    let error = lattice::workflow_template::save_draft(&store, &template).expect_err("unbound save fails");
    assert!(matches!(error, lattice::workflow_template::Error::ParamsUnbound));
    assert!(store.list_workflows().expect("workflows list").is_empty());

    let bound = lattice::workflow_template::bind(&template, &BTreeMap::new()).expect("defaults bind");
    assert!(matches!(
        lattice::workflow_template::plan_application(&bound, &missing),
        ApplicationPlan::CreateDraft { .. }
    ));
    // Tampering with bound content after `bind` is covered by the crate-internal
    // `workflow_template::tests`; fields are not writable from outside the crate.
}

fn copy_bundle(source: &str, target: &Path) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join(source);
    std::fs::create_dir_all(target).expect("bundle copy directory creates");
    for entry in std::fs::read_dir(source).expect("built-in bundle reads") {
        let entry = entry.expect("bundle entry reads");
        std::fs::copy(entry.path(), target.join(entry.file_name())).expect("bundle file copies");
    }
}

fn load_edited(source: &str, file: &str, edit: impl FnOnce(String) -> String) -> Vec<DiagnosticCode> {
    let root = tempfile::tempdir().expect("catalog root");
    let id = Path::new(source).file_name().expect("bundle id");
    let bundle = root.path().join(id);
    copy_bundle(source, &bundle);
    let text = std::fs::read_to_string(bundle.join(file)).expect("bundle file reads");
    std::fs::write(bundle.join(file), edit(text)).expect("bundle file writes");
    let catalog = lattice::workflow_template::load_catalog(root.path()).expect("catalog loads");
    catalog.diagnostics.iter().map(|diagnostic| diagnostic.code).collect()
}

fn repin(text: String, workflow: &str) -> String {
    let start = text.find("blake3 = \"").expect("workflow pin") + "blake3 = \"".len();
    let digest = blake3::hash(workflow.as_bytes()).to_hex().to_string();
    format!("{}{digest}{}", &text[..start], &text[start + digest.len()..])
}

// r[verify lattice.workflow_templates.params.scenario.declaration]
// r[verify lattice.workflow_templates.mode.scenario.enforced]
#[test]
fn edited_manifests_and_bindings_fail_closed_at_load() {
    assert_eq!(load_edited(PREVIEW_ROOT, "manifest.ncl", |text| text), Vec::new());
    let read_only = load_edited(PREVIEW_ROOT, "manifest.ncl", |text| text.replace("\"mutating\"", "\"read_only\""));
    assert_eq!(read_only, [DiagnosticCode::ModeViolation]);
    let drifted_default =
        load_edited(REVIEW_ROOT, "manifest.ncl", |text| text.replace("\"builtins/agent-evaluations\"", "\"other\""));
    assert_eq!(drifted_default, [DiagnosticCode::InvalidParamDeclaration]);
    let missing_node = load_edited(REVIEW_ROOT, "manifest.ncl", |text| {
        text.replace("node = \"Validate evaluation catalog\"", "node = \"Missing node\"")
    });
    assert_eq!(missing_node, [DiagnosticCode::InvalidParamDeclaration]);
    let unpinned = load_edited(REVIEW_ROOT, WORKFLOW_FILE, |text| text.replace("value = \"builtins/", "value = \"x/"));
    assert_eq!(unpinned, [DiagnosticCode::IdentityMismatch]);

    let workflow = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(REVIEW_ROOT).join(WORKFLOW_FILE))
        .expect("review workflow reads");
    let unquoted = workflow.replace("\\\"${LATTICE_PARAM_CATALOG_ROOT}\\\"", "${LATTICE_PARAM_CATALOG_ROOT}");
    assert_ne!(unquoted, workflow);
    let root = tempfile::tempdir().expect("unquoted catalog root");
    let bundle = root.path().join("agent-evaluation-catalog-review");
    copy_bundle(REVIEW_ROOT, &bundle);
    std::fs::write(bundle.join(WORKFLOW_FILE), &unquoted).expect("unquoted workflow writes");
    let manifest = std::fs::read_to_string(bundle.join("manifest.ncl")).expect("manifest reads");
    std::fs::write(bundle.join("manifest.ncl"), repin(manifest, &unquoted)).expect("repinned manifest writes");
    let catalog = lattice::workflow_template::load_catalog(root.path()).expect("unquoted catalog loads");
    let codes = catalog.diagnostics.iter().map(|diagnostic| diagnostic.code).collect::<Vec<_>>();
    assert_eq!(codes, [DiagnosticCode::InvalidParamDeclaration], "a re-pinned unquoted reference still fails");
}
