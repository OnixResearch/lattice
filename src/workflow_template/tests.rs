//! Bound content tampered after `bind` is refused at save, activation planning, and start.

const LABEL: &str = "run tool";
const KEY: &str = "LATTICE_PARAM_ROOT";
const WORKFLOW_BYTES: &[u8] = b"captured workflow bytes";

fn template() -> super::ValidatedBundle {
    let mut workflow =
        crate::engine::Workflow::new("tamper-fixture", crate::engine::workflow::WorkflowTimestamp::epoch());
    let node = workflow.add_shell_node(LABEL, "tool --root \"${LATTICE_PARAM_ROOT}\"");
    if let Some(crate::engine::workflow::NodeConfig::Shell { env, .. }) = workflow.configs.get_mut(&node.0) {
        env.insert(KEY.to_owned(), ".".to_owned());
    }
    let kinds = vec!["shell".to_owned()];
    super::validate_bundle(super::BundleValidationInput {
        manifest_bytes: b"captured manifest bytes".to_vec(),
        manifest: super::BundleManifest {
            schema_version: super::CURRENT_BUNDLE_SCHEMA_VERSION,
            id: "tamper-fixture".into(),
            display_name: "Tamper fixture".into(),
            description: String::new(),
            category: "tests".into(),
            workflow: super::FileBinding {
                path: "workflow.ncl".into(),
                blake3: super::ContentIdentity::from_bytes(WORKFLOW_BYTES),
            },
            orchestration: None,
            assets: Vec::new(),
            compatibility: super::CompatibilityRequirements {
                workflow_schema: super::CURRENT_WORKFLOW_SCHEMA_VERSION,
                orchestration_schema: None,
                node_kinds: kinds.clone(),
                runtime_kinds: kinds,
                feature_ids: vec!["lattice.workflow-template-bundle.v1".into()],
            },
            operator_notes: Vec::new(),
            non_claims: Vec::new(),
            mode: Some(super::BundleMode::Mutating),
            params: Some(vec![super::ParamDeclaration {
                name: "root".into(),
                value_type: super::ParamType::RelativePath { max_bytes: 64 },
                required: false,
                default: Some(super::ParamValue::Text(".".into())),
                bindings: vec![super::ParamTarget {
                    node: LABEL.into(),
                    shell_env: KEY.into(),
                }],
            }]),
        },
        files: vec![super::BundleFile {
            path: "workflow.ncl".into(),
            bytes: WORKFLOW_BYTES.to_vec(),
        }],
        workflow: Some(workflow),
        orchestration: None,
    })
    .expect("tamper fixture validates")
}

fn bound() -> super::ValidatedBundle {
    super::bind(&template(), &std::collections::BTreeMap::new()).expect("defaults bind")
}

fn shell_mut(bundle: &mut super::ValidatedBundle) -> (&mut String, &mut std::collections::HashMap<String, String>) {
    match bundle.workflow.configs.values_mut().next() {
        Some(crate::engine::workflow::NodeConfig::Shell { command, env, .. }) => (command, env),
        _ => panic!("fixture node is a shell node"),
    }
}

fn start_facts(bundle: &super::ValidatedBundle) -> super::ActivationFacts {
    super::ActivationFacts {
        explicit_save_requested: false,
        explicit_start_requested: true,
        saved_workflow_revision: Some(bundle.workflow_revision.clone()),
        current_orchestration_revision: bundle.orchestration_revision.clone(),
        runtime_admitted: true,
        effects_admitted: true,
        credentials_admitted: true,
        triggers_admitted: true,
        positive_budgets_admitted: true,
        actor_authority_admitted: true,
    }
}

fn tampered() -> Vec<(&'static str, super::ValidatedBundle)> {
    let mut command = bound();
    *shell_mut(&mut command).0 = "rm -rf --root \"${LATTICE_PARAM_ROOT}\"".to_owned();
    let mut value = bound();
    shell_mut(&mut value).1.insert(KEY.to_owned(), "other".to_owned());
    let mut revision = bound();
    *shell_mut(&mut revision).0 = "touch --root \"${LATTICE_PARAM_ROOT}\"".to_owned();
    revision.workflow_revision = crate::orchestration::workflow_revision(&revision.workflow).expect("revision");
    let mut values = bound();
    let other = std::collections::BTreeMap::from([("root".to_owned(), super::ParamValue::Text("x".into()))]);
    if let Some(bound) = values.params.as_mut() {
        bound.resolved = super::params::resolve(values.manifest.params(), &other).expect("other values resolve");
    }
    let mut mode = bound();
    mode.manifest.mode = Some(super::BundleMode::ReadOnly);
    let mut orchestration = bound();
    orchestration.orchestration_revision =
        Some(crate::orchestration::DefinitionRevision::try_from_hex("a".repeat(64)).expect("revision fixture"));
    let mut template_identity = bound();
    if let Some(bound) = template_identity.params.as_mut() {
        bound.template = super::ContentIdentity::from_bytes(b"another template");
    }
    let mut identity = bound();
    identity.identity = super::ContentIdentity::from_bytes(b"another bound identity");
    let mut credential = bound();
    add_param_credential(&mut credential.workflow);
    vec![
        ("command", command),
        ("value", value),
        ("revision", revision),
        ("values", values),
        ("mode", mode),
        ("orchestration", orchestration),
        ("template", template_identity),
        ("identity", identity),
        ("credential", credential),
    ]
}

fn add_param_credential(workflow: &mut crate::engine::Workflow) {
    let node = workflow.configs.keys().next().copied().expect("fixture node");
    let secret = crate::credentials::SecretRef::new("env", "TOKEN", None);
    let refs = crate::credentials::NodeCredentialRefs {
        env: std::collections::HashMap::from([(KEY.to_owned(), secret)]),
    };
    workflow.credentials.insert(node, refs);
}

#[test]
fn each_reverify_comparison_reports_its_own_drift() {
    let message = |case: &str| {
        let (_, bundle) = tampered().into_iter().find(|(name, _)| *name == case).expect("case exists");
        super::reverify(&bundle).expect_err(case).message
    };
    assert_eq!(message("orchestration"), "bound orchestration revision differs");
    assert_eq!(message("template"), "bound template identity differs");
    assert_eq!(message("identity"), "bound identity differs");
    assert_eq!(message("revision"), "bound workflow revision differs");
    assert_eq!(message("credential"), "bound workflow does not hold the resolved values");
}

#[test]
fn bind_refuses_a_parameter_credential() {
    let mut template = template();
    add_param_credential(&mut template.workflow);
    let refused = super::bind(&template, &std::collections::BTreeMap::new()).expect_err("credential shadows a value");
    assert_eq!(refused.code, super::DiagnosticCode::InvalidParamDeclaration);
}

#[test]
fn untampered_bound_content_reverifies() {
    let bundle = bound();
    assert!(super::reverify(&bundle).is_ok());
    assert!(super::reverify(&template()).is_err(), "an unbound template never verifies as bound");
}

#[test]
fn tampered_bound_content_is_refused_at_every_boundary() {
    let directory = tempfile::tempdir().expect("store directory");
    let store = crate::store::Store::new(directory.path().join("tamper.redb")).expect("store opens");
    let missing = super::ApplicationFacts {
        workflow: super::ExistingContent::Missing,
        orchestration: None,
    };
    for (case, bundle) in tampered() {
        let code = super::reverify(&bundle).expect_err(case).code;
        let expected_block = if case == "mode" {
            assert_eq!(code, super::DiagnosticCode::ModeViolation, "{case}");
            super::ActivationBlock::ModeViolation
        } else {
            assert_eq!(code, super::DiagnosticCode::BoundContentDrift, "{case}");
            super::ActivationBlock::BoundContentDrift
        };
        assert_eq!(super::plan_application(&bundle, &missing), super::ApplicationPlan::Conflict(code), "{case}");
        let blocked = super::ActivationPlan::Blocked(expected_block);
        assert_eq!(super::plan_activation(&bundle, &start_facts(&bundle)), blocked, "{case}");
        let saved = super::save_draft(&store, &bundle).expect_err(case);
        assert!(matches!(saved, super::Error::BoundContentDrift), "{case}");
        let started = super::shell::admit_activation(&store, &bundle, &start_facts(&bundle), "operator:local");
        assert!(matches!(started, Err(super::Error::ActivationBlocked(block)) if block == expected_block), "{case}");
    }
    assert!(store.list_workflows().expect("workflows list").is_empty(), "nothing tampered was saved");
}
