//! Q0: pure admission, publication, carry-forward and receipt rules, one test per refusal code.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use lattice_workflow_contract::*;

use super::*;

const IDENTITY_DIGITS: usize = 64;
const NODE_A: u64 = 1;
const NODE_B: u64 = 2;
const RUN_ID: u64 = 7;
const DEADLINE: u64 = 4_102_444_800_000;
const NOW: u64 = 1_800_000_000_000;
const BINARY: &str = "0000000000000000000000000000000000000000000000000000000000000001";
const OTHER_BINARY: &str = "0000000000000000000000000000000000000000000000000000000000000002";

fn identity(digit: char) -> Identity {
    Identity::new(format!("{BLAKE3_PREFIX}{}", digit.to_string().repeat(IDENTITY_DIGITS))).expect("test identity")
}

fn label(value: &str) -> Label {
    Label::new(value.to_string()).expect("test label")
}

fn contract_profile() -> CapabilityProfile {
    CapabilityProfile {
        schema: PROFILE_SCHEMA.to_string(),
        graph_schema: GRAPH_SCHEMA.to_string(),
        platforms: vec![label("linux-x86_64")],
        node_kinds: vec![label("build")],
        effect_classes: vec![EffectClass::Build],
        adapter_kinds: vec![label("local")],
        controls: ControlCapabilities {
            cancel: true,
            pause: false,
            resume: false,
        },
        limits: ContractLimits {
            maximum_nodes: 8,
            maximum_edges: 8,
            maximum_log_references: 8,
            maximum_log_read_bytes: 4_096,
            maximum_total_log_bytes: 16_384,
        },
        non_claims: lattice_workflow_contract::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    }
}

fn workflow_request() -> WorkflowRequest {
    let node = |id: &str, operation: char| WorkflowNode {
        id: label(id),
        kind: label("build"),
        effect_class: EffectClass::Build,
        adapter_kind: label("local"),
        operation_identity: identity(operation),
    };
    let mut graph = WorkflowGraph {
        schema: GRAPH_SCHEMA.to_string(),
        identity: identity('0'),
        nodes: vec![node("a", '1'), node("b", '2')],
        edges: vec![WorkflowEdge {
            source: label("a"),
            target: label("b"),
        }],
    };
    graph.identity = graph_identity(&graph);
    let mut request = WorkflowRequest {
        schema: REQUEST_SCHEMA.to_string(),
        identity: identity('0'),
        idempotency_key: identity('2'),
        profile_identity: profile_identity(&contract_profile()),
        platform: label("linux-x86_64"),
        kiln: KilnLinkage {
            effect_identity: identity('3'),
            attempt_identity: identity('4'),
            candidate_identity: identity('5'),
            pipeline_revision_identity: identity('6'),
            workflow_policy_identity: identity('7'),
        },
        source: None,
        graph,
        controls: RequestedControls {
            cancel: true,
            pause: false,
            resume: false,
        },
        observation: ObservationPolicy {
            maximum_log_references: 8,
            maximum_log_read_bytes: 4_096,
            maximum_total_log_bytes: 16_384,
        },
    };
    request.identity = lattice_workflow_contract::request_identity(&request);
    request
}

fn route(request: &WorkflowRequest) -> crate::workflow_exchange::WorkflowRoute {
    crate::workflow_exchange::WorkflowRoute {
        profile_identity: request.profile_identity.clone(),
        graph_identity: request.graph.identity.clone(),
        workflow_name: "checkpoint-test".to_string(),
        workflow_revision: identity('8'),
        effect_authority: crate::workflow_exchange::EffectAuthorityMode::RouteExact,
        source_binding: crate::workflow_exchange::SourceBindingMode::Unavailable,
        nodes: vec![
            crate::workflow_exchange::RouteNode {
                request_node_id: label("a"),
                operation_identity: identity('1'),
                workflow_node_id: NODE_A,
            },
            crate::workflow_exchange::RouteNode {
                request_node_id: label("b"),
                operation_identity: identity('2'),
                workflow_node_id: NODE_B,
            },
        ],
    }
}

fn record() -> crate::workflow_exchange::ExecutionRecord {
    let request = workflow_request();
    let route = route(&request);
    crate::workflow_exchange::initial_record(request, &route, identity('8'), identity('9'), RUN_ID)
        .expect("fixture record")
}

fn binding() -> CheckpointBinding {
    CheckpointBinding {
        schema: BINDING_SCHEMA.to_string(),
        vm_profile: identity('a'),
        checkpoint: CheckpointProfile::WorkflowNodeBoundaryV1,
        graph_identity: workflow_request().graph.identity,
        workflow_revision: identity('8'),
        root: std::path::PathBuf::from("/checkpoint"),
        workspace: None,
        binding: Some(identity('b')),
        environment: None,
        deadline_unix_ms: DEADLINE,
        non_claims: super::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    }
}

fn local() -> LocalRuntime {
    LocalRuntime {
        architecture: "x86_64".to_string(),
        runtime: CheckpointRuntime {
            lattice_revision: "lattice-test-runtime".to_string(),
            lattice_binary_blake3: BINARY.to_string(),
            environment: None,
        },
    }
}

fn outcome(node: u64, state: crate::workflow::gate::PersistedNodeState) -> crate::workflow::gate::PersistedNodeOutcome {
    let mut outputs = std::collections::BTreeMap::new();
    if state == crate::workflow::gate::PersistedNodeState::Completed {
        outputs.insert(
            crate::engine::workflow::PortId(node * 10),
            crate::engine::value::Value::from_string("out".to_string()),
        );
    }
    crate::workflow::gate::PersistedNodeOutcome {
        node_id: crate::engine::workflow::NodeId(node),
        state,
        input_hash: content_hex(&node.to_be_bytes()),
        outputs,
        error: None,
    }
}

struct Fixture {
    manifest: Manifest,
    bytes: Vec<u8>,
    marker: Option<CommitMarker>,
    request: RestartRequest,
    local: LocalRuntime,
    node_objects: BTreeMap<Identity, Vec<u8>>,
    workspace_objects: BTreeMap<String, Vec<u8>>,
    request_object: Option<Vec<u8>>,
    siblings: Vec<Manifest>,
    prior: Option<AdoptionRecord>,
    now: u64,
}

impl Fixture {
    fn new(completed: &[u64], uncertain: &[u64]) -> Self {
        let record = record();
        let mut node_objects = BTreeMap::new();
        let mut nodes = BTreeMap::new();
        for node in completed {
            let outcome = outcome(*node, crate::workflow::gate::PersistedNodeState::Completed);
            let (bytes, object) = node_outputs_object(*node, &outcome).expect("node object");
            node_objects.insert(object.clone(), bytes);
            nodes.insert(*node, completed_node(&record, &outcome, object).expect("completed node"));
        }
        let uncertain = uncertain.iter().copied().collect::<BTreeSet<_>>();
        let manifest = build_manifest(&ManifestInput {
            record: &record,
            binding: &binding(),
            producing_incarnation: 0,
            local: &local(),
            completed: &nodes,
            uncertain: &uncertain,
            workspace: &[],
            consumed: Consumed {
                model_requests: 3,
                cost_micros: 40,
                elapsed_ms: 1_000,
            },
            cancellation_requested: false,
        })
        .expect("manifest builds");
        let (bytes, checkpoint) = canonical(MANIFEST_SCHEMA, &manifest).expect("manifest identity");
        Self {
            request: RestartRequest {
                schema: RESTART_REQUEST_SCHEMA.to_string(),
                checkpoint: checkpoint.clone(),
                admission: "kiln-admission:test-1".to_string(),
                restarted_incarnation: 1,
                request: record.request.identity.clone(),
                execution: record.accepted.execution_identity.clone(),
                binding: Some(identity('b')),
            },
            marker: Some(CommitMarker { manifest: checkpoint }),
            manifest,
            bytes,
            local: local(),
            node_objects,
            workspace_objects: BTreeMap::new(),
            request_object: Some(serde_json::to_vec(&record.request).expect("request object")),
            siblings: Vec::new(),
            prior: None,
            now: NOW,
        }
    }

    fn reseal(&mut self, change: impl FnOnce(&mut Manifest)) {
        change(&mut self.manifest);
        let (bytes, checkpoint) = canonical(MANIFEST_SCHEMA, &self.manifest).expect("manifest identity");
        self.bytes = bytes;
        self.marker = Some(CommitMarker {
            manifest: checkpoint.clone(),
        });
        self.request.checkpoint = checkpoint;
    }

    fn admit(&self) -> Result<Admission, Refusal> {
        admit(&AdmissionInput {
            request: &self.request,
            marker: self.marker.as_ref(),
            manifest_bytes: &self.bytes,
            local: &self.local,
            node_objects: &self.node_objects,
            workspace_objects: &self.workspace_objects,
            request_object: self.request_object.as_deref(),
            siblings: &self.siblings,
            prior: self.prior.as_ref(),
            now_unix_ms: self.now,
        })
    }

    fn admitted(&self) -> AdmittedCheckpoint {
        match self.admit() {
            Ok(Admission::Fresh(admitted)) => *admitted,
            other => panic!("fixture must admit fresh, got {other:?}"),
        }
    }
}

fn refused(fixture: &Fixture) -> Refusal {
    fixture.admit().expect_err("fixture must be refused")
}

// r[verify lattice.application_checkpoint.admission]
#[test]
fn committed_manifest_admits_and_carries_budget_deadline_and_cancellation() {
    let fixture = Fixture::new(&[NODE_A], &[]);
    let admitted = fixture.admitted();
    assert_eq!(admitted.receipt.kind, RestartKind::ApplicationRestart);
    assert_eq!(admitted.receipt.restarted_incarnation, 1);
    assert_eq!(admitted.receipt.admission, "kiln-admission:test-1");
    assert_eq!(admitted.consumed, fixture.manifest.consumed);
    assert_eq!(admitted.deadline_unix_ms, DEADLINE);
    assert!(!admitted.cancellation_requested);
    assert_eq!(admitted.outcomes.len(), 1);
    assert!(admitted.uncertain.is_empty());
}

#[test]
fn identity_framing_is_length_prefixed_blake3() {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&4_u64.to_be_bytes());
    hasher.update(b"test");
    hasher.update(&2_u64.to_be_bytes());
    hasher.update(b"{}");
    let expected = format!("b3:{}", hasher.finalize().to_hex());
    assert_eq!(identity_of_bytes("test", b"{}").expect("identity").as_str(), expected);
}

#[test]
fn optional_manifest_fields_are_absent_when_unset() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.reseal(|manifest| manifest.binding = None);
    let text = String::from_utf8(fixture.bytes).expect("utf8");
    assert!(!text.contains("\"binding\""));
    assert!(!text.contains("\"environment\""));
}

#[test]
fn refuses_wrong_request_schema() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.request.schema = "lattice.application-restart-request.v2".to_string();
    assert_eq!(refused(&fixture), Refusal::Schema);
}

#[test]
fn refuses_wrong_manifest_schema() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.reseal(|manifest| manifest.schema = "lattice.application-checkpoint-manifest.v2".to_string());
    assert_eq!(refused(&fixture), Refusal::Schema);
}

#[test]
fn refuses_malformed_admission_reference() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.request.admission = String::new();
    assert_eq!(refused(&fixture), Refusal::AdmissionReference);
    fixture.request.admission = "has space".to_string();
    assert_eq!(refused(&fixture), Refusal::AdmissionReference);
}

// r[verify lattice.application_checkpoint.commit]
#[test]
fn refuses_manifest_without_commit_marker() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.marker = None;
    assert_eq!(refused(&fixture), Refusal::NotCommitted);
}

#[test]
fn refuses_marker_or_request_naming_another_manifest() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.request.checkpoint = identity('c');
    assert_eq!(refused(&fixture), Refusal::ManifestIdentity);
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.marker = Some(CommitMarker {
        manifest: identity('c'),
    });
    assert_eq!(refused(&fixture), Refusal::ManifestIdentity);
}

#[test]
fn refuses_non_canonical_manifest_bytes() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.bytes = serde_json::to_vec_pretty(&fixture.manifest).expect("pretty");
    let checkpoint = identity_of_bytes(MANIFEST_SCHEMA, &fixture.bytes).expect("identity");
    fixture.marker = Some(CommitMarker {
        manifest: checkpoint.clone(),
    });
    fixture.request.checkpoint = checkpoint;
    assert_eq!(refused(&fixture), Refusal::Encoding);
}

#[test]
fn refuses_wrong_runtime_revision() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.local.runtime.lattice_revision = "lattice-other-runtime".to_string();
    assert_eq!(refused(&fixture), Refusal::Runtime);
}

#[test]
fn portable_profile_admits_a_separately_built_binary_of_the_same_revision() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.local.runtime.lattice_binary_blake3 = OTHER_BINARY.to_string();
    assert!(matches!(fixture.admit(), Ok(Admission::Fresh(_))));
}

// r[verify lattice.application_checkpoint.portable]
#[test]
fn portable_profile_admits_separately_built_other_architecture() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.local.architecture = "aarch64".to_string();
    fixture.local.runtime.lattice_binary_blake3 = OTHER_BINARY.to_string();
    assert!(matches!(fixture.admit(), Ok(Admission::Fresh(_))));
}

#[test]
fn non_portable_profile_needs_same_architecture_and_binary() {
    let produced = local();
    let mut other_architecture = local();
    other_architecture.architecture = "aarch64".to_string();
    let mut other_binary = local();
    other_binary.runtime.lattice_binary_blake3 = OTHER_BINARY.to_string();
    let none = CheckpointProfile::None;
    let portable = CheckpointProfile::WorkflowNodeBoundaryV1;
    let check =
        |profile, local: &LocalRuntime| admit_runtime_facts(profile, &produced.architecture, &produced.runtime, local);
    assert_eq!(check(none, &other_architecture), Err(Refusal::Architecture));
    assert_eq!(check(none, &other_binary), Err(Refusal::Runtime));
    assert_eq!(check(none, &produced), Ok(()));
    assert_eq!(check(portable, &other_architecture), Ok(()));
    assert_eq!(check(portable, &other_binary), Ok(()));
}

#[test]
fn refuses_environment_mismatch() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.local.runtime.environment = Some("kiln-environment:other".to_string());
    assert_eq!(refused(&fixture), Refusal::Environment);
}

#[test]
fn refuses_crossed_request_execution_and_binding() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.request.request = identity('c');
    assert_eq!(refused(&fixture), Refusal::CrossedRequest);
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.request.execution = identity('c');
    assert_eq!(refused(&fixture), Refusal::CrossedExecution);
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.request.binding = Some(identity('c'));
    assert_eq!(refused(&fixture), Refusal::CrossedBinding);
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.request.binding = None;
    assert_eq!(refused(&fixture), Refusal::CrossedBinding);
}

#[test]
fn refuses_missing_or_tampered_objects() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.node_objects.clear();
    assert_eq!(refused(&fixture), Refusal::ObjectMissing);
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    for bytes in fixture.node_objects.values_mut() {
        bytes.push(b' ');
    }
    assert_eq!(refused(&fixture), Refusal::ObjectMismatch);
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.request_object = None;
    assert_eq!(refused(&fixture), Refusal::ObjectMissing);
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    let mut other = workflow_request();
    other.idempotency_key = identity('e');
    fixture.request_object = Some(serde_json::to_vec(&other).expect("request"));
    assert_eq!(refused(&fixture), Refusal::ObjectMismatch);
}

// r[verify lattice.application_checkpoint.workspace]
#[test]
fn refuses_missing_workspace_object_and_unsafe_paths() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    let content = b"workspace".to_vec();
    let digest = content_hex(&content);
    fixture.reseal(|manifest| {
        manifest.workspace_objects = vec![WorkspaceObject {
            path: "src/lib.rs".to_string(),
            blake3: digest.clone(),
            bytes: 9,
        }];
    });
    assert_eq!(refused(&fixture), Refusal::ObjectMissing);
    fixture.workspace_objects.insert(digest.clone(), content);
    assert!(matches!(fixture.admit(), Ok(Admission::Fresh(_))));
    for path in ["../escape", "/absolute", "a/./b", "a//b", ""] {
        let mut unsafe_path = Fixture::new(&[NODE_A], &[]);
        unsafe_path.reseal(|manifest| {
            manifest.workspace_objects = vec![WorkspaceObject {
                path: path.to_string(),
                blake3: digest.clone(),
                bytes: 9,
            }];
        });
        assert_eq!(refused(&unsafe_path), Refusal::WorkspacePath, "{path}");
    }
}

#[test]
fn refuses_duplicate_and_overlapping_nodes() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.reseal(|manifest| {
        let duplicate = manifest.completed_nodes[0].clone();
        manifest.completed_nodes.push(duplicate);
    });
    assert_eq!(refused(&fixture), Refusal::DuplicateNode);
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.reseal(|manifest| {
        manifest.uncertain_nodes.push(UncertainNode {
            workflow_node_id: NODE_A,
            adapter_identity: manifest.completed_nodes[0].adapter_identity.clone(),
            last_state: UncertainState::Running,
        });
    });
    assert_eq!(refused(&fixture), Refusal::DuplicateNode);
}

#[test]
fn refuses_node_limit() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.reseal(|manifest| {
        let template = manifest.completed_nodes[0].clone();
        manifest.completed_nodes = (0..=MAXIMUM_COMPLETED_NODES as u64)
            .map(|id| CompletedNode {
                workflow_node_id: id,
                ..template.clone()
            })
            .collect();
    });
    assert_eq!(refused(&fixture), Refusal::NodeLimit);
}

#[test]
fn refuses_stale_admission() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.request.restarted_incarnation = 0;
    assert_eq!(refused(&fixture), Refusal::StaleAdmission);
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    let mut newer = fixture.manifest.clone();
    newer.producing_incarnation = 1;
    fixture.siblings = vec![newer];
    assert_eq!(refused(&fixture), Refusal::StaleAdmission);
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    let mut prior = adoption_record(&fixture.admitted());
    prior.receipt.restarted_incarnation = 2;
    prior.receipt.admission = "kiln-admission:test-2".to_string();
    fixture.prior = Some(prior);
    assert_eq!(refused(&fixture), Refusal::StaleAdmission);
}

#[test]
fn identical_restart_is_idempotent_and_other_prior_is_not_fresh() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    let admitted = fixture.admitted();
    fixture.prior = Some(adoption_record(&admitted));
    assert!(matches!(fixture.admit(), Ok(Admission::Idempotent(receipt)) if receipt == admitted.receipt));
    let mut prior = adoption_record(&admitted);
    prior.receipt.restarted_incarnation = 1;
    prior.receipt.admission = "kiln-admission:older".to_string();
    fixture.request.restarted_incarnation = 2;
    fixture.prior = Some(prior);
    assert_eq!(refused(&fixture), Refusal::TargetNotFresh);
}

#[test]
fn refuses_expired_deadline() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    fixture.now = DEADLINE;
    assert_eq!(refused(&fixture), Refusal::DeadlineExpired);
}

// r[verify lattice.application_checkpoint.publish]
#[test]
fn refuses_sensitive_outputs_at_publication_and_admission() {
    let mut sensitive = outcome(NODE_A, crate::workflow::gate::PersistedNodeState::Completed);
    for value in sensitive.outputs.values_mut() {
        value.sensitive_fragments.push("secret".to_string());
    }
    let step = crate::workflow::gate::shell::checkpoint_to_results(std::slice::from_ref(&sensitive)).expect("steps");
    assert!(crate::workflow::gate::shell::checkpoint_from_results(&step).is_err());
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    let (bytes, object) = node_outputs_object(NODE_A, &sensitive).expect("object");
    fixture.node_objects = BTreeMap::from([(object.clone(), bytes)]);
    fixture.reseal(|manifest| manifest.completed_nodes[0].outputs_object = object);
    assert_eq!(refused(&fixture), Refusal::SensitiveOutput);
}

// r[verify lattice.application_checkpoint.carry_forward]
#[test]
fn older_checkpoint_keeps_newer_budget_cancellation_and_uncertainty() {
    let mut older = Fixture::new(&[NODE_A], &[]);
    let mut newer = Fixture::new(&[NODE_A], &[NODE_B]);
    newer.reseal(|manifest| {
        manifest.consumed.model_requests = 9;
        manifest.consumed.elapsed_ms = 5_000;
        manifest.cancellation_requested = true;
    });
    older.siblings = vec![newer.manifest.clone()];
    let admitted = older.admitted();
    assert_eq!(admitted.consumed.model_requests, 9);
    assert_eq!(admitted.consumed.cost_micros, 40);
    assert_eq!(admitted.consumed.elapsed_ms, 5_000);
    assert!(admitted.cancellation_requested);
    assert_eq!(admitted.deadline_unix_ms, DEADLINE);
    assert_eq!(admitted.uncertain.iter().map(|node| node.workflow_node_id).collect::<Vec<_>>(), vec![NODE_B]);
}

#[test]
fn refuses_sibling_with_different_deadline() {
    let mut fixture = Fixture::new(&[NODE_A], &[]);
    let mut sibling = fixture.manifest.clone();
    sibling.deadline_unix_ms = DEADLINE - 1;
    fixture.siblings = vec![sibling];
    assert_eq!(refused(&fixture), Refusal::CarryForward);
}

/// Deliberately broken restore variants must each be caught by the independent oracle.
#[test]
fn carry_forward_oracle_catches_dropped_cancellation_reset_budget_and_hidden_uncertainty() {
    let mut fixture = Fixture::new(&[NODE_A], &[NODE_B]);
    fixture.reseal(|manifest| manifest.cancellation_requested = true);
    let admitted = fixture.admitted();
    let fresh = record();
    let (completed, uncertain) = super::core::adopted_nodes(&admitted);
    let adopted = crate::workflow_exchange::adopt_checkpoint(&fresh, &crate::workflow_exchange::CheckpointAdoption {
        accepted: &admitted.manifest.execution.accepted,
        sequence: admitted.manifest.execution.sequence,
        cancellation_requested: admitted.cancellation_requested,
        completed: &completed,
        uncertain: &uncertain,
    })
    .expect("adoption");
    let adoption = adoption_record(&admitted);
    assert_eq!(verify_carry_forward(&admitted, &adoption, &adopted), Ok(()));
    assert_eq!(adopted.state, crate::workflow_exchange::DurableExecutionState::Unknown);

    let mut dropped = adopted.clone();
    dropped.cancellation_requested = false;
    assert_eq!(verify_carry_forward(&admitted, &adoption, &dropped), Err(Refusal::CarryForward));
    let mut reset = adoption.clone();
    reset.consumed = Consumed::default();
    assert_eq!(verify_carry_forward(&admitted, &reset, &adopted), Err(Refusal::CarryForward));
    let mut hidden = adopted.clone();
    hidden.state = crate::workflow_exchange::DurableExecutionState::Queued;
    assert_eq!(verify_carry_forward(&admitted, &adoption, &hidden), Err(Refusal::CarryForward));
    let mut replayed = adopted;
    replayed.nodes[1].state = NodeState::Pending;
    assert_eq!(verify_carry_forward(&admitted, &adoption, &replayed), Err(Refusal::CarryForward));
}

// r[verify lattice.application_checkpoint.restore]
#[test]
fn adoption_keeps_execution_and_adapter_identities_and_moves_sequence() {
    let fixture = Fixture::new(&[NODE_A], &[]);
    let admitted = fixture.admitted();
    let fresh = crate::workflow_exchange::initial_record(
        workflow_request(),
        &route(&workflow_request()),
        identity('8'),
        identity('9'),
        99,
    )
    .expect("target record");
    assert_ne!(fresh.accepted.execution_identity, admitted.manifest.execution.accepted.execution_identity);
    assert_eq!(admit_route(&fresh, &admitted), Ok(()));
    let (completed, uncertain) = super::core::adopted_nodes(&admitted);
    let adopted = crate::workflow_exchange::adopt_checkpoint(&fresh, &crate::workflow_exchange::CheckpointAdoption {
        accepted: &admitted.manifest.execution.accepted,
        sequence: admitted.manifest.execution.sequence,
        cancellation_requested: false,
        completed: &completed,
        uncertain: &uncertain,
    })
    .expect("adoption");
    assert_eq!(adopted.accepted, admitted.manifest.execution.accepted);
    assert_eq!(adopted.run_id, 99);
    assert!(adopted.sequence > admitted.manifest.execution.sequence);
    assert_eq!(adopted.state, crate::workflow_exchange::DurableExecutionState::Queued);
    assert_eq!(adopted.nodes[0].state, NodeState::Completed);
    assert_eq!(adopted.nodes[0].adapter_identity, fresh.nodes[0].adapter_identity);
    assert_eq!(adopted.nodes[1].state, NodeState::Pending);
}

#[test]
fn refuses_crossed_route_identities() {
    let fixture = Fixture::new(&[NODE_A], &[]);
    let admitted = fixture.admitted();
    let request = workflow_request();
    let target = |revision: char, profile: char, operation: char| {
        let mut route = route(&request);
        route.nodes[0].operation_identity = identity(operation);
        crate::workflow_exchange::initial_record(request.clone(), &route, identity(revision), identity(profile), 1)
            .expect("target record")
    };
    assert_eq!(admit_route(&target('8', 'c', '1'), &admitted), Err(Refusal::CrossedHandlerProfile));
    assert_eq!(admit_route(&target('c', '9', '1'), &admitted), Err(Refusal::CrossedWorkflowRevision));
    assert_eq!(admit_route(&target('8', '9', 'c'), &admitted), Err(Refusal::CrossedAdapter));
    let mut tampered = fixture.admitted();
    tampered.outcomes[0].0.receipt_identity = Some(identity('d'));
    assert_eq!(admit_route(&target('8', '9', '1'), &tampered), Err(Refusal::CrossedReceipt));
    let mut other = request.clone();
    other.idempotency_key = identity('e');
    other.identity = lattice_workflow_contract::request_identity(&other);
    let crossed =
        crate::workflow_exchange::initial_record(other.clone(), &route(&other), identity('8'), identity('9'), 1)
            .expect("other record");
    assert_eq!(admit_route(&crossed, &admitted), Err(Refusal::CrossedRequest));
}

#[test]
fn refuses_outputs_object_for_another_node() {
    let outcome = outcome(NODE_A, crate::workflow::gate::PersistedNodeState::Completed);
    assert_eq!(node_outputs_object(NODE_B, &outcome).map(|_| ()), Err(Refusal::CheckpointInvalid));
}

// r[verify lattice.application_checkpoint.unavailable]
#[test]
fn route_without_hooks_is_unavailable_and_cannot_publish() {
    assert_eq!(checkpoint_offer(None), CheckpointOffer::Unavailable);
    let mut none = binding();
    none.checkpoint = CheckpointProfile::None;
    assert_eq!(checkpoint_offer(Some(&none)), CheckpointOffer::Unavailable);
    assert_eq!(
        checkpoint_offer(Some(&binding())),
        CheckpointOffer::Declared(CheckpointProfile::WorkflowNodeBoundaryV1)
    );
    let record = record();
    let result = build_manifest(&ManifestInput {
        record: &record,
        binding: &none,
        producing_incarnation: 0,
        local: &local(),
        completed: &BTreeMap::new(),
        uncertain: &BTreeSet::new(),
        workspace: &[],
        consumed: Consumed::default(),
        cancellation_requested: false,
    });
    assert_eq!(result, Err(Refusal::CheckpointUnavailable));
}

#[test]
fn binding_requires_schema_non_claims_and_absolute_paths() {
    assert_eq!(validate_binding(&binding()), Ok(()));
    let mut wrong = binding();
    wrong.schema = "lattice.application-checkpoint-binding.v2".to_string();
    assert_eq!(validate_binding(&wrong), Err(Refusal::Schema));
    let mut wrong = binding();
    wrong.non_claims.pop();
    assert_eq!(validate_binding(&wrong), Err(Refusal::Schema));
    let mut wrong = binding();
    wrong.root = std::path::PathBuf::from("relative");
    assert_eq!(validate_binding(&wrong), Err(Refusal::WorkspacePath));
}

// r[verify lattice.application_checkpoint.receipt]
#[test]
fn receipt_kinds_distinguish_restart_continuation_and_fresh_retry() {
    assert_eq!(restart_kind(0, 1, true), Ok(RestartKind::ApplicationRestart));
    assert_eq!(restart_kind(1, 1, true), Err(Refusal::StaleAdmission));
    assert_eq!(restart_kind(1, 1, false), Ok(RestartKind::VmContinuation));
    assert_eq!(restart_kind(1, 2, false), Ok(RestartKind::FreshRetry));
    assert_eq!(restart_kind(2, 1, false), Err(Refusal::StaleAdmission));
}

#[test]
fn root_reports_io_and_store_refusals() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let blocked = directory.path().join("file");
    std::fs::write(&blocked, b"not a directory").expect("blocker");
    let root = CheckpointRoot::new(&blocked);
    assert_eq!(root.put_manifest(&identity('a'), b"{}"), Err(Refusal::Io));
    let store = crate::store::Store::new(directory.path().join("lattice.redb")).expect("store");
    let mut bound = binding();
    bound.root = directory.path().join("root");
    let hook = NodeBoundaryHook::for_route(&store, Some(&bound), &record(), HookSeed::default())
        .expect("hook")
        .expect("declared hook");
    let finished = crate::engine::executor::NodeProgress::Finished {
        node_id: crate::engine::workflow::NodeId(NODE_A),
        duration_millis: None,
        result: crate::workflow::gate::shell::checkpoint_to_results(&[outcome(
            NODE_A,
            crate::workflow::gate::PersistedNodeState::Completed,
        )])
        .expect("step")
        .remove(0),
    };
    hook.observe(&finished);
    assert_eq!(hook.last_published(), None, "an execution missing from the store publishes nothing");
}

#[test]
fn fixtures_parse_and_match_their_documented_outcomes() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/application-checkpoint");
    let manifest_bytes = std::fs::read(root.join("admitted-manifest.json")).expect("manifest fixture");
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes).expect("manifest parses");
    assert_eq!(validate_manifest(&manifest), Ok(()));
    let canonical_bytes = serde_json::to_vec(&manifest).expect("canonical");
    let expected = identity_of_bytes(MANIFEST_SCHEMA, &canonical_bytes).expect("identity");
    let marker: CommitMarker =
        serde_json::from_slice(&std::fs::read(root.join("admitted-commit.json")).expect("marker"))
            .expect("marker parses");
    assert_eq!(marker.manifest, expected);
    let receipt: RestartReceipt =
        serde_json::from_slice(&std::fs::read(root.join("admitted-receipt.json")).expect("receipt"))
            .expect("receipt parses");
    assert_eq!(receipt.checkpoint, expected);
    let request: RestartRequest =
        serde_json::from_slice(&std::fs::read(root.join("admitted-request.json")).expect("request"))
            .expect("request parses");
    assert_eq!(request.checkpoint, expected);
    assert_eq!(request.admission, receipt.admission);
    let binding: CheckpointBinding =
        serde_json::from_slice(&std::fs::read(root.join("admitted-binding.json")).expect("binding"))
            .expect("binding parses");
    assert_eq!(validate_binding(&binding), Ok(()));
    let refusals: BTreeMap<String, String> =
        serde_json::from_slice(&std::fs::read(root.join("refusals.json")).expect("refusals")).expect("refusals parse");
    for (name, code) in refusals {
        let bytes = std::fs::read(root.join(&name)).expect("refused fixture");
        let outcome = serde_json::from_slice::<Manifest>(&bytes)
            .map_err(|_| Refusal::Schema)
            .and_then(|manifest| validate_manifest(&manifest));
        assert_eq!(outcome.map_err(|refusal| refusal.code().to_string()), Err(code), "{name}");
    }
}
