//! Real selected-worker fixtures for the opt-in interaction-net profile.
//!
//! Every positive and producer-limit case runs the pinned Noble executable
//! (`LATTICE_NOBLE_BIN`) through the authored exact pack, ordinary pack
//! admission, the typed operation planner, the local read-only authority, the
//! durable operation-run owner, the existing process runner and operation-data
//! publication. The only stand-in executable is the descendant-teardown case,
//! which the real worker cannot reach because it spawns no descendants.

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use lattice::inet as profile;
use lattice::operation_pack as pack;
use lattice::pure_inet_profile as host;
use pack::shell::execution;

const FIXTURES: &str = "tests/fixtures/pure-inet";

fn noble() -> PathBuf {
    PathBuf::from(
        std::env::var_os("LATTICE_NOBLE_BIN")
            .expect("LATTICE_NOBLE_BIN must point at the pinned Noble executable; run inside nix develop"),
    )
}

// Produce literal Nickel, rather than permitting imports into an admitted file.
fn nickel(value: &impl serde::Serialize) -> String {
    fn literal(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Object(fields) => format!(
                "{{{}}}",
                fields
                    .iter()
                    .map(|(key, value)| format!("{} = {}", serde_json::to_string(key).unwrap(), literal(value)))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            serde_json::Value::Array(values) => {
                format!("[{}]", values.iter().map(literal).collect::<Vec<_>>().join(","))
            }
            value => value.to_string(),
        }
    }
    literal(&serde_json::to_value(value).expect("fixture value"))
}

struct Fixture {
    root: tempfile::TempDir,
    pack_root: PathBuf,
    admitted: pack::AdmittedPack,
    artifacts: PathBuf,
    instances: PathBuf,
    sequence: std::cell::Cell<u64>,
}

fn fixture_with(executable: &Path) -> Fixture {
    let root = tempfile::tempdir().expect("fixture root");
    let pack_root = root.path().join("pack");
    profile::route::pack::author(&pack_root, executable).expect("authored pack");
    assert!(profile::route::pack::author(&pack_root, executable).is_err(), "authoring is create-only");
    let candidate = pack::shell::load(&pack_root).expect("candidate");
    let facts = pack::AdmissionFacts {
        trusted_pack_identity: candidate.pack_identity.clone(),
        platform: pack::shell::platform_label(),
        dependency_identities: BTreeMap::new(),
        allowed_licenses: vec![
            profile::route::pack::CATALOG_LICENSE.to_string(),
            profile::route::pack::PRODUCER_LICENSE.to_string(),
        ],
        allowed_capabilities: candidate.manifest.admission.allowed_capabilities.clone(),
        evidence: BTreeMap::new(),
    };
    let lock = pack::create_lock(&candidate, &facts).expect("lock");
    let admitted = pack::admit(&candidate, &lock, &facts).expect("admitted pack");
    let artifacts = root.path().join("artifacts");
    std::fs::create_dir(&artifacts).expect("artifact root");
    let catalog = profile::route::catalog(&pack_root.join("noble")).expect("catalog");
    let instances = root.path().join("instances.ncl");
    std::fs::write(&instances, nickel(&profile::route::instance(&catalog, &artifacts).expect("instance")))
        .expect("instances");
    Fixture {
        root,
        pack_root,
        admitted,
        artifacts,
        instances,
        sequence: std::cell::Cell::new(0),
    }
}

fn fixture() -> Fixture {
    fixture_with(&noble())
}

fn write_artifacts(fixture: &Fixture, name: &str, program: &[u8], inputs: &[u8]) {
    std::fs::write(fixture.artifacts.join(format!("{name}.ngraph")), program).expect("program artifact");
    std::fs::write(fixture.artifacts.join(format!("{name}.vectors")), inputs).expect("inputs artifact");
}

fn arguments(fixture: &Fixture, name: &str, attempt: &str, ceilings: profile::route::Ceilings) -> serde_json::Value {
    let program = std::fs::read(fixture.artifacts.join(format!("{name}.ngraph"))).expect("program");
    let inputs = std::fs::read(fixture.artifacts.join(format!("{name}.vectors"))).expect("inputs");
    serde_json::json!({
        "attempt": attempt,
        "program": format!("{name}.ngraph"),
        "program_identity": host::blake3_identity(&program),
        "program_bytes": program.len(),
        "inputs": format!("{name}.vectors"),
        "inputs_identity": host::blake3_identity(&inputs),
        "inputs_bytes": inputs.len(),
        "fuel_interactions": ceilings.fuel_interactions,
        "max_input_bytes": ceilings.max_input_bytes,
        "max_retained_bytes": ceilings.max_retained_bytes,
        "max_output_bytes": ceilings.max_output_bytes,
    })
}

fn data_root(fixture: &Fixture) -> PathBuf {
    fixture.root.path().join("data")
}

fn execute_with(
    fixture: &Fixture,
    arguments: &serde_json::Value,
    prohibited: &[String],
    cancellation: Option<&AtomicBool>,
) -> Result<execution::Report, pack::Error> {
    fixture.sequence.set(fixture.sequence.get() + 1);
    let app_config = lattice::config::AppConfig {
        data_dir: data_root(fixture),
        ..lattice::config::AppConfig::default()
    };
    execution::run(execution::Request {
        app_config: &app_config,
        pack_root: &fixture.pack_root,
        admitted: &fixture.admitted,
        instances_path: &fixture.instances,
        instance_id: "pure-inet-artifacts",
        method: profile::route::METHOD,
        arguments_json: &arguments.to_string(),
        sequence: fixture.sequence.get(),
        prohibited_fragments: prohibited,
        cancellation,
    })
}

fn execute(fixture: &Fixture, arguments: &serde_json::Value) -> execution::Report {
    execute_with(fixture, arguments, &[], None).expect("admitted dispatch")
}

fn plan(fixture: &Fixture, arguments: &serde_json::Value) -> lattice::operation::InvocationPlan {
    lattice::operation::cli::admit_plan(lattice::operation::cli::PlanInput {
        catalog_path: &fixture.pack_root.join("operation-catalog.ncl"),
        instances_path: &fixture.instances,
        instance_id: "pure-inet-artifacts",
        method: profile::route::METHOD,
        arguments_json: &arguments.to_string(),
        executable_path: &fixture.pack_root.join("noble"),
    })
    .expect("plan")
}

fn store(fixture: &Fixture) -> lattice::store::Store {
    lattice::store::Store::new(lattice::config::store_path(&data_root(fixture))).expect("store")
}

fn published(
    fixture: &Fixture,
    report: &execution::Report,
) -> (lattice::operation_data::RecordMetadata, serde_json::Value) {
    let data = data_root(fixture);
    let plane = lattice::operation_data::Plane::open(&lattice::config::store_path(&data), &data).expect("plane");
    let version = report.operation_data_version.as_deref().expect("published version");
    let metadata = plane.load_metadata(version).expect("metadata").expect("record");
    let payload = plane.read_payload(&metadata).expect("payload");
    (metadata, serde_json::from_slice(&payload).expect("document"))
}

fn record_count(fixture: &Fixture) -> usize {
    let data = data_root(fixture);
    let plane = lattice::operation_data::Plane::open(&lattice::config::store_path(&data), &data).expect("plane");
    plane.list_metadata().expect("metadata").len()
}

fn ops(nodes: &[profile::Node]) -> profile::Program {
    let output = u32::try_from(nodes.len() - 1).expect("output");
    profile::Program::new(nodes.to_vec(), output).expect("program")
}

// r[verify lattice.pure_inet_profile.route.scenario.selected]
// r[verify lattice.pure_inet_profile.verification.scenario.positive]
// r[verify lattice.pure_inet_profile.purity.scenario.allowed]
#[test]
fn selected_worker_matches_the_independent_wrapping_reference() {
    use profile::Node::Add;
    use profile::Node::Const;
    use profile::Node::Input;
    use profile::Node::Mul;
    use profile::Node::Sub;
    let fixture = fixture();
    let programs = [
        ops(&[Input(0), Input(1), Add(0, 1)]),
        ops(&[Input(0), Input(1), Sub(0, 1)]),
        ops(&[Input(0), Input(1), Mul(0, 1)]),
        ops(&[
            Input(0),
            Input(1),
            Add(0, 1),
            Sub(0, 1),
            Mul(2, 3),
            Const(i64::MIN),
            Add(4, 5),
        ]),
        ops(&[
            Input(1),
            Input(0),
            Mul(1, 1),
            Mul(2, 2),
            Sub(3, 0),
            Const(-7),
            Mul(4, 5),
            Add(6, 2),
        ]),
    ];
    let pairs = [
        (i64::MAX, 1),
        (i64::MIN, -1),
        (i64::MIN, 1),
        (i64::MAX, i64::MAX),
        (9_007_199_254_740_993, -9_007_199_254_740_993),
        (0, 0),
        (-1, 9_007_199_254_740_992),
    ];
    let mut index = 0;
    for program in &programs {
        for (first, second) in pairs {
            index += 1;
            let inputs = host::PureInputs { first, second };
            let name = format!("case-{index}");
            write_artifacts(
                &fixture,
                &name,
                profile::text::render_program(program).as_bytes(),
                profile::text::render_inputs(inputs).as_bytes(),
            );
            let report =
                execute(&fixture, &arguments(&fixture, &name, &format!("attempt-{index}"), profile::route::CEILINGS));
            assert_eq!(report.status, execution::Status::Completed, "{name}");
            let (metadata, document) = published(&fixture, &report);
            assert_eq!(metadata.outcome, "completed");
            let expected = program.evaluate(inputs).expect("reference");
            assert_eq!(document["result"], host::format_canonical_i64(expected), "{name} {first} {second}");
            assert_eq!(document["authoritative"], true);
            assert_eq!(document["semantic_interactions"]["assurance"], "observed");
            assert!(document["counters"]["interactions"].is_string());
            assert_eq!(document["counters"]["interactions"], document["counters"]["expected_interactions"]);
            assert_eq!(document["binding"]["producer_revision"], profile::report::PRODUCER_SOURCE_REVISION);
        }
    }
}

// r[verify lattice.pure_inet_profile.recompute_integrity.scenario.match]
// r[verify lattice.pure_inet_profile.semantic_limit]
#[test]
fn two_fresh_admitted_evaluations_check_the_count_and_fix_the_exact_limit() {
    let fixture = fixture();
    write_artifacts(
        &fixture,
        "shared",
        &std::fs::read(format!("{FIXTURES}/program.ngraph")).unwrap(),
        &std::fs::read(format!("{FIXTURES}/inputs.vectors")).unwrap(),
    );
    let first = execute(&fixture, &arguments(&fixture, "shared", "attempt-1", profile::route::CEILINGS));
    let mut fresh = arguments(&fixture, "shared", "attempt-2", profile::route::CEILINGS);
    fresh["predecessor"] = first.invocation_identity.clone().into();
    let second = execute(&fixture, &fresh);
    assert_ne!(first.invocation_identity, second.invocation_identity);
    let first_document = published(&fixture, &first).1;
    let second_document = published(&fixture, &second).1;
    assert_eq!(second_document["binding"]["predecessor"], first.invocation_identity);
    let left = profile::count::Evaluation::from_document(&first_document).expect("first");
    let right = profile::count::Evaluation::from_document(&second_document).expect("second");
    let Ok(profile::count::Verdict::Consistent(checked)) = profile::count::compare(&left, &right, true) else {
        panic!("fresh complete evaluations agree");
    };
    assert_eq!(checked.assurance, profile::count::Assurance::Checked);
    let count = checked.value.parse::<u64>().expect("count");
    let at_count = profile::route::Ceilings {
        fuel_interactions: count,
        ..profile::route::CEILINGS
    };
    let exact = execute(&fixture, &arguments(&fixture, "shared", "attempt-3", at_count));
    assert_eq!(exact.status, execution::Status::Completed, "success at exactly the checked count");
    let below = profile::route::Ceilings {
        fuel_interactions: count - 1,
        ..profile::route::CEILINGS
    };
    let short = execute(&fixture, &arguments(&fixture, "shared", "attempt-4", below));
    assert_eq!(short.status, execution::Status::Failed);
    let (metadata, diagnostic) = published(&fixture, &short);
    assert_eq!(metadata.outcome, execution::NON_AUTHORITATIVE_OUTCOME);
    assert_eq!(
        (diagnostic["evaluation"].as_str(), diagnostic["exhaustion"]["limit"].as_str()),
        (Some("exhausted"), Some("fuel"))
    );
}

// r[verify lattice.pure_inet_profile.bounds.scenario.exhausted]
// r[verify lattice.pure_inet_profile.bounds.scenario.unmeasured]
#[test]
fn producer_ceilings_stop_the_real_worker_as_non_authoritative_exhaustion() {
    let fixture = fixture();
    write_artifacts(
        &fixture,
        "shared",
        &std::fs::read(format!("{FIXTURES}/program.ngraph")).unwrap(),
        &std::fs::read(format!("{FIXTURES}/inputs.vectors")).unwrap(),
    );
    for (index, (limit, ceilings)) in [
        ("retained-bytes", profile::route::Ceilings {
            max_retained_bytes: 64,
            ..profile::route::CEILINGS
        }),
        ("output-bytes", profile::route::Ceilings {
            max_output_bytes: 512,
            ..profile::route::CEILINGS
        }),
        ("input-bytes", profile::route::Ceilings {
            max_input_bytes: 64,
            ..profile::route::CEILINGS
        }),
    ]
    .into_iter()
    .enumerate()
    {
        let report = execute(&fixture, &arguments(&fixture, "shared", &format!("limit-{index}"), ceilings));
        assert_eq!(report.operation_state, lattice::operation::RunState::Failed, "{limit}");
        let (metadata, document) = published(&fixture, &report);
        assert_eq!(metadata.outcome, execution::NON_AUTHORITATIVE_OUTCOME);
        assert_eq!(document["authoritative"], false);
        assert!(document.get("result").is_none() && document.get("counters").is_none());
        assert_eq!(document["exhaustion"]["limit"], limit);
        assert!(document["exhaustion"]["cap"].is_string() && document["limits"]["max_output_bytes"].is_string());
        let record = store(&fixture).load_operation_run(&report.invocation_identity).expect("run").expect("record");
        assert!(record.output_identity.is_none(), "a diagnostic is never the run's result identity");
    }
}

// r[verify lattice.pure_inet_profile.identity.scenario.changed]
// r[verify lattice.pure_inet_profile.route.scenario.unavailable]
#[test]
fn changed_crossed_linked_or_unsupported_artifacts_are_refused_before_durable_start() {
    let fixture = fixture();
    let program = std::fs::read(format!("{FIXTURES}/program.ngraph")).unwrap();
    let inputs = std::fs::read(format!("{FIXTURES}/inputs.vectors")).unwrap();
    write_artifacts(&fixture, "shared", &program, &inputs);
    let admitted = arguments(&fixture, "shared", "attempt-1", profile::route::CEILINGS);
    let mut changed = program.clone();
    let position = changed.iter().rposition(|byte| *byte == b'6').expect("digit");
    changed[position] = b'5';
    std::fs::write(fixture.artifacts.join("shared.ngraph"), &changed).expect("changed bytes");
    let mut crossed = admitted.clone();
    crossed["inputs_identity"] = admitted["program_identity"].clone();
    let mut unknown = admitted.clone();
    unknown["resume_snapshot"] = "heap".into();
    let mut raised = admitted.clone();
    raised["max_output_bytes"] = (profile::route::CEILINGS.max_output_bytes + 1).into();
    let mut traversal = admitted.clone();
    traversal["program"] = "../shared.ngraph".into();
    for refused in [&admitted, &crossed, &unknown, &raised, &traversal] {
        assert!(execute_with(&fixture, refused, &[], None).is_err(), "{refused}");
    }
    std::fs::write(fixture.artifacts.join("shared.ngraph"), &program).expect("restored bytes");
    std::os::unix::fs::symlink(fixture.artifacts.join("shared.ngraph"), fixture.artifacts.join("linked.ngraph"))
        .expect("link");
    std::fs::write(fixture.artifacts.join("linked.vectors"), &inputs).expect("inputs");
    let mut linked = arguments(&fixture, "shared", "attempt-2", profile::route::CEILINGS);
    linked["program"] = "linked.ngraph".into();
    linked["inputs"] = "linked.vectors".into();
    assert!(execute_with(&fixture, &linked, &[], None).is_err());
    let cyclic = b"ngraph-ir/0 ngraph-i64/0\nsubject authored 0000000000000000000000000000000000000000000000000000000000000000 closure -\nslot input i64\nslot input i64\n%0 = input 0\n%1 = input 1\n%2 = i64.add %0 %3\n%3 = i64.mul %0 %1\nout %2\n";
    write_artifacts(&fixture, "cyclic", cyclic, &inputs);
    assert!(
        execute_with(&fixture, &arguments(&fixture, "cyclic", "attempt-3", profile::route::CEILINGS), &[], None)
            .is_err()
    );
    let equality = String::from_utf8(program.clone()).unwrap().replace("i64.mul %2 %3", "i64.eq %2 %3");
    write_artifacts(&fixture, "equality", equality.as_bytes(), &inputs);
    assert!(
        execute_with(&fixture, &arguments(&fixture, "equality", "attempt-4", profile::route::CEILINGS), &[], None)
            .is_err()
    );
    assert!(!data_root(&fixture).join("operation-data").exists() || record_count(&fixture) == 0);
    let report = execute(&fixture, &arguments(&fixture, "shared", "attempt-5", profile::route::CEILINGS));
    assert_eq!(report.status, execution::Status::Completed, "the restored exact artifact still runs");
}

// r[verify lattice.pure_inet_profile.recovery.scenario.cancelled]
#[test]
fn admitted_cancellation_of_the_real_worker_publishes_no_result() {
    let fixture = fixture();
    write_artifacts(
        &fixture,
        "shared",
        &std::fs::read(format!("{FIXTURES}/program.ngraph")).unwrap(),
        &std::fs::read(format!("{FIXTURES}/inputs.vectors")).unwrap(),
    );
    let cancelled = AtomicBool::new(true);
    let report = execute_with(
        &fixture,
        &arguments(&fixture, "shared", "attempt-1", profile::route::CEILINGS),
        &[],
        Some(&cancelled),
    )
    .expect("durable cancelled attempt");
    assert_ne!(report.operation_state, lattice::operation::RunState::Completed);
    assert!(report.operation_data_version.is_none());
    let record = store(&fixture).load_operation_run(&report.invocation_identity).expect("run").expect("record");
    assert!(record.output_identity.is_none());
}

fn executable_script(root: &Path, body: &str) -> PathBuf {
    let shell = which("sh");
    let path = root.join("stand-in");
    std::fs::write(&path, format!("#!{}\n{body}", shell.display())).expect("script");
    std::fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("mode");
    path
}

fn which(program: &str) -> PathBuf {
    std::env::var_os("PATH")
        .and_then(|paths| std::env::split_paths(&paths).map(|path| path.join(program)).find(|path| path.is_file()))
        .expect("program on PATH")
}

// r[verify lattice.pure_inet_profile.recovery.scenario.cancelled]
#[test]
fn cancellation_on_the_profile_route_tears_down_an_owned_descendant() {
    let scratch = tempfile::tempdir().expect("script root");
    let sleep = which("sleep");
    let script =
        executable_script(scratch.path(), &format!("{} 30 &\necho $! > descendant.pid\nwait\n", sleep.display()));
    let fixture = fixture_with(&script);
    write_artifacts(
        &fixture,
        "shared",
        &std::fs::read(format!("{FIXTURES}/program.ngraph")).unwrap(),
        &std::fs::read(format!("{FIXTURES}/inputs.vectors")).unwrap(),
    );
    let cancellation = std::sync::Arc::new(AtomicBool::new(false));
    let trigger = std::sync::Arc::clone(&cancellation);
    let pid_file = fixture.artifacts.join("descendant.pid");
    let watcher = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while !pid_file.exists() && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
        trigger.store(true, std::sync::atomic::Ordering::SeqCst);
        std::fs::read_to_string(&pid_file).ok()
    });
    let report = execute_with(
        &fixture,
        &arguments(&fixture, "shared", "attempt-1", profile::route::CEILINGS),
        &[],
        Some(&cancellation),
    )
    .expect("durable attempt");
    let pid = watcher.join().expect("watcher").expect("descendant started").trim().to_string();
    assert_eq!(report.operation_state, lattice::operation::RunState::Unknown);
    assert!(report.operation_data_version.is_none());
    let stat = PathBuf::from(format!("/proc/{pid}/stat"));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let is_gone = || std::fs::read_to_string(&stat).map_or(true, |text| text.split(' ').nth(2) == Some("Z"));
    while !is_gone() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(is_gone(), "the owned descendant {pid} survived cancellation");
}

// r[verify lattice.pure_inet_profile.recovery.scenario.restart]
// r[verify lattice.pure_inet_profile.recovery.scenario.recompute]
#[test]
fn restart_retains_uncertainty_and_admits_only_fresh_lineage_bound_recomputation() {
    let fixture = fixture();
    write_artifacts(
        &fixture,
        "shared",
        &std::fs::read(format!("{FIXTURES}/program.ngraph")).unwrap(),
        &std::fs::read(format!("{FIXTURES}/inputs.vectors")).unwrap(),
    );
    let original = arguments(&fixture, "shared", "attempt-1", profile::route::CEILINGS);
    let expected = plan(&fixture, &original);
    std::fs::create_dir_all(data_root(&fixture)).expect("data root");
    let started = store(&fixture).begin_operation_run(&expected, 1).expect("durable start");
    assert!(execute_with(&fixture, &original, &[], None).is_err(), "no blind redispatch of an unsettled attempt");
    assert_eq!(store(&fixture).load_operation_run(&expected.identity).expect("run"), Some(started.clone()));
    let budget = profile::lineage::Budget {
        maximum_attempts: 2,
        consumed_attempts: 1,
    };
    let mut predecessor = profile::lineage::Predecessor {
        invocation: &expected.identity,
        attempt: "attempt-1",
        state: started.state,
        is_reconciled: false,
    };
    assert!(profile::lineage::admit_recomputation(&predecessor, "attempt-2", budget).is_err());
    predecessor.is_reconciled = true;
    let admitted =
        profile::lineage::admit_recomputation(&predecessor, "attempt-2", budget).expect("explicit recomputation");
    let mut fresh = arguments(&fixture, "shared", &admitted.attempt, profile::route::CEILINGS);
    fresh["predecessor"] = admitted.predecessor_invocation.clone().into();
    let report = execute(&fixture, &fresh);
    assert_eq!(report.status, execution::Status::Completed);
    assert_ne!(report.invocation_identity, expected.identity);
    assert_eq!(published(&fixture, &report).1["binding"]["predecessor"], expected.identity);
    assert_eq!(
        store(&fixture).load_operation_run(&expected.identity).expect("run"),
        Some(started),
        "predecessor stays unsettled"
    );
}

// r[verify lattice.pure_inet_profile.results.scenario.incomplete]
#[test]
fn interrupted_publication_keeps_the_observation_without_a_usable_result() {
    let fixture = fixture();
    write_artifacts(
        &fixture,
        "shared",
        &std::fs::read(format!("{FIXTURES}/program.ngraph")).unwrap(),
        &std::fs::read(format!("{FIXTURES}/inputs.vectors")).unwrap(),
    );
    let original = arguments(&fixture, "shared", "attempt-1", profile::route::CEILINGS);
    let expected = plan(&fixture, &original);
    let refused = execute_with(&fixture, &original, &["noble-net-i64-rules".to_string()], None);
    assert!(refused.is_err(), "publication admission refused the document");
    let record = store(&fixture).load_operation_run(&expected.identity).expect("run").expect("record");
    assert_eq!(record.state, lattice::operation::RunState::Completed);
    assert!(record.output_identity.is_some());
    assert_eq!(record_count(&fixture), 0, "no operation data was published");
    assert!(execute_with(&fixture, &original, &[], None).is_err(), "the observed invocation is not redispatched");
}

// r[verify lattice.pure_inet_profile.ownership.scenario.stale]
#[test]
fn consumer_rejects_a_real_result_after_its_binding_is_superseded() {
    let fixture = fixture();
    write_artifacts(
        &fixture,
        "shared",
        &std::fs::read(format!("{FIXTURES}/program.ngraph")).unwrap(),
        &std::fs::read(format!("{FIXTURES}/inputs.vectors")).unwrap(),
    );
    let snapshot = "5".repeat(64);
    let mut bound = arguments(&fixture, "shared", "attempt-1", profile::route::CEILINGS);
    bound["consumer_binding"] = snapshot.clone().into();
    let document = published(&fixture, &execute(&fixture, &bound)).1;
    assert!(profile::lineage::admit_for_consumer(&document, &snapshot).is_ok());
    assert_eq!(
        profile::lineage::admit_for_consumer(&document, &"6".repeat(64)),
        Err(profile::Error::SupersededConsumerBinding)
    );
}

#[test]
fn catalog_drift_from_the_exact_profile_is_refused() {
    let fixture = fixture();
    let mut catalog = profile::route::catalog(&fixture.pack_root.join("noble")).expect("catalog");
    assert!(lattice::operation::catalog::validate(catalog.clone()).is_ok());
    catalog.types[0].methods[0].side_effect = lattice::operation::model::SideEffectClass::Mutation;
    catalog.types[0].revision = lattice::operation::identity::type_revision(&catalog.types[0]).expect("revision");
    assert!(lattice::operation::catalog::validate(catalog).is_err());
}

// r[verify lattice.pure_inet_profile.bounds]
#[test]
fn profile_deadline_stops_a_stalled_worker_as_unknown_without_a_result() {
    let scratch = tempfile::tempdir().expect("script root");
    let sleep = which("sleep");
    let script = executable_script(scratch.path(), &format!("exec {} 600\n", sleep.display()));
    let fixture = fixture_with(&script);
    write_artifacts(
        &fixture,
        "shared",
        &std::fs::read(format!("{FIXTURES}/program.ngraph")).unwrap(),
        &std::fs::read(format!("{FIXTURES}/inputs.vectors")).unwrap(),
    );
    let started = std::time::Instant::now();
    let report = execute(&fixture, &arguments(&fixture, "shared", "attempt-1", profile::route::CEILINGS));
    let elapsed = started.elapsed();
    assert_eq!(report.operation_state, lattice::operation::RunState::Unknown);
    assert!(report.operation_data_version.is_none());
    assert!(elapsed >= std::time::Duration::from_secs(profile::route::BOUNDS.timeout_seconds));
    assert!(elapsed < std::time::Duration::from_secs(profile::route::BOUNDS.timeout_seconds + 30));
}
