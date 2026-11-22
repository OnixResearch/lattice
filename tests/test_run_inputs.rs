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
    match fixture.admit_inputs(&fixture.store(), fixture::BUNDLE, &cli, file.as_deref()) {
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
        other => panic!("expected a parameter refusal, got {:?}", other.map(|admitted| admitted.identity().clone())),
    }
}

// r[verify lattice.workflow_templates.run_inputs]
// r[verify lattice.workflow_templates.run_inputs.scenario.typed]
#[test]
fn each_type_binds_from_both_sources_into_the_bound_revision() {
    let fixture = Fixture::new();
    let admitted = admit(&fixture, &["enabled=true", "count=5", "target=a/b-1.txt"], Some(r#"{"format": "json"}"#))
        .expect("typed inputs admit");
    let env = probe_env(admitted.bundle().workflow());
    let expected = [
        ("ENABLED", "true"),
        ("COUNT", "5"),
        ("FORMAT", "json"),
        ("TARGET", "a/b-1.txt"),
    ];
    for (key, value) in expected {
        assert_eq!(env.get(&format!("LATTICE_PARAM_{key}")).map(String::as_str), Some(value));
    }
    let names = admitted.identity().params.iter().map(|param| (param.name.as_str(), param.kind)).collect::<Vec<_>>();
    let kinds = [
        ("count", InputKind::Integer),
        ("enabled", InputKind::Bool),
        ("format", InputKind::Enum),
        ("target", InputKind::RelativePath),
    ];
    assert_eq!(names, kinds);
    assert!(admitted.identity().params.iter().all(|param| param.supplied));

    let saved = fixture.store().load_workflow(fixture::BUNDLE).expect("store reads").expect("saved workflow");
    let saved_revision = lattice::orchestration::workflow_revision(&saved).expect("revision");
    assert_ne!(admitted.identity().workflow, saved_revision, "inputs change the bound revision");
    let json_only =
        admit(&fixture, &[], Some(r#"{"format": "text", "count": 1, "enabled": false}"#)).expect("JSON types admit");
    assert_eq!(json_only.identity().workflow, saved_revision, "the saved values reproduce the saved revision");
    let defaults = json_only.identity().params.iter().filter(|param| !param.supplied).map(|param| param.name.as_str());
    assert_eq!(defaults.collect::<Vec<_>>(), ["target"]);

    let again =
        admit(&fixture, &["format=json", "target=a/b-1.txt"], Some(r#"{"enabled": true, "count": 5}"#)).unwrap();
    assert_eq!(again.identity(), admitted.identity(), "the identity covers values, not their source");
    let grants = |workflow| lattice::engine::effect::standalone::inspect_workflow(workflow).expect("inspects").grants;
    assert_eq!(grants(admitted.bundle().workflow()), grants(&saved), "inputs never change grants");
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
// r[verify lattice.workflow_templates.run_inputs.secrets.scenario.screened]
#[test]
fn token_shaped_paths_are_refused_and_ordinary_refs_admit() {
    let fixture = Fixture::new();
    let shaped = "aB3dE5fG7hJ9kL1mN3pQ".repeat(2);
    let github = format!("ghp_{shaped}");
    for value in [github.as_str(), "dir/AKIAIOSFODNN7EXAMPLE", shaped.as_str()] {
        let refused = admit(&fixture, &["format=json", &format!("target={value}")], None).unwrap_err();
        assert_eq!(refused, InputError::SecretLike {
            param: "target".into(),
            reason: SecretReason::Value
        });
        assert!(!refused.to_string().contains(value));
    }
    for value in [
        "dependabot/npm_and_yarn/express-4.18.2",
        ".github/workflows/pypi-publish.yml",
        "scripts/hf_upload_model.py",
        "data/ASIA_PACIFIC_REGIONS.csv",
        "feature/PROJ-1234-add-retry-to-upload-client",
        "src/ghp_/0123456789abcdef0123456789abcdef01234567",
    ] {
        admit(&fixture, &["format=json", &format!("target={value}")], None).expect(value);
    }
}

// r[verify lattice.workflow_templates.run_inputs.retained]
// r[verify lattice.workflow_templates.run_inputs.retained.scenario.exact]
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
    let load = admit(&fixture, &["format=json"], None).unwrap_err();
    assert!(
        matches!(load, InputError::TemplateLoad(_)),
        "the bound bundle's load diagnostic is surfaced: {load}"
    );

    let unparameterized = Fixture::with_params("[]");
    assert_eq!(admit(&unparameterized, &["format=json"], None).unwrap_err(), InputError::NotParameterized);
    assert_eq!(admit(&unparameterized, &[], Some("{}")).unwrap_err(), InputError::NotParameterized);

    let gated = Fixture::gated();
    assert_eq!(admit(&gated, &["format=json"], None).unwrap_err(), InputError::ApprovalGate);

    let other = Fixture::new();
    let store = other.store();
    let cli = args(&["format=json"]);
    let missing = other.admit_inputs(&store, "absent", &cli, None);
    assert!(matches!(missing, Err(lattice::workflow_template::Error::Inputs(InputError::WorkflowNotFound))));
}

/// The fixture template bound to a value for every parameter, none of them its default.
fn non_default_bound(fixture: &Fixture) -> lattice::workflow_template::ValidatedBundle {
    use lattice::workflow_template::ParamValue;
    let template = fixture::catalog_template(&fixture.catalog);
    let supplied = [
        ("enabled".to_string(), ParamValue::Bool(true)),
        ("count".to_string(), ParamValue::Integer(4)),
        ("format".to_string(), ParamValue::Text("json".into())),
        ("target".to_string(), ParamValue::Text("reports/nightly".into())),
    ]
    .into();
    let bound = lattice::workflow_template::bind(&template, &supplied).expect("non-default values bind");
    assert_ne!(bound.workflow_revision(), fixture.saved.workflow_revision(), "the binding is not the default");
    bound
}

fn admit_in(
    fixture: &Fixture,
    store: &lattice::store::Store,
) -> Result<lattice::workflow_template::inputs::Admitted, InputError> {
    match fixture.admit_inputs(store, fixture::BUNDLE, &args(&["format=text", "count=2"]), None) {
        Ok(admitted) => Ok(admitted),
        Err(lattice::workflow_template::Error::Inputs(error)) => Err(error),
        Err(other) => panic!("unexpected shell failure: {other}"),
    }
}

// r[verify lattice.workflow_templates.run_inputs.retained]
#[test]
fn a_workflow_saved_without_a_retained_binding_is_told_to_save_again() {
    let fixture = Fixture::new();
    let bound = non_default_bound(&fixture);
    let upgraded = tempfile::tempdir().expect("upgraded store directory");
    let store = lattice::store::Store::new(upgraded.path().join("lattice.redb")).expect("store opens");
    // What `save_draft` wrote before bindings were retained: the bound workflow alone.
    store.save_workflow_cas(bound.workflow(), None).expect("a save from before bindings were retained");
    let error = admit_in(&fixture, &store).unwrap_err();
    assert_eq!(error, InputError::BindingMissing);
    assert!(error.to_string().contains("save it again from its bundle"));

    let plan = lattice::workflow_template::save_draft(&store, &bound).expect("saving again succeeds");
    assert!(matches!(plan, lattice::workflow_template::ApplicationPlan::ExactAlreadyPresent { .. }));
    admit_in(&fixture, &store).expect("inputs admit after saving again");
}

#[test]
fn an_exact_resave_replaces_a_stale_binding() {
    let fixture = Fixture::new();
    let bound = non_default_bound(&fixture);
    let directory = tempfile::tempdir().expect("store directory");
    let store = lattice::store::Store::new(directory.path().join("lattice.redb")).expect("store opens");
    lattice::workflow_template::save_draft(&store, &bound).expect("first save");
    let current = store.load_workflow_template_binding(fixture::BUNDLE).unwrap().expect("binding retained");
    let mut stale = current.clone();
    stale.workflow.content_hash = fixture.saved.workflow_revision().content_hash.clone();
    store.save_workflow_template_binding(&stale).expect("stale binding writes");
    assert_eq!(admit_in(&fixture, &store).unwrap_err(), InputError::SavedWorkflowDrift);

    let plan = lattice::workflow_template::save_draft(&store, &bound).expect("exact save");
    assert!(matches!(plan, lattice::workflow_template::ApplicationPlan::ExactAlreadyPresent { .. }));
    assert_eq!(store.load_workflow_template_binding(fixture::BUNDLE).unwrap(), Some(current));
    admit_in(&fixture, &store).expect("the replaced binding admits");
}
