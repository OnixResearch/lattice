//! Animus #18 qualification attachments through Lattice's attachment port and
//! the `agent-revision` CLI shell. Attachments are encoded with Animus's own
//! `encode_attachment` for the subject Lattice resolves from a request.

use animus_runtime::qualification as animus;
use lattice::agent_revision::Command;
use lattice::agent_revision::CommandArgs;
use lattice::agent_revision::FixturePreparation;
use lattice::agent_revision::FixtureQualification;
use lattice::agent_revision::Ports;
use lattice::agent_revision::SelectArgs;
use lattice::agent_revision::seam;
use lattice::implementation_attempt::cli::Request;
use lattice::store::Store;
use lattice_core::agent_revision::Refusal;
use lattice_core::agent_revision::ports as verdict;

const FAMILY: &str = "rust-implementation";
const OBSERVED_AT_MS: u64 = 1_700_000_000_000;
const VALID_UNTIL_MS: u64 = 4_000_000_000_000;
const PLANNED: u64 = 3;
const ANIMUS_REVISION: &str = "8d6f67d38509899402df4d62e2d54321ff047f06";
const ANIMUS_ARCHIVE_BLAKE3: &str = "d0c2389c12cc273b6f4ebe11a15d3e18b69f9b169b4f80a41c412aaa5bac191e";

fn request(prompt: &str) -> Request {
    let envelope = lattice::implementation_attempt::ImplementationAttemptEnvelope {
        schema: lattice::implementation_attempt::IMPLEMENTATION_ATTEMPT_ENVELOPE_SCHEMA.into(),
        work_id: "work-18".into(),
        work_version: 1,
        source_kind: lattice::implementation_attempt::SourceKind::CairnTask,
        source_id: "task-18".into(),
        repository: "OnixResearch/lattice".into(),
        base: "a".repeat(40),
        readiness_snapshot: "b".repeat(64),
        task_spec_digest: "c".repeat(64),
        verification_profile: "f".repeat(64),
        attempt_id: "attempt-18".into(),
        fence: 7,
        workspace_id: "workspace-18".into(),
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

fn reference(label: &str) -> String {
    format!("blake3:{}", blake3::hash(label.as_bytes()).to_hex())
}

fn bound(kind: animus::ComponentKind) -> String {
    animus::component_identity(kind, b"v1")
}

/// A complete `task_family_evaluation` attachment document for the subject.
fn document(definition: &str, revision: &str) -> animus::AttachmentDocument {
    let components = animus::ComponentKind::ALL
        .into_iter()
        .map(|kind| animus::Component {
            kind,
            basis: animus::ComponentBasis::Observed,
            identity: Some(bound(kind)),
        })
        .collect();
    animus::AttachmentDocument {
        subject: animus::QualificationSubject {
            lattice_definition_identity: definition.into(),
            lattice_profile_revision: revision.into(),
            task_family: FAMILY.into(),
            components,
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
            planned: PLANNED,
            accepted: 2,
            rejected: 1,
            failed: 0,
            cancelled: 0,
            incomplete: 0,
        }),
        measures: Some(animus::Measures {
            repetitions: 1,
            tasks_covered: PLANNED,
            tasks_declared: PLANNED,
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

fn encode(document: &animus::AttachmentDocument) -> animus::Attachment {
    animus::encode_attachment(document).expect("valid attachment document")
}

/// One managed definition whose revision is prepared and not yet selected.
struct Prepared {
    root: tempfile::TempDir,
    store: std::path::PathBuf,
    base: Request,
    definition: String,
    revision: String,
}

impl Prepared {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("scratch");
        let store = root.path().join("lattice.redb");
        let base = request("make the candidate");
        let resolved = lattice::agent_revision::bundle::resolve(&base).expect("resolves");
        let prepared = Self {
            root,
            store,
            base,
            definition: resolved.definition.identity,
            revision: resolved.revision,
        };
        prepared.prepare(&prepared.base.clone(), "request.json");
        prepared
    }

    /// Desire and prepare `request` through a fixture preparation port.
    fn prepare(&self, request: &Request, name: &str) -> String {
        let path = self.write(name, &serde_json::to_vec(request).expect("request encodes"));
        let revision = lattice::agent_revision::bundle::resolve(request).expect("resolves").revision;
        let observation = verdict::Prepared::Prepared {
            artifacts: vec![reference(name)],
        };
        let mut preparation = FixturePreparation::new(vec![observation], Vec::new());
        let mut unused = FixtureQualification::new(Vec::new());
        let mut run = |command| {
            lattice::agent_revision::run_command_with_ports(&CommandArgs { command }, Ports {
                preparation: &mut preparation,
                qualification: &mut unused,
            })
        };
        run(Command::Desire {
            request: path,
            store: self.store.clone(),
        })
        .expect("desire");
        run(Command::Prepare {
            store: self.store.clone(),
            definition: self.definition.clone(),
            revision: revision.clone(),
        })
        .expect("prepare");
        assert!(unused.queries().is_empty());
        revision
    }

    fn write(&self, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = self.root.path().join(name);
        std::fs::write(&path, bytes).expect("file written");
        path
    }

    fn attach(&self, name: &str, attachment: &animus::Attachment) -> std::path::PathBuf {
        self.write(name, &attachment.canonical)
    }

    fn args(&self, qualification_ref: &str, attachments: Vec<std::path::PathBuf>) -> SelectArgs {
        SelectArgs {
            store: self.store.clone(),
            definition: self.definition.clone(),
            revision: self.revision.clone(),
            qualification_ref: qualification_ref.into(),
            task_family: FAMILY.into(),
            required_class: "task_family_evaluation".into(),
            expected_generation: 0,
            request_id: "promote-1".into(),
            qualification_attachments: attachments,
        }
    }

    fn subject(&self) -> verdict::Subject {
        verdict::Subject {
            definition_identity: self.definition.clone(),
            profile_revision: self.revision.clone(),
            task_family: FAMILY.into(),
        }
    }

    /// Promote through an injected port that answers this exact query with a
    /// well-formed `qualified` attachment for `subject`, as a faulty port
    /// could. Returns the result and how many queries the port received.
    fn answered(&self, subject: verdict::Subject) -> (Result<(), String>, usize) {
        let named = reference("injected-attachment");
        let query = verdict::Query {
            definition_identity: self.definition.clone(),
            profile_revision: self.revision.clone(),
            task_family: FAMILY.into(),
            required_class: verdict::EvidenceClass::TaskFamilyEvaluation,
            now_ms: 0,
        };
        let answer = verdict::Verdict::Qualified(verdict::Attachment {
            qualification_ref: named.clone(),
            subject,
            evidence_class: verdict::EvidenceClass::TaskFamilyEvaluation,
            protocol_identity: reference("protocol"),
            verifier_identity: reference("verifier"),
            observed_at_ms: OBSERVED_AT_MS,
            provenance: verdict::Provenance::Fixture,
        });
        let mut qualification = FixtureQualification::new(vec![(query, answer)]);
        let mut preparation = FixturePreparation::new(Vec::new(), Vec::new());
        let command = Command::Promote(self.args(&named, Vec::new()));
        let result = lattice::agent_revision::run_command_with_ports(&CommandArgs { command }, Ports {
            preparation: &mut preparation,
            qualification: &mut qualification,
        });
        (result, qualification.queries().len())
    }

    fn lifecycle(&self) -> lattice_core::agent_revision::state::Lifecycle {
        Store::new(&self.store)
            .expect("store")
            .load_lifecycle(&self.definition)
            .expect("load")
            .expect("managed")
            .1
    }

    /// Promote through the standalone CLI shell and require a refusal that
    /// leaves the lifecycle unchanged.
    fn refused(&self, args: SelectArgs) -> String {
        let before = self.lifecycle();
        let refused = standalone(Command::Promote(args)).expect_err("promotion refused");
        assert_eq!(self.lifecycle(), before, "a refusal changed the lifecycle");
        assert!(before.selected.is_none());
        refused
    }

    /// Require promotion to be refused as `not_qualified` with `reason`.
    fn not_qualified(&self, attachments: &[animus::Attachment], reason: &str) {
        let paths = attachments
            .iter()
            .enumerate()
            .map(|(index, attachment)| self.attach(&format!("attachment-{index}.json"), attachment))
            .collect();
        let refused = self.refused(self.args(&attachments[0].identity, paths));
        assert!(refused.contains("not_qualified"), "{refused}");
        assert!(refused.contains(&format!("\"reason\":\"{reason}\"")), "{refused}");
    }
}

fn standalone(command: Command) -> Result<(), String> {
    lattice::agent_revision::run_command(&CommandArgs { command })
}

/// Run the `lattice` binary and return its exit code and printed document.
fn cli(arguments: &[&std::ffi::OsStr]) -> (Option<i32>, serde_json::Value) {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("agent-revision")
        .args(arguments)
        .output()
        .expect("lattice runs");
    let document = serde_json::from_slice(&output.stdout).expect("one JSON document");
    (output.status.code(), document)
}

fn os(value: &str) -> &std::ffi::OsStr {
    std::ffi::OsStr::new(value)
}

#[test]
// r[verify lattice.agent_revisions.attachments]
// r[verify lattice.agent_revisions.attachments.scenario.qualified]
// r[verify lattice.agent_revisions.attachments.scenario.revoked]
fn a_qualified_attachment_promotes_through_the_cli_and_revocation_blocks_new_sessions() {
    let prepared = Prepared::new();
    let attachment = encode(&document(&prepared.definition, &prepared.revision));
    let mut written = attachment.canonical.clone();
    written.push(b'\n');
    let path = prepared.write("qualification.json", &written);
    let store = prepared.store.as_os_str();
    let promote = [
        os("promote"),
        os("--store"),
        store,
        os("--definition"),
        os(&prepared.definition),
        os("--revision"),
        os(&prepared.revision),
        os("--qualification-ref"),
        os(&attachment.identity),
        os("--task-family"),
        os(FAMILY),
        os("--required-class"),
        os("task_family_evaluation"),
        os("--expected-generation"),
        os("0"),
        os("--request-id"),
        os("promote-1"),
        os("--qualification-attachment"),
        path.as_os_str(),
    ];
    let (code, document) = cli(&promote);
    assert_eq!(code, Some(0), "{document}");
    assert_eq!(document["command"], "promote");
    assert_eq!(document["outcome"]["outcome"], "applied", "{document}");
    let lifecycle = prepared.lifecycle();
    let selected = lifecycle.selected.as_ref().expect("selected");
    assert_eq!(selected.revision, prepared.revision);
    assert_eq!(selected.qualification_ref, attachment.identity);
    assert_eq!(lifecycle.qualifications.len(), 1);
    assert_eq!(lifecycle.qualifications[0].attachment, verdict::Attachment {
        qualification_ref: attachment.identity.clone(),
        subject: prepared.subject(),
        evidence_class: verdict::EvidenceClass::TaskFamilyEvaluation,
        protocol_identity: bound(animus::ComponentKind::TaskProtocol),
        verifier_identity: bound(animus::ComponentKind::Verifier),
        observed_at_ms: OBSERVED_AT_MS,
        provenance: verdict::Provenance::Fixture,
    });
    let (code, document) = cli(&promote);
    assert_eq!(code, Some(0), "{document}");
    assert_eq!(document["outcome"]["outcome"], "already_applied", "{document}");

    let store = Store::new(&prepared.store).expect("store");
    let admitted = seam::admit(&store, &prepared.base, "session-a").expect("seam");
    assert!(matches!(admitted, seam::Decision::Admitted { .. }), "{admitted:?}");
    let admission = admitted.admission().expect("an admitted session records its admission mode");
    drop(store);
    let revoke = lattice::agent_revision::RevokeArgs {
        store: prepared.store.clone(),
        definition: prepared.definition.clone(),
        revision: None,
        qualification_ref: Some(attachment.identity.clone()),
        session: None,
    };
    standalone(Command::Revoke(revoke)).expect("revoke");
    let store = Store::new(&prepared.store).expect("store");
    assert_eq!(seam::admit(&store, &prepared.base, "session-b").expect("seam"), seam::Decision::Refused {
        refusal: Refusal::QualificationRevoked
    });
    assert_eq!(seam::effect(&store, &prepared.base, "session-a", admission).expect("seam"), seam::Effect::Blocked {
        refusal: Refusal::QualificationRevoked
    });
}

#[test]
// r[verify lattice.agent_revisions.attachments.cli]
// r[verify lattice.agent_revisions.attachments.scenario.rollback]
fn rollback_requires_the_attachment_for_the_earlier_revision() {
    let mut prepared = Prepared::new();
    let first = encode(&document(&prepared.definition, &prepared.revision));
    let first_path = prepared.attach("first.json", &first);
    standalone(Command::Promote(prepared.args(&first.identity, vec![first_path.clone()]))).expect("promote first");
    let first_revision = prepared.revision.clone();
    let second_revision = prepared.prepare(&request("another prompt"), "second-request.json");
    let second = encode(&document(&prepared.definition, &second_revision));
    prepared.revision.clone_from(&second_revision);
    let mut promote = prepared.args(&second.identity, vec![prepared.attach("second.json", &second)]);
    promote.expected_generation = 1;
    promote.request_id = "promote-2".into();
    standalone(Command::Promote(promote)).expect("promote second");

    prepared.revision = first_revision.clone();
    let mut rollback = prepared.args(&first.identity, Vec::new());
    rollback.expected_generation = 2;
    rollback.request_id = "rollback-1".into();
    let refused = standalone(Command::Rollback(rollback)).expect_err("no attachment");
    assert!(refused.contains("qualification_unavailable"), "{refused}");
    let mut rollback = prepared.args(&first.identity, vec![prepared.attach("second-for-first.json", &second)]);
    rollback.expected_generation = 2;
    rollback.request_id = "rollback-1".into();
    let refused = standalone(Command::Rollback(rollback)).expect_err("attachment for another revision");
    assert!(refused.contains("\"reason\":\"subject_mismatch\""), "{refused}");
    let mut rollback = prepared.args(&first.identity, vec![first_path]);
    rollback.expected_generation = 2;
    rollback.request_id = "rollback-1".into();
    standalone(Command::Rollback(rollback)).expect("rollback");
    let lifecycle = prepared.lifecycle();
    let selected = lifecycle.selected.expect("selected");
    assert_eq!(selected.revision, first_revision);
    assert_eq!(selected.qualification_ref, first.identity);
    assert_eq!(lifecycle.rollout_generation, 3);
}

#[test]
// r[verify lattice.agent_revisions.attachments.scenario.unavailable]
fn without_attachments_promotion_stays_unavailable() {
    let prepared = Prepared::new();
    let refused = prepared.refused(prepared.args(&reference("named"), Vec::new()));
    assert!(refused.contains("qualification_unavailable"), "{refused}");
    assert!(refused.contains("animus#18"), "{refused}");
}

#[test]
// r[verify lattice.agent_revisions.attachments.scenario.reasons]
fn a_wrong_definition_or_revision_is_subject_mismatch() {
    let prepared = Prepared::new();
    let other_revision =
        lattice::agent_revision::bundle::resolve(&request("another prompt")).expect("resolves").revision;
    assert_ne!(other_revision, prepared.revision);
    prepared.not_qualified(&[encode(&document(&prepared.definition, &other_revision))], "subject_mismatch");
    prepared
        .not_qualified(&[encode(&document(&reference("another-definition"), &prepared.revision))], "subject_mismatch");
}

#[test]
// r[verify lattice.agent_revisions.attachments.scenario.reasons]
fn another_task_family_is_scope_mismatch() {
    let prepared = Prepared::new();
    let mut scoped = document(&prepared.definition, &prepared.revision);
    scoped.subject.task_family = "release-notes".into();
    prepared.not_qualified(&[encode(&scoped)], "scope_mismatch");
}

#[test]
// r[verify lattice.agent_revisions.attachments.scenario.reasons]
fn another_required_class_is_insufficient_class() {
    let prepared = Prepared::new();
    let attachment = encode(&document(&prepared.definition, &prepared.revision));
    let mut args = prepared.args(&attachment.identity, vec![prepared.attach("attachment.json", &attachment)]);
    args.required_class = "contract_conformance".into();
    let refused = prepared.refused(args);
    assert!(refused.contains("not_qualified"), "{refused}");
    assert!(refused.contains("\"reason\":\"insufficient_class\""), "{refused}");
}

#[test]
// r[verify lattice.agent_revisions.attachments.scenario.reasons]
fn expired_evidence_is_stale() {
    let prepared = Prepared::new();
    let mut expired = document(&prepared.definition, &prepared.revision);
    expired.provenance.valid_until_ms = OBSERVED_AT_MS + 1;
    prepared.not_qualified(&[encode(&expired)], "stale");
}

#[test]
// r[verify lattice.agent_revisions.attachments.scenario.reasons]
fn evidence_observed_after_the_shell_clock_is_unusable() {
    let prepared = Prepared::new();
    let mut future = document(&prepared.definition, &prepared.revision);
    future.provenance.observed_at_ms = VALID_UNTIL_MS - 1;
    prepared.not_qualified(&[encode(&future)], "unusable");
}

#[test]
// r[verify lattice.agent_revisions.attachments.scenario.reasons]
fn distinct_attachments_from_one_source_are_conflicting() {
    let prepared = Prepared::new();
    let first = document(&prepared.definition, &prepared.revision);
    let mut second = first.clone();
    second.provenance.observed_at_ms = OBSERVED_AT_MS + 1;
    let (first, second) = (encode(&first), encode(&second));
    assert_ne!(first.identity, second.identity);
    prepared.not_qualified(&[first, second], "conflicting");
}

#[test]
// r[verify lattice.agent_revisions.attachments.scenario.reasons]
fn a_non_canonical_attachment_is_malformed() {
    let prepared = Prepared::new();
    let attachment = encode(&document(&prepared.definition, &prepared.revision));
    let mut spaced = b"{ ".to_vec();
    spaced.extend_from_slice(&attachment.canonical[1..]);
    let path = prepared.write("spaced.json", &spaced);
    let refused = prepared.refused(prepared.args(&attachment.identity, vec![path]));
    assert!(refused.contains("\"reason\":\"malformed\""), "{refused}");
}

#[test]
// r[verify lattice.agent_revisions.attachments.scenario.named_reference]
fn the_named_reference_must_equal_the_answer() {
    let prepared = Prepared::new();
    let attachment = encode(&document(&prepared.definition, &prepared.revision));
    let path = prepared.attach("attachment.json", &attachment);
    let refused = prepared.refused(prepared.args(&reference("another-attachment"), vec![path]));
    assert!(refused.contains("qualification_mismatch"), "{refused}");
}

#[test]
// r[verify lattice.agent_revisions.attachments.scenario.files]
fn oversized_symlinked_and_missing_files_are_refused() {
    let prepared = Prepared::new();
    let attachment = encode(&document(&prepared.definition, &prepared.revision));
    let valid = prepared.attach("attachment.json", &attachment);
    let oversized = prepared.write("oversized.json", &vec![b'x'; animus::MAX_ATTACHMENT_BYTES + 1]);
    let linked = prepared.root.path().join("linked.json");
    std::os::unix::fs::symlink(&valid, &linked).expect("symlink");
    let missing = prepared.root.path().join("missing.json");
    for (path, message) in [
        (oversized, "exceeds"),
        (linked, "not a regular file"),
        (missing, "is unavailable"),
        (prepared.root.path().to_path_buf(), "not a regular file"),
    ] {
        let refused = prepared.refused(prepared.args(&attachment.identity, vec![valid.clone(), path]));
        assert!(refused.contains(message), "{refused}");
        assert!(!refused.contains("refusal"), "{refused}");
    }
}

#[test]
// r[verify lattice.agent_revisions.attachments.vendor]
// r[verify lattice.agent_revisions.attachments.vendor.scenario.recorded]
fn the_vendored_producer_is_the_adopted_revision() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(root.join("vendor-cargo-deps/animus/source.ncl")).expect("snapshot metadata");
    assert!(source.contains(&format!("revision = \"{ANIMUS_REVISION}\"")), "{source}");
    assert!(source.contains(&format!("source_archive_blake3 = \"{ANIMUS_ARCHIVE_BLAKE3}\"")), "{source}");
    let script = std::fs::read_to_string(root.join("scripts/sync_animus.rs")).expect("sync script");
    assert!(script.contains(&format!("EXPECTED_REVISION: &str = \"{ANIMUS_REVISION}\"")));
    assert!(script.contains(&format!("SOURCE_ARCHIVE_BLAKE3: &str = \"{ANIMUS_ARCHIVE_BLAKE3}\"")));
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("manifest");
    assert!(manifest.contains(
        r#"animus-runtime = { path = "vendor-cargo-deps/animus/crates/animus-runtime", default-features = false, features = ["steel"] }"#
    ));
}

#[test]
// r[verify lattice.agent_revisions.attachments.subject]
// r[verify lattice.agent_revisions.attachments.subject.scenario.bound]
fn a_qualified_answer_for_the_exact_subject_promotes_through_an_injected_port() {
    let prepared = Prepared::new();
    let (result, queries) = prepared.answered(prepared.subject());
    result.expect("promotion");
    assert_eq!(queries, 1);
    let lifecycle = prepared.lifecycle();
    assert_eq!(lifecycle.selected.expect("selected").revision, prepared.revision);
    assert_eq!(lifecycle.qualifications[0].attachment.subject, prepared.subject());
}

#[test]
// r[verify lattice.agent_revisions.attachments.subject]
// r[verify lattice.agent_revisions.attachments.subject.scenario.foreign]
fn a_qualified_answer_for_another_subject_is_refused() {
    let prepared = Prepared::new();
    let other_revision = "9".repeat(64);
    assert_ne!(other_revision, prepared.revision);
    let cases = [
        (
            verdict::Subject {
                definition_identity: reference("another-definition"),
                ..prepared.subject()
            },
            "subject_mismatch",
        ),
        (
            verdict::Subject {
                profile_revision: other_revision,
                ..prepared.subject()
            },
            "subject_mismatch",
        ),
        (
            verdict::Subject {
                task_family: "release-notes".into(),
                ..prepared.subject()
            },
            "scope_mismatch",
        ),
    ];
    for (subject, reason) in cases {
        let before = prepared.lifecycle();
        let (result, queries) = prepared.answered(subject);
        let refused = result.expect_err("foreign subject refused");
        assert_eq!(queries, 1);
        assert!(refused.contains("not_qualified"), "{refused}");
        assert!(refused.contains(&format!("\"reason\":\"{reason}\"")), "{refused}");
        assert_eq!(prepared.lifecycle(), before);
    }
}

#[test]
// r[verify lattice.agent_revisions.attachments.scenario.provenance]
fn the_provenance_label_is_recorded_and_does_not_gate_promotion() {
    let prepared = Prepared::new();
    let mut live = document(&prepared.definition, &prepared.revision);
    live.provenance.label = animus::EvidenceLabel::Live;
    let attachment = encode(&live);
    let path = prepared.attach("live.json", &attachment);
    standalone(Command::Promote(prepared.args(&attachment.identity, vec![path]))).expect("promotion");
    let lifecycle = prepared.lifecycle();
    assert_eq!(lifecycle.qualifications[0].attachment.provenance, verdict::Provenance::Live);
    assert_eq!(lifecycle.selected.expect("selected").qualification_ref, attachment.identity);
}
