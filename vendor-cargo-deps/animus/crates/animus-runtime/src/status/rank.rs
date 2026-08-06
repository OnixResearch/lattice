pub(super) fn newest_observation<'a>(
    observations: &[&'a animus_adapters::AgentStatusObservation],
) -> Option<super::types::SelectedObservation<'a>> {
    observations
        .iter()
        .enumerate()
        .max_by(|left, right| compare_observations(left.1, right.1).then_with(|| right.0.cmp(&left.0)))
        .map(|(index, observation)| super::types::SelectedObservation { observation, index })
}

fn compare_observations(
    left: &animus_adapters::AgentStatusObservation,
    right: &animus_adapters::AgentStatusObservation,
) -> core::cmp::Ordering {
    left.sequence().cmp(&right.sequence()).then_with(|| right.source_id().cmp(left.source_id()))
}

pub(super) fn newest_conflicts(
    trusted: &[&animus_adapters::AgentStatusObservation],
    sequence: animus_adapters::StatusSequence,
    selected_state: animus_adapters::SemanticAgentState,
) -> bool {
    trusted
        .iter()
        .any(|observation| observation.sequence() == sequence && observation.state() != selected_state)
}

pub(super) fn reject_newest_conflicts(
    observations: &[&animus_adapters::AgentStatusObservation],
    sequence: animus_adapters::StatusSequence,
    rejected: &mut super::types::RejectionCollector,
) {
    for observation in observations {
        if observation.sequence() == sequence {
            rejected.push(observation, super::types::RejectedReason::ConflictingLifecycleAuthority);
        }
    }
}

pub(super) fn reject_non_selected(
    observations: &[&animus_adapters::AgentStatusObservation],
    selected: super::types::SelectedObservation<'_>,
    same_sequence_reason: super::types::RejectedReason,
    rejected: &mut super::types::RejectionCollector,
) {
    for (index, observation) in observations.iter().enumerate() {
        if index == selected.index {
            continue;
        }
        let reason = if observation.sequence() == selected.observation.sequence() {
            same_sequence_reason
        } else {
            super::types::RejectedReason::OlderSequence
        };
        rejected.push(observation, reason);
    }
}

pub(super) fn reject_heuristics(
    observations: &[&animus_adapters::AgentStatusObservation],
    reason: super::types::RejectedReason,
    rejected: &mut super::types::RejectionCollector,
) {
    for observation in observations {
        rejected.push(observation, reason);
    }
}

pub(super) fn fallback_reason(signals: super::types::FallbackSignals) -> super::types::FallbackReason {
    if !signals.has_target {
        return super::types::FallbackReason::NoObservation;
    }
    if signals.has_stale_lifecycle {
        return super::types::FallbackReason::StaleLifecycle;
    }
    if signals.has_untrusted_lifecycle {
        return super::types::FallbackReason::UntrustedLifecycle;
    }
    super::types::FallbackReason::NoTrustedLifecycle
}

pub(super) const fn freshness_for_unknown(reason: super::types::FallbackReason) -> super::types::Freshness {
    match reason {
        super::types::FallbackReason::StaleLifecycle => super::types::Freshness::Stale,
        super::types::FallbackReason::ConflictingLifecycleAuthority => super::types::Freshness::Conflicting,
        super::types::FallbackReason::NoObservation
        | super::types::FallbackReason::NoTrustedLifecycle
        | super::types::FallbackReason::UntrustedLifecycle => super::types::Freshness::Missing,
    }
}
