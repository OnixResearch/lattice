//! Agent revision shell (Lattice #25): resolution from exported requests,
//! version-checked persistence, injected ports, the standalone CLI's
//! unavailable ports, the CLI's exit status and the session-admission seam.

use lattice::agent_revision::Command;
use lattice::agent_revision::CommandArgs;
use lattice::agent_revision::FixturePreparation;
use lattice::agent_revision::FixtureQualification;
use lattice::agent_revision::Ports;
use lattice::agent_revision::SelectArgs;
use lattice::agent_revision::seam;
use lattice::implementation_attempt::cli::Request;
use lattice::store::Store;
use lattice_core::agent_revision::ports as verdict;
use lattice_core::agent_revision::state::Preparation;

const FAMILY: &str = "rust-implementation";

fn request(prompt: &str) -> Request {
    let envelope = lattice::implementation_attempt::ImplementationAttemptEnvelope {
        schema: lattice::implementation_attempt::IMPLEMENTATION_ATTEMPT_ENVELOPE_SCHEMA.into(),
        work_id: "work-25".into(),
        work_version: 1,
        source_kind: lattice::implementation_attempt::SourceKind::CairnTask,
        source_id: "task-25".into(),
        repository: "OnixResearch/lattice".into(),
        base: "a".repeat(40),
        readiness_snapshot: "b".repeat(64),
        task_spec_digest: "c".repeat(64),
        verification_profile: "f".repeat(64),
        attempt_id: "attempt-25".into(),
        fence: 7,
        workspace_id: "workspace-25".into(),
        worker_profile: "0".repeat(64),
        budget: lattice::implementation_attempt::AttemptBudget {
            max_turns: 1,
            max_seconds: 60,
            max_cost_micros: 1,
        },
        policy_digest: "d".repeat(64),
    };
    Request {
        schema: lattice::implementation_attempt::cli::REQUEST_SCHEMA.into(),
        admitted_envelope: envelope.identity(),
        envelope,
        expires_at_ms: u64::MAX,
        workspace_root: "/workspace".into(),
        read_paths: vec!["src".into()],
        write_paths: vec!["src".into()],
        program: "/usr/bin/worker".into(),
        sandbox_program: "/usr/bin/bwrap".into(),
        omp: lattice::candidate_worker::omp::Profile {
            revision: lattice::candidate_worker::omp::OMP_REVISION.into(),
            executable_blake3: "e".repeat(64),
            base_identity: "0".repeat(64),
            model: "lattice/model".into(),
            prompt: prompt.into(),
            model_broker: lattice::candidate_worker::broker::Profile {
                schema: lattice::candidate_worker::broker::SCHEMA.into(),
                endpoint: "https://model.invalid/v1/chat/completions".into(),
                model: "model".into(),
                credential: Some(lattice::credentials::SecretRef::new("env", "MODEL_TOKEN", None)),
                relay: lattice::candidate_worker::broker::Executable {
                    path: "/nix/store/relay".into(),
                    blake3: "a".repeat(64),
                },
                shell: lattice::candidate_worker::broker::Executable {
                    path: "/nix/store/shell".into(),
                    blake3: "b".repeat(64),
                },
                sandbox_blake3: "c".repeat(64),
                expires_at_ms: u64::MAX,
                max_requests: 1,
            },
        },
        output_limit_bytes: 4096,
        allow_unchanged: false,
        worker_compute: lattice_core::execution_binding::request::ComputeRequirement::HostLocal {},
        execution_binding: lattice_core::execution_binding::request::Declaration {
            schema: lattice_core::execution_binding::request::DECLARATION_SCHEMA.into(),
            serving: None,
            compute: lattice_core::execution_binding::request::ComputeDeclaration::HostLocal {},
            model_routes: lattice_core::execution_binding::request::RouteDeclaration::NoModel {},
        },
        worker_capability: Some(lattice_core::execution_binding::capability::Declaration {
            schema: lattice_core::execution_binding::capability::CAPABILITY_SCHEMA.into(),
            definition: "rust-implementer".into(),
            task_families: Some(lattice_core::execution_binding::capability::NameSet::Declared {
                names: vec![FAMILY.into()],
            }),
            tools: Some(lattice_core::execution_binding::capability::NameSet::Declared {
                names: vec!["edit".into(), "read".into(), "write".into()],
            }),
        }),
        network: None,
        dispatch_plan: None,
        repair: None,
        lineage: None,
        workflow_agent: None,
    }
}

fn reference(digit: char) -> String {
    format!("blake3:{}", blake3::hash(digit.to_string().as_bytes()).to_hex())
}

fn write_request(root: &std::path::Path, name: &str, request: &Request) -> std::path::PathBuf {
    let path = root.join(name);
    std::fs::write(&path, serde_json::to_vec(request).expect("request encodes")).expect("request written");
    path
}

fn run(
    command: Command,
    preparation: &mut FixturePreparation,
    qualification: &mut FixtureQualification,
) -> Result<(), String> {
    lattice::agent_revision::run_command_with_ports(&CommandArgs { command }, Ports {
        preparation,
        qualification,
    })
}

/// Desire and prepare `request` through the fixture preparation port.
fn desire_and_prepare(store: &std::path::Path, request: std::path::PathBuf, subject: (&str, &str), artifact: char) {
    let (definition, revision) = subject;
    let prepared = verdict::Prepared::Prepared {
        artifacts: vec![reference(artifact)],
    };
    let mut preparation = FixturePreparation::new(vec![prepared], Vec::new());
    let mut unused = FixtureQualification::new(Vec::new());
    let desire = Command::Desire {
        request,
        store: store.into(),
    };
    run(desire, &mut preparation, &mut unused).expect("desire");
    let prepare = Command::Prepare {
        store: store.into(),
        definition: definition.into(),
        revision: revision.into(),
    };
    run(prepare, &mut preparation, &mut unused).expect("prepare");
    assert_eq!(preparation.tickets().len(), 1);
}

/// Promote or roll back with a fixture port that qualifies exactly the
/// selected subject.
fn select_qualified(command: Command, qualification: char) -> Result<(), String> {
    let (Command::Promote(args) | Command::Rollback(args)) = &command else {
        panic!("not a selection");
    };
    let answers = vec![answer(&args.definition, &args.revision, qualification)];
    run(command, &mut FixturePreparation::default(), &mut FixtureQualification::new(answers))
}

/// Run the `lattice` binary's `agent-revision` subcommand.
fn cli(arguments: &[&std::ffi::OsStr]) -> (bool, serde_json::Value) {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("agent-revision")
        .args(arguments)
        .output()
        .expect("lattice runs");
    let printed = serde_json::from_slice(&output.stdout).unwrap_or(serde_json::Value::Null);
    (output.status.success(), printed)
}

fn select_args(
    store: &std::path::Path,
    definition: &str,
    revision: &str,
    qualification: char,
    expected: u64,
    request_id: &str,
) -> SelectArgs {
    SelectArgs {
        store: store.into(),
        definition: definition.into(),
        revision: revision.into(),
        qualification_ref: reference(qualification),
        task_family: FAMILY.into(),
        required_class: "task_family_evaluation".into(),
        expected_generation: expected,
        request_id: request_id.into(),
        qualification_attachments: Vec::new(),
    }
}

fn answer(definition: &str, revision: &str, qualification: char) -> (verdict::Query, verdict::Verdict) {
    let query = verdict::Query {
        definition_identity: definition.into(),
        profile_revision: revision.into(),
        task_family: FAMILY.into(),
        required_class: verdict::EvidenceClass::TaskFamilyEvaluation,
        now_ms: 0,
    };
    let attachment = verdict::Attachment {
        qualification_ref: reference(qualification),
        subject: verdict::Subject {
            definition_identity: definition.into(),
            profile_revision: revision.into(),
            task_family: FAMILY.into(),
        },
        evidence_class: verdict::EvidenceClass::TaskFamilyEvaluation,
        protocol_identity: reference('a'),
        verifier_identity: reference('b'),
        observed_at_ms: 1_795_200_000_000,
        provenance: verdict::Provenance::Fixture,
    };
    (query, verdict::Verdict::Qualified(attachment))
}

#[test]
// r[verify lattice.agent_revisions.bundle]
// r[verify lattice.agent_revisions.bundle.scenario.hosted]
// r[verify lattice.agent_revisions.bundle.scenario.inert]
fn resolution_uses_the_opaque_revision_and_reports_hosted_unknowns() {
    let base = request("make the candidate");
    let resolved = lattice::agent_revision::bundle::resolve(&base).expect("resolves");
    assert_eq!(resolved.revision, base.worker_profile_identity().expect("revision"));
    assert_eq!(resolved.bundle.routes[0].weight_identity, lattice_core::agent_revision::facts::Unexposed::Unknown);
    assert_eq!(resolved.bundle.routes[0].provider_model, "model");
    let encoded = serde_json::to_string(&resolved).expect("encodes");
    for material in [
        "model.invalid",
        "MODEL_TOKEN",
        "/usr/bin/worker",
        "make the candidate",
        "/workspace",
    ] {
        assert!(!encoded.contains(material), "bundle leaks {material}");
    }
    let mut prompt = base.clone();
    prompt.omp.prompt = "another prompt".into();
    let per_attempt = lattice::agent_revision::bundle::resolve(&prompt).expect("resolves");
    assert_eq!(per_attempt.revision, resolved.revision, "a task prompt is per-attempt, not a revision");
    let mut tools = base.clone();
    tools.worker_capability.as_mut().expect("capability").tools = None;
    let mut harness = base.clone();
    harness.omp.executable_blake3 = "9".repeat(64);
    let mut model = base.clone();
    model.omp.model_broker.model = "model-2".into();
    for changed in [tools, harness, model] {
        let other = lattice::agent_revision::bundle::resolve(&changed).expect("resolves");
        assert_ne!(other.revision, resolved.revision);
        assert_eq!(other.definition.identity, resolved.definition.identity);
    }
}

#[test]
// r[verify lattice.agent_revisions.persistence]
// r[verify lattice.agent_revisions.persistence.scenario.conflict]
fn commits_are_version_checked() {
    let root = tempfile::tempdir().expect("scratch");
    let store = Store::new(root.path().join("lattice.redb")).expect("store");
    let definition = reference('d');
    assert!(store.load_lifecycle(&definition).expect("load").is_none());
    let lifecycle = lattice_core::agent_revision::state::Lifecycle::new(&definition);
    assert_eq!(store.commit_lifecycle(0, &lifecycle).expect("first commit"), 1);
    assert!(matches!(store.commit_lifecycle(0, &lifecycle), Err(lattice::store::StoreError::Conflict(_))));
    assert_eq!(store.commit_lifecycle(1, &lifecycle).expect("second commit"), 2);
    assert_eq!(store.load_lifecycle(&definition).expect("load").map(|(version, _)| version), Some(2));
}

#[test]
// r[verify lattice.agent_revisions.promotion.scenario.unavailable]
// r[verify lattice.agent_revisions.shell]
// r[verify lattice.agent_revisions.shell.scenario.standalone]
fn standalone_ports_record_unknown_preparation_and_refuse_promotion() {
    let root = tempfile::tempdir().expect("scratch");
    let store_path = root.path().join("lattice.redb");
    let base = request("make the candidate");
    let resolved = lattice::agent_revision::bundle::resolve(&base).expect("resolves");
    let request_path = write_request(root.path(), "request.json", &base);
    let standalone = |command| lattice::agent_revision::run_command(&CommandArgs { command });
    standalone(Command::Desire {
        request: request_path.clone(),
        store: store_path.clone(),
    })
    .expect("desire");
    standalone(Command::Prepare {
        store: store_path.clone(),
        definition: resolved.definition.identity.clone(),
        revision: resolved.revision.clone(),
    })
    .expect("prepare reports unknown");
    let promote = || select_args(&store_path, &resolved.definition.identity, &resolved.revision, 'q', 0, "promote-1");
    let uncertain = standalone(Command::Promote(promote())).unwrap_err();
    assert!(uncertain.contains("preparation_uncertain"), "{uncertain}");
    let (_, lifecycle) = Store::new(&store_path)
        .expect("store")
        .load_lifecycle(&resolved.definition.identity)
        .expect("load")
        .expect("managed");
    assert!(matches!(lifecycle.candidates[&resolved.revision].preparation, Preparation::Unknown {
        generation: 1
    }));
    let prepared = verdict::Prepared::Prepared {
        artifacts: vec![reference('1')],
    };
    let mut preparation = FixturePreparation::new(vec![prepared], Vec::new());
    let mut unused = FixtureQualification::new(Vec::new());
    let reconcile = Command::Prepare {
        store: store_path.clone(),
        definition: resolved.definition.identity.clone(),
        revision: resolved.revision.clone(),
    };
    run(reconcile, &mut preparation, &mut unused).expect("embedder reconciles the unknown preparation");
    assert_eq!(preparation.tickets()[0].generation, 1);
    let refused = standalone(Command::Promote(promote())).unwrap_err();
    assert!(refused.contains("qualification_unavailable"), "{refused}");
    assert!(refused.contains("animus#18"), "{refused}");
    let (_, lifecycle) = Store::new(&store_path)
        .expect("store")
        .load_lifecycle(&resolved.definition.identity)
        .expect("load")
        .expect("managed");
    assert!(matches!(lifecycle.candidates[&resolved.revision].preparation, Preparation::Prepared {
        generation: 1
    }));
    assert!(lifecycle.selected.is_none());
    standalone(Command::Inspect {
        store: root.path().join("absent.redb"),
        definition: resolved.definition.identity,
    })
    .expect("inspect");
    assert!(!root.path().join("absent.redb").exists());
}

#[test]
// r[verify lattice.agent_revisions.seam]
// r[verify lattice.agent_revisions.seam.scenario.unmanaged]
// r[verify lattice.agent_revisions.promotion.scenario.idempotent]
// r[verify lattice.agent_revisions.ports]
// r[verify lattice.agent_revisions.ports.scenario.exact_subject]
fn fixture_ports_drive_the_lifecycle_and_the_seam() {
    let root = tempfile::tempdir().expect("scratch");
    let store_path = root.path().join("lattice.redb");
    let base = request("make the candidate");
    let resolved = lattice::agent_revision::bundle::resolve(&base).expect("resolves");
    let definition = resolved.definition.identity.clone();
    {
        let store = Store::new(&store_path).expect("store");
        let decision = seam::admit(&store, &base, "session-a").expect("seam");
        assert_eq!(decision, seam::Decision::Unmanaged);
        let admission = decision.admission().expect("unmanaged sessions start");
        assert_eq!(seam::effect(&store, &base, "session-a", admission).expect("seam"), seam::Effect::Unmanaged);
        assert_eq!(seam::close(&store, &base, "session-a", admission, &[]).expect("seam"), seam::Closure::Unmanaged);
        assert!(store.load_lifecycle(&definition).expect("load").is_none());
    }
    let request_path = write_request(root.path(), "request.json", &base);
    let prepared = verdict::Prepared::Prepared {
        artifacts: vec![reference('1')],
    };
    let prepared_other = verdict::Prepared::Prepared {
        artifacts: vec![reference('2')],
    };
    let mut preparation = FixturePreparation::new(vec![prepared, prepared_other], vec![verdict::Cleanup::Unknown]);
    let mut qualification = FixtureQualification::new(vec![answer(&definition, &resolved.revision, 'q')]);
    run(
        Command::Desire {
            request: request_path.clone(),
            store: store_path.clone(),
        },
        &mut preparation,
        &mut qualification,
    )
    .expect("desire");
    let prepare = || Command::Prepare {
        store: store_path.clone(),
        definition: definition.clone(),
        revision: resolved.revision.clone(),
    };
    run(prepare(), &mut preparation, &mut qualification).expect("prepare");
    run(prepare(), &mut preparation, &mut qualification).expect("already prepared");
    assert_eq!(preparation.tickets().len(), 1);
    let promote = || select_args(&store_path, &definition, &resolved.revision, 'q', 0, "promote-1");
    run(Command::Promote(promote()), &mut preparation, &mut qualification).expect("promote");
    run(Command::Promote(promote()), &mut preparation, &mut qualification).expect("lost acknowledgement reconciles");
    let store = Store::new(&store_path).expect("store");
    let (_, lifecycle) = store.load_lifecycle(&definition).expect("load").expect("managed");
    assert_eq!(lifecycle.rollout_generation, 1);
    assert_eq!(lifecycle.receipts.len(), 1);
    let decision = seam::admit(&store, &base, "session-a").expect("seam");
    assert!(matches!(decision, seam::Decision::Admitted { .. }), "{decision:?}");
    let admission = decision.admission().expect("admitted sessions start");
    assert_eq!(admission, seam::Admission::Managed);
    assert!(matches!(
        seam::effect(&store, &base, "session-a", admission).expect("seam"),
        seam::Effect::Permitted { .. }
    ));
    let mut changed = base.clone();
    changed.omp.executable_blake3 = "9".repeat(64);
    let refused = seam::admit(&store, &changed, "session-b").expect("seam");
    assert_eq!(refused, seam::Decision::Refused {
        refusal: lattice_core::agent_revision::Refusal::RevisionNotSelected
    });
    assert_eq!(refused.admission(), None);
    let other = lattice::agent_revision::bundle::resolve(&changed).expect("resolves");
    drop(store);
    let changed_path = write_request(root.path(), "changed.json", &changed);
    let desire_changed = Command::Desire {
        request: changed_path,
        store: store_path.clone(),
    };
    run(desire_changed, &mut preparation, &mut qualification).expect("desire the changed revision");
    let prepare_changed = Command::Prepare {
        store: store_path.clone(),
        definition: definition.clone(),
        revision: other.revision.clone(),
    };
    run(prepare_changed, &mut preparation, &mut qualification).expect("prepare the changed revision");
    let mismatched = select_args(&store_path, &definition, &other.revision, 'u', 1, "promote-2");
    let refused = run(Command::Promote(mismatched), &mut preparation, &mut qualification).unwrap_err();
    assert!(refused.contains("not_qualified") && refused.contains("subject_mismatch"), "{refused}");
    let asked = qualification.queries().last().expect("the port was asked");
    assert_eq!(asked.profile_revision, other.revision);
    let revoke = lattice::agent_revision::RevokeArgs {
        store: store_path.clone(),
        definition: definition.clone(),
        revision: None,
        qualification_ref: Some(reference('q')),
        session: None,
    };
    run(Command::Revoke(revoke), &mut preparation, &mut qualification).expect("revoke");
    let store = Store::new(&store_path).expect("store");
    let blocked = seam::effect(&store, &base, "session-a", admission).expect("seam");
    assert_eq!(blocked, seam::Effect::Blocked {
        refusal: lattice_core::agent_revision::Refusal::QualificationRevoked
    });
    assert!(qualification.queries().iter().all(|query| query.definition_identity == definition));
    let (_, lifecycle) = store.load_lifecycle(&definition).expect("load").expect("managed");
    assert_eq!(lifecycle.selected.map(|selected| selected.revision), Some(resolved.revision));
}

#[test]
// r[verify lattice.agent_revisions.seam]
// r[verify lattice.agent_revisions.seam.scenario.admission_mode]
fn the_recorded_admission_mode_keeps_unmanaged_sessions_and_fails_closed() {
    type Refusal = lattice_core::agent_revision::Refusal;
    let root = tempfile::tempdir().expect("scratch");
    let store_path = root.path().join("lattice.redb");
    let base = request("make the candidate");
    let resolved = lattice::agent_revision::bundle::resolve(&base).expect("resolves");
    let definition = resolved.definition.identity.clone();
    let legacy = {
        let store = Store::new(&store_path).expect("store");
        let decision = seam::admit(&store, &base, "session-legacy").expect("seam");
        let admission = decision.admission().expect("unmanaged sessions start");
        assert_eq!(admission, seam::Admission::Unmanaged);
        let claimed = seam::effect(&store, &base, "session-ghost", seam::Admission::Managed).expect("seam");
        assert_eq!(claimed, seam::Effect::Blocked {
            refusal: Refusal::UnknownSession
        });
        let closed = seam::close(&store, &base, "session-ghost", seam::Admission::Managed, &[]).expect("seam");
        assert_eq!(closed, seam::Closure::Refused {
            refusal: Refusal::UnknownSession
        });
        assert!(store.load_lifecycle(&definition).expect("load").is_none());
        admission
    };
    let request_path = write_request(root.path(), "request.json", &base);
    desire_and_prepare(&store_path, request_path, (&definition, &resolved.revision), '1');
    {
        let store = Store::new(&store_path).expect("store");
        let early = seam::admit(&store, &base, "session-early").expect("seam");
        assert_eq!(early, seam::Decision::Refused {
            refusal: Refusal::NoSelection
        });
        assert_eq!(early.admission(), None);
        assert_eq!(seam::effect(&store, &base, "session-legacy", legacy).expect("seam"), seam::Effect::Unmanaged);
    }
    let promote = select_args(&store_path, &definition, &resolved.revision, 'q', 0, "promote-1");
    select_qualified(Command::Promote(promote), 'q').expect("promote");
    let store = Store::new(&store_path).expect("store");
    let version = |store: &Store| store.load_lifecycle(&definition).expect("load").expect("managed").0;
    let before = version(&store);
    assert_eq!(seam::effect(&store, &base, "session-legacy", legacy).expect("seam"), seam::Effect::Unmanaged);
    let legacy_close = seam::close(&store, &base, "session-legacy", legacy, &[reference('k')]).expect("seam");
    assert_eq!(legacy_close, seam::Closure::Unmanaged);
    assert_eq!(version(&store), before);
    let claimed = seam::effect(&store, &base, "session-legacy", seam::Admission::Managed).expect("seam");
    assert_eq!(claimed, seam::Effect::Blocked {
        refusal: Refusal::UnknownSession
    });
    let managed = seam::admit(&store, &base, "session-new").expect("seam").admission().expect("admitted");
    assert_eq!(managed, seam::Admission::Managed);
    assert!(matches!(
        seam::effect(&store, &base, "session-new", managed).expect("seam"),
        seam::Effect::Permitted { .. }
    ));
    let closed = seam::close(&store, &base, "session-new", managed, &[reference('k')]).expect("seam");
    assert!(matches!(closed, seam::Closure::Closed { .. }), "{closed:?}");
    assert_eq!(seam::effect(&store, &base, "session-new", managed).expect("seam"), seam::Effect::Blocked {
        refusal: Refusal::SessionClosed
    });
    assert_eq!(seam::admit(&store, &base, "session-new").expect("seam"), seam::Decision::Refused {
        refusal: Refusal::SessionClosed
    });
    let unknown = seam::close(&store, &base, "session-gone", managed, &[]).expect("seam");
    assert_eq!(unknown, seam::Closure::Refused {
        refusal: Refusal::UnknownSession
    });
}

#[test]
// r[verify lattice.agent_revisions.shell]
// r[verify lattice.agent_revisions.shell.scenario.exit_status]
fn the_cli_exits_non_zero_on_every_refusal() {
    let root = tempfile::tempdir().expect("scratch");
    let store_path = root.path().join("lattice.redb");
    let store = store_path.as_os_str();
    let base = request("make the candidate");
    let resolved = lattice::agent_revision::bundle::resolve(&base).expect("resolves");
    let definition = resolved.definition.identity.clone();
    let mut changed = base.clone();
    changed.omp.executable_blake3 = "9".repeat(64);
    let other = lattice::agent_revision::bundle::resolve(&changed).expect("resolves");
    let request_path = write_request(root.path(), "request.json", &base);
    let changed_path = write_request(root.path(), "changed.json", &changed);
    let malformed_path = root.path().join("malformed.json");
    std::fs::write(&malformed_path, b"{}").expect("malformed request written");
    let os = std::ffi::OsStr::new;
    let (resolves, printed) = cli(&[os("resolve"), os("--request"), request_path.as_os_str()]);
    assert!(resolves, "{printed}");
    assert_eq!(printed["outcome"]["revision"], resolved.revision.as_str());
    let (malformed, _) = cli(&[os("resolve"), os("--request"), malformed_path.as_os_str()]);
    assert!(!malformed);
    let cleanup = [os("cleanup"), os("--store"), store, os("--definition"), os(&definition)];
    let (unmanaged_cleanup, _) = cli(&cleanup);
    assert!(!unmanaged_cleanup);
    let admit = |request: &std::path::Path, session: &str| {
        cli(&[
            os("admit-session"),
            os("--request"),
            request.as_os_str(),
            os("--store"),
            store,
            os("--session"),
            os(session),
        ])
    };
    let (unmanaged, printed) = admit(&request_path, "session-legacy");
    assert!(unmanaged, "{printed}");
    assert_eq!(printed["outcome"]["decision"], "unmanaged");
    desire_and_prepare(&store_path, request_path.clone(), (&definition, &resolved.revision), '1');
    let (unselected, printed) = admit(&request_path, "session-early");
    assert!(!unselected);
    assert_eq!(printed["refusal"]["code"], "no_selection");
    let promote = select_args(&store_path, &definition, &resolved.revision, 'q', 0, "promote-1");
    select_qualified(Command::Promote(promote), 'q').expect("promote R1");
    desire_and_prepare(&store_path, changed_path.clone(), (&definition, &other.revision), '2');
    let promote = select_args(&store_path, &definition, &other.revision, 's', 1, "promote-2");
    select_qualified(Command::Promote(promote), 's').expect("promote R2");
    let rollback = select_args(&store_path, &definition, &resolved.revision, 'q', 2, "rollback-1");
    select_qualified(Command::Rollback(rollback), 'q').expect("roll back to R1");
    let (admitted, printed) = admit(&request_path, "session-a");
    assert!(admitted, "{printed}");
    assert_eq!(printed["outcome"]["decision"], "admitted");
    let (not_selected, printed) = admit(&changed_path, "session-b");
    assert!(!not_selected);
    assert_eq!(printed["refusal"]["code"], "revision_not_selected");
    let (repeated, printed) = admit(&request_path, "session-a");
    assert!(repeated, "{printed}");
    assert_eq!(printed["outcome"]["admitted"]["outcome"], "already_pinned");
    let rollback = |revision: &str, qualification: char, expected: &str, request_id: &str| {
        cli(&[
            os("rollback"),
            os("--store"),
            store,
            os("--definition"),
            os(&definition),
            os("--revision"),
            os(revision),
            os("--qualification-ref"),
            os(&reference(qualification)),
            os("--task-family"),
            os(FAMILY),
            os("--required-class"),
            os("task_family_evaluation"),
            os("--expected-generation"),
            os(expected),
            os("--request-id"),
            os(request_id),
        ])
    };
    let (reconciled, printed) = rollback(&resolved.revision, 'q', "2", "rollback-1");
    assert!(reconciled, "{printed}");
    assert_eq!(printed["outcome"]["outcome"], "already_applied");
    let (unavailable, printed) = rollback(&other.revision, 's', "3", "rollback-2");
    assert!(!unavailable);
    assert_eq!(printed["refusal"]["code"], "qualification_unavailable");
    let (conflict, printed) = rollback(&other.revision, 's', "3", "rollback-1");
    assert!(!conflict);
    assert_eq!(printed["refusal"]["code"], "request_conflict");
    let (cleaned, printed) = cli(&cleanup);
    assert!(cleaned, "{printed}");
    assert_eq!(printed["outcome"], serde_json::json!([]));
}

#[test]
// r[verify lattice.agent_revisions.persistence]
// r[verify lattice.agent_revisions.persistence.scenario.retry]
fn an_interleaved_stale_write_is_reloaded_and_reapplied() {
    use lattice_core::agent_revision::ops::sessions;
    let root = tempfile::tempdir().expect("scratch");
    let store = Store::new(root.path().join("lattice.redb")).expect("store");
    let definition = reference('d');
    let lifecycle = lattice_core::agent_revision::state::Lifecycle::new(&definition);
    assert_eq!(store.commit_lifecycle(0, &lifecycle).expect("first commit"), 1);
    let rival = sessions::Target::Revision("a".repeat(64));
    let ours = sessions::Target::Revision("b".repeat(64));
    let attempts = std::cell::Cell::new(0_u32);
    let seen = std::cell::RefCell::new(Vec::new());
    let result =
        lattice::agent_revision::transact(&store, &definition, lattice::agent_revision::Seed::Existing, |current| {
            let attempt = attempts.get();
            attempts.set(attempt + 1);
            seen.borrow_mut().push(current.revoked.revisions.clone());
            if attempt == 0 {
                let (version, stored) = store.load_lifecycle(&definition).expect("load").expect("stored");
                let interleaved = sessions::revoke(&stored, &rival).expect("rival revokes").lifecycle;
                store.commit_lifecycle(version, &interleaved).expect("the rival commits between load and commit");
            }
            sessions::revoke(current, &ours)
        })
        .expect("the transition commits");
    assert_eq!(result, Ok(sessions::Revocation::Revoked));
    assert_eq!(attempts.get(), 2);
    assert!(seen.borrow()[0].is_empty());
    assert!(seen.borrow()[1].contains(&"a".repeat(64)));
    let (version, stored) = store.load_lifecycle(&definition).expect("load").expect("stored");
    assert_eq!(version, 3);
    assert!(stored.revoked.revisions.contains(&"a".repeat(64)));
    assert!(stored.revoked.revisions.contains(&"b".repeat(64)));
    let contended =
        lattice::agent_revision::transact(&store, &definition, lattice::agent_revision::Seed::Existing, |current| {
            let (version, stored) = store.load_lifecycle(&definition).expect("load").expect("stored");
            let session = sessions::Target::Session(format!("rival-{version}"));
            let interleaved = sessions::revoke(&stored, &session).expect("rival revokes").lifecycle;
            store.commit_lifecycle(version, &interleaved).expect("the rival always commits first");
            sessions::revoke(current, &sessions::Target::Revision("c".repeat(64)))
        });
    assert!(contended.expect_err("bounded retries report contention").contains("contended"));
    let (version, stored) = store.load_lifecycle(&definition).expect("load").expect("stored");
    assert_eq!(version, 7);
    assert!(!stored.revoked.revisions.contains(&"c".repeat(64)));
}

fn intent_digest(value: &str) -> String {
    format!("b3:{}", blake3::hash(value.as_bytes()).to_hex())
}

/// A synthetic two-obligation workflow intent naming `compiler`.
fn intent(compiler: &str, guidance: &str) -> lattice::workflow::intent::CompileRequest {
    use lattice::workflow::intent as wi;
    let cohort = wi::SourceCohort {
        producer_identity: intent_digest("producer"),
        selection_identity: intent_digest("selection"),
        obligations_identity: intent_digest("obligations"),
        profile_identity: intent_digest("profile"),
        context_identity: intent_digest("context"),
        source_identity: intent_digest("source"),
        policy_identity: intent_digest("policy"),
        dependency_identity: intent_digest("dependencies"),
    };
    let obligation = |id: &str, class: &str, prerequisites: Vec<String>| wi::Obligation {
        id: id.into(),
        obligation_identity: intent_digest(&format!("obligation:{id}")),
        guidance_identity: intent_digest(&format!("{guidance}:{id}")),
        remaining_condition_identity: intent_digest(&format!("remaining:{id}")),
        precondition_identity: intent_digest(&format!("precondition:{id}")),
        operation_class: class.into(),
        action_identity: intent_digest(&format!("action:{id}")),
        prerequisites,
        required_observations: vec![wi::ObservationContract {
            id: format!("evidence-{id}"),
            schema: "lattice.workflow-intent-fixture-result.v1".into(),
            predicate_identity: intent_digest(&format!("predicate:{id}")),
        }],
        required_capabilities: std::collections::BTreeSet::from(["native-cairn".into()]),
    };
    wi::CompileRequest {
        intent: wi::Definition {
            schema: wi::INTENT_SCHEMA.into(),
            cohort: cohort.clone(),
            coverage: wi::ContextCoverage::Complete,
            unsupported_fields: vec![],
            obligations: vec![
                obligation("B", "review_wait", vec!["A".into()]),
                obligation("A", "cairn_validate", vec![]),
            ],
            alternatives: vec![],
            selected_alternative: None,
        },
        facts: wi::CompilerFacts {
            expected_cohort: cohort,
            compiler_artifact_identity: compiler.into(),
            runtime_profile_identity: intent_digest("runtime-profile"),
            available_capabilities: std::collections::BTreeSet::from(["native-cairn".into()]),
            available_implementations: std::collections::BTreeSet::new(),
        },
    }
}

#[test]
// r[verify lattice.agent_revisions.workflow]
// r[verify lattice.agent_revisions.workflow.inspect]
fn a_compiled_plan_and_a_request_compose_an_inspectable_workflow_agent_bundle() {
    let worker = lattice::agent_revision::bundle::resolve(&request("make the candidate")).expect("resolves");
    let compiled =
        lattice::workflow::intent::compile(intent(&intent_digest("compiler"), "guidance")).expect("compiles");
    let composed =
        lattice::agent_revision::composite::compose("verify-change", &compiled, worker.clone()).expect("composes");
    let binding = composed.bundle.workflow.as_ref().expect("binding");
    assert_eq!(binding.plan.identity, compiled.identity());
    assert!(binding.plan.graph_identity.starts_with("blake3:"));
    assert_eq!(binding.plan.obligations.iter().map(|obligation| obligation.id.as_str()).collect::<Vec<_>>(), [
        "A", "B"
    ]);
    assert_eq!(binding.worker.revision, worker.revision);
    assert_eq!(composed.revision, composed.bundle.revision);
    assert_ne!(composed.revision, worker.revision);
    let encoded = serde_json::to_string(&composed).expect("encodes");
    for material in [
        "model.invalid",
        "MODEL_TOKEN",
        "/usr/bin/worker",
        "make the candidate",
        "/workspace",
    ] {
        assert!(!encoded.contains(material), "bundle leaks {material}");
    }
    let per_attempt = lattice::agent_revision::bundle::resolve(&request("another prompt")).expect("resolves");
    let same = lattice::agent_revision::composite::compose("verify-change", &compiled, per_attempt).expect("composes");
    assert_eq!(same.revision, composed.revision, "a task prompt is per-attempt");
    let guided =
        lattice::workflow::intent::compile(intent(&intent_digest("compiler"), "guidance-2")).expect("compiles");
    let adapted = lattice::agent_revision::composite::compose("verify-change", &guided, worker).expect("composes");
    assert_ne!(adapted.revision, composed.revision, "obligation guidance is part of the plan");
    assert_eq!(adapted.definition, composed.definition);

    let root = tempfile::tempdir().expect("scratch");
    let store_path = root.path().join("revisions.db");
    let store = Store::new(&store_path).expect("store");
    lattice::agent_revision::transact(
        &store,
        &composed.definition,
        lattice::agent_revision::Seed::Create,
        |lifecycle| lattice_core::agent_revision::ops::prepare::desire(lifecycle, &composed.bundle),
    )
    .expect("commits")
    .expect("desired");
    drop(store);
    let (is_success, printed) = cli(&[
        "inspect".as_ref(),
        "--store".as_ref(),
        store_path.as_os_str(),
        "--definition".as_ref(),
        composed.definition.as_ref(),
    ]);
    assert!(is_success, "inspect succeeds: {printed}");
    let slots = &printed["outcome"]["workflow_slots"];
    assert_eq!(slots["schema"], "lattice.workflow-agent-revision-slots.v1");
    assert_eq!(slots["desired"]["revision"], composed.revision.as_str());
    assert_eq!(slots["desired"]["workflow"]["plan_identity"], compiled.identity());
    assert_eq!(slots["desired"]["worker"]["revision"], binding.worker.revision.as_str());
    assert!(slots["selected_for_new_sessions"].is_null());

    let plain_store = root.path().join("plain.db");
    let plain = write_request(root.path(), "plain.json", &request("make the candidate"));
    let resolved = lattice::agent_revision::bundle::resolve(&request("make the candidate")).expect("resolves");
    let desire = Command::Desire {
        request: plain,
        store: plain_store.clone(),
    };
    run(desire, &mut FixturePreparation::default(), &mut FixtureQualification::default()).expect("desire");
    let (is_success, printed) = cli(&[
        "inspect".as_ref(),
        "--store".as_ref(),
        plain_store.as_os_str(),
        "--definition".as_ref(),
        resolved.definition.identity.as_ref(),
    ]);
    assert!(is_success);
    assert!(printed["outcome"].get("workflow_slots").is_none(), "a plain lifecycle has no workflow slots");
}

#[test]
// r[verify lattice.agent_revisions.workflow.inspect]
fn workflow_commands_refuse_an_intent_for_another_compiler_and_write_nothing() {
    let root = tempfile::tempdir().expect("scratch");
    let request = write_request(root.path(), "request.json", &request("make the candidate"));
    let intent_path = root.path().join("intent.json");
    let stale = intent(&intent_digest("another-compiler"), "guidance");
    std::fs::write(&intent_path, serde_json::to_vec(&stale).expect("encodes")).expect("intent written");
    let store = root.path().join("revisions.db");
    let pairing: [&std::ffi::OsStr; 6] = [
        "--request".as_ref(),
        request.as_os_str(),
        "--intent".as_ref(),
        intent_path.as_os_str(),
        "--name".as_ref(),
        "verify-change".as_ref(),
    ];
    let mut resolve: Vec<&std::ffi::OsStr> = vec!["resolve-workflow".as_ref()];
    resolve.extend(pairing);
    let (is_success, _) = cli(&resolve);
    assert!(!is_success, "a different compiler is refused, never rebound");
    let mut desire: Vec<&std::ffi::OsStr> = vec!["desire-workflow".as_ref()];
    desire.extend(pairing);
    desire.extend(["--store".as_ref(), store.as_os_str()]);
    let (is_success, _) = cli(&desire);
    assert!(!is_success);
    assert!(!store.exists(), "a refused intent creates no store");
}
