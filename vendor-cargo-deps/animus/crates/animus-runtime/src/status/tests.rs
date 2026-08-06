const TARGET_ID: &str = "pane:alpha";
const OTHER_TARGET_ID: &str = "pane:beta";
const LIFECYCLE_SOURCE: &str = "lifecycle:trusted";
const UNTRUSTED_SOURCE: &str = "lifecycle:untrusted";
const CONFLICT_SOURCE: &str = "lifecycle:conflict";
const HEURISTIC_SOURCE: &str = "heuristic:terminal";
const AGENT_LABEL: &str = "agent-alpha";
const CURRENT_SEQUENCE: animus_adapters::StatusSequence = animus_adapters::StatusSequence::new(12);
const OLDER_SEQUENCE: animus_adapters::StatusSequence = animus_adapters::StatusSequence::new(10);
const CURRENT_TTL: animus_adapters::StatusTtl =
    animus_adapters::StatusTtl::new(animus_adapters::StatusSequence::new(14));
const EXPIRED_TTL: animus_adapters::StatusTtl =
    animus_adapters::StatusTtl::new(animus_adapters::StatusSequence::new(11));
const VALID_HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const UNSAFE_FIXTURE_COUNT: usize = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ObservationSpec {
    kind: &'static str,
    state: &'static str,
    source_id: &'static str,
    target_id: &'static str,
    sequence: animus_adapters::StatusSequence,
    ttl: Option<animus_adapters::StatusTtl>,
}

fn request() -> animus_adapters::HostStatusRequest {
    animus_adapters::HostStatusRequest::new(TARGET_ID).expect("valid target should parse")
}

fn observation(spec: ObservationSpec) -> animus_adapters::AgentStatusObservation {
    animus_adapters::AgentStatusObservation::new(animus_adapters::AgentStatusObservationDraft {
        kind: spec.kind,
        state: spec.state,
        source_id: spec.source_id,
        target_id: spec.target_id,
        agent_label: AGENT_LABEL,
        session: None,
        message: Some("bounded status"),
        presentation: animus_adapters::AgentStatusPresentationDraft {
            title: Some("build pane"),
            display_agent_name: Some("Ralph"),
            activity_text: Some("checking"),
            custom_state_label: Some("needs input"),
        },
        sequence: spec.sequence,
        ttl: spec.ttl,
        evidence_hashes: &[VALID_HASH],
    })
    .expect("status observation should parse")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SpecFields {
    kind: &'static str,
    state: &'static str,
    source_id: &'static str,
}

fn spec(fields: SpecFields) -> ObservationSpec {
    ObservationSpec {
        kind: fields.kind,
        state: fields.state,
        source_id: fields.source_id,
        target_id: TARGET_ID,
        sequence: CURRENT_SEQUENCE,
        ttl: Some(CURRENT_TTL),
    }
}

#[test]
fn current_trusted_lifecycle_report_suppresses_heuristic_fallback() {
    let lifecycle = observation(spec(SpecFields {
        kind: "trusted_lifecycle",
        state: "working",
        source_id: LIFECYCLE_SOURCE,
    }));
    let heuristic = observation(spec(SpecFields {
        kind: "heuristic_fallback",
        state: "idle",
        source_id: HEURISTIC_SOURCE,
    }));

    let selection = crate::select_agent_status(&request(), CURRENT_SEQUENCE, &[lifecycle, heuristic]);

    assert_eq!(selection.semantic_state(), animus_adapters::SemanticAgentState::Working);
    assert_eq!(selection.message(), Some("bounded status"));
    assert_eq!(selection.presentation().custom_state_label(), Some("needs input"));
    assert_ne!(selection.semantic_state().as_str(), "needs input");
    let explanation = selection.explanation();
    assert_eq!(explanation.freshness(), crate::StatusFreshness::Current);
    assert_eq!(explanation.fallback_reason(), None);
    assert_eq!(explanation.evidence_hashes(), &[String::from(VALID_HASH)]);
    assert_eq!(
        explanation.selected_authority().expect("trusted authority should be selected").source_id(),
        LIFECYCLE_SOURCE
    );
    assert!(explanation.rejected_candidates().iter().any(|candidate| {
        candidate.authority().source_id() == HEURISTIC_SOURCE
            && candidate.reason() == crate::RejectedStatusReason::HeuristicSuppressed
    }));
}

#[test]
fn stale_lifecycle_degrades_to_current_heuristic_fallback() {
    let stale_lifecycle = observation(ObservationSpec {
        kind: "trusted_lifecycle",
        state: "working",
        source_id: LIFECYCLE_SOURCE,
        target_id: TARGET_ID,
        sequence: OLDER_SEQUENCE,
        ttl: Some(EXPIRED_TTL),
    });
    let heuristic = observation(spec(SpecFields {
        kind: "heuristic_fallback",
        state: "idle",
        source_id: HEURISTIC_SOURCE,
    }));

    let selection = crate::select_agent_status(&request(), CURRENT_SEQUENCE, &[stale_lifecycle, heuristic]);

    assert_eq!(selection.semantic_state(), animus_adapters::SemanticAgentState::Idle);
    assert_eq!(selection.explanation().fallback_reason(), Some(crate::StatusFallbackReason::StaleLifecycle));
    assert!(selection.explanation().rejected_candidates().iter().any(|candidate| {
        candidate.authority().source_id() == LIFECYCLE_SOURCE
            && candidate.reason() == crate::RejectedStatusReason::Stale
    }));
}

#[test]
fn untrusted_lifecycle_without_fallback_fails_closed_to_unknown() {
    let untrusted = observation(spec(SpecFields {
        kind: "untrusted_lifecycle",
        state: "done",
        source_id: UNTRUSTED_SOURCE,
    }));

    let selection = crate::select_agent_status(&request(), CURRENT_SEQUENCE, &[untrusted]);

    assert_eq!(selection.semantic_state(), animus_adapters::SemanticAgentState::Unknown);
    assert_eq!(selection.explanation().freshness(), crate::StatusFreshness::Missing);
    assert_eq!(selection.explanation().fallback_reason(), Some(crate::StatusFallbackReason::UntrustedLifecycle));
}

#[test]
fn conflicting_current_lifecycle_authorities_fail_closed_to_unknown() {
    let working = observation(spec(SpecFields {
        kind: "trusted_lifecycle",
        state: "working",
        source_id: LIFECYCLE_SOURCE,
    }));
    let blocked = observation(spec(SpecFields {
        kind: "trusted_lifecycle",
        state: "blocked",
        source_id: CONFLICT_SOURCE,
    }));
    let heuristic = observation(spec(SpecFields {
        kind: "heuristic_fallback",
        state: "idle",
        source_id: HEURISTIC_SOURCE,
    }));

    let selection = crate::select_agent_status(&request(), CURRENT_SEQUENCE, &[working, blocked, heuristic]);

    assert_eq!(selection.semantic_state(), animus_adapters::SemanticAgentState::Unknown);
    assert_eq!(selection.explanation().freshness(), crate::StatusFreshness::Conflicting);
    assert_eq!(
        selection.explanation().fallback_reason(),
        Some(crate::StatusFallbackReason::ConflictingLifecycleAuthority)
    );
    assert!(
        selection
            .explanation()
            .rejected_candidates()
            .iter()
            .any(|candidate| { candidate.reason() == crate::RejectedStatusReason::ConflictingLifecycleAuthority })
    );
}

#[test]
fn explanation_rejects_raw_evidence_leak_fixtures() {
    let fixtures = [
        ("RAW_PROMPT: summarize credentials", super::safe::RAW_PROMPT_MARKER),
        ("TERMINAL_SNAPSHOT: full pane text", super::safe::TERMINAL_SNAPSHOT_MARKER),
        ("PROVIDER_PAYLOAD: {json}", super::safe::PROVIDER_PAYLOAD_MARKER),
        ("API_KEY=secret", super::safe::API_KEY_MARKER),
        ("UNBOUNDED_DETECTOR_EVIDENCE: transcript", super::safe::UNBOUNDED_DETECTOR_MARKER),
    ];
    assert_eq!(fixtures.len(), UNSAFE_FIXTURE_COUNT);

    for (summary, marker) in fixtures {
        let error = crate::StatusEvidenceSummary::new(summary).expect_err("unsafe evidence should be rejected");
        assert_eq!(error, crate::StatusEvidenceError::UnsafeEvidenceMarker { marker });
    }
}

#[test]
fn selection_consumes_typed_observations_without_ambient_inputs() {
    let other_target = observation(ObservationSpec {
        target_id: OTHER_TARGET_ID,
        ..spec(SpecFields {
            kind: "trusted_lifecycle",
            state: "blocked",
            source_id: LIFECYCLE_SOURCE,
        })
    });
    let heuristic = observation(spec(SpecFields {
        kind: "heuristic_fallback",
        state: "idle",
        source_id: HEURISTIC_SOURCE,
    }));
    let observations = [other_target, heuristic];

    let first_selection = crate::select_agent_status(&request(), CURRENT_SEQUENCE, &observations);
    let second_selection = crate::select_agent_status(&request(), CURRENT_SEQUENCE, &observations);

    assert_eq!(first_selection, second_selection);
    assert_eq!(first_selection.semantic_state(), animus_adapters::SemanticAgentState::Idle);
    assert!(
        first_selection
            .explanation()
            .rejected_candidates()
            .iter()
            .any(|candidate| { candidate.reason() == crate::RejectedStatusReason::TargetMismatch })
    );
}
