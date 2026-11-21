//! Agent revision shell (Lattice #25): resolution from exported requests,
//! version-checked persistence, injected ports, the standalone CLI's
//! unavailable ports and the session-admission seam.

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
        evidence_class: verdict::EvidenceClass::TaskFamilyEvaluation,
        protocol_identity: reference('a'),
        verifier_identity: reference('b'),
        observed_at_ms: 1_795_200_000_000,
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
    let mut tools = base.clone();
    tools.worker_capability.as_mut().expect("capability").tools = None;
    let mut harness = base.clone();
    harness.omp.executable_blake3 = "9".repeat(64);
    let mut model = base.clone();
    model.omp.model_broker.model = "model-2".into();
    for changed in [prompt, tools, harness, model] {
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
        assert_eq!(seam::admit(&store, &base, "session-a").expect("seam"), seam::Decision::Unmanaged);
        assert_eq!(seam::effect(&store, &base, "session-a").expect("seam"), seam::Effect::Unmanaged);
        assert_eq!(seam::close(&store, &base, "session-a", &[]).expect("seam"), None);
        assert!(store.load_lifecycle(&definition).expect("load").is_none());
    }
    let request_path = write_request(root.path(), "request.json", &base);
    let prepared = verdict::Prepared::Prepared {
        artifacts: vec![reference('1')],
    };
    let mut preparation = FixturePreparation::new(vec![prepared], vec![verdict::Cleanup::Unknown]);
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
    assert!(matches!(seam::effect(&store, &base, "session-a").expect("seam"), seam::Effect::Permitted { .. }));
    let mut changed = base.clone();
    changed.omp.prompt = "another prompt".into();
    let refused = seam::admit(&store, &changed, "session-b").expect("seam");
    assert_eq!(refused, seam::Decision::Refused {
        refusal: lattice_core::agent_revision::Refusal::RevisionNotSelected
    });
    let other = lattice::agent_revision::bundle::resolve(&changed).expect("resolves");
    drop(store);
    let mismatched = select_args(&store_path, &definition, &other.revision, 'q', 1, "promote-2");
    let refused = run(Command::Promote(mismatched), &mut preparation, &mut qualification).unwrap_err();
    assert!(refused.contains("unknown_revision"), "{refused}");
    let revoke = lattice::agent_revision::RevokeArgs {
        store: store_path.clone(),
        definition: definition.clone(),
        revision: None,
        qualification_ref: Some(reference('q')),
        session: None,
    };
    run(Command::Revoke(revoke), &mut preparation, &mut qualification).expect("revoke");
    let store = Store::new(&store_path).expect("store");
    let blocked = seam::effect(&store, &base, "session-a").expect("seam");
    assert_eq!(blocked, seam::Effect::Blocked {
        refusal: lattice_core::agent_revision::Refusal::QualificationRevoked
    });
    assert!(qualification.queries().iter().all(|query| query.definition_identity == definition));
    let (mut other_subject, _) = answer(&definition, &other.revision, 'q');
    other_subject.now_ms = 1;
    let mismatch = lattice::agent_revision::QualificationPort::answer(&mut qualification, &other_subject);
    assert_eq!(mismatch, verdict::Verdict::NotQualified {
        reason: verdict::Unqualified::SubjectMismatch
    });
}
