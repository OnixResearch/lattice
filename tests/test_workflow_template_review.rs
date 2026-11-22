//! Review fixes: bound commands are strict simple commands, credentials never shadow values,
//! and schema 1 manifests may not name `mode` or `params`, even as `null`.

use std::path::Path;

use lattice::workflow_template::BundleMode;
use lattice::workflow_template::DiagnosticCode;
use lattice::workflow_template::ParamValue;

const LABEL: &str = "run tool";
const KEY: &str = "LATTICE_PARAM_TOOL";
const WORKFLOW_BYTES: &[u8] = b"captured workflow bytes";
const PREVIEW_WORKFLOW: &str = "builtins/workflow-templates/shell-preview/workflow.ncl";

fn input(command: &str, credentials: &[&str]) -> lattice::workflow_template::BundleValidationInput {
    let mut workflow =
        lattice::engine::Workflow::new("review-fixture", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let node = workflow.add_shell_node(LABEL, command);
    if let Some(lattice::engine::workflow::NodeConfig::Shell { env, .. }) = workflow.configs.get_mut(&node.0) {
        env.insert(KEY.to_owned(), "true".to_owned());
    }
    if !credentials.is_empty() {
        let refs = lattice::credentials::NodeCredentialRefs {
            env: credentials
                .iter()
                .map(|key| ((*key).to_owned(), lattice::credentials::SecretRef::new("env", "TOKEN", None)))
                .collect(),
        };
        workflow.credentials.insert(node.0, refs);
    }
    let kinds = vec!["shell".to_owned()];
    lattice::workflow_template::BundleValidationInput {
        manifest_bytes: b"captured manifest bytes".to_vec(),
        manifest: lattice::workflow_template::BundleManifest {
            schema_version: lattice::workflow_template::CURRENT_BUNDLE_SCHEMA_VERSION,
            id: "review-fixture".into(),
            display_name: "Review fixture".into(),
            description: String::new(),
            category: "tests".into(),
            workflow: lattice::workflow_template::FileBinding {
                path: "workflow.ncl".into(),
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
            mode: Some(BundleMode::Mutating),
            params: Some(vec![lattice::workflow_template::ParamDeclaration {
                name: "tool".into(),
                value_type: lattice::workflow_template::ParamType::Enum {
                    values: vec!["true".into(), "touch".into()],
                },
                required: false,
                default: Some(ParamValue::Text("true".into())),
                bindings: vec![lattice::workflow_template::ParamTarget {
                    node: LABEL.into(),
                    shell_env: KEY.into(),
                }],
            }]),
        },
        files: vec![lattice::workflow_template::BundleFile {
            path: "workflow.ncl".into(),
            bytes: WORKFLOW_BYTES.to_vec(),
        }],
        workflow: Some(workflow),
        orchestration: None,
    }
}

fn code(command: &str, credentials: &[&str]) -> Result<(), DiagnosticCode> {
    lattice::workflow_template::validate_bundle(input(command, credentials))
        .map(|_| ())
        .map_err(|diagnostic| diagnostic.code)
}

// r[verify lattice.workflow_templates.params.scenario.declaration]
#[test]
fn a_value_can_never_choose_the_program_or_reach_shell_syntax() {
    let invalid = Err(DiagnosticCode::InvalidParamDeclaration);
    assert_eq!(code("echo \"$(\"${LATTICE_PARAM_TOOL}\" output.txt)\"", &[]), invalid, "reviewer example");
    assert_eq!(code("\"${LATTICE_PARAM_TOOL}\" output.txt", &[]), invalid, "command position");
    assert_eq!(code("env \"${LATTICE_PARAM_TOOL}\" output.txt", &[]), invalid, "denied program");
    assert_eq!(code("/usr/bin/find . -exec \"${LATTICE_PARAM_TOOL}\"", &[]), invalid, "denied program by path");
    assert_eq!(code("tool -c \"${LATTICE_PARAM_TOOL}\"", &[]), invalid, "denied word");
    assert_eq!(code("tool \"${LATTICE_PARAM_TOOL}\" > out.txt", &[]), invalid, "redirection");
    assert_eq!(code("tool \"${LATTICE_PARAM_TOOL}\"; rm x", &[]), invalid, "control operator");

    let passing = "printf %s \"${LATTICE_PARAM_TOOL}\"";
    assert_eq!(code(passing, &[]), Ok(()));
    let template = lattice::workflow_template::validate_bundle(input(passing, &[])).expect("simple command admits");
    let touch = std::collections::BTreeMap::from([("tool".to_owned(), ParamValue::Text("touch".into()))]);
    let bound = lattice::workflow_template::bind(&template, &touch).expect("value binds");
    let commands = bound.workflow().configs.values().filter_map(|config| match config {
        lattice::engine::workflow::NodeConfig::Shell { command, .. } => Some(command.as_str()),
        _ => None,
    });
    assert_eq!(commands.collect::<Vec<_>>(), [passing], "the value never becomes command text");
}

// r[verify lattice.workflow_templates.params.scenario.declaration]
#[test]
fn credentials_never_shadow_parameter_values() {
    let passing = "printf %s \"${LATTICE_PARAM_TOOL}\"";
    assert_eq!(code(passing, &["API_TOKEN"]), Ok(()));
    assert_eq!(code(passing, &[KEY]), Err(DiagnosticCode::InvalidParamDeclaration), "target key shadowed");
    assert_eq!(code(passing, &["LATTICE_PARAM_OTHER"]), Err(DiagnosticCode::InvalidParamDeclaration));
}

fn load_legacy(extra: &str) -> Vec<DiagnosticCode> {
    let root = tempfile::tempdir().expect("catalog root");
    let bundle = root.path().join("legacy");
    std::fs::create_dir(&bundle).expect("bundle directory");
    let workflow = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(PREVIEW_WORKFLOW)).expect("workflow");
    std::fs::write(bundle.join("workflow.ncl"), &workflow).expect("workflow writes");
    let digest = blake3::hash(&workflow).to_hex();
    let manifest = format!(
        "{{ schema_version = 1, id = \"legacy\", display_name = \"Legacy\", description = \"\", category = \"tests\", workflow = {{ path = \"workflow.ncl\", blake3 = \"{digest}\" }}, assets = [], compatibility = {{ workflow_schema = 1, node_kinds = [\"shell\"], runtime_kinds = [\"shell\"], feature_ids = [\"lattice.workflow-template-bundle.v1\"] }}, operator_notes = [], non_claims = [] {extra} }}"
    );
    std::fs::write(bundle.join("manifest.ncl"), manifest).expect("manifest writes");
    let catalog = lattice::workflow_template::load_catalog(root.path()).expect("catalog loads");
    assert_eq!(catalog.bundles.len() + catalog.diagnostics.len(), 1);
    catalog.diagnostics.iter().map(|diagnostic| diagnostic.code).collect()
}

// r[verify lattice.workflow_templates.mode.scenario.compatibility]
#[test]
fn schema_one_manifests_may_not_name_mode_or_params_even_as_null() {
    assert_eq!(load_legacy(""), Vec::new());
    for extra in [
        ", mode = null",
        ", params = null",
        ", mode = \"mutating\"",
        ", params = []",
    ] {
        assert_eq!(load_legacy(extra), [DiagnosticCode::UnsupportedSchema], "{extra}");
    }
}
