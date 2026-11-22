//! Binding typed parameters into migrated built-in bundles without widening authority.

use std::collections::BTreeMap;
use std::path::Path;

use lattice::workflow_template::BundleMode;
use lattice::workflow_template::DiagnosticCode;
use lattice::workflow_template::ParamValue;
use lattice::workflow_template::ValidatedBundle;

const BUILTIN_ROOT: &str = "builtins/workflow-templates";
const CATALOG_REVIEW: &str = "agent-evaluation-catalog-review";
const CATALOG_PARAM: &str = "catalog_root";
const CATALOG_ENV: &str = "LATTICE_PARAM_CATALOG_ROOT";
const CATALOG_DEFAULT: &str = "builtins/agent-evaluations";
const CATALOG_NODE: &str = "Validate evaluation catalog";
const BUILTIN_IDS: [&str; 4] = [
    CATALOG_REVIEW,
    "cairn-agent-drain",
    "octet-agent-burndown",
    "shell-preview",
];

fn builtin_catalog() -> lattice::workflow_template::Catalog {
    lattice::workflow_template::load_catalog(&Path::new(env!("CARGO_MANIFEST_DIR")).join(BUILTIN_ROOT))
        .expect("built-in catalog loads")
}

fn builtin(id: &str) -> ValidatedBundle {
    builtin_catalog()
        .bundles
        .into_iter()
        .find(|bundle| bundle.manifest().id == id)
        .expect("built-in bundle exists")
}

fn values(pairs: &[(&str, ParamValue)]) -> BTreeMap<String, ParamValue> {
    pairs.iter().map(|(name, value)| ((*name).to_string(), value.clone())).collect()
}

fn root(value: &str) -> BTreeMap<String, ParamValue> {
    values(&[(CATALOG_PARAM, ParamValue::Text(value.to_string()))])
}

fn shell<'a>(bundle: &'a ValidatedBundle, label: &str) -> (&'a str, &'a std::collections::HashMap<String, String>) {
    let node = bundle.workflow().graph.nodes().find(|node| node.label == label).expect("labelled node exists");
    match bundle.workflow().configs.get(&node.id.0) {
        Some(lattice::engine::workflow::NodeConfig::Shell { command, env, .. }) => (command, env),
        _ => panic!("bound node is a shell node"),
    }
}

fn grants(bundle: &ValidatedBundle) -> Vec<lattice::engine::effect::standalone::core::Grant> {
    lattice::engine::effect::standalone::inspect_workflow(bundle.workflow())
        .expect("authority inspection succeeds")
        .grants
}

// r[verify lattice.workflow_templates.params.scenario.migration]
#[test]
fn built_in_bundles_are_schema_two_mutating_and_bind_their_defaults_exactly() {
    let catalog = builtin_catalog();
    assert!(catalog.diagnostics.is_empty(), "diagnostics: {:?}", catalog.diagnostics);
    let ids = catalog.bundles.iter().map(|bundle| bundle.manifest().id.as_str()).collect::<Vec<_>>();
    assert_eq!(ids, BUILTIN_IDS);
    for template in &catalog.bundles {
        let manifest = template.manifest();
        assert_eq!(manifest.schema_version, lattice::workflow_template::CURRENT_BUNDLE_SCHEMA_VERSION);
        assert_eq!(manifest.mode, Some(BundleMode::Mutating));
        let required = lattice::workflow_template::classification::required_mode(template.workflow());
        assert_eq!(required, BundleMode::Mutating);
        assert!(template.params().is_none(), "catalog entries are unbound templates");
        let bound = lattice::workflow_template::bind(template, &BTreeMap::new()).expect("defaults bind");
        let reviewed = &template.workflow().configs;
        assert_eq!(&bound.workflow().configs, reviewed, "defaults reproduce the reviewed workflow");
        assert_eq!(bound.workflow_revision(), template.workflow_revision());
        assert_eq!(bound.orchestration_revision(), template.orchestration_revision());
        assert_ne!(bound.identity(), template.identity());
        let bound_params = bound.params().expect("bound values are recorded");
        assert_eq!(&bound_params.template, template.identity());
        assert_eq!(&bound_params.workflow_revision, bound.workflow_revision());
        let expected = lattice::workflow_template::params::bound_identity(
            template.identity(),
            &bound_params.resolved,
            bound.workflow_revision(),
        );
        assert_eq!(bound.identity(), &expected);
        assert!(lattice::workflow_template::reverify(&bound).is_ok());
    }
    let review = catalog
        .bundles
        .iter()
        .find(|bundle| bundle.manifest().id == CATALOG_REVIEW)
        .expect("catalog review");
    let names = review.manifest().params().iter().map(|param| param.name.as_str()).collect::<Vec<_>>();
    assert_eq!(names, [CATALOG_PARAM]);
    assert_eq!(review.manifest().params()[0].default, Some(ParamValue::Text(CATALOG_DEFAULT.into())));
    let guarded = catalog.bundles.iter().filter(|bundle| bundle.manifest().category == "guarded-agents");
    assert!(guarded.clone().count() == 2 && guarded.into_iter().all(|bundle| bundle.manifest().params().is_empty()));
}

// r[verify lattice.workflow_templates.params.scenario.binding]
#[test]
fn values_bind_only_into_declared_environment_and_never_widen_grants() {
    let template = builtin(CATALOG_REVIEW);
    let defaults = lattice::workflow_template::bind(&template, &BTreeMap::new()).expect("defaults bind");
    let other = lattice::workflow_template::bind(&template, &root("fixtures/catalog")).expect("relative root binds");
    let (template_command, template_env) = shell(&template, CATALOG_NODE);
    let (command, env) = shell(&other, CATALOG_NODE);
    assert_eq!(command, template_command, "values never change command text");
    assert!(command.contains(&format!("\"${{{CATALOG_ENV}}}\"")));
    assert!(!command.contains("fixtures/catalog"));
    assert_eq!(template_env.get(CATALOG_ENV).map(String::as_str), Some(CATALOG_DEFAULT));
    assert_eq!(env.get(CATALOG_ENV).map(String::as_str), Some("fixtures/catalog"));
    assert_eq!(env.len(), template_env.len());

    assert_ne!(other.workflow_revision(), defaults.workflow_revision(), "the revision binds the resolved value");
    assert_ne!(other.identity(), defaults.identity(), "the bound identity drifts with the value");
    let again = lattice::workflow_template::bind(&template, &root("fixtures/catalog")).expect("same value binds");
    assert_eq!(again.identity(), other.identity());
    assert_eq!(again.workflow_revision(), other.workflow_revision());

    assert_eq!(grants(&other), grants(&template), "parameters never add or widen grants");
    let inspected = lattice::engine::effect::standalone::inspect_workflow(other.workflow()).expect("inspection");
    assert_eq!(inspected.workflow_revision, format!("blake3:{}", other.workflow_revision().content_hash));

    assert!(lattice::workflow_template::bind(&other, &BTreeMap::new()).is_err(), "a bound bundle cannot rebind");
}

// r[verify lattice.workflow_templates.params.scenario.bounds]
#[test]
fn rejected_values_produce_safe_bundle_diagnostics() {
    let template = builtin(CATALOG_REVIEW);
    for hostile in ["../outside", "/etc", "a; rm -rf /", "$(id)", "--root=/", "a b"] {
        let diagnostic = lattice::workflow_template::bind(&template, &root(hostile)).expect_err("hostile path fails");
        assert_eq!(diagnostic.code, DiagnosticCode::ParamOutOfBounds);
        assert!(!diagnostic.message.contains(hostile));
    }
    let wrong_type = values(&[(CATALOG_PARAM, ParamValue::Integer(1))]);
    let unknown = values(&[("command", ParamValue::Text("rm".into()))]);
    for (supplied, code) in [
        (wrong_type, DiagnosticCode::ParamTypeMismatch),
        (unknown, DiagnosticCode::UnknownParam),
    ] {
        let diagnostic = lattice::workflow_template::bind(&template, &supplied).expect_err("invalid values fail");
        assert_eq!((diagnostic.bundle.as_str(), diagnostic.code), (CATALOG_REVIEW, code));
    }
}

// r[verify lattice.workflow_templates.params.scenario.binding]
#[test]
fn bound_values_reach_the_process_as_one_unchanged_argument() {
    let template = builtin(CATALOG_REVIEW);
    let (command, _) = shell(&template, CATALOG_NODE);
    let probe = command
        .replace("lattice evaluation catalog --root", "printf '[%s]' --root")
        .replace(" --output json", "");
    assert_eq!(probe, format!("printf '[%s]' --root \"${{{CATALOG_ENV}}}\""));
    for value in ["fixtures/catalog", ".", "a.b_c-d/e", "./nested/dir"] {
        let bound = lattice::workflow_template::bind(&template, &root(value)).expect("value binds");
        let (_, env) = shell(&bound, CATALOG_NODE);
        let output =
            std::process::Command::new("sh").arg("-c").arg(&probe).envs(env).output().expect("probe shell runs");
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), format!("[--root][{value}]"));
    }
}
