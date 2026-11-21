//! Agent-revision session seam wiring over the real `agent-revision` and
//! `implementation-attempt` CLI shells.
//!
//! The fixture worker never starts a process: its program, sandbox and broker
//! runtimes are not immutable store artifacts, so the existing OMP route
//! blocks it at broker admission and publishes a `blocked` terminal result.
//! No OMP build, sandbox or model endpoint is needed. Attachments are encoded
//! with Animus's own `encode_attachment` for the subject Lattice resolves.

// r[verify lattice.agent_revisions.seam]
// r[verify lattice.agent_revisions.seam.scenario.attempt]
// r[verify lattice.implementation_attempt.consumer.scenario.admission_mode]
// r[verify lattice.worker_inventory.revisions.scenario.managed]
// r[verify lattice.worker_inventory.revisions.scenario.pinned]
// r[verify lattice.worker_inventory.qualification.scenario.qualified]

use animus_runtime::qualification as animus;

const FAMILY: &str = "rust-implementation";
const OBSERVED_AT_MS: u64 = 1_700_000_000_000;
const VALID_UNTIL_MS: u64 = 4_000_000_000_000;

#[derive(clap::Parser)]
struct RevisionCli {
    #[command(flatten)]
    args: crate::agent_revision::CommandArgs,
}

#[derive(clap::Parser)]
struct AttemptCli {
    #[command(flatten)]
    args: super::super::CommandArgs,
}

pub(super) struct Fixture {
    _root: tempfile::TempDir,
    pub root: std::path::PathBuf,
    pub state: std::path::PathBuf,
    pub store: std::path::PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        let root = tempfile::tempdir().expect("scratch");
        let canonical = root.path().canonicalize().expect("canonical scratch");
        std::fs::create_dir_all(canonical.join("workspace/src")).expect("workspace");
        Self {
            state: canonical.join("state"),
            store: canonical.join("revisions.redb"),
            root: canonical,
            _root: root,
        }
    }

    /// A host-local request of the `rust-implementer` definition.
    pub fn request(&self, attempt_id: &str, fence: u64) -> super::super::Request {
        let mut request = super::request(
            attempt_id,
            lattice_core::execution_binding::request::ComputeRequirement::HostLocal {},
            lattice_core::execution_binding::request::ComputeDeclaration::HostLocal {},
        );
        request.workspace_root = self.root.join("workspace");
        request.envelope.fence = fence;
        super::with_capability(request, capability())
    }

    pub fn write(&self, name: &str, request: &super::super::Request) -> (std::path::PathBuf, String) {
        let bytes = serde_json::to_vec(request).expect("request bytes");
        let path = self.root.join(name);
        std::fs::write(&path, &bytes).expect("request file");
        (path, blake3::hash(&bytes).to_hex().to_string())
    }

    pub fn directory(&self, request: &super::super::Request) -> std::path::PathBuf {
        super::super::artifacts::directory(&self.state, request)
    }

    fn text(path: &std::path::Path) -> &str {
        path.to_str().expect("UTF-8 path")
    }

    /// `lattice agent-revision ARGS... --store STORE` with a fixture
    /// preparation port and the standalone qualification port, so promotion
    /// is qualified only by the named attachment files.
    pub fn revision(&self, args: &[&str]) -> Result<(), String> {
        let mut argv = vec!["agent-revision"];
        argv.extend_from_slice(args);
        argv.extend(["--store", Self::text(&self.store)]);
        let cli = <RevisionCli as clap::Parser>::try_parse_from(argv).map_err(|error| error.to_string())?;
        let mut preparation = crate::agent_revision::FixturePreparation::new(
            vec![lattice_core::agent_revision::ports::Prepared::Prepared {
                artifacts: vec![format!("blake3:{}", blake3::hash(b"prepared-artifact").to_hex())],
            }],
            Vec::new(),
        );
        crate::agent_revision::run_command_with_ports(&cli.args, crate::agent_revision::Ports {
            preparation: &mut preparation,
            qualification: &mut crate::agent_revision::UnavailableQualification,
        })
    }

    /// `lattice implementation-attempt COMMAND --request REQUEST --state STATE [--revision-store
    /// STORE]` with the standalone compute and serving ports.
    pub async fn attempt(&self, command: &str, request: &std::path::Path, is_managed: bool) -> Result<(), String> {
        let mut argv = vec!["implementation-attempt", command, "--request", Self::text(request)];
        argv.extend(["--state", Self::text(&self.state)]);
        if is_managed {
            argv.extend(["--revision-store", Self::text(&self.store)]);
        }
        let cli = <AttemptCli as clap::Parser>::try_parse_from(argv).map_err(|error| error.to_string())?;
        super::super::run_command(&cli.args).await
    }

    /// Desire and prepare `request`'s revision; returns its definition and revision.
    pub fn prepared(&self, request: &super::super::Request, path: &std::path::Path) -> (String, String) {
        let resolved = crate::agent_revision::bundle::resolve(request).expect("resolves");
        let definition = resolved.definition.identity;
        self.revision(&["desire", "--request", Self::text(path)]).expect("desire");
        self.revision(&["prepare", "--definition", &definition, "--revision", &resolved.revision])
            .expect("prepare through the fixture port");
        (definition, resolved.revision)
    }

    /// Promote with an Animus #18 attachment for the exact subject; returns
    /// the attachment identity.
    pub fn promoted(&self, definition: &str, revision: &str) -> String {
        let attachment = animus::encode_attachment(&document(definition, revision)).expect("valid attachment");
        let path = self.root.join("attachment.json");
        std::fs::write(&path, &attachment.canonical).expect("attachment file");
        self.revision(&[
            "promote",
            "--definition",
            definition,
            "--revision",
            revision,
            "--qualification-ref",
            &attachment.identity,
            "--task-family",
            FAMILY,
            "--required-class",
            "task_family_evaluation",
            "--expected-generation",
            "0",
            "--request-id",
            "promote-1",
            "--qualification-attachment",
            Self::text(&path),
        ])
        .expect("promote with the attachment");
        attachment.identity
    }

    pub fn lifecycle(&self, definition: &str) -> lattice_core::agent_revision::state::Lifecycle {
        crate::store::Store::new(&self.store)
            .expect("store")
            .load_lifecycle(definition)
            .expect("load")
            .expect("managed")
            .1
    }

    pub fn inventory(&self, request: &std::path::Path, is_managed: bool) -> Result<serde_json::Value, String> {
        let store = is_managed.then_some(self.store.as_path());
        super::super::inspect::project(
            request,
            &self.state,
            &mut crate::execution_binding::serving::UnavailableOwner,
            super::super::session::revision::Store(store),
        )
    }
}

fn capability() -> lattice_core::execution_binding::capability::Declaration {
    lattice_core::execution_binding::capability::Declaration {
        schema: lattice_core::execution_binding::capability::CAPABILITY_SCHEMA.into(),
        definition: "rust-implementer".into(),
        task_families: Some(lattice_core::execution_binding::capability::NameSet::Declared {
            names: vec![FAMILY.into()],
        }),
        tools: Some(lattice_core::execution_binding::capability::NameSet::Declared {
            names: vec!["edit".into(), "read".into(), "write".into()],
        }),
    }
}

fn reference(label: &str) -> String {
    format!("blake3:{}", blake3::hash(label.as_bytes()).to_hex())
}

/// A complete fixture-labelled `task_family_evaluation` attachment document.
fn document(definition: &str, revision: &str) -> animus::AttachmentDocument {
    let bound = |kind| animus::component_identity(kind, b"v1");
    animus::AttachmentDocument {
        subject: animus::QualificationSubject {
            lattice_definition_identity: definition.into(),
            lattice_profile_revision: revision.into(),
            task_family: FAMILY.into(),
            components: animus::ComponentKind::ALL
                .into_iter()
                .map(|kind| animus::Component {
                    kind,
                    basis: animus::ComponentBasis::Observed,
                    identity: Some(bound(kind)),
                })
                .collect(),
            model: animus::ModelFacts {
                deployment: animus::ModelDeployment::Hosted,
                version_basis: animus::VersionBasis::ProviderDeclared,
                version: Some("model".into()),
                weights_identity: None,
            },
        },
        evidence: animus::EvidenceSource {
            class: animus::EvidenceClass::TaskFamilyEvaluation,
            protocol_identity: bound(animus::ComponentKind::TaskProtocol),
            verifier_identity: bound(animus::ComponentKind::Verifier),
            corpus_identity: Some(bound(animus::ComponentKind::Corpus)),
            source_identity: reference("run-1"),
        },
        outcomes: Some(animus::Outcomes {
            planned: 3,
            accepted: 2,
            rejected: 1,
            failed: 0,
            cancelled: 0,
            incomplete: 0,
        }),
        measures: Some(animus::Measures {
            repetitions: 1,
            tasks_covered: 3,
            tasks_declared: 3,
            time_to_acceptance_ms: None,
            covered_time: animus::CoveredTime {
                queue_ms: None,
                repair_ms: None,
                verification_ms: None,
            },
            costs: animus::Costs {
                unit: "cost-units".into(),
                model: None,
                tool: None,
                manager: None,
                verifier: None,
                unknown_remainder: true,
            },
            review_interventions: None,
            invalid_actions: None,
            false_completion_attempts: None,
        }),
        scope: animus::Scope {
            partitions: vec!["all".into()],
            limitations: vec!["synthetic-tasks-only".into()],
            missing_modes: vec!["live-model-route".into()],
        },
        provenance: animus::Provenance {
            label: animus::EvidenceLabel::Fixture,
            producer: "lattice-test".into(),
            producer_revision: "test".into(),
            observed_at_ms: OBSERVED_AT_MS,
            valid_until_ms: VALID_UNTIL_MS,
        },
        references: vec![animus::Reference {
            kind: "atif-trajectory".into(),
            identity: reference("trajectory"),
        }],
    }
}

pub(super) fn recorded(
    directory: &std::path::Path,
    request: &super::super::Request,
    identity: &str,
) -> super::super::session::intent::Record {
    let session = super::super::session::revision::identifier(request);
    let subject = super::super::session::intent::Subject {
        request_identity: identity,
        session: &session,
    };
    super::super::session::intent::read(directory, subject)
        .expect("readable intent")
        .expect("intent exists")
}

fn assert_not_started(directory: &std::path::Path) {
    for name in [
        super::super::binding::ADMITTED_FILE,
        super::super::session::intent::FILE,
        "candidate.redb",
        "raw-events.jsonl",
        "omp-observation.json",
    ] {
        assert!(!directory.join(name).exists(), "{name} must not exist");
    }
}

#[tokio::test]
// r[verify lattice.agent_revisions.revocation.scenario.retained]
async fn a_managed_attempt_is_pinned_closed_and_a_later_one_refused_after_revocation() {
    let fixture = Fixture::new();
    let first = fixture.request("attempt-managed", 7);
    let (first_path, first_identity) = fixture.write("first.json", &first);
    let (definition, revision) = fixture.prepared(&first, &first_path);
    let qualification_ref = fixture.promoted(&definition, &revision);

    fixture
        .attempt("run", &first_path, true)
        .await
        .expect("the fixture worker reaches a terminal result");
    let directory = fixture.directory(&first);
    let super::super::session::intent::Record::Owned(recorded) = recorded(&directory, &first, &first_identity) else {
        panic!("the run owns its intent");
    };
    assert_eq!(recorded.admission, crate::agent_revision::seam::Admission::Managed);
    let pin = recorded.pin.expect("a managed intent records its pin");
    assert_eq!(
        (pin.revision.as_str(), pin.qualification_ref.as_str()),
        (revision.as_str(), qualification_ref.as_str())
    );
    assert_eq!(pin.rollout_generation, 1);
    assert!(directory.join("raw-events.jsonl").exists(), "the worker step ran after the permit");
    let lifecycle = fixture.lifecycle(&definition);
    assert!(lifecycle.sessions.is_empty(), "the terminal attempt closed its session without checkpoints");

    let inventory = fixture.inventory(&first_path, true).expect("inventory");
    let profile = &inventory["profile"];
    assert_eq!(profile["revisions"]["desired"]["value"]["revision"], revision.as_str());
    assert_eq!(profile["revisions"]["prepared"]["value"], serde_json::json!([revision]));
    assert_eq!(profile["revisions"]["qualified"]["value"][0]["provenance"], "fixture");
    assert_eq!(
        profile["revisions"]["selected_for_new_sessions"]["value"]["qualification_ref"],
        qualification_ref.as_str()
    );
    assert_eq!(profile["task_families"]["support"], "qualified");
    assert_eq!(
        profile["capability_qualifications"],
        serde_json::json!([{
            "task_family": FAMILY,
            "qualification_ref": qualification_ref,
            "evidence_class": "task_family_evaluation",
            "provenance": "fixture",
        }])
    );
    assert!(!profile["wait_reasons"].to_string().contains("qualification_unbound"));
    let pinned = &inventory["instance"]["pinned"];
    assert_eq!(pinned["support"], "recorded");
    assert_eq!(pinned["value"]["status"], "released");
    assert_eq!(pinned["value"]["is_blocked"], false);
    assert_eq!(pinned["value"]["pin_identity"], pin.pin_identity.as_str());

    fixture.revision(&["revoke", "--definition", &definition, "--revision", &revision]).expect("revoke");
    let inventory = fixture.inventory(&first_path, true).expect("inventory");
    assert_eq!(inventory["instance"]["pinned"]["value"]["is_blocked"], true, "revocation reaches the closed pin");
    assert_eq!(inventory["profile"]["capability_qualifications"], serde_json::json!([]));
    assert!(inventory["profile"]["wait_reasons"].to_string().contains("revision_not_admitted"));

    let second = fixture.request("attempt-after-revocation", 8);
    let (second_path, _) = fixture.write("second.json", &second);
    let refused = fixture.attempt("run", &second_path, true).await;
    assert_eq!(refused, Err(super::super::refusal::Reason::RevisionNotAdmitted.message().to_owned()));
    assert_not_started(&fixture.directory(&second));
    let fence = fixture.state.join("works").join(blake3::hash(second.envelope.work_id.as_bytes()).to_hex().as_str());
    assert!(!fence.join(format!("{:020}", 8)).exists(), "a refused session records no fence");
    assert!(fixture.lifecycle(&definition).sessions.is_empty(), "a refused session is never pinned");

    let output_parent = fixture.root.join("exports");
    std::fs::create_dir(&output_parent).expect("output parent");
    let blocked = super::super::export::run(
        &first_path,
        &fixture.state,
        &output_parent,
        super::super::session::revision::Store(Some(&fixture.store)),
    );
    assert_eq!(blocked, Err(super::super::refusal::Reason::ExportRevoked.message().to_owned()));
    let record = super::super::refusal::record(&first.envelope, super::super::refusal::Reason::ExportRevoked);
    assert_eq!(record["reason"], "export_revoked");
    assert_eq!(record["dispatched"], true, "the attempt executed; a coordinator must never rerun it");
    for (reason, dispatched) in [
        (super::super::refusal::Reason::RevisionBlocked, false),
        (super::super::refusal::Reason::RevisionNotAdmitted, false),
        (super::super::refusal::Reason::SubstitutedRequest, true),
    ] {
        assert_eq!(super::super::refusal::record(&first.envelope, reason)["dispatched"], dispatched, "{reason:?}");
    }
    assert!(std::fs::read_dir(&output_parent).expect("exports").next().is_none(), "nothing is exported");
}

#[tokio::test]
async fn a_managed_definition_without_a_selection_refuses_the_attempt_before_any_write() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-unselected", 7);
    let (path, _) = fixture.write("request.json", &request);
    let (definition, _) = fixture.prepared(&request, &path);
    let refused = fixture.attempt("run", &path, true).await;
    assert_eq!(refused, Err(super::super::refusal::Reason::RevisionNotAdmitted.message().to_owned()));
    assert_not_started(&fixture.directory(&request));
    assert!(fixture.lifecycle(&definition).sessions.is_empty());
    let record = super::super::refusal::record(&request.envelope, super::super::refusal::Reason::RevisionNotAdmitted);
    assert_eq!(record["dispatched"], false);
    assert_eq!(record["reason"], "revision_not_admitted");
}

#[tokio::test]
// r[verify lattice.implementation_attempt.consumer.scenario.recovery]
async fn an_unmanaged_attempt_keeps_todays_intent_and_recovery_keeps_the_recorded_mode() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-unmanaged", 7);
    let (path, identity) = fixture.write("request.json", &request);
    fixture.attempt("run", &path, false).await.expect("standalone run");
    let directory = fixture.directory(&request);
    assert_eq!(
        std::fs::read(directory.join(super::super::session::intent::FILE)).expect("intent"),
        identity.as_bytes(),
        "without a store the intent is byte-identical to the request identity"
    );
    assert!(!directory.join(super::super::session::revision::BLOCK_FILE).exists());

    // The definition becomes managed later: the legacy intent stays unmanaged.
    let (definition, revision) = fixture.prepared(&request, &path);
    fixture.promoted(&definition, &revision);
    fixture.attempt("recover", &path, true).await.expect("an unmanaged attempt recovers with a store");
    fixture.attempt("recover", &path, false).await.expect("and without one");
    assert!(fixture.lifecycle(&definition).sessions.is_empty(), "recovery never pins an unmanaged session");
    let inventory = fixture.inventory(&path, true).expect("inventory");
    assert_eq!(inventory["instance"]["pinned"], serde_json::json!({"support": "unmanaged"}));
    assert_eq!(inventory["profile"]["revisions"]["selected_for_new_sessions"]["support"], "recorded");
    assert_eq!(
        std::fs::read(directory.join(super::super::session::intent::FILE)).expect("intent"),
        identity.as_bytes(),
        "recovery leaves the intent alone"
    );

    // A store-backed run of an unmanaged definition records `unmanaged`.
    let other = Fixture::new();
    let unmanaged = other.request("attempt-recorded-unmanaged", 7);
    let (unmanaged_path, unmanaged_identity) = other.write("request.json", &unmanaged);
    crate::store::Store::new(&other.store).expect("an empty revision store");
    other.attempt("run", &unmanaged_path, true).await.expect("unmanaged run with a store");
    let unmanaged_directory = other.directory(&unmanaged);
    assert_eq!(
        recorded(&unmanaged_directory, &unmanaged, &unmanaged_identity),
        super::super::session::intent::Record::Owned(super::super::session::intent::Recorded::UNMANAGED)
    );
    let bytes = std::fs::read(unmanaged_directory.join(super::super::session::intent::FILE)).expect("intent");
    let record: serde_json::Value = serde_json::from_slice(&bytes).expect("recorded intent");
    assert_eq!(record["admission"], "unmanaged");
    assert_eq!(record["schema"], super::super::session::intent::SCHEMA);
    let (definition, revision) = other.prepared(&unmanaged, &unmanaged_path);
    other.promoted(&definition, &revision);
    other.attempt("recover", &unmanaged_path, false).await.expect("recorded unmanaged needs no store");
    assert!(other.lifecycle(&definition).sessions.is_empty());
}

#[tokio::test]
async fn recovery_of_a_managed_attempt_requires_its_store_and_never_re_derives_the_mode() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-managed-recovery", 7);
    let (path, identity) = fixture.write("request.json", &request);
    let (definition, revision) = fixture.prepared(&request, &path);
    fixture.promoted(&definition, &revision);
    fixture.attempt("run", &path, true).await.expect("managed run");
    let directory = fixture.directory(&request);
    let intent = std::fs::read(directory.join(super::super::session::intent::FILE)).expect("intent");
    let error = fixture.attempt("recover", &path, false).await.expect_err("managed recovery without its store");
    assert!(error.contains("--revision-store"), "{error}");
    assert!(
        fixture
            .inventory(&path, false)
            .expect_err("managed inspection without its store")
            .contains("--revision-store")
    );
    fixture.attempt("recover", &path, true).await.expect("recovery with the store");
    fixture.attempt("run", &path, true).await.expect("a duplicate run reconciles");
    assert_eq!(std::fs::read(directory.join(super::super::session::intent::FILE)).expect("intent"), intent);
    assert!(matches!(
        recorded(&directory, &request, &identity),
        super::super::session::intent::Record::Owned(super::super::session::intent::Recorded {
            admission: crate::agent_revision::seam::Admission::Managed,
            pin: Some(_),
        })
    ));
}

/// Answers the provider allocation and, on revalidation (the call right
/// before spawn), revokes the attempt's session.
struct RevokingPort<'a> {
    fixture: &'a Fixture,
    definition: String,
    session: String,
    replies: crate::execution_binding::FixtureComputePort,
    calls: u32,
}

impl crate::execution_binding::ComputeAllocationPort for RevokingPort<'_> {
    fn observe_allocation(
        &mut self,
        query: &crate::execution_binding::AllocationQuery,
    ) -> Result<crate::execution_binding::AllocationReply, crate::execution_binding::ComputePortIssue> {
        self.calls += 1;
        if self.calls == 2 {
            self.fixture
                .revision(&["revoke", "--definition", &self.definition, "--session", &self.session])
                .expect("revoke the admitted session");
        }
        self.replies.observe_allocation(query)
    }
}

#[tokio::test]
async fn revocation_after_admission_blocks_the_attempt_before_spawn() {
    let fixture = Fixture::new();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/execution-binding/observations/positive/agent-a-1-active.json");
    let observation =
        lattice_core::execution_binding::validation::decode_observation(&std::fs::read(path).expect("fixture"))
            .expect("valid fixture");
    let mut request = super::request(
        &observation.allocation.attempt_id,
        lattice_core::execution_binding::request::ComputeRequirement::ProviderExecution {
            direct_accelerator: None,
        },
        lattice_core::execution_binding::request::ComputeDeclaration::ProviderAllocation {
            allocation_id: observation.allocation.id.clone(),
            host_id: observation.allocation.host_id.clone(),
            consumer_id: observation.allocation.consumer_id.clone(),
            owner_generation: observation.allocation.owner_generation,
        },
    );
    request.workspace_root = fixture.root.join("workspace");
    let request = super::with_capability(request, capability());
    let (request_path, identity) = fixture.write("request.json", &request);
    let (definition, revision) = fixture.prepared(&request, &request_path);
    fixture.promoted(&definition, &revision);
    let mut later = observation.clone();
    later.port.observation_sequence = 2;
    later.offer = None;
    let reply = |observed| Ok(crate::execution_binding::AllocationReply::Observed(Box::new(observed)));
    let mut port = RevokingPort {
        fixture: &fixture,
        definition: definition.clone(),
        session: super::super::session::revision::identifier(&request),
        replies: crate::execution_binding::FixtureComputePort::new(vec![reply(observation), reply(later)]),
        calls: 0,
    };
    let blocked = super::super::execution::run(
        &request_path,
        &fixture.state,
        Some(super::super::Ports {
            compute: &mut port,
            serving: Box::new(crate::execution_binding::serving::UnavailableOwner),
        }),
        super::super::session::revision::Store(Some(&fixture.store)),
    )
    .await;
    assert_eq!(blocked, Err(super::super::refusal::Reason::RevisionBlocked.message().to_owned()));
    assert_eq!(port.calls, 2, "admitted, then revalidated right before spawn");
    let directory = fixture.directory(&request);
    let block = super::super::session::revision::retained_block(&directory)
        .expect("readable")
        .expect("recorded once");
    assert_eq!(block["refusal"]["code"], "session_revoked");
    for name in ["raw-events.jsonl", "omp-observation.json"] {
        assert!(!directory.join(name).exists(), "{name}: the worker never spawned");
    }
    assert!(matches!(
        recorded(&directory, &request, &identity),
        super::super::session::intent::Record::Owned(super::super::session::intent::Recorded {
            admission: crate::agent_revision::seam::Admission::Managed,
            ..
        })
    ));
    assert!(fixture.lifecycle(&definition).sessions.is_empty(), "the blocked session is closed");
    fixture.attempt("recover", &request_path, true).await.expect("recovery reports the block");
    let inventory = fixture.inventory(&request_path, true).expect("inventory");
    assert_eq!(inventory["instance"]["lifecycle"]["state"], "revision_blocked_before_dispatch");
    assert_eq!(inventory["instance"]["liveness"]["state"], "not_started");
    assert_eq!(inventory["instance"]["pinned"]["value"]["is_blocked"], true);
    let observation = super::super::recovery::revision_blocked(
        &request,
        &directory,
        &super::super::binding::load(&directory, &request).expect("binding").expect("persisted"),
        &block,
    )
    .expect("a definite non-dispatch");
    assert_eq!(observation["status"], "not_dispatched_revision_blocked");
    assert_eq!(observation["revision_refusal"]["code"], "session_revoked");
}

/// Writes the managed-lifecycle v3 captures Kiln uses as fixtures, only when
/// `LATTICE_INVENTORY_CAPTURE_DIR` names an existing directory: desired,
/// prepared through the fixture port, promoted with an Animus #18 fixture
/// attachment, a live admitted session, a terminal attempt whose pin was
/// released, the same after revocation, and the profile after a refused
/// attempt.
// r[verify lattice.worker_inventory.captures]
// r[verify lattice.worker_inventory.captures.scenario.fixture]
#[tokio::test]
async fn capture_managed_worker_inventory_v3() {
    let Some(dir) = std::env::var_os("LATTICE_INVENTORY_CAPTURE_DIR") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let write = |name: &str, value: &serde_json::Value| {
        std::fs::write(dir.join(name), serde_json::to_vec_pretty(value).expect("json")).expect("write capture");
    };
    let fixture = Fixture::new();
    let run = fixture.request("attempt-managed-run", 7);
    let (run_path, _) = fixture.write("run.json", &run);
    let admitted = fixture.request("attempt-managed-admitted", 8);
    let (admitted_path, _) = fixture.write("admitted.json", &admitted);
    let resolved = crate::agent_revision::bundle::resolve(&run).expect("resolves");
    let (definition, revision) = (resolved.definition.identity, resolved.revision);
    fixture.revision(&["desire", "--request", Fixture::text(&run_path)]).expect("desire");
    write("managed-1-desired.json", &fixture.inventory(&run_path, true).expect("inventory"));
    fixture
        .revision(&["prepare", "--definition", &definition, "--revision", &revision])
        .expect("prepare");
    write("managed-2-prepared.json", &fixture.inventory(&run_path, true).expect("inventory"));
    fixture.promoted(&definition, &revision);
    write("managed-3-promoted.json", &fixture.inventory(&run_path, true).expect("inventory"));

    // A session admitted and bound, before its intent (a controller lost after binding).
    let session = super::super::session::revision::identifier(&admitted);
    fixture
        .revision(&[
            "admit-session",
            "--request",
            Fixture::text(&admitted_path),
            "--session",
            &session,
        ])
        .expect("admit the session");
    let directory = fixture.directory(&admitted);
    super::super::artifacts::private_directory(&fixture.state).expect("state");
    super::super::artifacts::private_directory(&directory).expect("attempt directory");
    super::super::binding::admit_and_persist(
        &admitted,
        &directory,
        &mut crate::execution_binding::UnavailableComputePort,
    )
    .expect("host-local binding");
    write("managed-4-admitted-session-live.json", &fixture.inventory(&admitted_path, true).expect("inventory"));

    fixture.attempt("run", &run_path, true).await.expect("managed run");
    write("managed-5-terminal-pin-released.json", &fixture.inventory(&run_path, true).expect("inventory"));

    fixture.revision(&["revoke", "--definition", &definition, "--revision", &revision]).expect("revoke");
    write("managed-6-revoked-live-session.json", &fixture.inventory(&admitted_path, true).expect("inventory"));
    write("managed-6-revoked-terminal.json", &fixture.inventory(&run_path, true).expect("inventory"));
    let refused = fixture.request("attempt-managed-refused", 9);
    let (refused_path, _) = fixture.write("refused.json", &refused);
    assert!(fixture.attempt("run", &refused_path, true).await.is_err(), "refused after revocation");
    write(
        "managed-7-refused-after-revocation.json",
        &fixture.inventory(&refused_path, true).expect("inventory"),
    );
}
