//! Native captured construction regression controls. The isolated worker is required.
use std::collections::BTreeMap;

use lattice::authoring::CapturedProgram;
use lattice::authoring::ProgramKind;
use lattice::authoring::SourceDraft;
use lattice::authoring::construction;
use lattice::authoring::{self};
use lattice::engine::effect::execution;
use lattice::engine::effect::standalone;
use lattice::headless::HeadlessRunStatus;
use lattice::store::Store;

fn program() -> CapturedProgram {
    CapturedProgram {
        entry: "local-shell-construction.ncl".into(),
        files: BTreeMap::from([(
            "local-shell-construction.ncl".into(),
            include_bytes!("../examples/construction/local-shell-construction.ncl").to_vec(),
        )]),
        kind: ProgramKind::Construction,
        parameters: BTreeMap::new(),
    }
}

fn selected_worker() {
    assert!(
        lattice::nickel::captured_worker().is_ok(),
        "qualification must select the exact companion worker artifact"
    );
}

// r[verify lattice.graph_construction.profile]
// r[verify lattice.graph_construction.closure]
// r[verify lattice.graph_construction.closure.reopen]
// r[verify lattice.graph_construction.origins]
#[test]
#[ignore = "requires exact captured native worker; Nix qualification runs ignored tests"]
fn captured_instances_reopen_with_exact_closure_and_native_origins() {
    selected_worker();
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let mut draft = SourceDraft {
        id: "construction".into(),
        workflow_name: "local-shell-demo".into(),
        expected_revision: None,
        program: program(),
        diagnostic: None,
        preview: None,
    };
    store.save_source_draft(&draft).unwrap();
    store.preview_source_draft(&mut draft).unwrap();
    let workflow = store.admit_source_draft(&draft).unwrap();
    assert!(workflow.construction_profile);
    let admitted = construction::admit_runtime(&workflow).unwrap().unwrap();
    assert_eq!(admitted.projection.nodes.len(), 6);
    assert_eq!(admitted.projection.edges.len(), 4);
    assert!(!admitted.projection.control_edges.is_empty());
    let observed = workflow.authoring.as_ref().unwrap().observation.as_ref().unwrap();
    assert!(
        observed
            .result
            .binding
            .observed
            .iter()
            .any(|module| module.name == "choregraph/graph-construction-v1.ncl")
    );
    assert!(
        observed
            .result
            .binding
            .observed
            .iter()
            .any(|module| module.name == "choregraph/graph-construction-contracts-v1.ncl")
    );
    let reopened = store.load_workflow("local-shell-demo").unwrap().unwrap();
    assert_eq!(
        reopened.authoring.as_ref().unwrap().source_revision,
        workflow.authoring.as_ref().unwrap().source_revision
    );
    assert_eq!(construction::admit_runtime(&reopened).unwrap().unwrap().projection, admitted.projection);
    let worker = reopened.graph.nodes().find(|node| node.label.contains("left")).unwrap();
    let origin = construction::lowering::origin_for(&reopened, &format!("node/{}", worker.id.0)).unwrap();
    assert_eq!(origin.template, "worker");
    let span = origin.location.unwrap();
    let captured = reopened.authoring.as_ref().unwrap();
    let bytes = &captured.observation.as_ref().unwrap().modules[&span.logical_module];
    assert!(std::str::from_utf8(&bytes[span.start..span.end]).unwrap().contains("shell-stdin"));
    let native = captured.construction.as_ref().unwrap();
    let output_port = worker.output_ports[0].id.0;
    assert!(native.port_map.contains_key(&output_port));
}

// r[verify lattice.graph_construction.authority]
#[tokio::test]
#[ignore = "requires exact captured native worker; Nix qualification runs ignored tests"]
async fn absent_authority_denies_before_shell_adapter_and_new_grants_do_not_match_v2() {
    selected_worker();
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let mut draft = SourceDraft {
        id: "authority".into(),
        workflow_name: "local-shell-demo".into(),
        expected_revision: None,
        program: program(),
        diagnostic: None,
        preview: None,
    };
    store.preview_source_draft(&mut draft).unwrap();
    let mut workflow = store.admit_source_draft(&draft).unwrap();
    let marker = directory.path().join("must-not-run");
    let first = workflow.graph.nodes().next().unwrap().id;
    let config = workflow.configs.get_mut(&first.0).unwrap();
    if let lattice::engine::workflow::NodeConfig::Shell { command, .. } = config {
        *command = format!("touch {}", marker.display());
    }
    authoring::reconcile_materialized(&mut workflow).unwrap();
    let requirements = standalone::inspect_workflow(&workflow).unwrap();
    assert!(requirements.grants.iter().all(|grant| grant.resource == construction::RESOURCE));
    assert!(requirements.grants.iter().all(|grant| grant.operation != "lattice/shell"));
    store.save_workflow_cas(&workflow, workflow.catalog_revision.as_ref()).unwrap();
    let run = store.next_run_id().unwrap();
    let summary = lattice::headless::run_saved_workflow_with_authority_provider(
        &store,
        "local-shell-demo",
        run,
        &execution::UnavailableRunContextSource,
    )
    .await
    .unwrap();
    assert_eq!(summary.status, HeadlessRunStatus::Failed);
    assert!(!marker.exists());
}

// r[verify lattice.graph_construction.drift]
// r[verify lattice.graph_construction.drift.registry]
#[test]
#[ignore = "requires exact captured native worker; Nix qualification runs ignored tests"]
fn mutated_closure_registry_and_projection_never_authorize_a_retained_receipt() {
    selected_worker();
    let workflow = authoring::shell::compile(&program()).unwrap();
    let mut changed = workflow.clone();
    changed
        .authoring
        .as_mut()
        .unwrap()
        .observation
        .as_mut()
        .unwrap()
        .modules
        .get_mut("local-shell-construction.ncl")
        .unwrap()
        .push(b' ');
    assert!(construction::admit_runtime(&changed).is_err());
    let mut changed = workflow.clone();
    changed.authoring.as_mut().unwrap().construction.as_mut().unwrap().projection_json.push(b' ');
    assert!(construction::admit_runtime(&changed).is_err());
    let mut changed = workflow.clone();
    changed.authoring.as_mut().unwrap().construction.as_mut().unwrap().binding_json.push(b' ');
    assert!(construction::admit_runtime(&changed).is_err());
    let mut changed = workflow.clone();
    changed.authoring.as_mut().unwrap().construction.as_mut().unwrap().port_map.clear();
    assert!(construction::admit_runtime(&changed).is_err());
    let mut changed = workflow.clone();
    changed
        .authoring
        .as_mut()
        .unwrap()
        .observation
        .as_mut()
        .unwrap()
        .modules
        .get_mut("choregraph/graph-construction-contracts-v1.ncl")
        .unwrap()
        .push(b' ');
    assert!(construction::admit_runtime(&changed).is_err());
    let mut changed = workflow.clone();
    changed
        .authoring
        .as_mut()
        .unwrap()
        .observation
        .as_mut()
        .unwrap()
        .result
        .binding
        .policy
        .push_str("-different");
    assert!(construction::admit_runtime(&changed).is_err());
    let mut changed = workflow.clone();
    changed.authoring.as_mut().unwrap().observation.as_mut().unwrap().worker_artifact.push('0');
    assert!(construction::admit_runtime(&changed).is_err());
    let mut changed = workflow.clone();
    let node = changed.graph.nodes().next().unwrap().id;
    if let lattice::engine::workflow::NodeConfig::Shell { command, .. } = changed.configs.get_mut(&node.0).unwrap() {
        command.push_str(" # different graph");
    }
    assert!(construction::admit_runtime(&changed).is_err());
    use choregraph_nickel::construction as c;
    let admitted = construction::admit_runtime(&workflow).unwrap().unwrap();
    let mut foreign = admitted.projection.clone();
    if let c::model::projection::ExpandedBinding::Slot { source_ref, .. } = &mut foreign.effects[0].request_binding {
        *source_ref = "node/999/invocation/dispatch-request".into();
    } else {
        panic!("selected construction effect must bind the invocation slot");
    }
    assert!(
        c::admission::lower_projection_json(
            &serde_json::to_vec(&foreign).unwrap(),
            &construction::registry(),
            c::model::profile::Limits::default()
        )
        .is_err()
    );
    let mut foreign = admitted.projection;
    foreign.effects[0].resource = "lattice://process/0".into();
    assert!(
        c::admission::lower_projection_json(
            &serde_json::to_vec(&foreign).unwrap(),
            &construction::registry(),
            c::model::profile::Limits::default()
        )
        .is_err()
    );
}
// r[verify lattice.graph_construction.drift.provenance]
#[test]
#[ignore = "requires exact captured native worker; Nix qualification runs ignored tests"]
fn equal_export_does_not_merge_distinct_source_or_observed_closure() {
    selected_worker();
    let original = authoring::shell::compile(&program()).unwrap();
    let mut edited = program();
    edited
        .files
        .get_mut("local-shell-construction.ncl")
        .unwrap()
        .extend_from_slice(b"\n# same output, distinct source\n");
    let changed = authoring::shell::compile(&edited).unwrap();
    let left = original.authoring.unwrap();
    let right = changed.authoring.unwrap();
    assert_eq!(left.evaluated_output, right.evaluated_output);
    assert_ne!(left.source_revision, right.source_revision);
    assert_ne!(
        left.observation.unwrap().result.binding.observed,
        right.observation.unwrap().result.binding.observed
    );
}

fn reference_shell(command: &str, with_stdin: bool) -> lattice::engine::workflow::NodeConfig {
    lattice::engine::workflow::NodeConfig::Shell {
        command: command.into(),
        cwd: None,
        env: std::collections::HashMap::new(),
        timeout_secs: 60,
        pipe_stdin: with_stdin,
    }
}

fn independent_reference() -> lattice::engine::Workflow {
    use lattice::engine::workflow::PortSpec;
    use lattice::engine::workflow::WorkflowTimestamp;
    let mut workflow = lattice::engine::Workflow::new("manual-reference", WorkflowTimestamp::epoch());
    workflow.construction_profile = true;
    workflow.construction_runtime = Some(construction::selected_runtime().unwrap());
    let outputs = [
        PortSpec::new("stdout", "string"),
        PortSpec::new("stderr", "string"),
        PortSpec::new("exit", "exitcode"),
    ];
    let source = workflow.graph.add_node("fork", &[], &outputs);
    workflow.set_node_config(source, reference_shell("printf 'alpha\\n'", false));
    let source_port = workflow.graph.node(source).unwrap().output_ports[0].id;
    let mut workers = Vec::new();
    for name in ["left", "right", "worker-a", "worker-b"] {
        let worker = workflow.graph.add_node(name, &[PortSpec::new("stdin", "string")], &outputs);
        workflow.set_node_config(worker, reference_shell("cat", true));
        let input = workflow.graph.node(worker).unwrap().input_ports[0].id;
        workflow.graph.add_edge(source_port, input).unwrap();
        workers.push(worker);
    }
    let join = workflow.graph.add_node("done", &[], &outputs);
    workflow.set_node_config(join, reference_shell("printf 'joined\\n'", false));
    workflow.control_dependencies = workers.into_iter().map(|worker| (worker.0, join.0)).collect();
    workflow
}

fn fixture_role<'a>(
    workflow: &lattice::engine::Workflow,
    node: &lattice::engine::workflow::Node,
    roles: &[&'a str],
) -> Option<&'a str> {
    if let Some(source) = &workflow.authoring {
        let construction = source.construction.as_ref()?;
        let origin = construction.origins.iter().find(|origin| {
            origin.element.strip_prefix("node/").and_then(|id| id.parse::<u64>().ok()) == Some(node.id.0)
        })?;
        roles.iter().copied().find(|role| origin.instance_path.iter().any(|part| part.as_str() == *role))
    } else {
        roles.iter().copied().find(|role| node.label == *role)
    }
}

async fn ordered_values(
    workflow: &lattice::engine::Workflow,
) -> (Vec<String>, std::collections::BTreeSet<(&'static str, &'static str)>) {
    use lattice::engine::effect::execution::RunContextProvider as _;
    use lattice::engine::executor::Executor;
    use lattice::engine::executor::NodeStatus;
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
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
    std::fs::write(directory.path().join("grant.json"), serde_json::to_vec(&profile).unwrap()).unwrap();
    std::fs::write(directory.path().join("authority.ncl"), "import \"grant.json\"\n").unwrap();
    let provider = standalone::OperatorRunContextSource::new(directory.path().join("authority.ncl"));
    let context = provider.load(store.next_run_id().unwrap(), Some(store)).unwrap();
    let mut executor = Executor::new();
    let (_, results) = executor.run_authorized(workflow, &std::collections::HashMap::new(), &context).await;
    let mut values = BTreeMap::new();
    for step in results {
        let node = workflow.graph.node(step.node_id).unwrap();
        let role = fixture_role(workflow, node, &["fork", "left", "right", "worker-a", "worker-b", "done"])
            .expect("every executed fixture node has an admitted instance role");
        let NodeStatus::Completed { outputs, .. } = step.status else {
            panic!("effectful node did not complete")
        };
        let stdout = outputs.get(&node.output_ports[0].id).unwrap().as_display_string();
        values.insert((*role).to_owned(), stdout);
    }
    let sequence = ["fork", "left", "right", "worker-a", "worker-b", "done"]
        .iter()
        .map(|role| values[*role].clone())
        .collect();
    // Required order is compared by admitted instance role, never by numeric ID or label.
    let dependencies = workflow
        .control_dependencies
        .iter()
        .map(|&(from, to)| {
            let role = |id| {
                let node = workflow.graph.node(lattice::engine::workflow::NodeId(id)).unwrap();
                fixture_role(workflow, node, &["fork", "left", "right", "worker-a", "worker-b", "done"]).unwrap()
            };
            (role(from), role(to))
        })
        .collect();
    (sequence, dependencies)
}

// r[verify lattice.graph_construction.reference]
// r[verify lattice.graph_construction.reference.values]
#[tokio::test]
#[ignore = "requires exact captured native worker and local shell authority; Nix qualification runs ignored tests"]
async fn generated_graph_agrees_with_independent_runtime_reference() {
    selected_worker();
    let generated = authoring::shell::compile(&program()).unwrap();
    let reference = independent_reference();
    let (actual, required) = ordered_values(&generated).await;
    let (expected, manual_dependencies) = ordered_values(&reference).await;
    assert_eq!(actual, expected);
    assert_eq!(actual[0], "alpha\n");
    assert_eq!(actual[5], "joined\n");
    assert_eq!(required, manual_dependencies);
    assert_eq!(required.len(), 4);
}

// r[verify lattice.graph_construction.control_order]
// r[verify lattice.graph_construction.control_order.barrier]
#[tokio::test]
#[ignore = "requires immutable selected local runtime; Nix qualification runs ignored tests"]
async fn v3_zero_data_barrier_waits_for_first_effect_completion() {
    use lattice::engine::effect::execution::RunContextProvider as _;
    use lattice::engine::executor::Executor;
    use lattice::engine::executor::NodeStatus;
    use lattice::engine::workflow::PortSpec;
    use lattice::engine::workflow::WorkflowTimestamp;
    let directory = tempfile::tempdir().unwrap();
    let path = |name: &str| format!("'{}'", directory.path().join(name).display().to_string().replace('\'', "'\"'\"'"));
    let started = directory.path().join("started");
    let gate = directory.path().join("continue");
    let second = directory.path().join("second");
    let command = format!("touch {}; while [ ! -e {} ]; do sleep 0.02; done", path("started"), path("continue"));
    let outputs = [
        PortSpec::new("stdout", "string"),
        PortSpec::new("stderr", "string"),
        PortSpec::new("exit", "exitcode"),
    ];
    let mut workflow = lattice::engine::Workflow::new("zero-data-barrier", WorkflowTimestamp::epoch());
    workflow.construction_profile = true;
    workflow.construction_runtime = Some(construction::selected_runtime().unwrap());
    let first = workflow.graph.add_node("first", &[], &outputs);
    workflow.set_node_config(first, reference_shell(&command, false));
    let follower = workflow.graph.add_node("second", &[], &outputs);
    workflow.set_node_config(follower, reference_shell(&format!("touch {}", path("second")), false));
    workflow.control_dependencies.push((first.0, follower.0));
    assert!(construction::admit_materialized(&workflow).is_ok());
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    store.save_workflow(&workflow).unwrap();
    let inspection = standalone::inspect_workflow(&workflow).unwrap();
    let profile = standalone::core::Profile {
        schema: inspection.schema,
        workflow_revision: inspection.workflow_revision,
        machine_identity: inspection.machine_identity,
        policy_ref: inspection.policy_ref,
        expires_at_unix_ms: 4_102_444_800_000,
        grants: inspection.grants,
        roots: BTreeMap::new(),
    };
    std::fs::write(directory.path().join("grant.json"), serde_json::to_vec(&profile).unwrap()).unwrap();
    std::fs::write(directory.path().join("authority.ncl"), "import \"grant.json\"\n").unwrap();
    let provider = standalone::OperatorRunContextSource::new(directory.path().join("authority.ncl"));
    let context = provider.load(store.next_run_id().unwrap(), Some(store)).unwrap();
    let execution = async {
        let mut executor = Executor::new();
        executor.run_authorized(&workflow, &std::collections::HashMap::new(), &context).await.1
    };
    let observe = async {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while !started.exists() {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("first effect starts under selected authority");
        assert!(!second.exists(), "second effect must not start before the first completes");
        std::fs::write(&gate, b"continue").unwrap();
    };
    let (results, ()) = tokio::join!(execution, observe);
    assert!(results.iter().all(|step| matches!(step.status, NodeStatus::Completed { .. })));
    assert!(second.exists());
}

// r[verify lattice.graph_construction.rejection]
#[test]
#[ignore = "requires immutable selected local runtime; Nix qualification runs ignored tests"]
fn invalid_materialized_bindings_never_reach_dispatch() {
    let baseline = independent_reference();
    let id = baseline.graph.nodes().next().unwrap().id;
    let mut unknown = baseline.clone();
    unknown.set_node_runner_settings(id, lattice::engine::workflow::NodeRunnerSettings::new("unknown", "default"));
    assert!(construction::admit_materialized(&unknown).is_err());

    let mut wrong_type = baseline.clone();
    let consumer = wrong_type.graph.nodes().find(|node| node.label == "left").unwrap().id;
    wrong_type.graph.node_mut(consumer).unwrap().input_ports[0].type_tag = "exitcode".into();
    assert!(construction::admit_materialized(&wrong_type).is_err());

    let mut invalid_expression = baseline.clone();
    if let lattice::engine::workflow::NodeConfig::Shell { command, .. } =
        invalid_expression.configs.get_mut(&id.0).unwrap()
    {
        command.push_str(" {{node/999/output/stdout}}");
    }
    assert!(construction::admit_materialized(&invalid_expression).is_err());

    let mut collision = baseline.clone();
    let duplicate = collision.graph.node(id).unwrap().output_ports[0].id;
    collision.graph.node_mut(consumer).unwrap().output_ports[0].id = duplicate;
    assert!(construction::admit_materialized(&collision).is_err());

    let mut cycle = baseline.clone();
    cycle.control_dependencies.push((cycle.control_dependencies[0].1, cycle.control_dependencies[0].0));
    assert!(construction::admit_materialized(&cycle).is_err());

    let admitted = construction::admit_materialized(&baseline).unwrap();
    let limits = choregraph_nickel::construction::model::profile::Limits {
        max_nodes: 1,
        ..choregraph_nickel::construction::model::profile::Limits::default()
    };
    assert!(
        choregraph_nickel::construction::admission::lower_projection_json(
            &serde_json::to_vec(&admitted.projection).unwrap(),
            &construction::registry(),
            limits,
        )
        .is_err()
    );
}

// r[verify lattice.graph_construction.rejection]
#[test]
#[ignore = "requires exact captured native worker; Nix qualification runs ignored tests"]
fn wrong_scope_handle_is_rejected_by_selected_elaborator() {
    selected_worker();
    let mut invalid = program();
    let source = String::from_utf8(invalid.files.remove("local-shell-construction.ncl").unwrap()).unwrap();
    let source = source.replace("\"left\" \"stdin\"", "\"missing\" \"stdin\"");
    invalid.files.insert("local-shell-construction.ncl".into(), source.into_bytes());
    assert!(authoring::shell::compile(&invalid).is_err());
}

fn stage_command<'a>(workflow: &'a lattice::engine::Workflow, stage: &str) -> &'a str {
    let node = workflow.graph.nodes().find(|node| fixture_role(workflow, node, &[stage]).is_some()).unwrap();
    match workflow.node_config(node.id).unwrap() {
        lattice::engine::workflow::NodeConfig::Shell { command, .. } => command,
        _ => panic!("selected construction stage must have a shell implementation"),
    }
}

async fn stage_stdout(workflow: &lattice::engine::Workflow) -> BTreeMap<String, String> {
    use lattice::engine::effect::execution::RunContextProvider as _;
    use lattice::engine::executor::Executor;
    use lattice::engine::executor::NodeStatus;
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
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
    std::fs::write(directory.path().join("grant.json"), serde_json::to_vec(&profile).unwrap()).unwrap();
    std::fs::write(directory.path().join("authority.ncl"), "import \"grant.json\"\n").unwrap();
    let provider = standalone::OperatorRunContextSource::new(directory.path().join("authority.ncl"));
    let context = provider.load(store.next_run_id().unwrap(), Some(store)).unwrap();
    let mut executor = Executor::new();
    let (_, results) = executor.run_authorized(workflow, &std::collections::HashMap::new(), &context).await;
    let mut outputs = BTreeMap::new();
    for step in results {
        let node = workflow.graph.node(step.node_id).unwrap();
        let stage = fixture_role(workflow, node, &["first", "second", "a", "b", "c"])
            .expect("every executed stage has an admitted instance role");
        let NodeStatus::Completed { outputs: values, .. } = step.status else {
            panic!("selected shell effect failed")
        };
        outputs.insert(stage.to_owned(), values.get(&node.output_ports[0].id).unwrap().as_display_string());
    }
    outputs
}

// r[verify lattice.graph_construction.editing]
// r[verify lattice.graph_construction.editing.scope]
// r[verify lattice.graph_construction.snapshot]
#[tokio::test]
#[ignore = "requires exact captured native worker and selected local shell authority; Nix qualification runs ignored tests"]
async fn source_template_and_instance_parameter_edits_readmit_independently() {
    selected_worker();
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let input = CapturedProgram {
        entry: "parameterized-shell-construction.ncl".into(),
        files: BTreeMap::from([(
            "parameterized-shell-construction.ncl".into(),
            include_bytes!("../examples/construction/parameterized-shell-construction.ncl").to_vec(),
        )]),
        kind: ProgramKind::ParameterizedConstruction,
        parameters: BTreeMap::new(),
    };
    let mut draft = SourceDraft {
        id: "initial".into(),
        workflow_name: "param-shell".into(),
        expected_revision: None,
        program: input.clone(),
        diagnostic: None,
        preview: None,
    };
    store.save_source_draft(&draft).unwrap();
    store.preview_source_draft(&mut draft).unwrap();
    let original = store.admit_source_draft(&draft).unwrap();
    assert_eq!(original.graph.node_count(), 4);
    let original_results = stage_stdout(&original).await;
    assert_eq!(original_results["first"], "first\n");
    assert_eq!(original_results["second"], "second\n");
    assert_eq!(original_results["a"], "worker\n");
    assert_eq!(original_results["b"], "worker\n");

    let mut template = SourceDraft {
        id: "template-edit".into(),
        workflow_name: "param-shell".into(),
        expected_revision: original.catalog_revision.clone(),
        program: input.clone(),
        diagnostic: None,
        preview: None,
    };
    let bytes = template.program.files.get_mut("parameterized-shell-construction.ncl").unwrap();
    let source = String::from_utf8(bytes.clone()).unwrap();
    let body = "g.operation \"body\" \"shell-source\" {command = g.parameter \"command\"} effect";
    assert_eq!(source.matches(body).count(), 1);
    *bytes = source
        .replace(
            body,
            "g.operation \"body\" \"shell-source\" {command = g.literal \"printf 'templated\\\\n'\"} effect",
        )
        .into_bytes();
    store.save_source_draft(&template).unwrap();
    store.preview_source_draft(&mut template).unwrap();
    let template_candidate = &template.preview.as_ref().unwrap().candidate;
    for stage in ["first", "second", "a", "b"] {
        assert_eq!(stage_command(template_candidate, stage), "printf 'templated\\n'");
    }
    assert_eq!(store.load_workflow("param-shell").unwrap().unwrap().catalog_revision, original.catalog_revision);
    let templated = store.admit_source_draft(&template).unwrap();
    let template_results = stage_stdout(&templated).await;
    assert_eq!(
        template_results,
        BTreeMap::from([
            ("a".into(), "templated\n".into()),
            ("b".into(), "templated\n".into()),
            ("first".into(), "templated\n".into()),
            ("second".into(), "templated\n".into()),
        ])
    );
    assert_eq!(original_results["first"], "first\n", "old admitted snapshot remains independently executable");
    assert_eq!(stage_command(&original, "second"), "printf 'second\\n'");

    let mut instance = SourceDraft {
        id: "instance-edit".into(),
        workflow_name: "param-shell".into(),
        expected_revision: templated.catalog_revision.clone(),
        program: input,
        diagnostic: None,
        preview: None,
    };
    instance.program.parameters.insert("first".into(), serde_json::json!("printf 'changed\\n'"));
    store.save_source_draft(&instance).unwrap();
    store.preview_source_draft(&mut instance).unwrap();
    let instance_candidate = &instance.preview.as_ref().unwrap().candidate;
    assert_eq!(instance_candidate.graph.node_count(), original.graph.node_count());
    assert_eq!(stage_command(instance_candidate, "first"), "printf 'changed\\n'");
    for stage in ["second", "a", "b"] {
        assert_eq!(stage_command(instance_candidate, stage), stage_command(&original, stage));
    }
    let admitted = store.admit_source_draft(&instance).unwrap();
    let reopened = store.load_workflow("param-shell").unwrap().unwrap();
    assert_eq!(reopened.catalog_revision, admitted.catalog_revision);
    assert_eq!(
        reopened.authoring.as_ref().unwrap().source_revision,
        admitted.authoring.as_ref().unwrap().source_revision
    );
    let instance_results = stage_stdout(&reopened).await;
    assert_eq!(instance_results["first"], "changed\n");
    for stage in ["second", "a", "b"] {
        assert_eq!(instance_results[stage], original_results[stage]);
    }
    assert_eq!(stage_stdout(&original).await, original_results);
    assert_eq!(stage_stdout(&templated).await, template_results);

    let mut topology = SourceDraft {
        id: "topology-edit".into(),
        workflow_name: "param-shell".into(),
        expected_revision: admitted.catalog_revision.clone(),
        program: instance.program.clone(),
        diagnostic: None,
        preview: None,
    };
    topology.program.parameters.insert("extra".into(), serde_json::json!(true));
    store.save_source_draft(&topology).unwrap();
    store.preview_source_draft(&mut topology).unwrap();
    assert_eq!(topology.preview.as_ref().unwrap().candidate.graph.node_count(), 5);
    assert_eq!(store.load_workflow("param-shell").unwrap().unwrap().graph.node_count(), 4);
    let expanded = store.admit_source_draft(&topology).unwrap();
    assert_eq!(stage_stdout(&expanded).await["c"], "worker\n");
    let source = authoring::shell::source_export(&expanded).unwrap();
    let archived: authoring::shell::AuthorExport = serde_json::from_slice(&source).unwrap();
    assert_eq!(
        archived.association.program.files["parameterized-shell-construction.ncl"].as_slice(),
        include_bytes!("../examples/construction/parameterized-shell-construction.ncl").as_slice()
    );
    assert!(authoring::shell::expanded_export(&expanded).contains("construction_profile = true"));
    let mut failed = SourceDraft {
        id: "failed".into(),
        workflow_name: "param-shell".into(),
        expected_revision: expanded.catalog_revision.clone(),
        program: topology.program.clone(),
        diagnostic: None,
        preview: None,
    };
    failed.program.files.get_mut("parameterized-shell-construction.ncl").unwrap().clear();
    store.save_source_draft(&failed).unwrap();
    assert!(store.preview_source_draft(&mut failed).is_err());
    assert!(store.load_source_draft("failed").unwrap().unwrap().diagnostic.is_some());
    assert_eq!(store.load_workflow("param-shell").unwrap().unwrap().catalog_revision, expanded.catalog_revision);
    assert!(store.admit_source_draft(&instance).is_err(), "stale draft cannot replace newer revision");
}

// r[verify lattice.graph_construction.drift.registry]
#[test]
#[ignore = "requires exact captured native worker; Nix qualification runs ignored tests"]
fn self_consistent_foreign_registry_implementation_cannot_admit_on_current_host() {
    use choregraph_nickel::construction as native;
    selected_worker();
    let workflow = authoring::shell::compile(&program()).unwrap();
    let observation = workflow.authoring.as_ref().unwrap().observation.as_ref().unwrap();
    let modules = observation
        .modules
        .iter()
        .map(|(logical_id, bytes)| native::association::CapturedModule {
            logical_id,
            bytes: bytes.as_slice(),
        })
        .collect::<Vec<_>>();
    let evaluator_binding = serde_json::to_vec(&observation.result.binding).unwrap();
    let materials = native::association::CompilationMaterials {
        modules: &modules,
        entry: &observation.entry,
        evaluator_identity: &observation.worker_artifact,
        evaluator_profile: nickel_eval::captured::PROFILE,
        evaluator_binding: &evaluator_binding,
        evaluated_output_blake3: &observation.result.binding.output_blake3,
        library: native::model::profile::LIBRARY_BYTES,
        contracts: native::model::profile::CONTRACTS_BYTES,
        lowerer_identity: env!("LATTICE_WORKFLOW_LOWERER_BLAKE3"),
        toolchain_identity: env!("LATTICE_DEPENDENCY_LOCK_BLAKE3"),
    };
    let host = construction::registry();
    let mut foreign = host.clone();
    foreign.implementation.push_str("-another-implementation");
    assert_eq!(foreign.target, host.target);
    assert_eq!(foreign.profile, host.profile);
    assert_eq!(foreign.operations, host.operations);
    assert_ne!(foreign.implementation, host.implementation);
    let candidate = observation.result.json.as_bytes();
    let limits = native::model::profile::Limits::default();
    let foreign_output = native::compile_json(candidate, &foreign, &materials, limits).unwrap();
    native::association::validate_compilation_binding(&foreign_output, candidate, &foreign, &materials, limits)
        .unwrap();
    let retained = workflow.authoring.as_ref().unwrap().construction.as_ref().unwrap();
    assert_ne!(
        foreign_output.binding,
        serde_json::from_slice::<native::association::CompilationBinding>(&retained.binding_json).unwrap()
    );
    assert_eq!(foreign_output.projection, serde_json::from_slice(&retained.projection_json).unwrap());
    let mut foreign_workflow = workflow.clone();
    let association = foreign_workflow.authoring.as_mut().unwrap().construction.as_mut().unwrap();
    association.binding_json = serde_json::to_vec(&foreign_output.binding).unwrap();
    association.projection_json = serde_json::to_vec(&foreign_output.projection).unwrap();
    association.origins = foreign_output.origins;
    assert!(construction::admit_runtime(&foreign_workflow).is_err());
    assert!(
        standalone::inspect_workflow(&foreign_workflow).is_err(),
        "effect authority inspection rejects a foreign compiled association before dispatch"
    );
}

// r[verify lattice.graph_construction.drift]
#[test]
#[ignore = "requires exact captured native worker and immutable local runtime; Nix qualification runs ignored tests"]
fn construction_receipts_cannot_change_under_ordinary_cas() {
    selected_worker();
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("state.redb")).unwrap();
    let mut draft = SourceDraft {
        id: "construction-receipt".into(),
        workflow_name: "local-shell-demo".into(),
        expected_revision: None,
        program: program(),
        diagnostic: None,
        preview: None,
    };
    store.preview_source_draft(&mut draft).unwrap();
    let admitted = store.admit_source_draft(&draft).unwrap();
    let mut altered = admitted.clone();
    altered.authoring.as_mut().unwrap().construction.as_mut().unwrap().projection_json.push(b' ');
    assert!(store.save_workflow_cas(&altered, admitted.catalog_revision.as_ref()).is_err());
    let mut altered = admitted.clone();
    altered.authoring.as_mut().unwrap().construction.as_mut().unwrap().binding_json.push(b' ');
    assert!(store.save_workflow_cas(&altered, admitted.catalog_revision.as_ref()).is_err());
    assert_eq!(store.load_workflow("local-shell-demo").unwrap().unwrap().authoring, admitted.authoring);
}
