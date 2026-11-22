//! Runs bound to typed inputs: effects, records, `runs show`, `authority inspect`, and inputless
//! stability.

mod common;
#[path = "support/run_inputs.rs"]
mod fixture;

use fixture::Fixture;
use fixture::args;

const DISTINCT_VALUE: &str = "probe/zzdistinctzz";

fn admit_ok(
    fixture: &Fixture,
    store: &lattice::store::Store,
    cli: &[&str],
) -> lattice::workflow_template::inputs::Admitted {
    fixture.admit_inputs(store, fixture::BUNDLE, &args(cli), None).expect("inputs admit")
}

/// An identity without its report's prefix, `b3:` in `runs show` and `blake3:` in `authority`.
fn hex(identity: &serde_json::Value) -> &str {
    let text = identity.as_str().expect("identity is text");
    text.split_once(':').map_or(text, |(_, hex)| hex)
}

fn stdout_of(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// A `runs show` report with its wall-clock fields removed.
fn timeless(report: &str) -> serde_json::Value {
    let mut value: serde_json::Value = serde_json::from_str(report).expect("report parses");
    value["lifecycle"]["started_epoch_ms"] = serde_json::Value::Null;
    value["lifecycle"]["finished_epoch_ms"] = serde_json::Value::Null;
    value
}

// r[verify lattice.workflow_templates.run_inputs.record]
// r[verify lattice.workflow_templates.run_inputs.inspection]
#[tokio::test]
async fn a_bound_run_records_its_input_identity_and_shows_it_without_values() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let admitted = admit_ok(&fixture, &store, &["format=json", "count=3", &format!("target={DISTINCT_VALUE}")]);
    let provider = common::authority::TestRunContextProvider::new();
    let identity = admitted.identity().clone();
    let summary = admitted.run_with_authority_provider(&store, &provider).await.expect("bound run completes");
    assert_eq!(summary.status, lattice::headless::HeadlessRunStatus::Succeeded);
    assert!(fixture.marker.exists(), "the admitted run reached its node effects");
    let details = store.load_run_details(summary.run_id).unwrap().expect("details recorded");
    let printed = details
        .nodes
        .iter()
        .flat_map(|node| &node.outputs)
        .any(|output| output.text.trim() == format!("false 3 json {DISTINCT_VALUE}"));
    assert!(printed, "the probe received each value as one argument");
    let recorded = store.load_run_inputs(summary.run_id).unwrap().expect("inputs recorded");
    assert_eq!(recorded.run_id, summary.run_id);
    assert_eq!(recorded.inputs, identity);
    drop(store);

    let report = lattice::run::shell::inspect_path(&fixture.store_path(), summary.run_id).expect("run inspects");
    assert!(report.valid, "diagnostics: {:?}", report.diagnostic_codes);
    let json = lattice::run::inspection::render_json(&report).expect("renders");
    let text = lattice::run::inspection::render_text(&report);
    for rendered in [&json, &text] {
        assert!(!rendered.contains("zzdistinctzz") && !rendered.contains(fixture::BUNDLE), "content is omitted");
    }
    assert!(text.contains(&format!("inputs template=b3:{} ", identity.template.as_str())), "{text}");
    assert!(text.contains("content=omitted\ninput count kind=integer supplied=true\n"), "{text}");
    assert!(text.contains("input enabled kind=bool supplied=false\n"), "{text}");
    let inputs = report.inputs.expect("inputs shown");
    assert_eq!(inputs.values, format!("b3:{}", identity.values.as_str()));
    assert_eq!(inputs.workflow_revision, format!("b3:{}", identity.workflow.content_hash.as_str()));
    let shown = inputs.params.iter().map(|param| (param.name.as_str(), param.supplied)).collect::<Vec<_>>();
    assert_eq!(shown, [("count", true), ("enabled", false), ("format", true), ("target", true)]);

    let mut foreign = recorded.clone();
    foreign.run_id = lattice::engine::RunId(summary.run_id.0 + 1);
    let record = lattice::store::RunRecord {
        run_id: summary.run_id,
        owner: lattice::store::RunOwner::Unscoped,
        workflow_name: fixture::BUNDLE.into(),
        started_epoch_ms: 1,
        finished_epoch_ms: Some(2),
        node_count: 2,
        cache_hits: 0,
        failures: 0,
    };
    let input = lattice::run::inspection::Input {
        record: &record,
        details: None,
        replay_events: None,
        orchestration: None,
    };
    let mismatched = lattice::run::inspection::inspect_with_inputs(input, Some(&foreign));
    assert!(!mismatched.valid && mismatched.inputs.is_none());
    assert_eq!(mismatched.diagnostic_codes, [lattice::run::inspection::DiagnosticCode::InputsIdentityMismatch]);
}

// r[verify lattice.workflow_templates.run_inputs.scenario.refused]
#[test]
fn refused_inputs_stop_before_any_effect_run_id_or_record() {
    let fixture = Fixture::new();
    for inputs in [
        &["count=9"][..],
        &["format=json", "count=9"],
        &["format=json", "target=../zzsecretzz"],
        &["format=json", "nope=1"],
    ] {
        let mut cli = vec!["run", fixture::BUNDLE];
        for input in inputs {
            cli.extend(["--input", input]);
        }
        let output = fixture.cli(&cli);
        assert!(!output.status.success());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("zzsecretzz"), "values are never echoed");
    }
    assert!(!fixture.marker.exists(), "no node ran");
    let store = fixture.store();
    assert!(store.list_runs(16).unwrap().is_empty(), "no run record");
    assert!(store.load_run_inputs(lattice::engine::RunId(0)).unwrap().is_none(), "no input record");
    assert_eq!(store.next_run_id().unwrap(), lattice::engine::RunId(0), "no run id was reserved");
}

// r[verify lattice.workflow_templates.run_inputs.authority]
#[test]
fn authority_inspect_with_inputs_reports_the_bound_revision_that_runs() {
    let fixture = Fixture::new();
    let inspect = |extra: &[&str]| {
        let mut cli = vec!["authority", "inspect", fixture::BUNDLE];
        cli.extend(extra);
        let output = fixture.cli(&cli);
        assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
        stdout_of(&output)
    };
    let file = fixture.input_file("inputs.json", r#"{"count": 2}"#);
    let file = file.to_str().expect("utf-8 path");
    let target = format!("target={DISTINCT_VALUE}");
    let bound_text = inspect(&["--input", "format=json", "--input", &target, "--input-file", file]);
    assert!(!bound_text.contains("zzdistinctzz"));
    let bound: serde_json::Value = serde_json::from_str(&bound_text).unwrap();
    let saved: serde_json::Value = serde_json::from_str(&inspect(&[])).unwrap();
    assert_ne!(bound["workflow_revision"], saved["workflow_revision"]);
    assert_eq!(bound["grants"], saved["grants"], "inputs never change grants");
    assert_eq!(
        bound["inputs"]["workflow_revision"], bound["workflow_revision"],
        "one hash renders one way in one report"
    );
    assert!(bound["workflow_revision"].as_str().is_some_and(|revision| revision.starts_with("blake3:")));

    let profile = lattice::engine::effect::standalone::core::Profile {
        schema: bound["schema"].as_str().unwrap().into(),
        workflow_revision: bound["workflow_revision"].as_str().unwrap().into(),
        machine_identity: bound["machine_identity"].as_str().unwrap().into(),
        policy_ref: bound["policy_ref"].as_str().unwrap().into(),
        expires_at_unix_ms: 4_102_444_800_000,
        grants: serde_json::from_value(bound["grants"].clone()).unwrap(),
        roots: Default::default(),
    };
    std::fs::write(fixture.dir.path().join("authority.json"), serde_json::to_vec(&profile).unwrap()).unwrap();
    let authority = fixture.dir.path().join("authority.ncl");
    std::fs::write(&authority, "import \"authority.json\"\n").unwrap();
    fixture.write_config(Some(&authority));

    let other = fixture.cli(&[
        "run",
        fixture::BUNDLE,
        "--input",
        "format=text",
        "--input",
        &target,
        "--input-file",
        file,
    ]);
    assert!(!other.status.success(), "other values have another revision the profile does not grant");
    assert!(!fixture.marker.exists());
    let run = fixture.cli(&[
        "run",
        fixture::BUNDLE,
        "--input",
        "format=json",
        "--input",
        &target,
        "--input-file",
        file,
        "--output",
        "json",
    ]);
    assert!(run.status.success(), "stderr: {}", String::from_utf8_lossy(&run.stderr));
    assert!(fixture.marker.exists(), "the inspected values run under the inspected grants");
    let summary: serde_json::Value = serde_json::from_slice(&run.stdout).unwrap();
    let shown = fixture.cli(&["runs", "show", &summary["run_id"].to_string(), "--output", "json"]);
    let shown: serde_json::Value = serde_json::from_slice(&shown.stdout).unwrap();
    for key in ["template", "bound", "workflow_revision", "values"] {
        assert_eq!(hex(&shown["inputs"][key]), hex(&bound["inputs"][key]), "the run records the inspected {key}");
        assert!(shown["inputs"][key].as_str().is_some_and(|value| value.starts_with("b3:")));
    }
    assert_eq!(shown["inputs"]["params"], bound["inputs"]["params"]);
}

// r[verify lattice.workflow_templates.run_inputs.scenario.inputless]
#[test]
fn inputless_runs_keep_identity_records_and_output_bytes() {
    let fixture = Fixture::new();
    let saved = fixture.store().load_workflow(fixture::BUNDLE).unwrap().unwrap();
    let plain = tempfile::tempdir().unwrap();
    let plain_state = plain.path().join("state");
    std::fs::create_dir_all(&plain_state).unwrap();
    let plain_config = plain.path().join("config.ncl");
    let state = serde_json::to_string(&plain_state).unwrap();
    std::fs::write(&plain_config, format!("{{ data_dir = {state}, default_timeout_secs = 5, shell = \"sh\", }}\n"))
        .unwrap();
    let store = lattice::store::Store::new(plain_state.join("lattice.redb")).unwrap();
    let mut hand_copy = saved.clone();
    hand_copy.catalog_revision = None;
    store.save_workflow(&hand_copy).expect("hand save without a template");
    drop(store);

    let expected =
        serde_json::to_string_pretty(&lattice::engine::effect::standalone::inspect_workflow(&saved).unwrap()).unwrap();
    assert_eq!(stdout_of(&fixture.cli(&["authority", "inspect", fixture::BUNDLE])), format!("{expected}\n"));

    let templated = fixture.cli(&["run", fixture::BUNDLE, "--output", "json"]);
    let hand_saved = fixture::lattice(&plain_config, &["run", fixture::BUNDLE, "--output", "json"]);
    assert_eq!(templated.stdout, hand_saved.stdout, "same run identity and summary bytes");
    assert_eq!(templated.status.code(), hand_saved.status.code());
    let show =
        |config: &std::path::Path| stdout_of(&fixture::lattice(config, &["runs", "show", "0", "--output", "json"]));
    let (templated_report, hand_report) = (show(&fixture.config), show(&plain_config));
    assert_eq!(timeless(&templated_report), timeless(&hand_report));
    assert!(!templated_report.contains("\"inputs\""), "inputless reports keep their shape");
    assert!(fixture.store().load_run_inputs(lattice::engine::RunId(0)).unwrap().is_none());
    let records = fixture.store().list_runs(4).unwrap();
    let keys = serde_json::to_value(&records[0]).unwrap().as_object().unwrap().keys().cloned().collect::<Vec<_>>();
    let expected_keys = [
        "cache_hits",
        "failures",
        "finished_epoch_ms",
        "node_count",
        "owner",
        "run_id",
        "started_epoch_ms",
        "workflow_name",
    ];
    assert_eq!(keys, expected_keys);
}

struct FailingAuthority;

impl lattice::engine::effect::execution::RunContextProvider for FailingAuthority {
    fn load(
        &self,
        _run_id: lattice::engine::RunId,
        _store: Option<lattice::store::Store>,
    ) -> Result<lattice::engine::effect::execution::RunContext, lattice::engine::effect::execution::Error> {
        Err(lattice::engine::effect::execution::Error::AuthoritySource)
    }
}

// r[verify lattice.workflow_templates.run_inputs.record]
#[tokio::test]
async fn an_orphaned_input_record_is_never_listed_or_shown_as_a_run() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let admitted = admit_ok(&fixture, &store, &["format=json"]);
    let failed = admitted.run_with_authority_provider(&store, &FailingAuthority).await;
    assert!(matches!(failed, Err(lattice::headless::HeadlessRunError::Validation(_))), "{failed:?}");
    let orphan = lattice::engine::RunId(0);
    assert!(store.load_run_inputs(orphan).unwrap().is_some(), "the reserved id keeps its input record");
    assert!(store.list_runs(8).unwrap().is_empty(), "the orphan is never listed as a run");
    assert_eq!(store.next_run_id().unwrap(), lattice::engine::RunId(1), "its id is never reused");
    drop(store);
    let shown = lattice::run::shell::inspect_path(&fixture.store_path(), orphan);
    assert!(matches!(shown, Err(lattice::run::shell::Error::NotFound(id)) if id == orphan));
}
