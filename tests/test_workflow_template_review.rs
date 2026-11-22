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
fn shell_builtins_wrappers_and_case_variants_are_refused_in_any_word() {
    let invalid = Err(DiagnosticCode::InvalidParamDeclaration);
    for command in [
        "trap \"${LATTICE_PARAM_TOOL}\" EXIT",
        "jobs -x \"${LATTICE_PARAM_TOOL}\" out",
        "coproc \"${LATTICE_PARAM_TOOL}\" out",
        "mapfile -c1 -C \"${LATTICE_PARAM_TOOL}\" lines",
        "enable -f \"${LATTICE_PARAM_TOOL}\" x",
        "ENV \"${LATTICE_PARAM_TOOL}\"",
        "tool sh \"${LATTICE_PARAM_TOOL}\"",
        "tool l BASH \"${LATTICE_PARAM_TOOL}\"",
        "- \"${LATTICE_PARAM_TOOL}\"",
    ] {
        assert_eq!(code(command, &[]), invalid, "{command:?}");
    }
    assert_eq!(code("tar -xf - -C \"${LATTICE_PARAM_TOOL}\"", &[]), Ok(()), "`-` stays valid as an argument");
}

// r[verify lattice.workflow_templates.params.scenario.declaration]
#[test]
fn receiving_node_cwd_and_env_hold_no_expressions() {
    for place in ["cwd", "env value", "env key"] {
        let mut fixture = input("printf %s \"${LATTICE_PARAM_TOOL}\"", &[]);
        let workflow = fixture.workflow.as_mut().expect("fixture workflow");
        let producer = workflow.add_shell_node("producer", "printf x");
        let port = workflow.graph.nodes().find(|node| node.id == producer).expect("producer").output_ports[0].id;
        let expression = format!("{{{{node:{}:port:{}}}}}", producer.0, port.0);
        let receiving = workflow.graph.nodes().find(|node| node.label == LABEL).expect("receiving node").id;
        let Some(lattice::engine::workflow::NodeConfig::Shell { cwd, env, .. }) =
            workflow.configs.get_mut(&receiving.0)
        else {
            panic!("receiving node is a shell node");
        };
        match place {
            "cwd" => *cwd = Some(expression),
            "env value" => drop(env.insert("PATH".to_owned(), expression)),
            _ => drop(env.insert(expression, "x".to_owned())),
        }
        let declared = fixture.manifest.params.clone().expect("fixture params");
        let pure = lattice::workflow_template::params::validate_bindings(
            &declared,
            fixture.workflow.as_ref().expect("workflow"),
        );
        assert_eq!(pure.expect_err(place).code, lattice::workflow_template::params::ParamErrorCode::TargetExpression);
        let loaded = lattice::workflow_template::validate_bundle(fixture)
            .map(|_| ())
            .map_err(|diagnostic| diagnostic.code);
        assert_eq!(loaded, Err(DiagnosticCode::ParamTargetExpression), "{place}");
    }
}

fn env_code(command: &str, env: &[(&str, &str)]) -> Result<(), DiagnosticCode> {
    let mut fixture = input(command, &[]);
    let workflow = fixture.workflow.as_mut().expect("fixture workflow");
    let receiving = workflow.graph.nodes().find(|node| node.label == LABEL).expect("receiving node").id;
    if let Some(lattice::engine::workflow::NodeConfig::Shell { env: node_env, .. }) =
        workflow.configs.get_mut(&receiving.0)
    {
        node_env.extend(env.iter().map(|(key, value)| ((*key).to_owned(), (*value).to_owned())));
    }
    lattice::workflow_template::validate_bundle(fixture)
        .map(|_| ())
        .map_err(|diagnostic| diagnostic.code)
}

// r[verify lattice.workflow_templates.params.scenario.declaration]
#[test]
fn receiving_node_env_cannot_let_the_shell_run_a_value() {
    let printf = "printf %s \"${LATTICE_PARAM_TOOL}\"";
    let refused = Err(DiagnosticCode::ParamTargetEnv);
    assert_eq!(env_code(printf, &[("BASH_FUNC_printf%%", "() { \"$2\" out; }")]), refused, "exported function");
    assert_eq!(env_code(printf, &[("BASH_FUNC_printf", "x")]), refused, "BASH_FUNC_ key");
    assert_eq!(env_code(printf, &[("printf%%", "x")]), refused, "%% key");
    let subscript = "a[$(\"${LATTICE_PARAM_TOOL}\" out)]";
    assert_eq!(env_code(printf, &[("x", subscript)]), refused, "arithmetic subscript");
    assert_eq!(env_code(printf, &[("x", "a[`id`]")]), refused, "backtick");
    for evaluator in ["let x", "declare -i y=x", "typeset -i y=x", "local -i y=x"] {
        let command = format!("{evaluator} \"${{LATTICE_PARAM_TOOL}}\"");
        assert_eq!(env_code(&command, &[]), Err(DiagnosticCode::InvalidParamDeclaration), "{evaluator}");
    }
    assert_eq!(env_code(printf, &[("BASH_ENV", "${LATTICE_PARAM_TOOL}")]), refused, "BASH_ENV sources a value");
    for key in ["BASH_ENV", "ENV", "SHELLOPTS", "BASHOPTS", "PS4"] {
        assert_eq!(env_code(printf, &[(key, "plain")]), refused, "{key}");
    }

    assert_eq!(env_code(printf, &[("RUST_LOG", "info")]), Ok(()), "ordinary static env stays valid");
    let mut fixture = input(printf, &[]);
    let workflow = fixture.workflow.as_mut().expect("fixture workflow");
    if let Some(lattice::engine::workflow::NodeConfig::Shell { env, .. }) = workflow.configs.values_mut().next() {
        env.insert("RUST_LOG".to_owned(), "info".to_owned());
    }
    let template = lattice::workflow_template::validate_bundle(fixture).expect("ordinary env admits");
    let touch = std::collections::BTreeMap::from([("tool".to_owned(), ParamValue::Text("touch".into()))]);
    assert!(lattice::workflow_template::bind(&template, &touch).is_ok(), "ordinary env binds");
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
