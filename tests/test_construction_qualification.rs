//! Issue #20/#21 qualification controls for the captured local-shell construction profile.
//! Every positive case is paired with a negative control through the same production route.
//! These are bounded consumer observations, not semantic-equivalence or release evidence.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use choregraph_nickel::construction as native;
use lattice::authoring::CapturedProgram;
use lattice::authoring::ProgramKind;
use lattice::authoring::SourceDraft;
use lattice::authoring::construction;
use lattice::authoring::{self};
use lattice::engine::Workflow;
use lattice::engine::effect::standalone;
use lattice::engine::workflow::NodeConfig;
use lattice::headless::HeadlessRunStatus;
use lattice::headless::{self};
use lattice::store::Store;

const LOCAL_SHELL: &str = "local-shell-construction.ncl";
const PARAMETERIZED: &str = "parameterized-shell-construction.ncl";
const ROLES: [&str; 6] = ["fork", "left", "right", "worker-a", "worker-b", "done"];
const WORKERS: [&str; 4] = ["left", "right", "worker-a", "worker-b"];
const STAGES: [&str; 5] = ["first", "second", "a", "b", "c"];
const SEQUENCE_SOURCE: &str = r#"# Two effects ordered only by a Unit/zero-data control dependency.
let g = import "./choregraph/graph-construction-v1.ncl" in
{
  parameters = {first = "printf 'first\\n'", second = "printf 'second\\n'"},
  build = fun p =>
    let effect = [{
      key = "invoke", from_state = "ready", action = "start",
      resource = "lattice://process/construction-shell-v1", ability = "process/execute",
      request = g.slot_binding "dispatch-request" "invocation" "dispatch-request",
      payload = g.slot_binding "resolved-payload" "invocation" "resolved-payload",
    }] in
    g.finish "guarded-sequence" "lattice-local-shell-v1" "lattice-local-shell-registry-v1"
      {} [] (g.sequence "root" [
        g.wire "first" [g.operation "first" "shell-source" {command = g.literal p.first} effect] [] [] [],
        g.wire "second" [g.operation "second" "shell-source" {command = g.literal p.second} effect] [] [] [],
      ]),
}
"#;

fn selected_worker() {
    assert!(
        lattice::nickel::captured_worker().is_ok(),
        "qualification must select the exact companion worker artifact"
    );
}

fn captured(entry: &str, bytes: &[u8], kind: ProgramKind) -> CapturedProgram {
    CapturedProgram {
        entry: entry.into(),
        files: BTreeMap::from([(entry.into(), bytes.to_vec())]),
        kind,
        parameters: BTreeMap::new(),
    }
}

fn local_shell() -> CapturedProgram {
    captured(
        LOCAL_SHELL,
        include_bytes!("../examples/construction/local-shell-construction.ncl"),
        ProgramKind::Construction,
    )
}

fn parameterized(parameters: &[(&str, String)]) -> CapturedProgram {
    let mut program = captured(
        PARAMETERIZED,
        include_bytes!("../examples/construction/parameterized-shell-construction.ncl"),
        ProgramKind::ParameterizedConstruction,
    );
    for (name, value) in parameters {
        program.parameters.insert((*name).to_owned(), serde_json::json!(value));
    }
    program
}

fn sequence(first: String, second: String) -> CapturedProgram {
    let mut program = captured("sequence.ncl", SEQUENCE_SOURCE.as_bytes(), ProgramKind::ParameterizedConstruction);
    program.parameters.insert("first".into(), serde_json::json!(first));
    program.parameters.insert("second".into(), serde_json::json!(second));
    program
}

fn quoted(path: &std::path::Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\"'\"'"))
}

/// Source preview and exact admission through the durable store route.
fn admit(store: &Store, id: &str, name: &str, program: CapturedProgram) -> Workflow {
    let mut draft = SourceDraft {
        id: id.into(),
        workflow_name: name.into(),
        expected_revision: store.load_workflow(name).unwrap().and_then(|workflow| workflow.catalog_revision),
        program,
        diagnostic: None,
        preview: None,
    };
    store.save_source_draft(&draft).unwrap();
    store.preview_source_draft(&mut draft).unwrap();
    store.admit_source_draft(&draft).unwrap()
}

/// Exact run-scoped authority for one admitted revision, loaded by the operator source.
fn exact_authority(
    directory: &std::path::Path,
    label: &str,
    workflow: &Workflow,
) -> standalone::OperatorRunContextSource {
    let inspection = standalone::inspect_workflow(workflow).unwrap();
    let profile = standalone::core::Profile {
        schema: inspection.schema,
        workflow_revision: inspection.workflow_revision,
        machine_identity: inspection.machine_identity,
        policy_ref: inspection.policy_ref,
        expires_at_unix_ms: 4_102_444_800_000,
        grants: inspection.grants,
        roots: BTreeMap::new(),
    };
    let grant = format!("{label}-grant.json");
    std::fs::write(directory.join(&grant), serde_json::to_vec(&profile).unwrap()).unwrap();
    let authority = directory.join(format!("{label}-authority.ncl"));
    std::fs::write(&authority, format!("import \"{grant}\"\n")).unwrap();
    standalone::OperatorRunContextSource::new(authority)
}

fn role_of(workflow: &Workflow, node: u64, roles: &[&'static str]) -> Option<&'static str> {
    let origin = workflow
        .authoring
        .as_ref()?
        .construction
        .as_ref()?
        .origins
        .iter()
        .find(|origin| origin.element == format!("node/{node}"))?;
    roles.iter().copied().find(|role| origin.instance_path.iter().any(|part| part == role))
}

fn stage(workflow: &Workflow, role: &str) -> lattice::engine::workflow::NodeId {
    workflow.graph.nodes().find(|node| role_of(workflow, node.id.0, &STAGES) == Some(role)).unwrap().id
}

fn command(workflow: &Workflow, node: lattice::engine::workflow::NodeId) -> (&str, bool) {
    match workflow.node_config(node).unwrap() {
        NodeConfig::Shell {
            command, pipe_stdin, ..
        } => (command, *pipe_stdin),
        _ => panic!("selected construction node must use the shell implementation"),
    }
}

fn materials_for<'a>(
    observation: &'a lattice::authoring::EvaluatorObservation,
    modules: &'a [native::association::CapturedModule<'a>],
    evaluator_binding: &'a [u8],
) -> native::association::CompilationMaterials<'a> {
    native::association::CompilationMaterials {
        modules,
        entry: &observation.entry,
        evaluator_identity: &observation.worker_artifact,
        evaluator_profile: nickel_eval::captured::PROFILE,
        evaluator_binding,
        evaluated_output_blake3: &observation.result.binding.output_blake3,
        library: native::model::profile::LIBRARY_BYTES,
        contracts: native::model::profile::CONTRACTS_BYTES,
        lowerer_identity: env!("LATTICE_WORKFLOW_LOWERER_BLAKE3"),
        toolchain_identity: env!("LATTICE_DEPENDENCY_LOCK_BLAKE3"),
    }
}

/// Run admission replays the captured worker synchronously before the first effect,
/// so the bound covers that replay as well as process start.
async fn wait_for(path: &std::path::Path) {
    tokio::time::timeout(std::time::Duration::from_secs(120), async {
        while !path.exists() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("selected effect starts under its exact authority");
}

fn refused(outcome: Result<headless::HeadlessRunSummary, headless::HeadlessRunError>) -> bool {
    match outcome {
        Err(_) => true,
        Ok(summary) => summary.status == HeadlessRunStatus::Failed && summary.completed == 0,
    }
}

// r[verify lattice.graph_construction.instances]
// r[verify lattice.graph_construction.instances.remap]
#[test]
#[ignore = "requires exact captured native worker and immutable local runtime; host qualification runs ignored tests"]
fn generated_instances_keep_identities_shared_handle_and_attached_facts() {
    selected_worker();
    let workflow = authoring::shell::compile(&local_shell()).unwrap();
    let admitted = construction::admit_runtime(&workflow).unwrap().unwrap();
    let projection = &admitted.projection;
    let roles = workflow
        .graph
        .nodes()
        .map(|node| (role_of(&workflow, node.id.0, &ROLES).unwrap(), node.id))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(roles.keys().copied().collect::<BTreeSet<_>>(), ROLES.into_iter().collect());
    assert_eq!(workflow.graph.node_count(), ROLES.len(), "a reused output handle adds no operation");
    assert_eq!(projection.nodes.len(), ROLES.len());

    let origins = &workflow.authoring.as_ref().unwrap().construction.as_ref().unwrap().origins;
    let origin =
        |role: &str| origins.iter().find(|origin| origin.element == format!("node/{}", roles[role].0)).unwrap();
    assert_eq!(origin("left").template, "worker");
    assert_eq!(origin("right").template, "worker");
    assert_ne!(origin("left").instance_path, origin("right").instance_path);
    assert_ne!(roles["left"], roles["right"]);
    let left_ports = &workflow.graph.node(roles["left"]).unwrap().input_ports;
    let right_ports = &workflow.graph.node(roles["right"]).unwrap().input_ports;
    assert_ne!(left_ports[0].id, right_ports[0].id);

    let fork = workflow.graph.node(roles["fork"]).unwrap();
    let shared = fork.output_ports.iter().find(|port| port.label == "stdout").unwrap().id;
    let consumers = workflow
        .graph
        .edges()
        .iter()
        .filter(|edge| edge.source == shared)
        .map(|edge| workflow.graph.port_owner(edge.target).unwrap())
        .collect::<BTreeSet<_>>();
    let expected = WORKERS.iter().map(|role| roles[role]).collect::<BTreeSet<_>>();
    assert_eq!(consumers, expected, "nested exposed stdin and cross-instance handle resolve to each instance");
    assert_eq!(workflow.graph.edges().len(), WORKERS.len());

    let registry = construction::registry();
    for node in &projection.nodes {
        let role = role_of(&workflow, node.id, &ROLES).unwrap();
        let signature = registry.operations.iter().find(|signature| signature.operation == node.operation).unwrap();
        assert_eq!(node.implementation, signature.implementation);
        let effects = projection.effects.iter().filter(|effect| effect.node_id == node.id).collect::<Vec<_>>();
        assert_eq!(effects.len(), 1, "{role} keeps exactly its own effect");
        assert_eq!(effects[0].resource, construction::RESOURCE);
        let native::model::projection::ExpandedBinding::Slot { source_ref, .. } = &effects[0].request_binding else {
            panic!("selected effect binds its invocation slot")
        };
        assert_eq!(source_ref, &format!("node/{}/invocation/dispatch-request", node.id));
        let (actual, is_stdin) = command(&workflow, lattice::engine::workflow::NodeId(node.id));
        let expected = match role {
            "fork" => ("printf 'alpha\\n'", false),
            "done" => ("printf 'joined\\n'", false),
            _ => ("cat", true),
        };
        assert_eq!((actual, is_stdin), expected, "{role} configuration follows its instance after remapping");
        assert_eq!(node.operation, if is_stdin { "shell-stdin" } else { "shell-source" });
    }
    let controls = workflow
        .control_dependencies
        .iter()
        .map(|&(from, to)| (role_of(&workflow, from, &ROLES).unwrap(), role_of(&workflow, to, &ROLES).unwrap()))
        .collect::<BTreeSet<_>>();
    assert_eq!(controls, WORKERS.iter().map(|worker| (*worker, "done")).collect());
}

// r[verify lattice.graph_construction.instances.sharing]
#[test]
#[ignore = "requires exact captured native worker and immutable local runtime; host qualification runs ignored tests"]
fn fanout_and_join_follow_selected_copy_and_drop_rules() {
    selected_worker();
    let workflow = authoring::shell::compile(&local_shell()).unwrap();
    let observation = workflow.authoring.as_ref().unwrap().observation.as_ref().unwrap();
    let modules = observation
        .modules
        .iter()
        .map(|(logical_id, bytes)| native::association::CapturedModule {
            logical_id,
            bytes: bytes.as_slice(),
        })
        .collect::<Vec<_>>();
    let binding = serde_json::to_vec(&observation.result.binding).unwrap();
    let materials = materials_for(observation, &modules, &binding);
    let candidate = observation.result.json.as_bytes();
    let limits = native::model::profile::Limits::default();
    let host = construction::registry();
    assert!(native::compile_json(candidate, &host, &materials, limits).is_ok());
    let projection = serde_json::to_vec(&construction::admit_runtime(&workflow).unwrap().unwrap().projection).unwrap();
    for (name, copyable, droppable) in [("string", false, true), ("exitcode", true, false)] {
        let mut restricted = host.clone();
        let ty = restricted.types.iter_mut().find(|ty| ty.name == name).unwrap();
        ty.copyable = copyable;
        ty.droppable = droppable;
        assert!(
            native::compile_json(candidate, &restricted, &materials, limits).is_err(),
            "{name} fanout/discard must obey the selected registry during elaboration"
        );
        assert!(
            native::admission::lower_projection_json(&projection, &restricted, limits).is_err(),
            "{name} fanout/discard must obey the selected registry during admission"
        );
    }
}

// r[verify lattice.graph_construction.rejection]
// r[verify lattice.graph_construction.rejection.overflow]
#[test]
#[ignore = "requires exact captured native worker and immutable local runtime; host qualification runs ignored tests"]
fn generation_and_expansion_overflow_reject_before_any_candidate() {
    selected_worker();
    let mut capped = local_shell();
    let source = String::from_utf8(capped.files.remove(LOCAL_SHELL).unwrap()).unwrap();
    let generation = "g.generate 2 [\"worker-a\", \"worker-b\"]";
    assert_eq!(source.matches(generation).count(), 1);
    capped.files.insert(
        LOCAL_SHELL.into(),
        source.replace(generation, "g.generate 1 [\"worker-a\", \"worker-b\"]").into_bytes(),
    );
    assert!(authoring::shell::compile(&capped).is_err(), "source generation cap is enforced by the library");

    let workflow = authoring::shell::compile(&local_shell()).unwrap();
    let observation = workflow.authoring.as_ref().unwrap().observation.as_ref().unwrap();
    let modules = observation
        .modules
        .iter()
        .map(|(logical_id, bytes)| native::association::CapturedModule {
            logical_id,
            bytes: bytes.as_slice(),
        })
        .collect::<Vec<_>>();
    let binding = serde_json::to_vec(&observation.result.binding).unwrap();
    let materials = materials_for(observation, &modules, &binding);
    let candidate = observation.result.json.as_bytes();
    let host = construction::registry();
    let fits = native::model::profile::Limits {
        max_nodes: ROLES.len(),
        ..native::model::profile::Limits::default()
    };
    assert!(native::compile_json(candidate, &host, &materials, fits).is_ok());
    for limits in [
        native::model::profile::Limits {
            max_nodes: ROLES.len() - 1,
            ..native::model::profile::Limits::default()
        },
        native::model::profile::Limits {
            max_instances: 1,
            ..native::model::profile::Limits::default()
        },
    ] {
        assert!(native::compile_json(candidate, &host, &materials, limits).is_err());
    }
}

// r[verify lattice.graph_construction.drift]
#[test]
#[ignore = "requires exact captured native worker and immutable local runtime; host qualification runs ignored tests"]
fn parameter_member_and_elaborator_drift_invalidate_the_binding() {
    selected_worker();
    let workflow = authoring::shell::compile(&parameterized(&[])).unwrap();
    assert!(construction::admit_runtime(&workflow).unwrap().is_some());

    let mut parameter = workflow.clone();
    let source = parameter.authoring.as_mut().unwrap();
    Arc::make_mut(&mut source.program)
        .parameters
        .insert("first".into(), serde_json::json!("printf 'drift\\n'"));
    source.source_revision = source.program.identity().unwrap();
    assert!(construction::admit_runtime(&parameter).is_err(), "a changed parameter needs a new evaluation");
    assert!(standalone::inspect_workflow(&parameter).is_err());

    let mut elaborator = workflow.clone();
    elaborator.authoring.as_mut().unwrap().compiler.lowerer_adapter =
        lattice::authoring::ContentIdentity::from_bytes(b"another elaborator");
    assert!(construction::admit_runtime(&elaborator).is_err());
    let mut evaluator = workflow.clone();
    evaluator.authoring.as_mut().unwrap().compiler.evaluator_source.push_str("-other");
    assert!(construction::admit_runtime(&evaluator).is_err());

    let mut wrapped = local_shell();
    wrapped.entry = "wrapper.ncl".into();
    wrapped.files.insert("wrapper.ncl".into(), b"import \"./local-shell-construction.ncl\"\n".to_vec());
    let imported = authoring::shell::compile(&wrapped).unwrap();
    assert!(construction::admit_runtime(&imported).unwrap().is_some());
    assert!(imported.authoring.as_ref().unwrap().observation.as_ref().unwrap().modules.contains_key(LOCAL_SHELL));
    let mut observed = imported.clone();
    observed
        .authoring
        .as_mut()
        .unwrap()
        .observation
        .as_mut()
        .unwrap()
        .modules
        .get_mut(LOCAL_SHELL)
        .unwrap()
        .push(b'\n');
    assert!(construction::admit_runtime(&observed).is_err(), "an imported member cannot drift under its receipt");
    let mut retained = imported;
    let source = retained.authoring.as_mut().unwrap();
    Arc::make_mut(&mut source.program).files.get_mut(LOCAL_SHELL).unwrap().push(b'\n');
    source.source_revision = source.program.identity().unwrap();
    assert!(construction::admit_runtime(&retained).is_err());
    assert!(standalone::inspect_workflow(&retained).is_err());
}

// r[verify lattice.graph_construction.authority]
// r[verify lattice.graph_construction.inert_authoring]
// r[verify lattice.graph_construction.authority.same_graph]
// r[verify lattice.graph_construction.inert_authoring.marker]
#[tokio::test]
#[ignore = "requires exact captured native worker and immutable local runtime; host qualification runs ignored tests"]
async fn same_constructed_graph_runs_only_with_its_exact_authority() {
    selected_worker();
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("effect-ran");
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let workflow = admit(
        &store,
        "effectful",
        "param-shell",
        parameterized(&[("first", format!("touch {}", quoted(&marker)))]),
    );
    let first = stage(&workflow, "first");
    assert!(construction::lowering::origin_for(&workflow, &format!("node/{}", first.0)).is_some());
    assert!(authoring::shell::source_export(&workflow).is_ok());
    assert!(authoring::shell::expanded_export(&workflow).contains("touch"));
    let mut with_credentials = workflow.clone();
    with_credentials.set_node_credentials(first, lattice::credentials::NodeCredentialRefs {
        env: std::collections::HashMap::from([(
            "TOKEN".into(),
            lattice::credentials::SecretRef::new("env", "CONSTRUCTION_QUALIFICATION_TOKEN", None),
        )]),
    });
    assert!(construction::admit_runtime(&with_credentials).is_err(), "credentials cannot join an admitted graph");
    assert!(standalone::inspect_workflow(&with_credentials).is_err());
    assert!(!marker.exists(), "preview, admission, navigation, export and inspection start no workload");
    assert!(store.list_runs(10).unwrap().is_empty());

    let denied = headless::run_saved_workflow(&store, "param-shell", store.next_run_id().unwrap()).await.unwrap();
    assert_eq!(denied.status, HeadlessRunStatus::Failed);
    assert_eq!(denied.completed, 0);
    assert!(!marker.exists(), "absent authority fails before the shell adapter");

    let other = authoring::shell::compile(&parameterized(&[])).unwrap();
    let foreign = exact_authority(directory.path(), "foreign", &other);
    let mismatched = headless::run_saved_workflow_with_authority_provider(
        &store,
        "param-shell",
        store.next_run_id().unwrap(),
        &foreign,
    )
    .await;
    assert!(refused(mismatched));
    assert!(!marker.exists(), "authority for another graph does not authorize this one");

    let exact = exact_authority(directory.path(), "exact", &workflow);
    let summary = headless::run_saved_workflow_with_authority_provider(
        &store,
        "param-shell",
        store.next_run_id().unwrap(),
        &exact,
    )
    .await
    .unwrap();
    assert_eq!(summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(summary.completed, 4);
    assert!(marker.exists(), "the same admitted graph runs with exact local fixture authority");
    assert!(store.load_run(summary.run_id).unwrap().is_some());
}

// r[verify lattice.graph_construction.snapshot]
// r[verify lattice.graph_construction.snapshot.readmit]
#[tokio::test]
#[ignore = "requires exact captured native worker and immutable local runtime; host qualification runs ignored tests"]
async fn in_flight_run_keeps_its_exact_admitted_snapshot_while_source_readmits() {
    selected_worker();
    let directory = tempfile::tempdir().unwrap();
    let path = |name: &str| directory.path().join(name);
    let store = Store::new(path("state.redb")).unwrap();
    let blocking = format!(
        "touch {}; while [ ! -e {} ]; do sleep 0.02; done; touch {}",
        quoted(&path("started")),
        quoted(&path("continue")),
        quoted(&path("old-ran"))
    );
    let original = admit(&store, "original", "param-shell", parameterized(&[("first", blocking)]));
    let authority = exact_authority(directory.path(), "original", &original);
    let started = path("started");
    let run = headless::run_saved_workflow_with_authority_provider(
        &store,
        "param-shell",
        store.next_run_id().unwrap(),
        &authority,
    );
    let readmit = async {
        wait_for(&started).await;
        let newer = admit(
            &store,
            "newer",
            "param-shell",
            parameterized(&[("first", format!("touch {}", quoted(&path("new-ran"))))]),
        );
        assert_ne!(newer.catalog_revision, original.catalog_revision);
        std::fs::write(path("continue"), b"continue").unwrap();
        newer
    };
    let (summary, newer) = tokio::join!(run, readmit);
    assert_eq!(summary.unwrap().status, HeadlessRunStatus::Succeeded);
    assert!(path("old-ran").exists(), "the in-flight run completes its admitted snapshot");
    assert!(!path("new-ran").exists(), "readmission does not mutate the running graph");
    let current = store.load_workflow("param-shell").unwrap().unwrap();
    assert_eq!(current.catalog_revision, newer.catalog_revision);
    assert!(command(&current, stage(&current, "first")).0.contains("new-ran"));

    let stale = headless::run_saved_workflow_with_authority_provider(
        &store,
        "param-shell",
        store.next_run_id().unwrap(),
        &authority,
    )
    .await;
    assert!(refused(stale));
    assert!(!path("new-ran").exists(), "the old snapshot's authority does not authorize the new revision");
}

fn sequence_store(
    directory: &std::path::Path,
    first: String,
    second: String,
) -> (Store, Workflow, standalone::OperatorRunContextSource) {
    let store = Store::new(directory.join("state.redb")).unwrap();
    let workflow = admit(&store, "sequence", "guarded-sequence", sequence(first, second));
    assert_eq!(workflow.control_dependencies.len(), 1);
    assert!(workflow.graph.edges().is_empty(), "the sequence carries no payload connection");
    let authority = exact_authority(directory, "sequence", &workflow);
    (store, workflow, authority)
}

async fn snapshot_run(
    store: &Store,
    workflow: Workflow,
    authority: &standalone::OperatorRunContextSource,
    cancellation: Arc<AtomicBool>,
) -> headless::SnapshotRunOutcome {
    headless::run_workflow_snapshot_with_authority_provider_and_cancellation(
        store,
        headless::SnapshotRunRequest {
            workflow,
            run_id: store.next_run_id().unwrap(),
            owner: lattice::store::RunOwner::Unscoped,
            agent_runtime: lattice::agent_node::RuntimeOptions::default(),
            cancellation,
        },
        authority,
    )
    .await
    .unwrap()
}

// r[verify lattice.graph_construction.runtime_controls]
// r[verify lattice.graph_construction.runtime_controls.result]
#[tokio::test]
#[ignore = "requires exact captured native worker and immutable local runtime; host qualification runs ignored tests"]
async fn result_validation_and_cancellation_stop_the_dependent_effect() {
    selected_worker();
    for (first, is_success) in [("printf 'ok\\n'", true), ("exit 3", false)] {
        let directory = tempfile::tempdir().unwrap();
        let marker = directory.path().join("second-ran");
        let (store, workflow, authority) =
            sequence_store(directory.path(), first.into(), format!("touch {}", quoted(&marker)));
        let outcome = snapshot_run(&store, workflow, &authority, Arc::new(AtomicBool::new(false))).await;
        assert!(!outcome.cancelled);
        assert_eq!(outcome.summary.status == HeadlessRunStatus::Succeeded, is_success, "{first}");
        assert_eq!(marker.exists(), is_success, "a rejected result never releases its dependent: {first}");
    }

    let directory = tempfile::tempdir().unwrap();
    let started = directory.path().join("started");
    let marker = directory.path().join("second-ran");
    let (store, workflow, authority) = sequence_store(
        directory.path(),
        format!("touch {}; while :; do sleep 0.02; done", quoted(&started)),
        format!("touch {}", quoted(&marker)),
    );
    let cancellation = Arc::new(AtomicBool::new(false));
    let cancel = async {
        wait_for(&started).await;
        cancellation.store(true, Ordering::Release);
    };
    let (outcome, ()) = tokio::join!(snapshot_run(&store, workflow, &authority, Arc::clone(&cancellation)), cancel);
    assert!(outcome.cancelled, "owned process termination is observed");
    assert_eq!(outcome.summary.status, HeadlessRunStatus::Failed);
    assert!(!marker.exists(), "cancellation stops the dependent effect");
}

fn budgeted_definition(workflow: &Workflow, limit: u64) -> lattice::orchestration::Definition {
    use lattice::orchestration::*;
    Definition {
        schema_version: current_schema_version(),
        id: DefinitionId::try_from_text("construction-budget").unwrap(),
        objective: "Run one constructed graph round under an enforced budget".into(),
        workflow: workflow_revision(workflow).unwrap(),
        completion: CompletionPolicy::RunSucceeded,
        retry: None,
        budgets: BudgetPolicy {
            enforced: BudgetDimensions {
                rounds: true,
                ..BudgetDimensions::default()
            },
            limits: BudgetLimits {
                rounds: limit,
                ..BudgetLimits::default()
            },
            per_round_upper_bound: BudgetCharge {
                rounds: limit,
                ..BudgetCharge::default()
            },
        },
        authority: AuthorityPolicy {
            approvers: BTreeSet::from(["operator:construction".to_owned()]),
        },
        triggers: Vec::new(),
        redaction: RedactionPolicy {
            retain_safe_summaries: true,
        },
    }
}

// r[verify lattice.graph_construction.runtime_controls.budget]
#[tokio::test]
#[ignore = "requires exact captured native worker and immutable local runtime; host qualification runs ignored tests"]
async fn orchestration_budget_gates_constructed_rounds() {
    selected_worker();
    for (limit, is_funded) in [(1, true), (0, false)] {
        let directory = tempfile::tempdir().unwrap();
        let marker = directory.path().join("second-ran");
        let (store, workflow, authority) =
            sequence_store(directory.path(), "printf 'first\\n'".into(), format!("touch {}", quoted(&marker)));
        let started = lattice::orchestration::start_with_execution(
            &store,
            budgeted_definition(&workflow, limit),
            "operator:construction",
            lattice::orchestration::ProductExecution {
                agent_runtime: lattice::agent_node::RuntimeOptions::default(),
                authority_provider: &authority,
            },
        )
        .await;
        let runs = store.list_runs(10).unwrap();
        if is_funded {
            let instance = started.unwrap();
            assert_eq!(instance.status, lattice::orchestration::Status::Completed);
            assert_eq!(instance.budgets.consumed.rounds, 1);
            assert_eq!(runs.len(), 1);
            assert!(marker.exists());
        } else {
            assert!(started.is_err() || started.is_ok_and(|instance| instance.budgets.consumed.rounds == 0));
            assert!(runs.is_empty(), "an exhausted budget dispatches no constructed round");
            assert!(!marker.exists());
        }
    }
}
