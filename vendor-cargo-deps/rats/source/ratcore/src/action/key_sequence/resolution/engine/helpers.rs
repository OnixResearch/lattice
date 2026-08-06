use std::collections::BTreeMap;

use super::super::super::ActionBindingId;
use super::super::super::KeySequence;
use super::super::super::LogicalKeystroke;
use super::super::BindingDecision;
use super::super::PendingContinuation;
use super::Match;
use super::MatchKind;
use super::Selection;

pub(super) fn ambiguous_selection<'a>(matches: &[Match<'a>], precedence: i16) -> Selection<'a> {
    let mut ids = matches.iter().map(|candidate| candidate.binding.id.clone()).collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    Selection {
        winner: None,
        winner_id: None,
        winner_decision: BindingDecision::Ambiguous,
        pending: Vec::new(),
        precedence: Some(precedence),
        ambiguous_ids: ids,
        modal_blocked: false,
        blocked_reason: None,
    }
}

pub(super) fn blocked_selection(precedence: i16, reason: String) -> Selection<'static> {
    Selection {
        winner: None,
        winner_id: None,
        winner_decision: BindingDecision::ModalBlocked,
        pending: Vec::new(),
        precedence: Some(precedence),
        ambiguous_ids: Vec::new(),
        modal_blocked: true,
        blocked_reason: Some(reason),
    }
}

pub(super) fn empty_selection() -> Selection<'static> {
    Selection {
        winner: None,
        winner_id: None,
        winner_decision: BindingDecision::Shadowed,
        pending: Vec::new(),
        precedence: None,
        ambiguous_ids: Vec::new(),
        modal_blocked: false,
        blocked_reason: None,
    }
}

pub(super) fn project_continuations(prefix: &KeySequence, matches: &[Match<'_>]) -> Vec<PendingContinuation> {
    let next_index = prefix.strokes().len();
    let mut grouped: BTreeMap<LogicalKeystroke, (Vec<ActionBindingId>, Vec<super::super::super::super::ActionId>)> =
        BTreeMap::new();
    for candidate in matches {
        let Some(stroke) = candidate.binding.sequence.strokes().get(next_index) else {
            continue;
        };
        let entry = grouped.entry(stroke.clone()).or_default();
        entry.0.push(candidate.binding.id.clone());
        entry.1.push(candidate.binding.action_id.clone());
    }
    grouped
        .into_iter()
        .map(|(stroke, (mut binding_ids, mut action_ids))| {
            binding_ids.sort();
            binding_ids.dedup();
            action_ids.sort();
            action_ids.dedup();
            PendingContinuation {
                stroke,
                binding_ids,
                action_ids,
            }
        })
        .collect()
}

pub(super) fn relation(input: &KeySequence, binding: &KeySequence) -> Option<MatchKind> {
    if input == binding {
        Some(MatchKind::Exact)
    } else if binding.starts_with(input) {
        Some(MatchKind::Prefix)
    } else {
        None
    }
}
