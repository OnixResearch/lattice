/// Pure status-selection result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    target_id: String,
    semantic_state: animus_adapters::SemanticAgentState,
    message: Option<String>,
    presentation: animus_adapters::AgentStatusPresentation,
    explanation: super::explain::DecisionExplanation,
}

impl Selection {
    fn from_observation(parts: SelectionParts<'_>) -> Self {
        let selected_authority = super::types::AuthoritySummary::from_observation(parts.observation);
        let evidence_hashes = parts.observation.evidence_hashes().to_vec();
        Self {
            target_id: String::from(parts.request.target_id()),
            semantic_state: parts.observation.state(),
            message: parts.observation.message().map(String::from),
            presentation: parts.observation.presentation().clone(),
            explanation: super::explain::DecisionExplanation::new(super::explain::DecisionExplanationParts {
                target_id: parts.request.target_id(),
                selected_authority: Some(selected_authority),
                rejected_candidates: parts.rejected_candidates,
                freshness: super::types::Freshness::Current,
                fallback_reason: parts.fallback_reason,
                evidence_hashes,
                evidence_summary: parts.evidence_summary,
            }),
        }
    }

    fn unknown(
        request: &animus_adapters::HostStatusRequest,
        rejected_candidates: Vec<super::types::RejectedCandidate>,
        freshness: super::types::Freshness,
        fallback_reason: super::types::FallbackReason,
    ) -> Self {
        Self {
            target_id: String::from(request.target_id()),
            semantic_state: animus_adapters::SemanticAgentState::Unknown,
            message: None,
            presentation: animus_adapters::AgentStatusPresentation::empty(),
            explanation: super::explain::DecisionExplanation::new(super::explain::DecisionExplanationParts {
                target_id: request.target_id(),
                selected_authority: None,
                rejected_candidates,
                freshness,
                fallback_reason: Some(fallback_reason),
                evidence_hashes: Vec::new(),
                evidence_summary: super::safe::EvidenceSummary::unknown(),
            }),
        }
    }

    /// Returns the selected target identity.
    #[must_use]
    pub fn target_id(&self) -> &str {
        self.target_id.as_str()
    }

    /// Returns the selected semantic lifecycle state.
    #[must_use]
    pub const fn semantic_state(&self) -> animus_adapters::SemanticAgentState {
        self.semantic_state
    }

    /// Returns the selected bounded lifecycle message, if any.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Returns presentation-only metadata from the selected observation.
    #[must_use]
    pub const fn presentation(&self) -> &animus_adapters::AgentStatusPresentation {
        &self.presentation
    }

    /// Returns the bounded explanation for this decision.
    #[must_use]
    pub const fn explanation(&self) -> &super::explain::DecisionExplanation {
        &self.explanation
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SelectionParts<'a> {
    request: &'a animus_adapters::HostStatusRequest,
    observation: &'a animus_adapters::AgentStatusObservation,
    rejected_candidates: Vec<super::types::RejectedCandidate>,
    fallback_reason: Option<super::types::FallbackReason>,
    evidence_summary: super::safe::EvidenceSummary,
}

/// Selects one semantic status authority for a target without ambient I/O.
#[must_use]
pub fn select(
    request: &animus_adapters::HostStatusRequest,
    current_sequence: animus_adapters::StatusSequence,
    observations: &[animus_adapters::AgentStatusObservation],
) -> Selection {
    let mut rejected = super::types::RejectionCollector::with_capacity(observations.len());
    let classified = classify(request, current_sequence, observations, &mut rejected);
    if !classified.trusted.is_empty() {
        return select_trusted(request, &classified, rejected);
    }
    select_fallback(request, &classified, rejected)
}

fn classify<'a>(
    request: &animus_adapters::HostStatusRequest,
    current_sequence: animus_adapters::StatusSequence,
    observations: &'a [animus_adapters::AgentStatusObservation],
    rejected: &mut super::types::RejectionCollector,
) -> super::types::ClassifiedObservations<'a> {
    let mut trusted = Vec::with_capacity(observations.len());
    let mut heuristics = Vec::with_capacity(observations.len());
    let mut signals = super::types::FallbackSignals::default();
    assert!(trusted.capacity() >= observations.len());
    assert!(heuristics.capacity() >= observations.len());

    for observation in observations {
        if observation.target_id() != request.target_id() {
            rejected.push(observation, super::types::RejectedReason::TargetMismatch);
            continue;
        }
        signals.has_target = true;

        if !observation.is_current_at(current_sequence) {
            if matches!(
                observation.kind(),
                animus_adapters::AgentStatusObservationKind::TrustedLifecycle
                    | animus_adapters::AgentStatusObservationKind::UntrustedLifecycle
            ) {
                signals.has_stale_lifecycle = true;
            }
            rejected.push(observation, super::types::RejectedReason::Stale);
            continue;
        }

        match observation.kind() {
            animus_adapters::AgentStatusObservationKind::TrustedLifecycle => {
                trusted.push(observation);
            }
            animus_adapters::AgentStatusObservationKind::UntrustedLifecycle => {
                signals.has_untrusted_lifecycle = true;
                rejected.push(observation, super::types::RejectedReason::UntrustedLifecycle);
            }
            animus_adapters::AgentStatusObservationKind::HeuristicFallback => {
                heuristics.push(observation);
            }
        }
    }

    super::types::ClassifiedObservations {
        trusted,
        heuristics,
        signals,
    }
}

fn select_trusted(
    request: &animus_adapters::HostStatusRequest,
    classified: &super::types::ClassifiedObservations<'_>,
    mut rejected: super::types::RejectionCollector,
) -> Selection {
    assert!(!classified.trusted.is_empty());
    assert!(classified.trusted.len() <= classified.trusted.capacity());
    let Some(selected) = super::rank::newest_observation(classified.trusted.as_slice()) else {
        return Selection::unknown(
            request,
            rejected.into_vec(),
            super::types::Freshness::Missing,
            super::types::FallbackReason::NoTrustedLifecycle,
        );
    };
    let has_conflict = super::rank::newest_conflicts(
        classified.trusted.as_slice(),
        selected.observation.sequence(),
        selected.observation.state(),
    );

    if has_conflict {
        super::rank::reject_newest_conflicts(
            classified.trusted.as_slice(),
            selected.observation.sequence(),
            &mut rejected,
        );
        super::rank::reject_heuristics(
            classified.heuristics.as_slice(),
            super::types::RejectedReason::HeuristicSuppressed,
            &mut rejected,
        );
        return Selection::unknown(
            request,
            rejected.into_vec(),
            super::types::Freshness::Conflicting,
            super::types::FallbackReason::ConflictingLifecycleAuthority,
        );
    }

    super::rank::reject_non_selected(
        classified.trusted.as_slice(),
        selected,
        super::types::RejectedReason::DuplicateLifecycleAuthority,
        &mut rejected,
    );
    super::rank::reject_heuristics(
        classified.heuristics.as_slice(),
        super::types::RejectedReason::HeuristicSuppressed,
        &mut rejected,
    );
    Selection::from_observation(SelectionParts {
        request,
        observation: selected.observation,
        rejected_candidates: rejected.into_vec(),
        fallback_reason: None,
        evidence_summary: super::safe::EvidenceSummary::trusted_lifecycle(),
    })
}

fn select_fallback(
    request: &animus_adapters::HostStatusRequest,
    classified: &super::types::ClassifiedObservations<'_>,
    mut rejected: super::types::RejectionCollector,
) -> Selection {
    assert!(classified.trusted.is_empty());
    assert!(classified.heuristics.len() <= classified.heuristics.capacity());
    if let Some(selected) = super::rank::newest_observation(classified.heuristics.as_slice()) {
        super::rank::reject_non_selected(
            classified.heuristics.as_slice(),
            selected,
            super::types::RejectedReason::OlderSequence,
            &mut rejected,
        );
        return Selection::from_observation(SelectionParts {
            request,
            observation: selected.observation,
            rejected_candidates: rejected.into_vec(),
            fallback_reason: Some(super::rank::fallback_reason(classified.signals)),
            evidence_summary: super::safe::EvidenceSummary::heuristic_fallback(),
        });
    }

    let reason = super::rank::fallback_reason(classified.signals);
    Selection::unknown(request, rejected.into_vec(), super::rank::freshness_for_unknown(reason), reason)
}
