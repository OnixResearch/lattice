//! Worker inventory v2: definition identity, declared capability, #25 slots,
//! single-shot liveness and closed wait reasons, projected by the pure core.

// r[verify lattice.worker_inventory.definition]
// r[verify lattice.worker_inventory.definition.scenario.stable]
// r[verify lattice.worker_inventory.definition.scenario.basis]
// r[verify lattice.worker_inventory.capability]
// r[verify lattice.worker_inventory.capability.scenario.undeclared]
// r[verify lattice.worker_inventory.capability.scenario.declared]
// r[verify lattice.worker_inventory.capability.scenario.malformed]
// r[verify lattice.worker_inventory.liveness]
// r[verify lattice.worker_inventory.liveness.scenario.unknown]
// r[verify lattice.worker_inventory.liveness.scenario.single_shot]
// r[verify lattice.worker_inventory.wait_reasons]
// r[verify lattice.worker_inventory.wait_reasons.scenario.causes]
// r[verify lattice.worker_inventory.wait_reasons.scenario.disclosure]
// r[verify lattice.worker_inventory.revisions]
// r[verify lattice.worker_inventory.revisions.scenario.unpublished]

use lattice::execution_binding::core::admission::bound::*;
use lattice::execution_binding::core::admission::*;
use lattice::execution_binding::core::capability::CAPABILITY_SCHEMA;
use lattice::execution_binding::core::capability::Defect as CapabilityDefect;
use lattice::execution_binding::core::capability::MAX_CAPABILITY_NAMES;
use lattice::execution_binding::core::capability::MAX_NAME_BYTES;
use lattice::execution_binding::core::capability::NameSet;
use lattice::execution_binding::core::capability::is_name;
use lattice::execution_binding::core::capability::validate;
use lattice::execution_binding::core::projection::inventory::*;
use lattice::execution_binding::core::projection::lineage::*;
use lattice::execution_binding::core::projection::retained::*;
use lattice::execution_binding::core::projection::wait::Code as WaitCode;
use lattice::execution_binding::core::projection::wait::FactField;
use lattice::execution_binding::core::projection::wait::MAX_WAIT_REASONS;
use lattice::execution_binding::core::projection::wait::Reason as WaitReason;
use lattice::execution_binding::core::projection::wait::Subject as WaitSubject;
use lattice::execution_binding::core::projection::wait::Unbounded;
use lattice::execution_binding::core::projection::wait::finish;
use lattice::execution_binding::core::projection::*;
use lattice::execution_binding::core::request::*;

const ROUTE: &str = "blake3:9999999999999999999999999999999999999999999999999999999999999999";
const DEFINITION: &str = "blake3:8888888888888888888888888888888888888888888888888888888888888888";
static NO_PORT: ServingReadiness = ServingReadiness::NoServingPort {};

type Capability = lattice::execution_binding::core::capability::Declaration;

fn routes() -> RouteDeclaration {
    RouteDeclaration::Routes {
        routes: vec![NamedRoute {
            name: "primary".into(),
            route_identity: ROUTE.into(),
        }],
    }
}

fn input<'a>(routes: &'a RouteDeclaration, compute: &'a ComputeRequirement) -> ProfileInput<'a> {
    ProfileInput {
        owner: ProfileOwner::ImplementationAttempt,
        revision: "revision-a",
        default_definition: ATTEMPT_DEFINITION,
        capability: None,
        qualification_ref: None,
        adapter: Offered::Declared("omp-jsonl"),
        protocol_revision: Offered::Declared("omp-revision"),
        executable_identity: Offered::Declared("eeee"),
        inputs: Offered::Declared("workspace-snapshot"),
        outputs: Offered::Declared("candidate-receipt"),
        sessions: Offered::Declared("single-shot-no-session"),
        checkpoint: Offered::Unavailable,
        workspace: Offered::Declared("admitted-read-write-paths"),
        network: Offered::Declared("attempt-local-model-broker"),
        isolation: Offered::Declared("required-sandbox"),
        deadline_secs: Offered::Declared(60),
        max_output_bytes: Offered::Declared(4096),
        model_routes: Offered::Declared(routes),
        compute: Offered::Declared(compute),
        serving_readiness: &NO_PORT,
    }
}

fn declaration(task_families: Option<NameSet>, tools: Option<NameSet>) -> Capability {
    Capability {
        schema: CAPABILITY_SCHEMA.into(),
        definition: "rust-implementer".into(),
        task_families,
        tools,
    }
}

fn declared(names: &[&str]) -> Option<NameSet> {
    Some(NameSet::Declared {
        names: names.iter().map(|name| (*name).to_owned()).collect(),
    })
}

fn codes(reasons: &[WaitReason]) -> Vec<WaitCode> {
    reasons.iter().map(|reason| reason.code).collect()
}

#[test]
fn undeclared_capability_is_unknown_never_empty_and_explained() {
    let (routes, compute) = (routes(), ComputeRequirement::HostLocal {});
    let facts = project_profile(&input(&routes, &compute)).expect("facts");
    for names in [&facts.task_families, &facts.tools] {
        assert_eq!((names.support, names.value.as_ref()), (SupportLevel::Unknown, None));
    }
    assert_eq!(
        (facts.definition.name.as_str(), facts.definition.basis),
        (ATTEMPT_DEFINITION, DefinitionBasis::OwnerDefault)
    );
    assert_eq!(codes(&facts.wait_reasons), [
        WaitCode::CapabilityUndeclared,
        WaitCode::CapabilityUndeclared,
        WaitCode::QualificationUnbound,
        WaitCode::RevisionSelectionUnpublished,
        WaitCode::ServingPortAbsent,
    ]);
    assert_eq!(facts.wait_reasons[0], WaitReason {
        code: WaitCode::CapabilityUndeclared,
        subject: WaitSubject::Fact {
            field: FactField::TaskFamilies
        },
        owner: "configuration".into(),
    });
    let wire = serde_json::to_value(&facts).expect("facts json");
    assert_eq!(wire["schema"], "lattice.worker-profile-facts.v2");
    assert_eq!(wire["task_families"], serde_json::json!({"support": "unknown", "value": null}));
    assert_eq!(
        wire["wait_reasons"][4],
        serde_json::json!({
            "code": "serving_port_absent",
            "subject": {"kind": "fact", "field": "serving_readiness"},
            "owner": "kiln#49",
        })
    );
}

#[test]
fn declared_capability_is_sorted_canonical_and_unavailable_stays_distinct() {
    let (routes, compute) = (routes(), ComputeRequirement::HostLocal {});
    let capability = declaration(declared(&["rust-implementation", "cairn-task"]), Some(NameSet::Unavailable {}));
    let mut profile = input(&routes, &compute);
    profile.capability = Some(&capability);
    let facts = project_profile(&profile).expect("facts");
    assert_eq!(facts.task_families.support, SupportLevel::Declared);
    assert_eq!(
        facts.task_families.value.as_deref(),
        Some(&["cairn-task".to_owned(), "rust-implementation".to_owned()][..])
    );
    assert_eq!((facts.tools.support, facts.tools.value.as_ref()), (SupportLevel::Unavailable, None));
    assert!(!codes(&facts.wait_reasons).contains(&WaitCode::CapabilityUndeclared));
    let reordered = declaration(declared(&["cairn-task", "rust-implementation"]), Some(NameSet::Unavailable {}));
    assert_eq!(reordered.canonical(), capability.canonical(), "order never changes the canonical form");
    profile.qualification_ref = Some("blake3:abababababababababababababababababababababababababababababababab");
    let qualified = project_profile(&profile).expect("qualified");
    assert_eq!(qualified.task_families.support, SupportLevel::Qualified);
    assert_eq!(qualified.tools.support, SupportLevel::Unavailable, "unavailable never becomes qualified");
    let only_tools = declaration(None, declared(&["read"]));
    profile.capability = Some(&only_tools);
    let facts = project_profile(&profile).expect("facts");
    assert_eq!(facts.task_families.support, SupportLevel::Unknown, "an absent set stays undeclared");
    assert_eq!(facts.wait_reasons[0].subject, WaitSubject::Fact {
        field: FactField::TaskFamilies
    });
}

#[test]
fn malformed_declarations_are_refused_with_typed_defects() {
    let at_bound = (0..MAX_CAPABILITY_NAMES).map(|index| format!("tool-{index}")).collect::<Vec<_>>();
    let over_bound = (0..=MAX_CAPABILITY_NAMES).map(|index| format!("tool-{index}")).collect::<Vec<_>>();
    let longest = "a".repeat(MAX_NAME_BYTES);
    let too_long = "a".repeat(MAX_NAME_BYTES + 1);
    assert_eq!(validate(&declaration(None, Some(NameSet::Declared { names: at_bound }))), Ok(()));
    assert_eq!(validate(&declaration(declared(&[&longest]), None)), Ok(()));
    let cases = [
        (declaration(None, Some(NameSet::Declared { names: over_bound })), CapabilityDefect::TooMany {
            field: "tools",
        }),
        (declaration(declared(&[&too_long]), None), CapabilityDefect::Name { field: "task_families" }),
        (declaration(None, declared(&[])), CapabilityDefect::Empty { field: "tools" }),
        (declaration(None, declared(&["read", "read"])), CapabilityDefect::Duplicate { field: "tools" }),
        (declaration(None, declared(&["/usr/bin/bash"])), CapabilityDefect::Name { field: "tools" }),
        (declaration(None, declared(&["cargo test"])), CapabilityDefect::Name { field: "tools" }),
        (declaration(None, declared(&["Read"])), CapabilityDefect::Name { field: "tools" }),
        (declaration(None, declared(&["https://model.invalid/v1"])), CapabilityDefect::Name {
            field: "tools",
        }),
        (declaration(None, declared(&["-flag"])), CapabilityDefect::Name { field: "tools" }),
    ];
    let (routes, compute) = (routes(), ComputeRequirement::HostLocal {});
    for (capability, defect) in &cases {
        assert_eq!(validate(capability), Err(*defect), "{capability:?}");
        let mut profile = input(&routes, &compute);
        profile.capability = Some(capability);
        assert_eq!(project_profile(&profile), Err(FactsDefect::Capability { capability: *defect }));
    }
    let mut wrong_schema = declaration(None, None);
    wrong_schema.schema = "lattice.worker-capability.v2".into();
    assert_eq!(validate(&wrong_schema), Err(CapabilityDefect::Schema));
    let mut bad_definition = declaration(None, None);
    bad_definition.definition = "Rust Implementer".into();
    assert_eq!(validate(&bad_definition), Err(CapabilityDefect::Definition));
    let unknown_field = serde_json::json!({"schema": CAPABILITY_SCHEMA, "definition": "a", "prompt": "make it"});
    assert!(serde_json::from_value::<Capability>(unknown_field).is_err(), "unknown fields are refused");
    for name in ["", " ", "a/b", "a b", "a=b", "é", "A"] {
        assert!(!is_name(name), "{name:?}");
    }
}

#[test]
fn definition_identity_is_stable_across_revisions_and_distinct_across_bases() {
    let (routes, compute) = (routes(), ComputeRequirement::HostLocal {});
    let first = declaration(declared(&["rust-implementation"]), None);
    let second = declaration(declared(&["rust-implementation", "review"]), declared(&["read"]));
    let mut old = input(&routes, &compute);
    old.capability = Some(&first);
    let mut new = input(&routes, &compute);
    new.revision = "revision-b";
    new.executable_identity = Offered::Declared("ffff");
    new.capability = Some(&second);
    let (old, new) = (project_profile(&old).expect("old"), project_profile(&new).expect("new"));
    assert_ne!(old.revision, new.revision);
    assert_eq!(old.definition, new.definition, "a new revision keeps its definition");
    assert_eq!(old.definition.basis, DefinitionBasis::Declared);
    assert!(old.definition.identity.starts_with("blake3:") && old.definition.identity.len() == 71);
    let mut named_like_default = declaration(None, None);
    named_like_default.definition = ATTEMPT_DEFINITION.into();
    let mut declared_default = input(&routes, &compute);
    declared_default.capability = Some(&named_like_default);
    let declared_default = project_profile(&declared_default).expect("declared");
    let owner_default = project_profile(&input(&routes, &compute)).expect("default");
    assert_eq!(declared_default.definition.name, owner_default.definition.name);
    assert_ne!(declared_default.definition.identity, owner_default.definition.identity);
    let mut configured = input(&routes, &compute);
    configured.owner = ProfileOwner::ConfiguredRuntime;
    assert_ne!(project_profile(&configured).expect("configured").definition, owner_default.definition);
    let mut unnamed = input(&routes, &compute);
    unnamed.default_definition = "Not A Name";
    assert_eq!(project_profile(&unnamed), Err(FactsDefect::Definition));
}

#[test]
fn revision_slots_are_owned_unknowns() {
    let (routes, compute) = (routes(), ComputeRequirement::HostLocal {});
    let facts = project_profile(&input(&routes, &compute)).expect("facts");
    let slots = &facts.revisions;
    for slot in [
        &slots.desired,
        &slots.prepared,
        &slots.qualified,
        &slots.selected_for_new_sessions,
        &slots.pinned,
    ] {
        assert_eq!(slot, &OwnedUnknown::unknown("lattice#25"));
    }
    let wire = serde_json::to_value(&facts).expect("facts json");
    assert_eq!(wire["revisions"]["pinned"], serde_json::json!({"support": "unknown", "owner": "lattice#25"}));
}

fn bound() -> Admitted {
    let declaration = Declaration {
        schema: DECLARATION_SCHEMA.into(),
        serving: None,
        compute: ComputeDeclaration::HostLocal {},
        model_routes: routes(),
    };
    let requirement = WorkerRequirement {
        compute: ComputeRequirement::HostLocal {},
        model: ModelRequirement::Routes {
            names: vec!["primary".into()],
        },
    };
    let attempt = AttemptContext {
        work_id: "work-24",
        attempt_id: "attempt-24",
        fence: 7,
        envelope_identity: "envelope-24",
        profile_revision: "profile-24",
    };
    match admit(&declaration, &requirement, attempt, ProviderReading::NotRead) {
        Decision::Admitted(admitted) => *admitted,
        other => panic!("expected admission, got {other:?}"),
    }
}

const BINDING_ONLY: [(&str, &str); 1] = [("execution-binding.json", "b1")];
const DISPATCHED: [(&str, &str); 2] = [("execution-binding.json", "b1"), ("intent.json", "i1")];

fn state<'a>(admitted: &'a Admitted, artifacts: &'a [(&'a str, &'a str)]) -> AttemptState<'a> {
    AttemptState {
        admitted: Some(admitted),
        request_identity: "request-24",
        definition_identity: DEFINITION,
        admitted_until_epoch_ms: 2_000,
        has_intent: false,
        has_refusal: false,
        terminal_disposition: None,
        has_cancel_request: false,
        artifacts,
        serving_readiness: &NO_PORT,
    }
}

fn instance(state: &AttemptState<'_>, observed_at_epoch_ms: i64) -> WorkerInstanceObservation {
    project_instance(state, observed_at_epoch_ms).expect("projection").expect("instance")
}

#[test]
fn single_shot_liveness_is_unknown_when_missing_or_stale_and_never_idle_or_live() {
    let admitted = bound();
    let dispatched = AttemptState {
        has_intent: true,
        ..state(&admitted, &DISPATCHED)
    };
    let inside = instance(&dispatched, 1_000);
    assert_eq!(inside.definition_identity, DEFINITION);
    assert_eq!(inside.schema, "lattice.worker-instance-observation.v2");
    assert_eq!(inside.liveness, Liveness {
        runtime: RuntimeClass::SingleShot,
        state: LivenessState::Unknown,
        unknown_cause: Some(UnknownCause::Unobserved),
        retained_generation: 2,
        observed_at_epoch_ms: 1_000,
    });
    assert_eq!(codes(&inside.wait_reasons), [
        WaitCode::ServingPortAbsent,
        WaitCode::DispatchOutcomeUnknown,
        WaitCode::LivenessUnobserved,
        WaitCode::HostReachabilityUnknown,
        WaitCode::AllocationLivenessUnknown,
    ]);
    let after = instance(&dispatched, 2_001);
    assert_eq!(
        (after.liveness.state, after.liveness.unknown_cause),
        (LivenessState::Unknown, Some(UnknownCause::Stale))
    );
    assert!(codes(&after.wait_reasons).contains(&WaitCode::LivenessStale));
    assert_eq!(instance(&dispatched, -1).liveness.unknown_cause, Some(UnknownCause::Unobserved));
    for observed in [inside, after] {
        let text = serde_json::to_string(&observed).expect("instance json");
        for forbidden in ["\"idle\"", "\"live\"", "\"running\"", "\"healthy\"", "\"available\""] {
            assert!(!text.contains(forbidden), "{forbidden}");
        }
    }
}

#[test]
fn retained_lifecycle_maps_to_single_shot_liveness_and_reasons() {
    let admitted = bound();
    let admitted_only = instance(&state(&admitted, &BINDING_ONLY), 1_000);
    assert_eq!(admitted_only.liveness.state, LivenessState::NotStarted);
    assert_eq!(admitted_only.liveness.retained_generation, 1);
    assert!(codes(&admitted_only.wait_reasons).contains(&WaitCode::AwaitingDispatch));
    let expired = instance(&state(&admitted, &BINDING_ONLY), 2_001);
    assert!(codes(&expired.wait_reasons).contains(&WaitCode::AdmissionExpired));
    assert_eq!(expired.liveness.state, LivenessState::NotStarted, "an expired admission never started");
    let refused = AttemptState {
        has_intent: true,
        has_refusal: true,
        ..state(&admitted, &DISPATCHED)
    };
    let refused = instance(&refused, 1_000);
    assert_eq!((refused.liveness.state, refused.wait_reasons.len()), (LivenessState::NotStarted, 0));
    let terminal = AttemptState {
        has_intent: true,
        terminal_disposition: Some("failed"),
        ..state(&admitted, &DISPATCHED)
    };
    let terminal = instance(&terminal, 5_000);
    assert_eq!(terminal.liveness.state, LivenessState::Exited);
    assert_eq!(terminal.liveness.unknown_cause, None);
    assert_eq!(terminal.liveness.retained_generation, 3, "the terminal result grows the generation");
    assert!(terminal.wait_reasons.is_empty(), "a finished instance waits for nothing");
    let cancelling = AttemptState {
        has_intent: true,
        has_cancel_request: true,
        ..state(&admitted, &DISPATCHED)
    };
    assert!(codes(&instance(&cancelling, 1_000).wait_reasons).contains(&WaitCode::CancellationUnobserved));
    let crossed = AttemptState {
        definition_identity: "omp-attempt-worker",
        ..state(&admitted, &BINDING_ONLY)
    };
    assert_eq!(project_instance(&crossed, 1), Err(StateDefect::Definition));
}

fn reading(readiness: &str, freshness: &str) -> SnapshotReading {
    SnapshotReading::Observed {
        snapshot_identity: format!("blake3:{}", "d".repeat(64)),
        source_generation: 1,
        sequence: 4,
        readiness: readiness.into(),
        freshness: freshness.into(),
    }
}

#[test]
fn serving_causes_name_published_routes_once_and_deterministically() {
    let cases = [
        (reading("draining", "too-old"), Some(WaitCode::ServingSnapshotNotCurrent)),
        (reading("draining", "current"), Some(WaitCode::ServingNotReady)),
        (reading("ready", "current"), None),
        (SnapshotReading::Unavailable {}, Some(WaitCode::ServingSnapshotUnavailable)),
        (SnapshotReading::PortIssue {}, Some(WaitCode::ServingPortIssue)),
    ];
    let (routes, compute) = (routes(), ComputeRequirement::HostLocal {});
    let admitted = bound();
    for (snapshot, expected) in cases {
        let readiness = ServingReadiness::Observed {
            routes: vec![RouteReadiness {
                route_name: "primary".into(),
                service_id: "svc-private-name".into(),
                snapshot,
            }],
        };
        let mut profile = input(&routes, &compute);
        profile.serving_readiness = &readiness;
        let facts = project_profile(&profile).expect("facts");
        assert_eq!(facts, project_profile(&profile).expect("again"), "deterministic");
        let route_reasons = facts
            .wait_reasons
            .iter()
            .filter(|reason| matches!(reason.subject, WaitSubject::Route { .. }))
            .collect::<Vec<_>>();
        assert_eq!(
            route_reasons.iter().map(|reason| reason.code).collect::<Vec<_>>(),
            expected.into_iter().collect::<Vec<_>>()
        );
        for reason in &route_reasons {
            assert_eq!(reason.subject, WaitSubject::Route { name: "primary".into() }, "a published route name");
            assert_eq!(reason.owner, "animus#17");
        }
        let reasons = serde_json::to_string(&facts.wait_reasons).expect("reasons json");
        assert!(!reasons.contains("svc-private-name"), "service ids stay in readiness, not in reasons");
        let observed = AttemptState {
            has_intent: true,
            serving_readiness: &readiness,
            ..state(&admitted, &DISPATCHED)
        };
        let instance = instance(&observed, 1_000);
        assert_eq!(
            instance
                .wait_reasons
                .iter()
                .filter(|reason| matches!(reason.subject, WaitSubject::Route { .. }))
                .count(),
            usize::from(expected.is_some())
        );
        assert!(!codes(&instance.wait_reasons).contains(&WaitCode::ServingPortAbsent));
    }
}

#[test]
fn wait_reasons_are_sorted_deduplicated_and_bounded() {
    let reason = |code| WaitReason::fact(code, FactField::Lifecycle);
    let reasons = vec![
        reason(WaitCode::LivenessStale),
        reason(WaitCode::CapabilityUndeclared),
        reason(WaitCode::LivenessStale),
    ];
    assert_eq!(codes(&finish(reasons).expect("bounded")), [WaitCode::CapabilityUndeclared, WaitCode::LivenessStale]);
    let unbounded = (0..=MAX_WAIT_REASONS)
        .map(|index| WaitReason::route(WaitCode::ServingNotReady, &format!("route-{index}")))
        .collect();
    assert_eq!(finish(unbounded), Err(Unbounded));
    assert_eq!(WaitCode::AwaitingDispatch.owner(), "kiln#45");
    assert_eq!(WaitCode::DispatchOutcomeUnknown.owner(), "lattice#6");
}
