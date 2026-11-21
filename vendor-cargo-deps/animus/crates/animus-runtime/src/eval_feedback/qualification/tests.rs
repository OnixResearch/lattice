use alloc::string::String;
use alloc::vec::Vec;

use super::*;

const DEFINITION_LABEL: &str = "definition";
const REVISION_LABEL: &str = "revision";
const TASK_FAMILY: &str = "campaign.exploration";
const OBSERVED_AT_MS: u64 = 1_790_000_000_000;
const VALID_UNTIL_MS: u64 = 1_792_592_000_000;
const NOW_MS: u64 = 1_790_000_100_000;
const PLANNED: u64 = 4;

fn identity(label: &str) -> String {
    framed_identity(&[label.as_bytes()])
}

fn hex(label: &str) -> String {
    identity(label).trim_start_matches("blake3:").into()
}

fn component(kind: ComponentKind, label: &str) -> Component {
    Component {
        kind,
        basis: ComponentBasis::Observed,
        identity: Some(component_identity(kind, label.as_bytes())),
    }
}

fn subject() -> QualificationSubject {
    let components = ComponentKind::ALL
        .into_iter()
        .map(|kind| match kind {
            ComponentKind::SemanticRole | ComponentKind::Skills | ComponentKind::MemoryPolicy => {
                Component {
                    kind,
                    basis: ComponentBasis::Unavailable,
                    identity: None,
                }
            }
            ComponentKind::Permissions | ComponentKind::Network => Component {
                basis: ComponentBasis::Declared,
                ..component(kind, "declared")
            },
            _ => component(kind, "v1"),
        })
        .collect();
    QualificationSubject {
        lattice_definition_identity: identity(DEFINITION_LABEL),
        lattice_profile_revision: hex(REVISION_LABEL),
        task_family: TASK_FAMILY.into(),
        components,
        model: ModelFacts {
            deployment: ModelDeployment::Scripted,
            version_basis: VersionBasis::Declared,
            version: Some("model:scripted:signal-greedy".into()),
            weights_identity: None,
        },
    }
}

fn bound(kind: ComponentKind) -> String {
    component_identity(kind, b"v1")
}

fn document() -> AttachmentDocument {
    AttachmentDocument {
        subject: subject(),
        evidence: EvidenceSource {
            class: EvidenceClass::TaskFamilyEvaluation,
            protocol_identity: bound(ComponentKind::TaskProtocol),
            verifier_identity: bound(ComponentKind::Verifier),
            corpus_identity: Some(bound(ComponentKind::Corpus)),
            source_identity: identity("run"),
        },
        outcomes: Some(Outcomes {
            planned: PLANNED,
            accepted: 1,
            rejected: 1,
            failed: 1,
            cancelled: 0,
            incomplete: 1,
        }),
        measures: Some(Measures {
            repetitions: 1,
            tasks_covered: 2,
            tasks_declared: PLANNED,
            time_to_acceptance_ms: None,
            covered_time: CoveredTime {
                queue_ms: None,
                repair_ms: None,
                verification_ms: Some(40),
            },
            costs: Costs {
                unit: "cost-units".into(),
                model: Some(10),
                tool: None,
                manager: Some(3),
                verifier: Some(2),
                unknown_remainder: true,
            },
            review_interventions: None,
            invalid_actions: Some(0),
            false_completion_attempts: None,
        }),
        scope: Scope {
            partitions: alloc::vec!["all".into()],
            limitations: alloc::vec!["synthetic-tasks-only".into()],
            missing_modes: alloc::vec!["live-model-route".into()],
        },
        provenance: Provenance {
            label: EvidenceLabel::Fixture,
            producer: "unit-test".into(),
            producer_revision: "test".into(),
            observed_at_ms: OBSERVED_AT_MS,
            valid_until_ms: VALID_UNTIL_MS,
        },
        references: alloc::vec![Reference {
            kind: "atif-trajectory".into(),
            identity: identity("trajectory"),
        }],
    }
}

fn bytes(document: &AttachmentDocument) -> Vec<u8> {
    encode_attachment(document)
        .expect("valid document")
        .canonical
}

fn expected<'a>(
    definition: &'a str,
    revision: &'a str,
    current: &'a [CurrentComponent<'a>],
) -> ExpectedSubject<'a> {
    ExpectedSubject {
        lattice_definition_identity: definition,
        lattice_profile_revision: revision,
        task_family: TASK_FAMILY,
        required_class: EvidenceClass::TaskFamilyEvaluation,
        now_ms: NOW_MS,
        required_components: &[],
        current_components: current,
    }
}

fn reason(bytes: &[u8], expected: &ExpectedSubject<'_>) -> RejectionReason {
    validate_attachment(bytes, expected)
        .expect_err("rejected")
        .reason
}

fn decode_reason(bytes: &[u8]) -> RejectionReason {
    decode_attachment(bytes).expect_err("rejected").reason
}

fn decode_field(bytes: &[u8]) -> &'static str {
    decode_attachment(bytes).expect_err("rejected").field
}

fn encode_field(document: &AttachmentDocument) -> &'static str {
    encode_attachment(document).expect_err("rejected").field
}

#[test]
// r[verify agent_harness_runtime.qualification_attachment.attachment]
// r[verify agent_harness_runtime.qualification_attachment.validator]
// r[verify agent_harness_runtime.qualification_attachment.boundary]
fn canonical_attachment_round_trips_and_qualifies_the_exact_subject() {
    let encoded = encode_attachment(&document()).expect("valid");
    assert_eq!(
        encoded.identity,
        alloc::format!("blake3:{}", blake3::hash(&encoded.canonical).to_hex())
    );
    let decoded = decode_attachment(&encoded.canonical).expect("canonical");
    assert_eq!(decoded, encoded);
    let (definition, revision) = (identity(DEFINITION_LABEL), hex(REVISION_LABEL));
    let expected = expected(&definition, &revision, &[]);
    let validated = validate_attachment(&encoded.canonical, &expected).expect("qualifies");
    assert_eq!(validated.identity, encoded.identity);
    let answer = qualify(&[encoded.canonical.as_slice()], &expected);
    assert_eq!(
        answer,
        QualificationAnswer::Qualified(Qualification {
            qualification_ref: encoded.identity,
            evidence_class: EvidenceClass::TaskFamilyEvaluation,
            protocol_identity: bound(ComponentKind::TaskProtocol),
            verifier_identity: bound(ComponentKind::Verifier),
            observed_at_ms: OBSERVED_AT_MS,
            provenance_label: EvidenceLabel::Fixture,
        })
    );
    assert_eq!(
        RejectionReason::SubjectMismatch.as_str(),
        "subject_mismatch"
    );
    assert_eq!(
        EvidenceClass::DeploymentAdmission.as_str(),
        "deployment_admission"
    );
    assert_eq!(EvidenceLabel::Fixture.as_str(), "fixture");
    assert_eq!(EvidenceLabel::Live.as_str(), "live");
    let mut live = document();
    live.provenance.label = EvidenceLabel::Live;
    let live = bytes(&live);
    let QualificationAnswer::Qualified(live) = qualify(&[live.as_slice()], &expected) else {
        panic!("live evidence qualifies");
    };
    assert_eq!(live.provenance_label, EvidenceLabel::Live);
}

#[test]
// r[verify agent_harness_runtime.qualification_attachment.subject]
fn subject_keys_and_model_facts_are_strictly_spelled() {
    let mut prefixed = document();
    prefixed.subject.lattice_profile_revision = identity(REVISION_LABEL);
    assert!(encode_attachment(&prefixed).is_err());
    let mut bare = document();
    bare.subject.lattice_definition_identity = hex(DEFINITION_LABEL);
    assert!(encode_attachment(&bare).is_err());
    let mut family = document();
    family.subject.task_family = "Campaign Exploration".into();
    assert!(encode_attachment(&family).is_err());
    let mut missing_kind = document();
    missing_kind.subject.components.pop();
    assert!(encode_attachment(&missing_kind).is_err());
    let mut unavailable_with_identity = document();
    unavailable_with_identity.subject.components[0].basis = ComponentBasis::Unavailable;
    assert!(encode_attachment(&unavailable_with_identity).is_err());

    let mut hosted = document();
    hosted.subject.model = ModelFacts {
        deployment: ModelDeployment::Hosted,
        version_basis: VersionBasis::ProviderDeclared,
        version: Some("provider-model-2026-10-01".into()),
        weights_identity: None,
    };
    assert!(encode_attachment(&hosted).is_ok());
    hosted.subject.model.weights_identity = Some(identity("weights"));
    assert!(encode_attachment(&hosted).is_err());
    hosted.subject.model.version_basis = VersionBasis::VerifiedArtifact;
    assert!(encode_attachment(&hosted).is_err());
    let mut local = document();
    local.subject.model = ModelFacts {
        deployment: ModelDeployment::Local,
        version_basis: VersionBasis::VerifiedArtifact,
        version: None,
        weights_identity: Some(identity("weights")),
    };
    assert!(encode_attachment(&local).is_ok());

    let (definition, revision) = (identity(DEFINITION_LABEL), hex(REVISION_LABEL));
    let prefixed_revision = identity(REVISION_LABEL);
    let encoded = bytes(&document());
    assert_eq!(
        reason(&encoded, &expected(&definition, &prefixed_revision, &[])),
        RejectionReason::Malformed
    );
    assert_eq!(
        reason(&encoded, &expected(&hex(DEFINITION_LABEL), &revision, &[])),
        RejectionReason::Malformed
    );
}

#[test]
// r[verify agent_harness_runtime.qualification_attachment.attachment]
fn malformed_and_noncanonical_documents_are_rejected() {
    let encoded = bytes(&document());
    let text = String::from_utf8(encoded.clone()).expect("ascii");
    assert_eq!(decode_reason(b"not json"), RejectionReason::Malformed);
    assert_eq!(
        decode_reason(&encoded[..encoded.len() - 1]),
        RejectionReason::Malformed
    );
    let spaced = text.replacen(',', ", ", 1);
    assert_eq!(decode_reason(spaced.as_bytes()), RejectionReason::Malformed);
    let extra = text.replacen('{', "{\"activate\":true,", 1);
    assert_eq!(decode_reason(extra.as_bytes()), RejectionReason::Malformed);
    let throughput = text.replacen('{', "{\"throughput_tokens_per_second\":900,", 1);
    assert_eq!(
        decode_reason(throughput.as_bytes()),
        RejectionReason::Malformed
    );
    let schema = text.replace("qualification-attachment.v1", "qualification-attachment.v2");
    assert_eq!(decode_reason(schema.as_bytes()), RejectionReason::Malformed);
    let leading_zero = text.replace("\"planned\":4", "\"planned\":04");
    assert_eq!(
        decode_reason(leading_zero.as_bytes()),
        RejectionReason::Malformed
    );

    let mut dropped_failure = document();
    if let Some(outcomes) = dropped_failure.outcomes.as_mut() {
        outcomes.failed = 0;
    }
    assert!(encode_attachment(&dropped_failure).is_err());
    let mut unbound_verifier = document();
    unbound_verifier.evidence.verifier_identity = identity("other-verifier");
    assert!(encode_attachment(&unbound_verifier).is_err());
    let mut declared_with_outcomes = document();
    declared_with_outcomes.evidence.class = EvidenceClass::DeclaredSupport;
    assert!(encode_attachment(&declared_with_outcomes).is_err());
    let mut conformance_with_measures = document();
    conformance_with_measures.evidence.class = EvidenceClass::ContractConformance;
    assert!(encode_attachment(&conformance_with_measures).is_err());
    conformance_with_measures.measures = None;
    assert!(encode_attachment(&conformance_with_measures).is_ok());

    let root_schema = "\"schema\":\"animus.qualification-attachment.v1\"";
    let reordered = alloc::format!(
        "{{{root_schema},{}",
        text.replacen(&alloc::format!(",{root_schema}"), "", 1)
            .trim_start_matches('{')
    );
    assert_ne!(reordered, text);
    assert_eq!(reordered.len(), text.len());
    assert_eq!(decode_field(reordered.as_bytes()), "attachment.canonical");
    let mut oversized = encoded.clone();
    oversized.resize(MAX_ATTACHMENT_BYTES + 1, b' ');
    assert_eq!(decode_field(&oversized), "attachment.bytes");
    assert_eq!(decode_field(b"not json"), "attachment.syntax");
}

#[test]
// r[verify agent_harness_runtime.qualification_attachment.redaction]
fn unsafe_material_cannot_be_represented() {
    let oversized = "x".repeat(65);
    let unsafe_values = [
        "/home/user/.config/provider/key",
        "Write a function that parses the hidden test input",
        "sk-live-0123456789",
        "provider:sk-live-0123456789",
        "bearer-abc",
        "UPPER",
        "../escape",
        oversized.as_str(),
        "db_password:hunter2",
        "sk_live_fixture",
        "private_key:9f86d081",
        "openai_api-key:0123abcd",
        "auth_bearer-abc",
    ];
    for value in unsafe_values {
        let mut limitation = document();
        limitation.scope.limitations = alloc::vec![String::from(value)];
        assert!(encode_attachment(&limitation).is_err(), "{value}");
        let mut producer = document();
        producer.provenance.producer = String::from(value);
        assert!(encode_attachment(&producer).is_err(), "{value}");
    }
    for ordinary in [
        "risk-analysis",
        "task-family.review",
        "desk-check",
        "task-family",
        "risk-assessment",
        "task_family",
    ] {
        let mut family = document();
        family.subject.task_family = String::from(ordinary);
        assert!(encode_attachment(&family).is_ok(), "{ordinary}");
    }
    let mut reference = document();
    reference.references[0].identity = "atif:trial:1".into();
    assert!(encode_attachment(&reference).is_err());
    let text = String::from_utf8(bytes(&document())).expect("ascii");
    let injected = text.replace("synthetic-tasks-only", "/etc/passwd");
    assert_eq!(
        decode_reason(injected.as_bytes()),
        RejectionReason::Malformed
    );
    let escaped = text.replace("synthetic-tasks-only", "a\\nb");
    assert_eq!(
        decode_reason(escaped.as_bytes()),
        RejectionReason::Malformed
    );
    assert!(!text.contains('/'));
    assert!(!text.contains(' '));
}

#[test]
// r[verify agent_harness_runtime.qualification_attachment.attachment]
fn the_iterative_parser_keeps_its_depth_item_and_key_bounds() {
    let nested = |levels: usize| {
        let mut text = "[".repeat(levels);
        text.push('1');
        text.push_str(&"]".repeat(levels));
        text
    };
    // Six containers put the scalar at depth 6, the deepest allowed.
    let deepest = nested(6);
    let parsed = super::json::parse(deepest.as_bytes()).expect("depth 6");
    assert_eq!(super::json::encode(&parsed), deepest.as_bytes());
    assert!(super::json::parse(nested(7).as_bytes()).is_none());
    // An empty container counts as a value at its own depth.
    assert!(super::json::parse(b"[[[[[[[]]]]]]]").is_some());
    assert!(super::json::parse(b"[[[[[[[[]]]]]]]]").is_none());
    assert!(super::json::parse(b"[[[[[[[]]]]]]").is_none());

    let items = |count: usize| alloc::format!("[{}]", alloc::vec!["0"; count].join(","));
    assert!(super::json::parse(items(64).as_bytes()).is_some());
    assert!(super::json::parse(items(65).as_bytes()).is_none());
    let members = |count: usize| {
        let entries: Vec<String> = (0..count)
            .map(|index| alloc::format!("\"k{index:02}\":0"))
            .collect();
        alloc::format!("{{{}}}", entries.join(","))
    };
    assert!(super::json::parse(members(64).as_bytes()).is_some());
    assert!(super::json::parse(members(65).as_bytes()).is_none());

    let object = br#"{"a":[1,{"b":null}],"c":{"d":true,"e":"x"}}"#;
    let parsed = super::json::parse(object).expect("object");
    assert_eq!(super::json::encode(&parsed), object);
    for refused in [
        br#"{"a":1,"a":2}"#.as_slice(),
        b"[1,]",
        b"[1 ]",
        b"{}x",
        b"[1",
        b"{\"a\"1}",
        b"",
    ] {
        assert!(super::json::parse(refused).is_none(), "{refused:?}");
    }
    assert!(super::json::parse(b"[]").is_some());
    assert!(super::json::parse(b"{}").is_some());
}

#[test]
// r[verify agent_harness_runtime.qualification_attachment.validator]
fn cross_profile_scope_and_class_mismatches_are_not_qualified() {
    let encoded = bytes(&document());
    let (definition, revision) = (identity(DEFINITION_LABEL), hex(REVISION_LABEL));
    let other_revision = hex("other-revision");
    let other_definition = identity("other-definition");
    assert_eq!(
        reason(&encoded, &expected(&definition, &other_revision, &[])),
        RejectionReason::SubjectMismatch
    );
    assert_eq!(
        reason(&encoded, &expected(&other_definition, &revision, &[])),
        RejectionReason::SubjectMismatch
    );
    let family = ExpectedSubject {
        task_family: "code.repair",
        ..expected(&definition, &revision, &[])
    };
    assert_eq!(reason(&encoded, &family), RejectionReason::ScopeMismatch);
    for class in [
        EvidenceClass::DeclaredSupport,
        EvidenceClass::ContractConformance,
        EvidenceClass::DeploymentAdmission,
    ] {
        let required = ExpectedSubject {
            required_class: class,
            ..expected(&definition, &revision, &[])
        };
        assert_eq!(
            reason(&encoded, &required),
            RejectionReason::InsufficientClass
        );
    }
    let mut declared = document();
    declared.evidence = EvidenceSource {
        class: EvidenceClass::DeclaredSupport,
        protocol_identity: identity("declaration-schema"),
        verifier_identity: identity("declaring-owner"),
        corpus_identity: None,
        source_identity: identity("declaration"),
    };
    declared.outcomes = None;
    declared.measures = None;
    let declared = bytes(&declared);
    assert_eq!(
        reason(&declared, &expected(&definition, &revision, &[])),
        RejectionReason::InsufficientClass
    );
}

#[test]
// r[verify agent_harness_runtime.qualification_attachment.applicability]
fn stale_changed_and_unavailable_components_are_not_qualified() {
    let encoded = bytes(&document());
    let (definition, revision) = (identity(DEFINITION_LABEL), hex(REVISION_LABEL));
    let late = ExpectedSubject {
        now_ms: VALID_UNTIL_MS,
        ..expected(&definition, &revision, &[])
    };
    assert_eq!(reason(&encoded, &late), RejectionReason::Stale);
    let early = ExpectedSubject {
        now_ms: OBSERVED_AT_MS - 1,
        ..expected(&definition, &revision, &[])
    };
    assert_eq!(reason(&encoded, &early), RejectionReason::Unusable);
    for kind in [
        ComponentKind::Prompt,
        ComponentKind::ModelRoute,
        ComponentKind::ToolImplementation,
        ComponentKind::Permissions,
        ComponentKind::ContextControls,
        ComponentKind::Environment,
        ComponentKind::Verifier,
        ComponentKind::TaskProtocol,
    ] {
        let changed = component_identity(kind, b"v2");
        let current = [CurrentComponent {
            kind,
            identity: Some(&changed),
        }];
        assert_eq!(
            reason(&encoded, &expected(&definition, &revision, &current)),
            RejectionReason::Stale,
            "{kind:?}"
        );
        let unchanged_identity = document()
            .subject
            .component(kind)
            .and_then(|component| component.identity.clone())
            .expect("component identity");
        let unchanged = [CurrentComponent {
            kind,
            identity: Some(&unchanged_identity),
        }];
        assert!(
            validate_attachment(&encoded, &expected(&definition, &revision, &unchanged)).is_ok()
        );
    }
    let verifier_gone = [CurrentComponent {
        kind: ComponentKind::Verifier,
        identity: None,
    }];
    assert_eq!(
        reason(&encoded, &expected(&definition, &revision, &verifier_gone)),
        RejectionReason::Unusable
    );
    let requires_skills = ExpectedSubject {
        required_components: &[ComponentKind::Skills],
        ..expected(&definition, &revision, &[])
    };
    assert_eq!(
        reason(&encoded, &requires_skills),
        RejectionReason::Unusable
    );
}

#[test]
// r[verify agent_harness_runtime.qualification_attachment.replay]
fn duplicate_and_conflicting_sets_follow_replay_rules() {
    let first = bytes(&document());
    let mut other_claim = document();
    if let Some(outcomes) = other_claim.outcomes.as_mut() {
        outcomes.accepted = 2;
        outcomes.rejected = 0;
    }
    let second = bytes(&other_claim);
    let (definition, revision) = (identity(DEFINITION_LABEL), hex(REVISION_LABEL));
    let expected = expected(&definition, &revision, &[]);
    assert!(matches!(
        qualify(&[first.as_slice(), first.as_slice()], &expected),
        QualificationAnswer::Qualified(_)
    ));
    assert_eq!(
        qualify(&[first.as_slice(), second.as_slice()], &expected),
        QualificationAnswer::NotQualified(RejectionReason::Conflicting)
    );
    assert_eq!(qualify(&[], &expected), QualificationAnswer::Unavailable);
    let mut stale = document();
    stale.provenance.valid_until_ms = NOW_MS;
    let stale = bytes(&stale);
    assert_eq!(
        qualify(&[b"garbage".as_slice(), stale.as_slice()], &expected),
        QualificationAnswer::NotQualified(RejectionReason::Stale)
    );

    let report = replay_attachments(&[
        first.as_slice(),
        first.as_slice(),
        second.as_slice(),
        b"{}".as_slice(),
    ]);
    assert_eq!(report.accepted.len(), 2);
    assert_eq!(report.duplicates, 1);
    assert_eq!(
        report.rejected,
        alloc::vec![(3, RejectionReason::Malformed)]
    );
    assert_eq!(report.conflicting_sources, alloc::vec![identity("run")]);
    let replayed = decode_attachment(&first).expect("replay");
    let measures = replayed.document.measures.expect("measures");
    assert_eq!(measures.costs.tool, None);
    assert_eq!(measures.time_to_acceptance_ms, None);
}

#[test]
// r[verify agent_harness_runtime.qualification_attachment.replay]
fn conflicts_survive_a_stale_unusable_or_changed_member() {
    let first = bytes(&document());
    let (definition, revision) = (identity(DEFINITION_LABEL), hex(REVISION_LABEL));
    let conflicting = QualificationAnswer::NotQualified(RejectionReason::Conflicting);

    // Time: the second member expires first; the set stays conflicting.
    let expires_at = NOW_MS + 1_000;
    let mut expiring = document();
    expiring.provenance.valid_until_ms = expires_at;
    let expiring = bytes(&expiring);
    let now = expected(&definition, &revision, &[]);
    let later = ExpectedSubject {
        now_ms: expires_at,
        ..now
    };
    assert_eq!(reason(&expiring, &later), RejectionReason::Stale);
    assert!(validate_attachment(&first, &later).is_ok());
    for at in [now, later] {
        assert_eq!(
            qualify(&[first.as_slice(), expiring.as_slice()], &at),
            conflicting
        );
    }

    // Required component: the first member lacks skills evidence.
    let mut with_skills = document();
    with_skills.subject.components = with_skills
        .subject
        .components
        .into_iter()
        .map(|entry| {
            if entry.kind == ComponentKind::Skills {
                component(ComponentKind::Skills, "v1")
            } else {
                entry
            }
        })
        .collect();
    let with_skills = bytes(&with_skills);
    let requires_skills = ExpectedSubject {
        required_components: &[ComponentKind::Skills],
        ..now
    };
    assert_eq!(reason(&first, &requires_skills), RejectionReason::Unusable);
    assert!(validate_attachment(&with_skills, &requires_skills).is_ok());
    assert_eq!(
        qualify(
            &[first.as_slice(), with_skills.as_slice()],
            &requires_skills
        ),
        conflicting
    );

    // Current component: the first member's environment changed.
    let new_environment = component_identity(ComponentKind::Environment, b"v2");
    let mut moved = document();
    moved.subject.components = moved
        .subject
        .components
        .into_iter()
        .map(|entry| {
            if entry.kind == ComponentKind::Environment {
                component(ComponentKind::Environment, "v2")
            } else {
                entry
            }
        })
        .collect();
    let moved = bytes(&moved);
    let current = [CurrentComponent {
        kind: ComponentKind::Environment,
        identity: Some(&new_environment),
    }];
    let changed = expected(&definition, &revision, &current);
    assert_eq!(reason(&first, &changed), RejectionReason::Stale);
    assert!(validate_attachment(&moved, &changed).is_ok());
    assert_eq!(
        qualify(&[moved.as_slice(), first.as_slice()], &changed),
        conflicting
    );
}

#[test]
// r[verify agent_harness_runtime.qualification_attachment.measures]
fn denominators_and_costs_keep_failures_and_unknowns() {
    let unknown = encode_attachment(&document()).expect("valid");
    assert_eq!(cost_per_accepted(&unknown), CostPerAccepted::UnknownCost);
    let mut known = document();
    if let Some(measures) = known.measures.as_mut() {
        measures.costs.tool = Some(5);
        measures.costs.unknown_remainder = false;
    }
    let known_attachment = encode_attachment(&known).expect("valid");
    assert_eq!(
        cost_per_accepted(&known_attachment),
        CostPerAccepted::Defined {
            known_total: 20,
            accepted: 1
        }
    );
    let mut none_accepted = known;
    if let Some(outcomes) = none_accepted.outcomes.as_mut() {
        outcomes.accepted = 0;
        outcomes.rejected = 2;
    }
    let none_accepted = encode_attachment(&none_accepted).expect("valid");
    assert_eq!(
        cost_per_accepted(&none_accepted),
        CostPerAccepted::NoAcceptedCompletion
    );
    let mut zero_for_unknown = document();
    if let Some(measures) = zero_for_unknown.measures.as_mut() {
        measures.costs.unknown_remainder = false;
    }
    assert!(encode_attachment(&zero_for_unknown).is_err());

    let mut inflated_coverage = document();
    if let Some(outcomes) = inflated_coverage.outcomes.as_mut() {
        outcomes.rejected = 0;
        outcomes.failed = 2;
    }
    assert_eq!(encode_field(&inflated_coverage), "measures.task_coverage");
    let mut uncovered_completion = document();
    if let Some(measures) = uncovered_completion.measures.as_mut() {
        measures.tasks_covered = 1;
    }
    assert_eq!(
        encode_field(&uncovered_completion),
        "measures.task_coverage"
    );
    let mut time_without_acceptance = document();
    if let Some(outcomes) = time_without_acceptance.outcomes.as_mut() {
        outcomes.accepted = 0;
        outcomes.rejected = 2;
    }
    if let Some(measures) = time_without_acceptance.measures.as_mut() {
        measures.time_to_acceptance_ms = Some(0);
    }
    assert_eq!(
        encode_field(&time_without_acceptance),
        "measures.time_to_acceptance_ms"
    );
    let mut timed_acceptance = document();
    if let Some(measures) = timed_acceptance.measures.as_mut() {
        measures.time_to_acceptance_ms = Some(0);
    }
    assert!(encode_attachment(&timed_acceptance).is_ok());
}

#[test]
// r[verify agent_harness_runtime.qualification_attachment.measures]
fn equal_budget_comparison_counts_only_independently_accepted_completions() {
    let baseline = encode_attachment(&document()).expect("valid");
    let mut faster = document();
    faster.subject.model.version = Some("model:faster".into());
    if let Some(outcomes) = faster.outcomes.as_mut() {
        outcomes.accepted = 0;
        outcomes.rejected = 2;
    }
    let faster = encode_attachment(&faster).expect("valid");
    assert_eq!(
        compare_equal_budget(&baseline, &faster),
        Ok(EqualBudgetComparison::NoAcceptedGain {
            baseline: 1,
            candidate: 0
        })
    );
    assert_eq!(
        compare_equal_budget(&faster, &baseline),
        Ok(EqualBudgetComparison::CandidateMoreAccepted {
            baseline: 0,
            candidate: 1
        })
    );
    let mut other_verifier = document();
    let verifier = component_identity(ComponentKind::Verifier, b"v2");
    other_verifier
        .evidence
        .verifier_identity
        .clone_from(&verifier);
    if let Some(component) = other_verifier
        .subject
        .components
        .iter_mut()
        .find(|component| component.kind == ComponentKind::Verifier)
    {
        component.identity = Some(verifier);
    }
    let other_verifier = encode_attachment(&other_verifier).expect("valid");
    assert_eq!(
        compare_equal_budget(&baseline, &other_verifier),
        Err(ComparisonRefusal::UnmatchedControls(Some(
            ComponentKind::Verifier
        )))
    );
}

#[test]
// r[verify agent_harness_runtime.qualification_attachment.applicability]
// r[verify agent_harness_runtime.qualification_attachment.authority]
fn applicability_requires_scoped_decisions_and_never_grants_authority() {
    let attachment = encode_attachment(&document()).expect("valid");
    let policy = ApplicabilityPolicy {
        now_ms: NOW_MS,
        required: &[],
    };
    let current = subject();
    assert_eq!(
        classify_applicability(&attachment, &current, &policy, None),
        Applicability::Current
    );
    let review = review_qualification(&Applicability::Current);
    assert!(review.can_request_scaffold_admission);
    assert!(!review.activates_scaffold);
    assert!(!review.changes_runtime_selection_policy);

    let mut revised = subject();
    revised.lattice_profile_revision = hex("revision-two");
    if let Some(component) = revised
        .components
        .iter_mut()
        .find(|component| component.kind == ComponentKind::Prompt)
    {
        component.identity = Some(component_identity(ComponentKind::Prompt, b"v2"));
    }
    let historical = classify_applicability(&attachment, &revised, &policy, None);
    assert_eq!(
        historical,
        Applicability::Historical {
            changed: alloc::vec![ComponentKind::Prompt],
            revision_changed: true,
            expired: false
        }
    );
    let review = review_qualification(&historical);
    assert!(!review.can_request_scaffold_admission);
    assert!(!review.activates_scaffold);

    let decision = CompatibilityDecision {
        decision_identity: identity("decision"),
        attachment_identity: attachment.identity.clone(),
        from_profile_revision: hex(REVISION_LABEL),
        to_profile_revision: hex("revision-two"),
        task_family: TASK_FAMILY.into(),
        evidence_class: EvidenceClass::TaskFamilyEvaluation,
        waived: alloc::vec![ComponentKind::Prompt],
    };
    let reuse = classify_applicability(&attachment, &revised, &policy, Some(&decision));
    assert_eq!(
        reuse,
        Applicability::CompatibleReuse {
            decision_identity: identity("decision"),
            changed: alloc::vec![ComponentKind::Prompt]
        }
    );
    assert!(!review_qualification(&reuse).can_request_scaffold_admission);
    let revised_revision = hex("revision-two");
    let lattice = expected(&current.lattice_definition_identity, &revised_revision, &[]);
    assert_eq!(
        reason(&attachment.canonical, &lattice),
        RejectionReason::SubjectMismatch
    );

    let evaluator_waiver = CompatibilityDecision {
        waived: alloc::vec![ComponentKind::Prompt, ComponentKind::Verifier],
        ..decision.clone()
    };
    assert!(matches!(
        classify_applicability(&attachment, &revised, &policy, Some(&evaluator_waiver)),
        Applicability::Historical { .. }
    ));
    let other_attachment = CompatibilityDecision {
        attachment_identity: identity("another"),
        ..decision.clone()
    };
    assert!(matches!(
        classify_applicability(&attachment, &revised, &policy, Some(&other_attachment)),
        Applicability::Historical { .. }
    ));
    let expired = ApplicabilityPolicy {
        now_ms: VALID_UNTIL_MS,
        required: &[],
    };
    assert!(matches!(
        classify_applicability(&attachment, &revised, &expired, Some(&decision)),
        Applicability::Historical { expired: true, .. }
    ));
    let mut other_definition = subject();
    other_definition.lattice_definition_identity = identity("other-definition");
    assert_eq!(
        classify_applicability(&attachment, &other_definition, &policy, Some(&decision)),
        Applicability::Inapplicable(RejectionReason::SubjectMismatch)
    );
    let requires_role = ApplicabilityPolicy {
        now_ms: NOW_MS,
        required: &[ComponentKind::SemanticRole],
    };
    assert_eq!(
        classify_applicability(&attachment, &current, &requires_role, None),
        Applicability::Unusable {
            component: ComponentKind::SemanticRole
        }
    );
}
