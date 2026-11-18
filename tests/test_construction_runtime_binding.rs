//! Runtime/executable boundary controls for the immutable construction profile.
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use lattice::authoring::AssociationState;
use lattice::authoring::CapturedProgram;
use lattice::authoring::CompilerIdentity;
use lattice::authoring::ProgramKind;
use lattice::authoring::SourceAssociation;
use lattice::authoring::construction;
use lattice::authoring::{self};
use lattice::engine::Workflow;
use lattice::engine::effect::standalone;
use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::PortSpec;
use lattice::engine::workflow::WorkflowTimestamp;

fn shell_workflow(name: &str) -> Workflow {
    let mut workflow = Workflow::new(name, WorkflowTimestamp::epoch());
    let outputs = [
        PortSpec::new("stdout", "string"),
        PortSpec::new("stderr", "string"),
        PortSpec::new("exit", "exitcode"),
    ];
    let node = workflow.graph.add_node("first", &[], &outputs);
    workflow.set_node_config(node, NodeConfig::Shell {
        command: "printf 'legacy-ok'".into(),
        cwd: None,
        env: HashMap::new(),
        timeout_secs: 60,
        pipe_stdin: false,
    });
    workflow
}

// r[verify lattice.graph_construction.compatibility]
#[tokio::test]
async fn materialized_flat_fork_retains_v2_execution() {
    let mut workflow = shell_workflow("legacy-flat-fork");
    workflow.authoring = Some(retained_fork(ProgramKind::Workflow, false));
    assert!(construction::admit_runtime(&workflow).unwrap().is_none());
    let inspection = standalone::inspect_workflow(&workflow).unwrap();
    assert!(inspection.grants.iter().any(|grant| grant.operation == "lattice/shell"));
    let (_, steps) = Executor::new().run(&workflow, &HashMap::new()).await;
    assert_eq!(steps.len(), 1);
    assert!(matches!(steps[0].status, NodeStatus::Completed { .. }));
}

fn retained_fork(kind: ProgramKind, receipt: bool) -> SourceAssociation {
    let program = CapturedProgram {
        entry: "main.ncl".into(),
        files: BTreeMap::from([("main.ncl".into(), b"{}".to_vec())]),
        kind,
        parameters: BTreeMap::new(),
    };
    SourceAssociation {
        source_revision: program.identity().unwrap(),
        program: Arc::new(program),
        compiler: CompilerIdentity::current().unwrap(),
        evaluated_output: authoring::ContentIdentity::from_bytes(b"original"),
        expanded_artifact: authoring::ContentIdentity::from_bytes(b"before-fork"),
        state: AssociationState::MaterializedFork,
        observation: None,
        // A retained receipt's presence is lineage, not permission to replay its contents.
        construction: receipt.then(|| authoring::ConstructionAssociation {
            projection_json: b"{}".to_vec(),
            binding_json: b"{}".to_vec(),
            origins: Vec::new(),
            port_map: BTreeMap::new(),
        }),
    }
}

#[test]
fn retained_construction_forks_cannot_inspect_as_v2_shell_authority() {
    for (kind, receipt) in [
        (ProgramKind::Construction, false),
        (ProgramKind::ParameterizedConstruction, false),
        (ProgramKind::Workflow, true),
        (ProgramKind::Parameterized, true),
    ] {
        let mut workflow = shell_workflow("downgraded-construction-fork");
        workflow.authoring = Some(retained_fork(kind, receipt));
        assert!(!workflow.construction_profile);
        assert!(workflow.control_dependencies.is_empty());
        assert_eq!(
            construction::admit_runtime(&workflow).unwrap_err(),
            "retained construction lineage requires construction profile"
        );
        assert!(standalone::inspect_workflow(&workflow).is_err(), "must not issue v2 shell grants");
    }
}

#[derive(Clone, Default)]
struct BoundarySpy {
    adapters: Arc<AtomicUsize>,
    credentials: Arc<AtomicUsize>,
}

impl lattice::credentials::CredentialProvider for BoundarySpy {
    fn resolve(
        &self,
        reference: &lattice::credentials::SecretRef,
    ) -> Result<lattice::credentials::SecretValue, lattice::credentials::CredentialError> {
        self.credentials.fetch_add(1, Ordering::SeqCst);
        Err(lattice::credentials::CredentialError::ProviderFailure {
            identity: reference.identity(),
            message: "credential spy reached".into(),
        })
    }
}

impl lattice::http::HttpClient for BoundarySpy {
    fn execute(
        &self,
        _request: &lattice::http::HttpRequestPlan,
    ) -> Result<lattice::http::HttpResponse, lattice::http::HttpError> {
        self.adapters.fetch_add(1, Ordering::SeqCst);
        Ok(lattice::http::HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body: "http-spy-response".into(),
        })
    }
}

impl lattice::file_node::FileSystem for BoundarySpy {
    fn execute(
        &self,
        plan: &lattice::file_node::FileOperationPlan,
    ) -> Result<lattice::file_node::FileOperationResult, lattice::file_node::FileError> {
        self.adapters.fetch_add(1, Ordering::SeqCst);
        lattice::file_node::FileSystem::execute(&lattice::file_node::LocalFs, plan)
    }
}

impl lattice::image_node::ImageIo for BoundarySpy {
    fn load(
        &self,
        _plan: &lattice::image_node::ImageInputPlan,
    ) -> Result<lattice::engine::value::ImageValue, lattice::image_node::ImageError> {
        self.adapters.fetch_add(1, Ordering::SeqCst);
        Err(lattice::image_node::ImageError::Io("image spy reached".into()))
    }
}

impl lattice::agent_node::Adapter for BoundarySpy {
    fn execution_identity(
        &self,
        _request: &lattice::agent_node::Request,
    ) -> Result<String, lattice::agent_node::Error> {
        Ok("construction-boundary-spy".into())
    }

    fn execute(
        &self,
        _request: &lattice::agent_node::Request,
        _context: &lattice::agent_node::ExecutionContext,
    ) -> Result<lattice::agent_node::Response, lattice::agent_node::Error> {
        self.adapters.fetch_add(1, Ordering::SeqCst);
        Err(lattice::agent_node::Error::PreflightRejected("agent spy reached".into()))
    }
}

fn boundary_executor(spy: &BoundarySpy) -> Executor {
    Executor::new_with_runtime_clients(
        Box::new(spy.clone()),
        Box::new(spy.clone()),
        Box::new(spy.clone()),
        Box::new(spy.clone()),
        Box::new(spy.clone()),
    )
}

fn effect_workflow(directory: &std::path::Path) -> Workflow {
    let mut workflow = Workflow::new("construction-compatibility-boundary", WorkflowTimestamp::epoch());
    workflow.add_shell_node("shell", format!("printf shell-ran > {:?}", directory.join("shell-marker")));
    workflow.add_http_node("http", "GET", "https://construction-boundary.invalid/");
    let file = workflow.add_file_node(
        "file",
        lattice::file_node::FileOperationKind::Write,
        directory.join("file-marker").to_string_lossy(),
    );
    workflow.set_node_config(file, NodeConfig::File {
        operation: lattice::file_node::FileOperationKind::Write,
        path: directory.join("file-marker").to_string_lossy().into_owned(),
        content: Some("file-ran".into()),
        overwrite: false,
        allowed_roots: vec![directory.to_string_lossy().into_owned()],
    });
    let image = workflow.add_image_input_node("image", directory.join("image.png").to_string_lossy());
    workflow.set_node_config(image, NodeConfig::ImageInput {
        path: directory.join("image.png").to_string_lossy().into_owned(),
        allowed_roots: vec![directory.to_string_lossy().into_owned()],
        max_bytes: 1024,
    });
    workflow.add_agent_node(
        "agent",
        lattice::agent_node::Target::Pi,
        "Return the fixture result.",
        lattice::agent_node::OutputMode::Text,
    );
    workflow
}

// r[verify lattice.graph_construction.authority]
#[tokio::test]
async fn construction_compatibility_denies_every_adapter_and_credentials_through_both_public_paths() {
    for lineage in 0..4 {
        for with_credentials in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let mut workflow = effect_workflow(directory.path());
            match lineage {
                0 => workflow.construction_profile = true,
                1 => workflow.authoring = Some(retained_fork(ProgramKind::Construction, false)),
                2 => workflow.authoring = Some(retained_fork(ProgramKind::ParameterizedConstruction, false)),
                _ => workflow.authoring = Some(retained_fork(ProgramKind::Workflow, true)),
            }
            let nodes: Vec<_> = workflow.graph.nodes().map(|node| node.id).collect();
            if with_credentials {
                for node in &nodes {
                    workflow.set_node_credentials(*node, lattice::credentials::NodeCredentialRefs {
                        env: HashMap::from([(
                            "TOKEN".into(),
                            lattice::credentials::SecretRef::new("env", "CONSTRUCTION_BOUNDARY_TOKEN", None),
                        )]),
                    });
                }
            }
            let spy = BoundarySpy::default();
            let mut executor = boundary_executor(&spy);
            for node in &nodes {
                let result = executor.execute_node(&workflow, *node, &HashMap::new()).await;
                assert!(matches!(
                    result.status,
                    NodeStatus::Failed { ref error } if error == "construction effects require admitted authority"
                ));
            }
            let (_, results) = executor.run(&workflow, &HashMap::new()).await;
            assert_eq!(results.len(), nodes.len());
            assert!(results.iter().all(|result| matches!(
                &result.status,
                NodeStatus::Failed { error } if error == "construction effects require admitted authority"
            )));
            assert_eq!(spy.adapters.load(Ordering::SeqCst), 0);
            assert_eq!(spy.credentials.load(Ordering::SeqCst), 0);
            assert!(!directory.path().join("shell-marker").exists());
            assert!(!directory.path().join("file-marker").exists());
        }
    }
}

#[tokio::test]
async fn legacy_compatibility_fixture_reaches_selected_adapters_and_real_local_markers() {
    let directory = tempfile::tempdir().unwrap();
    let workflow = effect_workflow(directory.path());
    let spy = BoundarySpy::default();
    let mut executor = boundary_executor(&spy);
    let (_, results) = executor.run(&workflow, &HashMap::new()).await;
    assert_eq!(results.len(), 5);
    assert_eq!(spy.adapters.load(Ordering::SeqCst), 4);
    assert_eq!(spy.credentials.load(Ordering::SeqCst), 0);
    assert_eq!(std::fs::read_to_string(directory.path().join("shell-marker")).unwrap(), "shell-ran");
    assert_eq!(std::fs::read_to_string(directory.path().join("file-marker")).unwrap(), "file-ran");
    for result in results {
        match workflow.graph.node(result.node_id).unwrap().label.as_str() {
            "shell" | "http" | "file" => assert!(matches!(result.status, NodeStatus::Completed { .. })),
            "image" => {
                assert!(matches!(result.status, NodeStatus::Failed { error } if error.contains("image spy reached")))
            }
            "agent" => {
                assert!(matches!(result.status, NodeStatus::Failed { error } if error.contains("agent spy reached")))
            }
            label => panic!("unexpected fixture node: {label}"),
        }
    }
}

#[tokio::test]
async fn legacy_compatibility_credential_fixture_reaches_provider_before_adapters() {
    let directory = tempfile::tempdir().unwrap();
    let mut workflow = effect_workflow(directory.path());
    let nodes: Vec<_> = workflow
        .graph
        .nodes()
        .filter(|node| matches!(node.label.as_str(), "shell" | "http" | "agent"))
        .map(|node| node.id)
        .collect();
    for node in &nodes {
        workflow.set_node_credentials(*node, lattice::credentials::NodeCredentialRefs {
            env: HashMap::from([(
                "TOKEN".into(),
                lattice::credentials::SecretRef::new("env", "CONSTRUCTION_BOUNDARY_TOKEN", None),
            )]),
        });
    }
    let spy = BoundarySpy::default();
    let executor = boundary_executor(&spy);
    for node in nodes {
        let result = executor.execute_node(&workflow, node, &HashMap::new()).await;
        assert!(matches!(result.status, NodeStatus::Failed { .. }));
    }
    assert_eq!(spy.credentials.load(Ordering::SeqCst), 3);
    assert_eq!(spy.adapters.load(Ordering::SeqCst), 0);
    assert!(!directory.path().join("shell-marker").exists());
    assert!(!directory.path().join("file-marker").exists());
}

#[tokio::test]
#[ignore = "requires immutable Nix-store runtime, shell and captured evaluator"]
async fn captured_no_control_fork_retains_v3_authority_and_rejects_saved_profile_downgrade() {
    use lattice::engine::effect::execution::RunContextProvider as _;

    let directory = tempfile::tempdir().unwrap();
    let store = lattice::store::Store::new(directory.path().join("state.redb")).unwrap();
    let source = br#"
let g = import "./choregraph/graph-construction-v1.ncl" in
let effect = [{
  key = "invoke", from_state = "ready", action = "start",
  resource = "lattice://process/construction-shell-v1", ability = "process/execute",
  request = g.slot_binding "dispatch-request" "invocation" "dispatch-request",
  payload = g.slot_binding "resolved-payload" "invocation" "resolved-payload",
}] in
g.finish "captured-no-controls" "lattice-local-shell-v1" "lattice-local-shell-registry-v1"
  {} [] (g.wire "root" [
    g.operation "first" "shell-source" {command = g.literal "printf captured-ok"} effect,
  ] [] [] [])
"#;
    let mut draft = authoring::SourceDraft {
        id: "no-control-construction".into(),
        workflow_name: "captured-no-controls".into(),
        expected_revision: None,
        program: CapturedProgram {
            entry: "main.ncl".into(),
            files: BTreeMap::from([("main.ncl".into(), source.to_vec())]),
            kind: ProgramKind::Construction,
            parameters: BTreeMap::new(),
        },
        diagnostic: None,
        preview: None,
    };
    store.preview_source_draft(&mut draft).unwrap();
    let mut workflow = store.admit_source_draft(&draft).unwrap();
    assert!(workflow.control_dependencies.is_empty());
    assert!(construction::admit_runtime(&workflow).unwrap().is_some());
    let first = workflow.graph.nodes().next().unwrap().id;
    let NodeConfig::Shell { command, .. } = workflow.configs.get_mut(&first.0).unwrap() else {
        panic!("captured fixture must compile to shell")
    };
    *command = "printf fork-ok".into();
    let fork = store.save_workflow_cas(&workflow, workflow.catalog_revision.as_ref()).unwrap();
    assert_eq!(fork.authoring.as_ref().unwrap().state, AssociationState::MaterializedFork);
    assert_eq!(fork.authoring.as_ref().unwrap().construction, workflow.authoring.as_ref().unwrap().construction);
    assert!(construction::admit_runtime(&fork).unwrap().is_some());
    let inspection = standalone::inspect_workflow(&fork).unwrap();
    assert_eq!(inspection.grants.len(), 1);
    assert_eq!(inspection.grants[0].resource, construction::RESOURCE);
    assert_ne!(inspection.grants[0].operation, "lattice/shell");
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
    let context = provider.load(store.next_run_id().unwrap(), Some(store.clone())).unwrap();
    let mut executor = Executor::new();
    let (_, results) = executor.run_authorized(&fork, &HashMap::new(), &context).await;
    assert_eq!(results.len(), 1);
    let NodeStatus::Completed { outputs, .. } = &results[0].status else {
        panic!("exactly authorized construction fork must complete: {:?}", results[0].status)
    };
    let stdout = fork.graph.node(first).unwrap().output_ports[0].id;
    assert_eq!(outputs[&stdout].as_display_string(), "fork-ok");

    let mut downgraded = fork;
    downgraded.construction_profile = false;
    downgraded.construction_runtime = None;
    store.save_workflow_cas(&downgraded, downgraded.catalog_revision.as_ref()).unwrap();
    let reopened = store.load_workflow("captured-no-controls").unwrap().unwrap();
    assert_eq!(reopened.authoring.as_ref().unwrap().state, AssociationState::MaterializedFork);
    assert!(!reopened.construction_profile);
    assert_eq!(
        construction::admit_runtime(&reopened).unwrap_err(),
        "retained construction lineage requires construction profile"
    );
    assert!(standalone::inspect_workflow(&reopened).is_err(), "saved fork must never produce v2 shell grants");
    let (_, denied) = executor.run_authorized(&reopened, &HashMap::new(), &context).await;
    assert_eq!(denied.len(), 1);
    assert!(matches!(
        &denied[0].status,
        NodeStatus::Failed { error } if error.contains("retained construction lineage requires construction profile")
    ));
}

// r[verify lattice.graph_construction.control_order]
#[test]
#[ignore = "requires immutable Nix-store runtime and shell selection"]
fn flat_v3_controls_admit_with_runtime_binding() {
    let mut workflow = shell_workflow("flat-controls");
    let outputs = [
        PortSpec::new("stdout", "string"),
        PortSpec::new("stderr", "string"),
        PortSpec::new("exit", "exitcode"),
    ];
    let second = workflow.graph.add_node("second", &[], &outputs);
    workflow.set_node_config(second, NodeConfig::Shell {
        command: "printf 'second'".into(),
        cwd: None,
        env: HashMap::new(),
        timeout_secs: 60,
        pipe_stdin: false,
    });
    let first = workflow.graph.nodes().find(|node| node.label == "first").unwrap().id;
    workflow.control_dependencies.push((first.0, second.0));
    workflow.construction_profile = true;
    let mut flat = serde_json::to_value(lattice::workflow_format::workflow_to_portable(&workflow)).unwrap();
    assert!(flat.get("construction_runtime").is_none());
    // Flat source supplies controls/profile but not an executable claim; import binds the selected
    // host.
    let imported = lattice::workflow_format::from_nickel_value(flat.take(), &BTreeMap::new()).unwrap();
    assert!(imported.construction_runtime.is_some());
    let admitted = construction::admit_runtime(&imported).unwrap().unwrap();
    assert_eq!(admitted.projection.control_edges.len(), 1);
    assert_eq!(standalone::inspect_workflow(&imported).unwrap().grants.len(), 2);
}

#[test]
#[ignore = "requires immutable Nix-store runtime, shell and captured evaluator"]
fn captured_flat_v3_controls_replay_before_authority() {
    let mut workflow = shell_workflow("captured-flat-controls");
    let outputs = [
        PortSpec::new("stdout", "string"),
        PortSpec::new("stderr", "string"),
        PortSpec::new("exit", "exitcode"),
    ];
    let second = workflow.graph.add_node("second", &[], &outputs);
    workflow.set_node_config(second, NodeConfig::Shell {
        command: "printf 'second'".into(),
        cwd: None,
        env: HashMap::new(),
        timeout_secs: 60,
        pipe_stdin: false,
    });
    let first = workflow.graph.nodes().find(|node| node.label == "first").unwrap().id;
    workflow.control_dependencies.push((first.0, second.0));
    workflow.construction_profile = true;
    workflow.construction_runtime = Some(construction::selected_runtime().unwrap());
    let program = CapturedProgram {
        entry: "flat.ncl".into(),
        files: BTreeMap::from([(
            "flat.ncl".into(),
            lattice::workflow_format::workflow_to_nickel(&workflow).into_bytes(),
        )]),
        kind: ProgramKind::Workflow,
        parameters: BTreeMap::new(),
    };
    let captured = authoring::shell::compile(&program).unwrap();
    assert_eq!(captured.authoring.as_ref().unwrap().state, AssociationState::CapturedModules);
    assert_eq!(construction::admit_runtime(&captured).unwrap().unwrap().projection.control_edges.len(), 1);
    assert_eq!(standalone::inspect_workflow(&captured).unwrap().grants.len(), 2);

    let mut source_changed = captured.clone();
    let source = source_changed.authoring.as_mut().unwrap();
    std::sync::Arc::make_mut(&mut source.program)
        .files
        .get_mut("flat.ncl")
        .unwrap()
        .extend_from_slice(b"\n# changed retained source\n");
    // Updating the source digest cannot make the previous observation cover new bytes.
    source.source_revision = source.program.identity().unwrap();
    assert!(construction::admit_runtime(&source_changed).is_err());
    assert!(standalone::inspect_workflow(&source_changed).is_err());

    for runtime_changed in [false, true] {
        let mut stale = workflow.clone();
        let binding = stale.construction_runtime.as_mut().unwrap();
        if runtime_changed {
            binding.runtime_artifact.push('0');
        } else {
            binding.shell_artifact.push('0');
        }
        let mut stale_program = program.clone();
        stale_program
            .files
            .insert("flat.ncl".into(), lattice::workflow_format::workflow_to_nickel(&stale).into_bytes());
        assert!(authoring::shell::compile(&stale_program).is_err());
    }
}

#[test]
#[ignore = "requires immutable Nix-store runtime and shell selection"]
fn same_named_changed_backend_or_runtime_refuses_old_authority() {
    let mut workflow = shell_workflow("backend-facts");
    workflow.construction_profile = true;
    workflow.construction_runtime = Some(construction::selected_runtime().unwrap());
    let approved = standalone::inspect_workflow(&workflow).unwrap();
    let mut shell_changed = workflow.clone();
    shell_changed.construction_runtime.as_mut().unwrap().shell_artifact.push('0');
    assert_ne!(
        lattice::orchestration::workflow_revision(&workflow).unwrap(),
        lattice::orchestration::workflow_revision(&shell_changed).unwrap()
    );
    assert!(standalone::inspect_workflow(&shell_changed).is_err());
    let mut runtime_changed = workflow.clone();
    runtime_changed.construction_runtime.as_mut().unwrap().runtime_artifact.push('0');
    assert_ne!(
        approved.workflow_revision,
        format!("blake3:{}", lattice::orchestration::workflow_revision(&runtime_changed).unwrap().content_hash)
    );
    assert!(construction::admit_runtime(&runtime_changed).is_err());
    assert!(standalone::inspect_workflow(&runtime_changed).is_err());
    let temp = tempfile::tempdir().unwrap();
    let old_shell = std::path::Path::new(&workflow.construction_runtime.as_ref().unwrap().shell_path);
    let mutable_same_name = temp.path().join(old_shell.file_name().unwrap());
    std::fs::copy(old_shell, &mutable_same_name).unwrap();
    let retained = temp.path().join("retained.json");
    std::fs::write(&retained, serde_json::to_vec(&workflow).unwrap()).unwrap();
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "changed_backend_child", "--nocapture"])
        .env("LATTICE_TEST_RETAINED_WORKFLOW", &retained)
        .env("LATTICE_CONSTRUCTION_SHELL", &mutable_same_name)
        .status()
        .unwrap();
    assert!(status.success(), "same-named mutable backend must invalidate retained authority");
}

#[test]
#[ignore = "requires immutable Nix-store test executable and selected Bash backend"]
fn immutable_non_bash_backend_is_rejected_at_selection() {
    let executable = std::env::current_exe().unwrap();
    assert!(
        executable.starts_with("/nix/store/"),
        "run this regression from an immutable Nix-store test executable"
    );
    let status = std::process::Command::new(&executable)
        .args(["--exact", "unreviewed_immutable_backend_child", "--nocapture"])
        .env("LATTICE_TEST_UNREVIEWED_BACKEND_CHILD", "1")
        .env("LATTICE_CONSTRUCTION_SHELL", &executable)
        .status()
        .unwrap();
    assert!(status.success(), "immutable non-Bash executable must be refused before authority");
}

#[test]
fn unreviewed_immutable_backend_child() {
    if std::env::var_os("LATTICE_TEST_UNREVIEWED_BACKEND_CHILD").is_none() {
        return;
    }
    assert!(
        construction::selected_runtime().is_err(),
        "an immutable but unreviewed executable cannot become the construction shell"
    );
}

#[test]
fn changed_backend_child() {
    let Some(path) = std::env::var_os("LATTICE_TEST_RETAINED_WORKFLOW") else {
        return;
    };
    let workflow: Workflow = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert!(construction::admit_runtime(&workflow).is_err());
    assert!(standalone::inspect_workflow(&workflow).is_err());
}

#[test]
fn worker_selection_child() {
    if std::env::var_os("LATTICE_TEST_MUTABLE_WORKER_CHILD").is_none() {
        return;
    }
    assert!(lattice::nickel::captured_worker().is_err(), "mutable selected worker must fail closed");
}

#[test]
fn mutable_and_replaced_worker_paths_never_admit() {
    let temp = tempfile::tempdir().unwrap();
    let worker = temp.path().join("closed-worker");
    let replacement = temp.path().join("replacement");
    std::fs::copy(std::env::current_exe().unwrap(), &worker).unwrap();
    std::fs::copy(std::env::current_exe().unwrap(), &replacement).unwrap();
    for phase in 0..2 {
        if phase == 1 {
            std::fs::rename(&replacement, &worker).unwrap();
        }
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "worker_selection_child", "--nocapture"])
            .env("LATTICE_TEST_MUTABLE_WORKER_CHILD", "1")
            .env("LATTICE_CLOSED_EVAL_WORKER", &worker)
            .status()
            .unwrap();
        assert!(status.success(), "mutable worker selection must reject before and after replacement");
    }
}
