// r[verify lattice.render_binding.producer]
// r[verify lattice.render_binding.admission]
// r[verify lattice.render_binding.acceptance]
// r[verify lattice.render_binding.recovery]
// r[verify lattice.render_binding.truthfulness]
// Native contract regressions only. Process qualification is scripts/qualify-render.py.
//! Consumer regressions. Native producer execution below supplies adversarial
//! contract bytes; it is NOT qualification of the Lattice-to-iroh process rail.
use std::collections::BTreeMap;
use std::path::Path;

use lattice::render::Authority;
use lattice::render::Budget;
use lattice::render::Outcome;
use lattice::render::Request;
use lattice::render::model;
use lattice::render::shell;
use lattice::render::{self};
use molten_batch_core::*;
use molten_orchestrator_codec as codec;
use molten_render_pilot as render_pilot;

struct Fixture {
    _root: tempfile::TempDir,
    request: Request,
    snapshot: ProgressSnapshot,
    objects: BTreeMap<Id, Vec<u8>>,
}

fn text(path: &Path) -> String {
    path.to_str().unwrap().to_string()
}
fn fixture() -> Fixture {
    let root = tempfile::tempdir().unwrap();
    let component = b"native contract fixture only; not a Wasm component";
    let component_file = root.path().join("renderer.wasm");
    std::fs::write(&component_file, component).unwrap();
    let renderer = render_pilot::content_id(ContentKind::Component, component).unwrap();
    let mut manifest = render_pilot::frozen_manifest(renderer).unwrap();
    manifest.width = 8;
    manifest.height = 6;
    manifest.columns = 2;
    manifest.rows = 2;
    manifest.samples = 1;
    manifest.bounces = 1;
    manifest.runtime.max_attempts = 2;
    let authority_file = root.path().join("authority.json");
    let request = Request {
        schema: render::SCHEMA.into(),
        execution: "execution-exact".into(),
        attempt: "attempt-4".into(),
        fence: "fence-9".into(),
        budget: Budget {
            identity: "governing-budget-1".into(),
            deadline_epoch_millis: u64::MAX / 2,
            task_attempts: 10,
            fuel: 10_000_000_000,
            task_wall_millis: 600_000,
        },
        manifest,
        route: model::Route {
            executable: text(&component_file),
            executable_identity: format!("b3:{}", "1".repeat(64)),
            client_root: text(root.path()),
            controller: format!("{}@127.0.0.1:40000", Id([3; 32])),
            cluster: Id([1; 32]),
            generation: 7,
            policy_ref: Id([2; 32]),
            fixture_fault: None,
        },
        component_file: text(&component_file),
        authority_file: text(&authority_file),
        journal_root: text(&root.path().join("journal")),
    };
    let authority = Authority {
        execution: request.execution.clone(),
        attempt: request.attempt.clone(),
        fence: request.fence.clone(),
        budget_identity: request.budget.identity.clone(),
        current: true,
    };
    std::fs::write(authority_file, serde_json::to_vec(&authority).unwrap()).unwrap();
    let prepared = request.prepare().unwrap();
    let mut outputs = BTreeMap::new();
    for index in 0..request.manifest.tile_count() {
        outputs.insert(format!("tile-{index:03}"), render_pilot::render_tile(&request.manifest, index).unwrap());
    }
    let tiles: Vec<_> = outputs.values().cloned().collect();
    outputs.insert("assemble".into(), render_pilot::assemble(&request.manifest, &tiles).unwrap());
    let (snapshot, objects) = build_snapshot(&request, &prepared.job, &outputs);
    Fixture {
        _root: root,
        request,
        snapshot,
        objects,
    }
}

fn build_snapshot(
    request: &Request,
    job: &JobSpec,
    outputs: &BTreeMap<String, Vec<u8>>,
) -> (ProgressSnapshot, BTreeMap<Id, Vec<u8>>) {
    let run = request.run_id().unwrap();
    let spec_ref = codec::identity("job-spec.v1", job).unwrap();
    let ids: BTreeMap<_, _> = outputs
        .iter()
        .map(|(task, bytes)| (task.clone(), render_pilot::content_id(ContentKind::Data, bytes).unwrap()))
        .collect();
    let mut tasks = BTreeMap::new();
    let mut objects = BTreeMap::new();
    for task in &job.tasks {
        let node = Id([4; 32]);
        let assignment = codec::identity(
            "assignment-id.v1",
            &(request.route.cluster, request.route.generation, run, &task.id, node, 1u32, spec_ref),
        )
        .unwrap();
        let fence = Fence {
            cluster: request.route.cluster,
            generation: request.route.generation,
            run,
            task: task.id.clone(),
            assignment,
            spec_ref,
            node,
            epoch: 1,
        };
        let mut inputs = task.inputs.clone();
        inputs.extend(task.dependencies.iter().map(|name| ids[name]));
        let execution = ExecutionRequest {
            fence: fence.clone(),
            policy_ref: request.route.policy_ref,
            component: task.component,
            inputs,
            resources: task.resources,
            fuel: task.fuel,
            timeout_ms: task.timeout_ms,
            max_output_bytes: task.max_output_bytes,
        };
        let request_ref = codec::identity("execution-request.v1", &execution).unwrap();
        let output = ids[&task.id];
        let result = codec::identity("execution-result.v1", &ExecutionResult {
            request_ref,
            outcome: ExecutionOutcome::Succeeded { output },
            stopped: true,
        })
        .unwrap();
        tasks.insert(task.id.clone(), ProgressTask::Accepted {
            output,
            result,
            fence: Box::new(fence),
            request: request_ref,
        });
        objects.insert(output, outputs[&task.id].clone());
    }
    (
        ProgressSnapshot {
            schema: PROGRESS_SCHEMA.into(),
            cluster: request.route.cluster,
            generation: request.route.generation,
            run,
            spec_ref,
            phase: RunPhase::Succeeded,
            cancel_requested: false,
            tasks,
            state_ref: Id([5; 32]),
            state_revision: 12,
            watermark: ProgressCursor {
                service_id: "fixture".into(),
                stream_epoch: 1,
                revision: 12,
                ordinal: 0,
            },
        },
        objects,
    )
}

struct BatchFixture {
    snapshot: ProgressSnapshot,
    objects: BTreeMap<Id, Vec<u8>>,
    submitted: bool,
    submits: usize,
    cancels: usize,
    fail_observe: bool,
}
impl BatchFixture {
    fn new(f: &Fixture) -> Self {
        Self {
            snapshot: f.snapshot.clone(),
            objects: f.objects.clone(),
            submitted: false,
            submits: 0,
            cancels: 0,
            fail_observe: false,
        }
    }
}
impl shell::Batch for BatchFixture {
    fn put(&mut self, kind: ContentKind, file: &Path) -> anyhow::Result<Id> {
        Ok(render_pilot::content_id(kind, &std::fs::read(file)?)?)
    }
    fn submit(&mut self, _: &str, _: &Path) -> anyhow::Result<Id> {
        self.submits += 1;
        self.submitted = true;
        Ok(self.snapshot.run)
    }
    fn snapshot(&mut self, run: Id) -> anyhow::Result<ProgressSnapshot> {
        anyhow::ensure!(self.submitted && !self.fail_observe && run == self.snapshot.run, "unavailable observation");
        Ok(self.snapshot.clone())
    }
    fn get(&mut self, id: Id, file: &Path) -> anyhow::Result<()> {
        use std::io::Write as _;
        let mut f = std::fs::OpenOptions::new().write(true).create_new(true).open(file)?;
        f.write_all(&self.objects[&id])?;
        Ok(())
    }
    fn cancel(&mut self, _: Id) -> anyhow::Result<()> {
        self.cancels += 1;
        Ok(())
    }
}

#[test]
fn complete_accepted_artifacts_are_committed_and_duplicates_reused() {
    let f = fixture();
    let accepted = model::accept(&f.request, &f.snapshot, &f.objects).unwrap();
    assert_eq!(accepted.tiles.len(), 4);
    assert!(accepted.measured_usage.is_none());
    assert_eq!(accepted.budget, f.request.budget);
    let mut batch = BatchFixture::new(&f);
    let first = shell::step(&f.request, &mut batch, shell::Action::Advance, shell::Fault::None).unwrap();
    assert_eq!(first.outcome, Outcome::Completed);
    shell::verify_committed(&f.request, &first).unwrap();
    let again = shell::step(&f.request, &mut batch, shell::Action::Advance, shell::Fault::None).unwrap();
    assert_eq!(again.accepted_manifest, first.accepted_manifest);
    assert_eq!(again.recovery, "reused_committed_artifacts");
    assert_eq!(batch.submits, 1);
}

#[test]
fn reordered_partial_acceptance_and_duplicates_preserve_committed_image() {
    let mut expected_image = None;
    for order in [["tile-000", "tile-001", "tile-002", "tile-003"], [
        "tile-003", "tile-002", "tile-001", "tile-000",
    ]] {
        let f = fixture();
        let mut batch = BatchFixture::new(&f);
        batch.snapshot.phase = RunPhase::Pending;
        for task in batch.snapshot.tasks.values_mut() {
            *task = ProgressTask::Pending { ready_at: 0 };
        }
        let initial = shell::step(&f.request, &mut batch, shell::Action::Advance, shell::Fault::None).unwrap();
        assert_eq!(initial.outcome, Outcome::Pending);
        for (index, name) in order.into_iter().enumerate() {
            *batch.snapshot.tasks.get_mut(name).unwrap() = f.snapshot.tasks[name].clone();
            batch.snapshot.state_revision += 1;
            batch.snapshot.watermark.revision = batch.snapshot.state_revision;
            let progress = shell::step(&f.request, &mut batch, shell::Action::Observe, shell::Fault::None).unwrap();
            assert_eq!(progress.outcome, Outcome::Pending, "tiles alone cannot complete assembly");
            if index == 0 {
                let duplicate =
                    shell::step(&f.request, &mut batch, shell::Action::Observe, shell::Fault::None).unwrap();
                assert_eq!(duplicate.outcome, Outcome::Pending, "duplicate acceptance cannot add coverage");
            }
        }
        *batch.snapshot.tasks.get_mut("assemble").unwrap() = f.snapshot.tasks["assemble"].clone();
        batch.snapshot.phase = RunPhase::Succeeded;
        batch.snapshot.state_revision += 1;
        batch.snapshot.watermark.revision = batch.snapshot.state_revision;
        let completed = shell::step(&f.request, &mut batch, shell::Action::Observe, shell::Fault::None).unwrap();
        assert_eq!(completed.outcome, Outcome::Completed);
        shell::verify_committed(&f.request, &completed).unwrap();
        assert_eq!(batch.submits, 1, "reordered observation never resubmits");
        let image = std::fs::read(
            Path::new(&f.request.journal_root).join(f.request.id().unwrap().to_string()).join("image.ppm"),
        )
        .unwrap();
        if let Some(expected) = &expected_image {
            assert_eq!(&image, expected, "completion order cannot change committed pixels");
        } else {
            expected_image = Some(image);
        }
    }
}

#[test]
fn identities_invalidate_scene_camera_renderer_sampling_seed_partition_encoding_and_attempt_reuse() {
    let f = fixture();
    let original = f.request.id().unwrap();
    let mut variations = Vec::new();
    let mut request = f.request.clone();
    request.manifest.camera.origin[0] += 1;
    variations.push(request);
    let mut request = f.request.clone();
    request.manifest.renderer = Id([9; 32]);
    variations.push(request);
    let mut request = f.request.clone();
    request.manifest.samples += 1;
    variations.push(request);
    let mut request = f.request.clone();
    request.manifest.bounces += 1;
    variations.push(request);
    let mut request = f.request.clone();
    request.manifest.seed += 1;
    variations.push(request);
    let mut request = f.request.clone();
    request.manifest.columns = 1;
    variations.push(request);
    let mut request = f.request.clone();
    request.manifest.output = "different".into();
    variations.push(request);
    let mut request = f.request.clone();
    request.manifest.runtime.engine_profile = "different".into();
    variations.push(request);
    let mut request = f.request.clone();
    request.attempt = "new-attempt".into();
    variations.push(request);
    let mut request = f.request.clone();
    let mut scene = request.manifest.scene().unwrap();
    scene.spheres[0].albedo[0] ^= 1;
    request.manifest.scene_bytes = codec::frame(render_pilot::SCENE_SCHEMA, &scene).unwrap();
    variations.push(request);
    for request in variations {
        assert_ne!(request.id().unwrap(), original);
        assert!(model::accept(&request, &f.snapshot, &f.objects).is_err());
    }
}

#[test]
fn request_bounds_include_assembly_and_all_retries() {
    let f = fixture();
    let prepared = f.request.prepare().unwrap();
    assert_eq!(prepared.job.tasks.len(), 5);
    let mut request = f.request.clone();
    request.budget.task_attempts = 9;
    assert!(request.prepare().is_err());
    let mut request = f.request.clone();
    request.budget.fuel -= 1;
    assert!(request.prepare().is_err());
    let mut request = f.request.clone();
    request.budget.task_wall_millis -= 1;
    assert!(request.prepare().is_err());
    let mut request = f.request.clone();
    request.manifest.height = 8;
    request.manifest.columns = 8;
    request.manifest.rows = 8;
    assert!(request.prepare().is_err());
    let mut request = f.request.clone();
    request.manifest.width = 1025;
    assert!(request.prepare().is_err());
}

#[test]
fn missing_extra_crossed_wrong_length_and_exit_zero_partial_cannot_complete() {
    let f = fixture();
    let mut cases = Vec::new();
    let mut s = f.snapshot.clone();
    s.tasks.remove("tile-000");
    cases.push(s);
    let mut s = f.snapshot.clone();
    s.tasks.insert("tile-extra".into(), ProgressTask::Cancelled);
    cases.push(s);
    let mut s = f.snapshot.clone();
    s.run = Id([8; 32]);
    cases.push(s);
    let mut s = f.snapshot.clone();
    s.generation += 1;
    cases.push(s);
    let mut s = f.snapshot.clone();
    s.tasks.insert("tile-000".into(), ProgressTask::Pending { ready_at: 0 });
    cases.push(s);
    let mut s = f.snapshot.clone();
    if let ProgressTask::Accepted { fence, .. } = s.tasks.get_mut("tile-000").unwrap() {
        fence.epoch += 1;
    }
    cases.push(s);
    let mut s = f.snapshot.clone();
    s.cancel_requested = true;
    cases.push(s);
    for snapshot in cases {
        assert!(model::accept(&f.request, &snapshot, &f.objects).is_err());
    }
    let mut objects = f.objects.clone();
    objects.values_mut().next().unwrap().pop();
    assert!(model::accept(&f.request, &f.snapshot, &objects).is_err());
    let mut objects = f.objects.clone();
    objects.values_mut().next().unwrap()[0] ^= 1;
    assert!(model::accept(&f.request, &f.snapshot, &objects).is_err());
}

#[test]
fn rehashed_wrong_pixels_and_coordinates_fail_producer_strict_verification() {
    let f = fixture();
    let mut outputs: BTreeMap<_, _> = f
        .snapshot
        .tasks
        .iter()
        .map(|(name, task)| {
            let ProgressTask::Accepted { output, .. } = task else {
                panic!()
            };
            (name.clone(), f.objects[output].clone())
        })
        .collect();
    // Self-consistent content identity AND raw pixel digest are not strict pixels.
    let image = outputs.get_mut("assemble").unwrap();
    image[render_pilot::FINAL_HEADER] ^= 0x80;
    let digest = blake3::hash(&image[render_pilot::FINAL_HEADER..]);
    image[48..80].copy_from_slice(digest.as_bytes());
    assert!(render_pilot::decode_final(&f.request.manifest, image).is_ok());
    assert!(render_pilot::verify_final(&f.request.manifest, image).is_err());
    let prepared = f.request.prepare().unwrap();
    let (snapshot, objects) = build_snapshot(&f.request, &prepared.job, &outputs);
    assert!(model::accept(&f.request, &snapshot, &objects).is_err());
    // Rebinding both accepted content and result IDs cannot authorize an overlap.
    let mut outputs: BTreeMap<_, _> = f
        .snapshot
        .tasks
        .iter()
        .map(|(name, task)| {
            let ProgressTask::Accepted { output, .. } = task else {
                panic!()
            };
            (name.clone(), f.objects[output].clone())
        })
        .collect();
    outputs.insert("tile-001".into(), outputs["tile-000"].clone());
    let (snapshot, objects) = build_snapshot(&f.request, &prepared.job, &outputs);
    assert!(model::accept(&f.request, &snapshot, &objects).is_err());
}

#[test]
fn lost_ack_reconciles_exact_run_and_never_resubmits() {
    let f = fixture();
    let mut batch = BatchFixture::new(&f);
    let lost = shell::step(&f.request, &mut batch, shell::Action::Advance, shell::Fault::LostSubmitAck).unwrap();
    assert_eq!(lost.outcome, Outcome::Unknown);
    assert_eq!(batch.submits, 1);
    let recovered = shell::step(&f.request, &mut batch, shell::Action::Advance, shell::Fault::None).unwrap();
    assert_eq!(recovered.outcome, Outcome::Completed);
    assert_eq!(recovered.recovery, "restored_workflow_state");
    assert_eq!(batch.submits, 1);
    assert_eq!(recovered.budget_identity, f.request.budget.identity);
}

#[test]
fn intent_without_ack_or_observation_remains_unknown_even_after_reopen() {
    let f = fixture();
    let mut batch = BatchFixture::new(&f);
    let lost = shell::step(&f.request, &mut batch, shell::Action::Advance, shell::Fault::AfterIntent).unwrap();
    assert_eq!(lost.outcome, Outcome::Unknown);
    assert_eq!(batch.submits, 0);
    let reopened = shell::step(&f.request, &mut batch, shell::Action::Advance, shell::Fault::None).unwrap();
    assert_eq!(reopened.outcome, Outcome::Unknown);
    assert_eq!(batch.submits, 0);
}

#[test]
fn interrupted_observation_and_publication_restore_without_new_computation() {
    for fault in [shell::Fault::AfterObservation, shell::Fault::BeforePublication] {
        let f = fixture();
        let mut batch = BatchFixture::new(&f);
        assert_eq!(
            shell::step(&f.request, &mut batch, shell::Action::Advance, fault).unwrap().outcome,
            Outcome::Unknown
        );
        assert_eq!(
            shell::step(&f.request, &mut batch, shell::Action::Observe, shell::Fault::None).unwrap().outcome,
            Outcome::Completed
        );
        assert_eq!(batch.submits, 1);
    }
}

#[test]
fn cancellation_request_is_not_stop_and_late_success_is_not_accepted() {
    let f = fixture();
    let mut batch = BatchFixture::new(&f);
    batch.submitted = true;
    let requested = shell::step(&f.request, &mut batch, shell::Action::Cancel, shell::Fault::None).unwrap();
    assert_eq!(requested.outcome, Outcome::CancellationRequested);
    assert!(!requested.remote_stop_observed);
    let late = shell::step(&f.request, &mut batch, shell::Action::Observe, shell::Fault::None).unwrap();
    assert_eq!(late.outcome, Outcome::CancellationRequested);
    assert!(late.accepted_manifest.is_none());
    batch.snapshot.phase = RunPhase::Cancelled;
    batch.snapshot.cancel_requested = true;
    batch.snapshot.state_revision += 1;
    let stopped = shell::step(&f.request, &mut batch, shell::Action::Observe, shell::Fault::None).unwrap();
    assert_eq!(stopped.outcome, Outcome::Stopped);
    assert!(stopped.remote_stop_observed);
}

#[test]
fn supersession_and_owner_revocation_block_late_acceptance() {
    for revoke in [false, true] {
        let f = fixture();
        let mut batch = BatchFixture::new(&f);
        shell::step(&f.request, &mut batch, shell::Action::Advance, shell::Fault::LostSubmitAck).unwrap();
        if revoke {
            let mut authority: Authority =
                serde_json::from_slice(&std::fs::read(&f.request.authority_file).unwrap()).unwrap();
            authority.attempt = "new-owner".into();
            std::fs::write(&f.request.authority_file, serde_json::to_vec(&authority).unwrap()).unwrap();
        } else {
            shell::step(&f.request, &mut batch, shell::Action::Supersede, shell::Fault::None).unwrap();
        }
        let late = shell::step(&f.request, &mut batch, shell::Action::Observe, shell::Fault::None).unwrap();
        assert_eq!(late.outcome, Outcome::Superseded);
        assert!(late.accepted_manifest.is_none());
        assert_eq!(batch.submits, 1);
    }
}

#[test]
fn exact_operation_graph_is_opt_in_and_mutation_has_no_read_only_grant() {
    let f = fixture();
    // Metadata-only fixture; the actual Lattice binary is exercised by the process rail.
    let executable = std::path::PathBuf::from(std::env::var_os("LATTICE_CAIRN_BIN").expect("pinned native fixture"));
    let file = f._root.path().join("binding.json");
    std::fs::write(&file, serde_json::to_vec(&f.request).unwrap()).unwrap();
    let catalog = render::operation::catalog(&executable).unwrap();
    let instances = render::operation::instance(&catalog, f._root.path(), &file, f.request.id().unwrap());
    let (plan, workflow) = render::cli::admitted_graph(&catalog, &instances, &executable, "advance").unwrap();
    let diagnostics = lattice::workflow_validation::validate_workflow(&workflow);
    assert!(!lattice::workflow_validation::has_fatal_diagnostics(&diagnostics), "{diagnostics:?}");
    let catalog = lattice::operation::catalog::validate(catalog).unwrap();
    let mut instances = lattice::operation::instance::validate(instances, &catalog).unwrap();
    let instance = instances.remove("admitted-render").unwrap();
    assert!(lattice::operation::shell::LocalReadOnlyAuthorityProvider::new(&catalog, &instance, &plan).is_err());
    assert!(lattice::operation::runtime::process_request_plan(&plan).is_ok());
    // An absolute file alone cannot substitute different manifest/attempt bytes.
    let mut changed = f.request.clone();
    changed.attempt = "crossed".into();
    std::fs::write(&file, serde_json::to_vec(&changed).unwrap()).unwrap();
    assert!(lattice::operation::runtime::process_request_plan(&plan).is_err());
}

#[test]
fn host_dispatch_provenance_rejects_crossed_compute_runs_but_preserves_explicit_observer() {
    let f = fixture();
    let executable = std::path::PathBuf::from(std::env::var_os("LATTICE_CAIRN_BIN").expect("pinned native fixture"));
    let binding_file = f._root.path().join("binding.json");
    std::fs::write(&binding_file, serde_json::to_vec(&f.request).unwrap()).unwrap();
    let catalog = render::operation::catalog(&executable).unwrap();
    let instances = render::operation::instance(&catalog, f._root.path(), &binding_file, f.request.id().unwrap());
    let (plan, _) = render::cli::admitted_graph(&catalog, &instances, &executable, "advance").unwrap();
    let credentials = lattice::credentials::EmptyCredentialProvider;
    assert!(lattice::operation::runtime::prepare_process_request(&plan, &credentials, None).is_err());
    // Synthetic contexts exercise the consumer contract, not a real engine run.
    let owner = shell::HostExecution {
        run_id: 71,
        node_id: 0,
        authority_run_identity: format!("blake3:{}", "a".repeat(64)),
        workflow_revision: format!("blake3:{}", "b".repeat(64)),
        invocation_identity: plan.identity.clone(),
    };
    let prepared = lattice::operation::runtime::prepare_process_request(&plan, &credentials, Some(&owner)).unwrap();
    let argument = |flag: &str| {
        let index = prepared.process.args.iter().position(|value| value == flag).unwrap();
        prepared.process.args[index + 1].clone()
    };
    let permit_path = argument("--permit");
    let permit_id = Id::parse(&argument("--permit-identity")).unwrap();
    let permit = shell::load_permit(&f.request, Path::new(&permit_path), permit_id).unwrap();
    let mut batch = BatchFixture::new(&f);
    let receipt = shell::step_bound(&f.request, &mut batch, &permit, shell::Fault::None).unwrap();
    assert_eq!(receipt.host_execution, Some(owner.clone()));
    assert_eq!(receipt.observed_host_execution, Some(owner.clone()));
    let output = lattice::agent_node::ProcessOutput {
        status_code: Some(0),
        stdout: serde_json::to_vec(&receipt).unwrap(),
        stderr: vec![],
    };
    assert_eq!(
        lattice::operation::runtime::classify_process_output(&plan, output.clone(), &[], Some(&owner))
            .unwrap()
            .class,
        lattice::operation::runtime::ObservationClass::Completed
    );
    let mut crossed = owner.clone();
    crossed.run_id += 1;
    assert!(lattice::operation::runtime::prepare_process_request(&plan, &credentials, Some(&crossed)).is_err());
    assert!(lattice::operation::runtime::classify_process_output(&plan, output, &[], Some(&crossed)).is_err());
    let (observe, _) = render::cli::admitted_graph(&catalog, &instances, &executable, "observe").unwrap();
    crossed.invocation_identity = observe.identity.clone();
    assert!(lattice::operation::runtime::prepare_process_request(&observe, &credentials, Some(&crossed)).is_ok());
    assert_eq!(batch.submits, 1);
}

#[test]
fn expired_original_budget_cancels_without_new_submit_or_late_acceptance() {
    let mut f = fixture();
    f.request.budget.deadline_epoch_millis = 1;
    let mut batch = BatchFixture::new(&f);
    let receipt = shell::step(&f.request, &mut batch, shell::Action::Advance, shell::Fault::None).unwrap();
    assert_eq!(receipt.outcome, Outcome::CancellationRequested);
    assert_eq!(batch.submits, 0);
    assert_eq!(batch.cancels, 1);
    assert!(receipt.accepted_manifest.is_none());
}
