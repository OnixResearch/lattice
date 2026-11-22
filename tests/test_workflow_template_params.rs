//! Typed workflow-template parameters: declarations, value bounds, and resolved identity.

use std::collections::BTreeMap;

use lattice::workflow_template::ParamDeclaration;
use lattice::workflow_template::ParamTarget;
use lattice::workflow_template::ParamType;
use lattice::workflow_template::ParamValue;
use lattice::workflow_template::params::ParamErrorCode;

const SHELL_NODE: &str = "run tool";
const HOSTILE_VALUES: &[&str] = &[
    "../outside",
    "/etc",
    "a b",
    "a;rm -rf /",
    "$(touch pwned)",
    "`id`",
    "a|b",
    "-rf",
    "dir/--force",
    "a//b",
    "a/",
    "\"quoted\"",
    "'single'",
    "glob*",
    "new\nline",
    "",
];

fn target(key: &str) -> Vec<ParamTarget> {
    vec![ParamTarget {
        node: SHELL_NODE.to_string(),
        shell_env: key.to_string(),
    }]
}

fn declarations() -> Vec<ParamDeclaration> {
    vec![
        ParamDeclaration {
            name: "limit".into(),
            value_type: ParamType::Integer { min: 1, max: 10 },
            required: false,
            default: Some(ParamValue::Integer(5)),
            bindings: target("LATTICE_PARAM_LIMIT"),
        },
        ParamDeclaration {
            name: "format".into(),
            value_type: ParamType::Enum {
                values: vec!["json".into(), "text".into()],
            },
            required: true,
            default: None,
            bindings: target("LATTICE_PARAM_FORMAT"),
        },
        ParamDeclaration {
            name: "root".into(),
            value_type: ParamType::RelativePath { max_bytes: 16 },
            required: false,
            default: Some(ParamValue::Text(".".into())),
            bindings: target("LATTICE_PARAM_ROOT"),
        },
        ParamDeclaration {
            name: "strict".into(),
            value_type: ParamType::Bool {},
            required: false,
            default: Some(ParamValue::Bool(false)),
            bindings: target("LATTICE_PARAM_STRICT"),
        },
    ]
}

fn supplied(pairs: &[(&str, ParamValue)]) -> BTreeMap<String, ParamValue> {
    pairs.iter().map(|(name, value)| ((*name).to_string(), value.clone())).collect()
}

fn text(value: &str) -> ParamValue {
    ParamValue::Text(value.to_string())
}

fn rejected(pairs: &[(&str, ParamValue)]) -> lattice::workflow_template::ParamError {
    lattice::workflow_template::params::resolve(&declarations(), &supplied(pairs)).expect_err("values are rejected")
}

// r[verify lattice.workflow_templates.params.scenario.bounds]
#[test]
fn resolution_applies_defaults_and_binds_a_stable_identity() {
    let declared = declarations();
    let first = lattice::workflow_template::params::resolve(&declared, &supplied(&[("format", text("json"))]))
        .expect("required value with defaults resolves");
    let again = lattice::workflow_template::params::resolve(&declared, &supplied(&[("format", text("json"))]))
        .expect("same values resolve");
    assert_eq!(first, again);
    assert_eq!(first.get("limit"), Some(&ParamValue::Integer(5)));
    assert_eq!(first.get("root"), Some(&text(".")));
    assert_eq!(first.get("strict"), Some(&ParamValue::Bool(false)));
    let names = first.values().iter().map(|param| param.name.as_str()).collect::<Vec<_>>();
    assert_eq!(names, ["format", "limit", "root", "strict"]);

    for changed in [
        supplied(&[("format", text("text"))]),
        supplied(&[("format", text("json")), ("limit", ParamValue::Integer(6))]),
        supplied(&[("format", text("json")), ("root", text("crates/core"))]),
        supplied(&[("format", text("json")), ("strict", ParamValue::Bool(true))]),
    ] {
        let other = lattice::workflow_template::params::resolve(&declared, &changed).expect("changed values resolve");
        assert_ne!(other.identity(), first.identity(), "identity covers every resolved value");
    }
    let explicit_defaults = supplied(&[
        ("format", text("json")),
        ("limit", ParamValue::Integer(5)),
        ("root", text(".")),
        ("strict", ParamValue::Bool(false)),
    ]);
    let explicit =
        lattice::workflow_template::params::resolve(&declared, &explicit_defaults).expect("explicit defaults resolve");
    assert_eq!(explicit.identity(), first.identity(), "identity covers values, not how they were supplied");
}

// r[verify lattice.workflow_templates.params.scenario.bounds]
#[test]
fn unknown_wrong_type_out_of_bounds_and_missing_values_are_rejected_safely() {
    let missing = rejected(&[]);
    assert_eq!((missing.code, missing.param.as_deref()), (ParamErrorCode::MissingRequired, Some("format")));

    let unknown = rejected(&[("format", text("json")), ("Injected; $(id)", text("x"))]);
    assert_eq!((unknown.code, unknown.param.as_deref()), (ParamErrorCode::UnknownParam, None));
    assert!(!unknown.to_string().contains("Injected"));

    for (name, value) in [
        ("limit", text("5")),
        ("strict", text("true")),
        ("root", ParamValue::Integer(1)),
        ("format", ParamValue::Bool(true)),
    ] {
        let error = rejected(&[("format", text("json")), (name, value)]);
        assert_eq!((error.code, error.param.as_deref()), (ParamErrorCode::WrongType, Some(name)));
    }
    for (name, value) in [
        ("limit", ParamValue::Integer(0)),
        ("limit", ParamValue::Integer(11)),
        ("format", text("xml")),
        ("root", text("a/very/long/relative/path")),
    ] {
        let error = rejected(&[("format", text("json")), (name, value)]);
        assert_eq!((error.code, error.param.as_deref()), (ParamErrorCode::OutOfBounds, Some(name)));
    }
    for hostile in HOSTILE_VALUES {
        let error = rejected(&[("format", text("json")), ("root", text(hostile))]);
        assert_eq!(error.code, ParamErrorCode::OutOfBounds, "{hostile:?} is rejected");
        if !hostile.is_empty() {
            assert!(!error.to_string().contains(hostile), "diagnostic never echoes a value");
        }
    }
}

// r[verify lattice.workflow_templates.params.scenario.declaration]
#[test]
fn unsafe_or_inconsistent_declarations_are_rejected() {
    assert!(lattice::workflow_template::params::validate_declarations(&declarations()).is_ok());
    let mutations: Vec<fn(&mut Vec<ParamDeclaration>)> = vec![
        |declared| declared[1].name = declared[0].name.clone(),
        |declared| declared[0].name = "Limit".into(),
        |declared| declared[0].value_type = ParamType::Integer { min: 2, max: 1 },
        |declared| declared[1].value_type = ParamType::Enum { values: Vec::new() },
        |declared| {
            declared[1].value_type = ParamType::Enum {
                values: vec!["json".into(), "json".into()],
            }
        },
        |declared| {
            declared[1].value_type = ParamType::Enum {
                values: vec!["a b".into()],
            }
        },
        |declared| declared[2].value_type = ParamType::RelativePath { max_bytes: 0 },
        |declared| declared[1].default = Some(text("json")),
        |declared| declared[0].default = None,
        |declared| declared[0].default = Some(ParamValue::Integer(99)),
        |declared| declared[0].bindings = Vec::new(),
        |declared| declared[0].bindings = target("PATH"),
        |declared| declared[0].bindings = target("LATTICE_PARAM_"),
        |declared| declared[0].bindings = target("LATTICE_PARAM_lower"),
        |declared| declared[1].bindings = target("LATTICE_PARAM_LIMIT"),
        |declared| declared[0].bindings[0].node = String::new(),
        |declared| {
            for index in 0..lattice::workflow_template::params::MAX_PARAMS {
                let mut extra = declared[0].clone();
                extra.name = format!("extra_{index}");
                extra.bindings = target(&format!("LATTICE_PARAM_EXTRA_{index}"));
                declared.push(extra);
            }
        },
    ];
    for (index, mutate) in mutations.into_iter().enumerate() {
        let mut declared = declarations();
        mutate(&mut declared);
        let error = lattice::workflow_template::params::validate_declarations(&declared)
            .expect_err("unsafe declaration is rejected");
        assert_eq!(error.code, ParamErrorCode::InvalidDeclaration, "mutation {index}");
        let resolved = lattice::workflow_template::params::resolve(&declared, &supplied(&[("format", text("json"))]));
        assert!(resolved.is_err(), "resolve rechecks declarations, mutation {index}");
    }
}

// r[verify lattice.workflow_templates.params.scenario.declaration]
#[test]
fn manifest_parameter_types_are_closed() {
    let parse = |value: serde_json::Value| serde_json::from_value::<ParamDeclaration>(value);
    let declaration = serde_json::json!({
        "name": "root",
        "type": { "kind": "relative_path", "max_bytes": 64 },
        "required": false,
        "default": ".",
        "bindings": [{ "node": "run tool", "shell_env": "LATTICE_PARAM_ROOT" }],
    });
    assert!(parse(declaration.clone()).is_ok());
    for (pointer, replacement) in [
        ("/type", serde_json::json!({ "kind": "shell", "max_bytes": 64 })),
        ("/type", serde_json::json!({ "kind": "string" })),
        ("/type", serde_json::json!({ "kind": "bool", "min": 1 })),
        ("/type", serde_json::json!({ "kind": "relative_path", "max_bytes": 64, "pattern": ".*" })),
        (
            "/bindings",
            serde_json::json!([{ "node": "run tool", "shell_env": "LATTICE_PARAM_ROOT", "command": "x" }]),
        ),
    ] {
        let mut changed = declaration.clone();
        *changed.pointer_mut(pointer).expect("fixture field exists") = replacement;
        assert!(parse(changed).is_err(), "{pointer} rejects an open or unknown shape");
    }
    let mut extra = declaration;
    extra["template"] = serde_json::json!("--root {root}");
    assert!(parse(extra).is_err(), "a declaration has no free-form command template field");
}

// r[verify lattice.workflow_templates.params.scenario.declaration]
// r[verify lattice.workflow_templates.mode.scenario.compatibility]
#[test]
fn authoring_contract_accepts_schema_two_and_rejects_open_or_ambient_shapes() {
    let fixture = |name: &str| {
        std::fs::canonicalize(format!("tests/fixtures/workflow-templates/{name}.ncl")).expect("policy fixture exists")
    };
    let valid = lattice::nickel::evaluate_file(&fixture("manifest-policy-valid")).expect("valid manifest evaluates");
    let manifest = serde_json::from_value::<lattice::workflow_template::BundleManifest>(valid)
        .expect("contract output is a loader manifest");
    assert_eq!(manifest.mode, Some(lattice::workflow_template::BundleMode::ReadOnly));
    assert!(lattice::workflow_template::params::validate_declarations(manifest.params()).is_ok());
    for invalid in [
        "manifest-policy-invalid-mode",
        "manifest-policy-invalid-schema",
        "manifest-policy-invalid-type",
        "manifest-policy-invalid-env",
    ] {
        assert!(lattice::nickel::evaluate_file(&fixture(invalid)).is_err(), "{invalid} is rejected");
    }
}
