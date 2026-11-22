//! Typed `--input` values for saved workflows: typing, refusal, secrets, records and inspection.

#[path = "support/run_inputs.rs"]
mod fixture;

use std::collections::BTreeMap;

use fixture::Fixture;
use fixture::args;
use lattice::workflow_template::inputs::InputError;
use lattice::workflow_template::inputs::InputKind;
use lattice::workflow_template::inputs::SecretReason;
use lattice::workflow_template::params::ParamErrorCode;

fn admit(
    fixture: &Fixture,
    cli: &[&str],
    file: Option<&str>,
) -> Result<lattice::workflow_template::inputs::Admitted, InputError> {
    let cli = args(cli);
    let file = file.map(|json| fixture.input_file("inputs.json", json));
    let store = fixture.store();
    match lattice::workflow_template::admit_saved_inputs(
        &store,
        fixture::BUNDLE,
        &fixture.request(&cli, file.as_deref()),
    ) {
        Ok(admitted) => Ok(admitted),
        Err(lattice::workflow_template::Error::Inputs(error)) => Err(error),
        Err(other) => panic!("unexpected shell failure: {other}"),
    }
}

fn probe_env(workflow: &lattice::engine::Workflow) -> BTreeMap<String, String> {
    let node = workflow.graph.nodes().find(|node| node.label == fixture::PROBE).expect("probe node");
    match workflow.configs.get(&node.id.0) {
        Some(lattice::engine::workflow::NodeConfig::Shell { env, .. }) => env.clone().into_iter().collect(),
        _ => panic!("probe is a shell node"),
    }
}

fn param_code(result: Result<lattice::workflow_template::inputs::Admitted, InputError>) -> ParamErrorCode {
    match result {
        Err(InputError::Param(error)) => error.code,
        other => panic!("expected a parameter refusal, got {:?}", other.map(|admitted| admitted.identity)),
    }
}

// r[verify lattice.workflow_templates.run_inputs]
// r[verify lattice.workflow_templates.run_inputs.scenario.typed]
#[test]
fn each_type_binds_from_both_sources_into_the_bound_revision() {
    let fixture = Fixture::new();
    let admitted = admit(&fixture, &["enabled=true", "count=5", "target=a/b-1.txt"], Some(r#"{"format": "json"}"#))
        .expect("typed inputs admit");
    let env = probe_env(admitted.bundle.workflow());
    let expected = [
        ("ENABLED", "true"),
        ("COUNT", "5"),
        ("FORMAT", "json"),
        ("TARGET", "a/b-1.txt"),
    ];
    for (key, value) in expected {
        assert_eq!(env.get(&format!("LATTICE_PARAM_{key}")).map(String::as_str), Some(value));
    }
    let names = admitted.identity.params.iter().map(|param| (param.name.as_str(), param.kind)).collect::<Vec<_>>();
    let kinds = [
        ("count", InputKind::Integer),
        ("enabled", InputKind::Bool),
        ("format", InputKind::Enum),
        ("target", InputKind::RelativePath),
    ];
    assert_eq!(names, kinds);
    assert!(admitted.identity.params.iter().all(|param| param.supplied));

    let saved = fixture.store().load_workflow(fixture::BUNDLE).expect("store reads").expect("saved workflow");
    let saved_revision = lattice::orchestration::workflow_revision(&saved).expect("revision");
    assert_ne!(admitted.identity.workflow, saved_revision, "inputs change the bound revision");
    let json_only =
        admit(&fixture, &[], Some(r#"{"format": "text", "count": 1, "enabled": false}"#)).expect("JSON types admit");
    assert_eq!(json_only.identity.workflow, saved_revision, "the saved values reproduce the saved revision");
    let defaults = json_only.identity.params.iter().filter(|param| !param.supplied).map(|param| param.name.as_str());
    assert_eq!(defaults.collect::<Vec<_>>(), ["target"]);

    let again =
        admit(&fixture, &["format=json", "target=a/b-1.txt"], Some(r#"{"enabled": true, "count": 5}"#)).unwrap();
    assert_eq!(again.identity, admitted.identity, "the identity covers values, not their source");
    let grants = |workflow| lattice::engine::effect::standalone::inspect_workflow(workflow).expect("inspects").grants;
    assert_eq!(grants(admitted.bundle.workflow()), grants(&saved), "inputs never change grants");
}

// r[verify lattice.workflow_templates.run_inputs.scenario.refused]
#[test]
fn wrong_unknown_missing_and_duplicate_inputs_are_refused_without_echo() {
    let fixture = Fixture::new();
    let format = ["format=json"];
    let with = |extra: &str| admit(&fixture, &[format[0], extra], None);
    assert_eq!(param_code(with("count=0")), ParamErrorCode::OutOfBounds);
    assert_eq!(param_code(with("count=6")), ParamErrorCode::OutOfBounds);
    for wrong in [
        "count=x",
        "count=+3",
        "count=03",
        "count=2.0",
        "enabled=yes",
        "enabled=TRUE",
    ] {
        assert_eq!(param_code(with(wrong)), ParamErrorCode::WrongType, "{wrong}");
    }
    for path in [
        "target=../up",
        "target=/abs",
        "target=-opt",
        "target=a//b",
        "target=a b",
    ] {
        assert_eq!(param_code(with(path)), ParamErrorCode::OutOfBounds, "{path}");
    }
    assert_eq!(param_code(admit(&fixture, &["format=xml"], None)), ParamErrorCode::OutOfBounds);
    for json in [
        r#"{"format": "json", "enabled": "true"}"#,
        r#"{"format": "json", "count": 2.5}"#,
        r#"{"format": 1}"#,
    ] {
        assert_eq!(param_code(admit(&fixture, &[], Some(json))), ParamErrorCode::WrongType, "{json}");
    }
    let unknown = admit(&fixture, &[format[0], "zzunknownzz=1"], None).unwrap_err();
    assert_eq!(
        unknown,
        InputError::Param(lattice::workflow_template::ParamError {
            code: ParamErrorCode::UnknownParam,
            param: None
        })
    );
    assert!(!unknown.to_string().contains("zzunknownzz"));
    let missing = admit(&fixture, &["count=2"], None);
    assert_eq!(param_code(missing), ParamErrorCode::MissingRequired);
    let duplicate = admit(&fixture, &["format=json", "count=2"], Some(r#"{"count": 3}"#));
    assert_eq!(duplicate.unwrap_err(), InputError::DuplicateInput);
    assert_eq!(admit(&fixture, &["format=json", "format=text"], None).unwrap_err(), InputError::DuplicateInput);
    assert_eq!(admit(&fixture, &["format"], None).unwrap_err(), InputError::MalformedInput);
    assert_eq!(admit(&fixture, &[], Some("[1]")).unwrap_err(), InputError::MalformedFile);
    for refusal in [with("target=../zzvaluezz"), with("count=zzvaluezz")] {
        assert!(!refusal.unwrap_err().to_string().contains("zzvaluezz"), "diagnostics never echo a value");
    }
}

// r[verify lattice.workflow_templates.run_inputs.secrets]
#[test]
fn secret_like_values_names_and_credential_targets_are_refused() {
    let fixture = Fixture::new();
    let token = "Ab1".repeat(11);
    for value in ["ghp_0123456789abcdefABCD", "dir/AKIAIOSFODNN7EXAMPLE", token.as_str()] {
        let refused = admit(&fixture, &["format=json", &format!("target={value}")], None).unwrap_err();
        assert_eq!(refused, InputError::SecretLike {
            param: "target".into(),
            reason: SecretReason::Value
        });
        assert!(!refused.to_string().contains(value));
    }
    admit(
        &fixture,
        &[
            "format=json",
            "target=src/ghp_/0123456789abcdef0123456789abcdef01234567",
        ],
        None,
    )
    .expect("short prefixes and lowercase hex are ordinary paths");

    let template = fixture::catalog_template(&fixture.catalog);
    let declared = template.manifest().params().to_vec();
    let supplied = BTreeMap::new();
    lattice::workflow_template::inputs::screen(&declared, template.workflow(), &supplied).expect("fixture is clean");
    let mut named = declared.clone();
    named[0].name = "api_token".into();
    let by_name = lattice::workflow_template::inputs::screen(&named, template.workflow(), &supplied).unwrap_err();
    assert_eq!(by_name, InputError::SecretLike {
        param: "api_token".into(),
        reason: SecretReason::Name
    });
    let mut keyed = declared.clone();
    keyed[0].bindings[0].shell_env = "LATTICE_PARAM_PRIVATE_KEY".into();
    let by_key = lattice::workflow_template::inputs::screen(&keyed, template.workflow(), &supplied).unwrap_err();
    assert!(matches!(by_key, InputError::SecretLike {
        reason: SecretReason::Name,
        ..
    }));

    let mut credentialed = template.workflow().clone();
    let probe = credentialed.graph.nodes().find(|node| node.label == fixture::PROBE).expect("probe").id;
    let mut refs = lattice::credentials::NodeCredentialRefs::default();
    let reference = lattice::credentials::SecretRef::new("env", "UPSTREAM", None);
    refs.env.insert("LATTICE_PARAM_TARGET".into(), reference);
    credentialed.set_node_credentials(probe, refs);
    let target = lattice::workflow_template::inputs::screen(&declared, &credentialed, &supplied).unwrap_err();
    assert_eq!(target, InputError::SecretLike {
        param: "target".into(),
        reason: SecretReason::CredentialTarget
    });
}

// r[verify lattice.workflow_templates.run_inputs.retained]
#[test]
fn only_an_exact_retained_parameterized_template_accepts_inputs() {
    let plain = Fixture::new();
    let store = plain.store();
    let mut edited = store.load_workflow(fixture::BUNDLE).unwrap().unwrap();
    let marker = edited.graph.nodes().find(|node| node.label == "marker").expect("marker").id;
    edited.configs.insert(marker.0, lattice::engine::workflow::NodeConfig::shell("true"));
    store.save_workflow(&edited).expect("operator edit saves");
    drop(store);
    assert_eq!(admit(&plain, &["format=json"], None).unwrap_err(), InputError::SavedWorkflowDrift);

    let fixture = Fixture::new();
    std::fs::write(fixture.catalog.join(fixture::BUNDLE).join("manifest.ncl"), "{}").expect("catalog edit");
    assert_eq!(admit(&fixture, &["format=json"], None).unwrap_err(), InputError::TemplateUnavailable);

    let unparameterized = Fixture::with_params("[]");
    assert_eq!(admit(&unparameterized, &["format=json"], None).unwrap_err(), InputError::NotParameterized);
    assert_eq!(admit(&unparameterized, &[], Some("{}")).unwrap_err(), InputError::NotParameterized);

    let unsaved = Fixture::new();
    let store = unsaved.store();
    let mut copy = store.load_workflow(fixture::BUNDLE).unwrap().unwrap();
    copy.meta.name = "hand-saved".into();
    copy.catalog_revision = None;
    store.save_workflow(&copy).expect("plain save");
    let cli = args(&["format=json"]);
    let refused = lattice::workflow_template::admit_saved_inputs(&store, "hand-saved", &unsaved.request(&cli, None));
    assert!(matches!(refused, Err(lattice::workflow_template::Error::Inputs(InputError::NotParameterized))));
    let missing = lattice::workflow_template::admit_saved_inputs(&store, "absent", &unsaved.request(&cli, None));
    assert!(matches!(missing, Err(lattice::workflow_template::Error::Inputs(InputError::WorkflowNotFound))));
}

#[test]
fn the_input_file_must_be_a_bounded_regular_file() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let cli = Vec::new();
    let oversized = format!("{{\"format\": \"json\", \"pad\": \"{}\"}}", " ".repeat(65_536));
    let oversized = fixture.input_file("oversized.json", &oversized);
    let missing = fixture.dir.path().join("missing.json");
    for path in [oversized.as_path(), missing.as_path(), fixture.dir.path()] {
        let refused =
            lattice::workflow_template::admit_saved_inputs(&store, fixture::BUNDLE, &fixture.request(&cli, Some(path)));
        assert!(matches!(refused, Err(lattice::workflow_template::Error::InputFile)), "{}", path.display());
    }
    let bounded = fixture.input_file("bounded.json", r#"{"format": "json"}"#);
    lattice::workflow_template::admit_saved_inputs(&store, fixture::BUNDLE, &fixture.request(&cli, Some(&bounded)))
        .expect("a bounded regular file is read");
}

#[test]
fn approval_gates_are_detected_for_refusal() {
    let fixture = Fixture::new();
    let template = fixture::catalog_template(&fixture.catalog);
    assert!(!lattice::workflow_template::inputs::has_approval_gate(template.workflow()));
    let mut gated = template.workflow().clone();
    lattice::tui::authoring::node::create_approval(&mut gated, "gate", "review-gate", "operator").expect("gate adds");
    assert!(lattice::workflow_template::inputs::has_approval_gate(&gated));
}
