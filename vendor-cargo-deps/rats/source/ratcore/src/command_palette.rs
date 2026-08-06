//! Renderer-neutral command palette state, transitions, and diagnostics.
//!
//! Renderers own event translation and effects. This module owns command
//! filtering, active-row repair, navigation, selection, and cancellation.

use std::cmp::Reverse;
use std::collections::BTreeSet;

use crate::action_registry::ActionId;
use crate::action_registry::CommandPaletteItemProjection;
use crate::fuzzy::FuzzyScoreInput;
use crate::fuzzy::MatchOffset;
use crate::fuzzy::fuzzy_score;

/// Policy used when keyboard navigation reaches the first or last match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPaletteNavigationPolicy {
    /// Continue at the opposite edge.
    Wrap,
    /// Stay on the current edge row.
    Clamp,
    /// Reject the movement and emit a stable diagnostic.
    Reject,
}

/// An action available to the command palette model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteCandidate {
    action_id: ActionId,
    label: String,
    disabled_reason: Option<String>,
}

impl From<&CommandPaletteItemProjection> for CommandPaletteCandidate {
    fn from(projection: &CommandPaletteItemProjection) -> Self {
        if projection.enabled {
            Self::enabled(projection.action_id.clone(), projection.label.clone())
        } else {
            Self::disabled(
                projection.action_id.clone(),
                projection.label.clone(),
                projection.disabled_reason.clone().unwrap_or_else(|| "action registry reports disabled".to_string()),
            )
        }
    }
}

impl CommandPaletteCandidate {
    /// Creates an enabled action candidate.
    #[must_use]
    pub fn enabled(action_id: impl Into<ActionId>, label: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            label: label.into(),
            disabled_reason: None,
        }
    }

    /// Creates a disabled action candidate with an explicit reason.
    #[must_use]
    pub fn disabled(action_id: impl Into<ActionId>, label: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            label: label.into(),
            disabled_reason: Some(reason.into()),
        }
    }

    /// Returns the stable action identity.
    #[must_use]
    pub fn action_id(&self) -> &ActionId {
        &self.action_id
    }

    /// Returns the human-readable action label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the reason this candidate cannot be activated.
    #[must_use]
    pub fn disabled_reason(&self) -> Option<&str> {
        self.disabled_reason.as_deref()
    }

    /// Returns whether this candidate can be activated.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.disabled_reason.is_none()
    }
}

/// A filtered command palette row with renderer-neutral highlight metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteMatch {
    candidate_index: usize,
    score: i32,
    positions: Vec<usize>,
}

impl CommandPaletteMatch {
    /// Returns the candidate index in [`CommandPaletteModel::candidates`].
    #[must_use]
    pub fn candidate_index(&self) -> usize {
        self.candidate_index
    }

    /// Returns the fuzzy-match score.
    #[must_use]
    pub fn score(&self) -> i32 {
        self.score
    }

    /// Returns character positions to highlight in the candidate label.
    #[must_use]
    pub fn positions(&self) -> &[usize] {
        &self.positions
    }
}

/// Authoritative renderer-neutral command palette state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteModel {
    candidates: Vec<CommandPaletteCandidate>,
    matches: Vec<CommandPaletteMatch>,
    query: String,
    active_action_id: Option<ActionId>,
    selected_action_id: Option<ActionId>,
    open: bool,
    cancelled: bool,
    navigation_policy: CommandPaletteNavigationPolicy,
}

impl CommandPaletteModel {
    /// Builds a model and reports malformed or ambiguous initial candidates.
    #[must_use]
    pub fn build(
        candidates: Vec<CommandPaletteCandidate>,
        navigation_policy: CommandPaletteNavigationPolicy,
        open: bool,
    ) -> CommandPaletteBuild {
        let (candidates, diagnostics) = validated_initial_candidates(candidates);
        let mut model = Self {
            candidates,
            matches: Vec::new(),
            query: String::new(),
            active_action_id: None,
            selected_action_id: None,
            open,
            cancelled: false,
            navigation_policy,
        };
        model.recompute_matches(false);
        CommandPaletteBuild { model, diagnostics }
    }

    /// Returns all accepted candidates in source order.
    #[must_use]
    pub fn candidates(&self) -> &[CommandPaletteCandidate] {
        &self.candidates
    }

    /// Returns filtered candidates in deterministic score order.
    #[must_use]
    pub fn matches(&self) -> &[CommandPaletteMatch] {
        &self.matches
    }

    /// Returns the current query.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns the active action, if a matching row exists.
    #[must_use]
    pub fn active_action_id(&self) -> Option<&ActionId> {
        self.active_action_id.as_ref()
    }

    /// Returns the last selected action.
    #[must_use]
    pub fn selected_action_id(&self) -> Option<&ActionId> {
        self.selected_action_id.as_ref()
    }

    /// Returns whether the palette is open.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Returns whether the latest close was a cancellation.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Returns the configured edge-navigation policy.
    #[must_use]
    pub fn navigation_policy(&self) -> CommandPaletteNavigationPolicy {
        self.navigation_policy
    }

    /// Returns a candidate by typed identity.
    #[must_use]
    pub fn candidate(&self, action_id: &ActionId) -> Option<&CommandPaletteCandidate> {
        self.candidates.iter().find(|candidate| candidate.action_id() == action_id)
    }

    fn recompute_matches(&mut self, reset_active: bool) {
        let previous_active = if reset_active {
            None
        } else {
            self.active_action_id.clone()
        };
        self.matches = score_candidates(&self.candidates, &self.query);
        self.active_action_id = previous_active
            .filter(|action_id| {
                self.matches.iter().any(|row| self.candidates[row.candidate_index()].action_id() == action_id)
            })
            .or_else(|| self.first_match_action_id());
    }

    fn first_match_action_id(&self) -> Option<ActionId> {
        self.matches.first().map(|row| self.candidates[row.candidate_index()].action_id().clone())
    }

    fn active_match_index(&self) -> Option<usize> {
        let active = self.active_action_id.as_ref()?;
        self.matches.iter().position(|row| self.candidates[row.candidate_index()].action_id() == active)
    }
}

/// Result of initial command palette model construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteBuild {
    /// Accepted model. Invalid initial candidates are omitted.
    pub model: CommandPaletteModel,
    /// Stable diagnostics for omitted candidates.
    pub diagnostics: Vec<CommandPaletteDiagnostic>,
}

/// Direction requested by command palette navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPaletteDirection {
    /// Move to the previous filtered action.
    Previous,
    /// Move to the next filtered action.
    Next,
}

/// Typed command palette input understood by [`reduce_command_palette`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandPaletteMessage {
    /// Open and reset the palette for a new interaction.
    Open,
    /// Replace the query and reset the active row to the first result.
    ReplaceQuery(String),
    /// Replace the candidate projection.
    ReplaceCandidates(Vec<CommandPaletteCandidate>),
    /// Move according to the model's explicit navigation policy.
    Move(CommandPaletteDirection),
    /// Make a visible action the active row.
    Hover(ActionId),
    /// Activate the current row.
    Activate,
    /// Activate an explicit action, for pointer and replay adapters.
    ActivateAction(ActionId),
    /// Cancel and request that the renderer close the palette.
    Cancel,
}

/// Effect request emitted by an accepted transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandPaletteOutput {
    /// Invoke the selected action through renderer/application infrastructure.
    Select(ActionId),
    /// Close the palette without selecting an action.
    Close,
}

/// Stable command palette diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandPaletteDiagnosticCode {
    /// An action identity is empty.
    EmptyActionId,
    /// More than one candidate has the same action identity.
    DuplicateActionId,
    /// Query text contains a control character.
    MalformedQuery,
    /// A message references an action absent from the candidate projection.
    UnknownAction,
    /// A hover message references an action absent from current results.
    FilteredAction,
    /// Activation targeted a disabled action.
    DisabledAction,
    /// Activation or movement requires a matching row, but there are none.
    NoMatchingAction,
    /// Edge movement was rejected by policy.
    NavigationBoundary,
}

/// Stable renderer-neutral diagnostic emitted by the reducer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteDiagnostic {
    /// Machine-readable diagnostic code.
    pub code: CommandPaletteDiagnosticCode,
    /// Typed action identity associated with the diagnostic, when available.
    pub action_id: Option<ActionId>,
    /// Human-readable context suitable for logs and development UI.
    pub message: String,
}

impl CommandPaletteDiagnostic {
    fn new(code: CommandPaletteDiagnosticCode, action_id: Option<ActionId>, message: impl Into<String>) -> Self {
        Self {
            code,
            action_id,
            message: message.into(),
        }
    }
}

/// Complete result of one command palette reduction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteTransition {
    /// Next model, equal to the input model when the message was rejected.
    pub model: CommandPaletteModel,
    /// Whether the reducer accepted the message.
    pub accepted: bool,
    /// Optional effect request for the renderer/application adapter.
    pub output: Option<CommandPaletteOutput>,
    /// Stable diagnostics produced while interpreting the message.
    pub diagnostics: Vec<CommandPaletteDiagnostic>,
}

impl CommandPaletteTransition {
    fn accepted(model: CommandPaletteModel, output: Option<CommandPaletteOutput>) -> Self {
        Self {
            model,
            accepted: true,
            output,
            diagnostics: Vec::new(),
        }
    }

    fn rejected(model: &CommandPaletteModel, diagnostic: CommandPaletteDiagnostic) -> Self {
        Self {
            model: model.clone(),
            accepted: false,
            output: None,
            diagnostics: vec![diagnostic],
        }
    }
}

/// Applies one message without renderer or asynchronous effects.
#[must_use]
pub fn reduce_command_palette(model: &CommandPaletteModel, message: CommandPaletteMessage) -> CommandPaletteTransition {
    match message {
        CommandPaletteMessage::Open => {
            let mut next = model.clone();
            next.query.clear();
            next.selected_action_id = None;
            next.cancelled = false;
            next.open = true;
            next.recompute_matches(true);
            CommandPaletteTransition::accepted(next, None)
        }
        CommandPaletteMessage::ReplaceQuery(query) => replace_query(model, query),
        CommandPaletteMessage::ReplaceCandidates(candidates) => replace_candidates(model, candidates),
        CommandPaletteMessage::Move(direction) => move_active(model, direction),
        CommandPaletteMessage::Hover(action_id) => hover_action(model, action_id),
        CommandPaletteMessage::Activate => activate_current(model),
        CommandPaletteMessage::ActivateAction(action_id) => activate_action(model, &action_id),
        CommandPaletteMessage::Cancel => {
            let mut next = model.clone();
            next.open = false;
            next.cancelled = true;
            CommandPaletteTransition::accepted(next, Some(CommandPaletteOutput::Close))
        }
    }
}

fn replace_query(model: &CommandPaletteModel, query: String) -> CommandPaletteTransition {
    if query.chars().any(char::is_control) {
        return CommandPaletteTransition::rejected(
            model,
            CommandPaletteDiagnostic::new(
                CommandPaletteDiagnosticCode::MalformedQuery,
                None,
                "command palette query must not contain control characters",
            ),
        );
    }
    let mut next = model.clone();
    next.query = query;
    next.recompute_matches(true);
    CommandPaletteTransition::accepted(next, None)
}

fn replace_candidates(
    model: &CommandPaletteModel,
    candidates: Vec<CommandPaletteCandidate>,
) -> CommandPaletteTransition {
    if let Some(diagnostic) = validate_candidates(&candidates).into_iter().next() {
        return CommandPaletteTransition::rejected(model, diagnostic);
    }
    let mut next = model.clone();
    next.candidates = candidates;
    next.recompute_matches(false);
    CommandPaletteTransition::accepted(next, None)
}

fn move_active(model: &CommandPaletteModel, direction: CommandPaletteDirection) -> CommandPaletteTransition {
    let Some(current_index) = model.active_match_index() else {
        return CommandPaletteTransition::rejected(
            model,
            CommandPaletteDiagnostic::new(
                CommandPaletteDiagnosticCode::NoMatchingAction,
                None,
                "command palette has no matching action to navigate",
            ),
        );
    };
    let final_index = model.matches.len().saturating_sub(1);
    let at_boundary = matches!(direction, CommandPaletteDirection::Previous) && current_index == 0
        || matches!(direction, CommandPaletteDirection::Next) && current_index == final_index;
    if at_boundary && model.navigation_policy == CommandPaletteNavigationPolicy::Reject {
        return CommandPaletteTransition::rejected(
            model,
            CommandPaletteDiagnostic::new(
                CommandPaletteDiagnosticCode::NavigationBoundary,
                model.active_action_id.clone(),
                "command palette movement crossed a configured navigation boundary",
            ),
        );
    }
    let next_index = match (direction, model.navigation_policy, at_boundary) {
        (CommandPaletteDirection::Previous, CommandPaletteNavigationPolicy::Wrap, true) => final_index,
        (CommandPaletteDirection::Next, CommandPaletteNavigationPolicy::Wrap, true) => 0,
        (_, CommandPaletteNavigationPolicy::Clamp, true) => current_index,
        (CommandPaletteDirection::Previous, _, false) => current_index.saturating_sub(1),
        (CommandPaletteDirection::Next, _, false) => current_index.saturating_add(1),
        (_, CommandPaletteNavigationPolicy::Reject, true) => unreachable!("handled above"),
    };
    let mut next = model.clone();
    next.active_action_id = Some(model.candidates[model.matches[next_index].candidate_index()].action_id().clone());
    CommandPaletteTransition::accepted(next, None)
}

fn hover_action(model: &CommandPaletteModel, action_id: ActionId) -> CommandPaletteTransition {
    if model.candidate(&action_id).is_none() {
        return CommandPaletteTransition::rejected(
            model,
            CommandPaletteDiagnostic::new(
                CommandPaletteDiagnosticCode::UnknownAction,
                Some(action_id),
                "command palette hover referenced an unknown action",
            ),
        );
    }
    if !model.matches.iter().any(|row| model.candidates[row.candidate_index()].action_id() == &action_id) {
        return CommandPaletteTransition::rejected(
            model,
            CommandPaletteDiagnostic::new(
                CommandPaletteDiagnosticCode::FilteredAction,
                Some(action_id),
                "command palette hover referenced an action outside current results",
            ),
        );
    }
    let mut next = model.clone();
    next.active_action_id = Some(action_id);
    CommandPaletteTransition::accepted(next, None)
}

fn activate_current(model: &CommandPaletteModel) -> CommandPaletteTransition {
    let Some(action_id) = model.active_action_id.as_ref() else {
        return CommandPaletteTransition::rejected(
            model,
            CommandPaletteDiagnostic::new(
                CommandPaletteDiagnosticCode::NoMatchingAction,
                None,
                "command palette has no matching action to activate",
            ),
        );
    };
    activate_action(model, action_id)
}

fn activate_action(model: &CommandPaletteModel, action_id: &ActionId) -> CommandPaletteTransition {
    let Some(candidate) = model.candidate(action_id) else {
        return CommandPaletteTransition::rejected(
            model,
            CommandPaletteDiagnostic::new(
                CommandPaletteDiagnosticCode::UnknownAction,
                Some(action_id.clone()),
                "command palette activation referenced an unknown action",
            ),
        );
    };
    if let Some(reason) = candidate.disabled_reason() {
        return CommandPaletteTransition::rejected(
            model,
            CommandPaletteDiagnostic::new(
                CommandPaletteDiagnosticCode::DisabledAction,
                Some(action_id.clone()),
                format!("command palette action is disabled: {reason}"),
            ),
        );
    }
    let mut next = model.clone();
    next.selected_action_id = Some(action_id.clone());
    next.cancelled = false;
    next.open = false;
    CommandPaletteTransition::accepted(next, Some(CommandPaletteOutput::Select(action_id.clone())))
}

fn score_candidates(candidates: &[CommandPaletteCandidate], query: &str) -> Vec<CommandPaletteMatch> {
    let mut matches = candidates
        .iter()
        .enumerate()
        .filter_map(|(candidate_index, candidate)| {
            let label_match = fuzzy_score(FuzzyScoreInput {
                text: candidate.label(),
                query,
            });
            let id_match = fuzzy_score(FuzzyScoreInput {
                text: candidate.action_id().as_str(),
                query,
            });
            match (label_match, id_match) {
                (None, None) => None,
                (Some(label), Some(id)) if id.score > label.score => Some(CommandPaletteMatch {
                    candidate_index,
                    score: id.score,
                    positions: Vec::new(),
                }),
                (Some(label), _) => Some(CommandPaletteMatch {
                    candidate_index,
                    score: label.score,
                    positions: label.positions.into_iter().map(MatchOffset::get).collect(),
                }),
                (None, Some(id)) => Some(CommandPaletteMatch {
                    candidate_index,
                    score: id.score,
                    positions: Vec::new(),
                }),
            }
        })
        .collect::<Vec<_>>();
    matches.sort_by_key(|row| Reverse(row.score));
    matches
}

fn validated_initial_candidates(
    candidates: Vec<CommandPaletteCandidate>,
) -> (Vec<CommandPaletteCandidate>, Vec<CommandPaletteDiagnostic>) {
    let mut accepted = Vec::with_capacity(candidates.len());
    let mut diagnostics = Vec::new();
    let mut action_ids = BTreeSet::new();
    for candidate in candidates {
        if candidate.action_id().is_empty() {
            diagnostics.push(CommandPaletteDiagnostic::new(
                CommandPaletteDiagnosticCode::EmptyActionId,
                Some(candidate.action_id().clone()),
                "command palette action ID must not be empty",
            ));
        } else if !action_ids.insert(candidate.action_id().clone()) {
            diagnostics.push(CommandPaletteDiagnostic::new(
                CommandPaletteDiagnosticCode::DuplicateActionId,
                Some(candidate.action_id().clone()),
                "command palette action IDs must be unique",
            ));
        } else {
            accepted.push(candidate);
        }
    }
    (accepted, diagnostics)
}

fn validate_candidates(candidates: &[CommandPaletteCandidate]) -> Vec<CommandPaletteDiagnostic> {
    let mut action_ids = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for candidate in candidates {
        if candidate.action_id().is_empty() {
            diagnostics.push(CommandPaletteDiagnostic::new(
                CommandPaletteDiagnosticCode::EmptyActionId,
                Some(candidate.action_id().clone()),
                "command palette action ID must not be empty",
            ));
        } else if !action_ids.insert(candidate.action_id()) {
            diagnostics.push(CommandPaletteDiagnostic::new(
                CommandPaletteDiagnosticCode::DuplicateActionId,
                Some(candidate.action_id().clone()),
                "command palette action IDs must be unique",
            ));
        }
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidates() -> Vec<CommandPaletteCandidate> {
        vec![
            CommandPaletteCandidate::enabled("action.open", "Open file"),
            CommandPaletteCandidate::disabled("action.build", "Build project", "build is running"),
            CommandPaletteCandidate::enabled("action.settings", "Settings"),
        ]
    }

    fn model() -> CommandPaletteModel {
        CommandPaletteModel::build(candidates(), CommandPaletteNavigationPolicy::Wrap, true).model
    }

    #[test]
    fn query_filters_and_resets_active_action() {
        let transition = reduce_command_palette(&model(), CommandPaletteMessage::ReplaceQuery("set".to_string()));

        assert!(transition.accepted);
        assert_eq!(transition.model.matches().len(), 1);
        assert_eq!(transition.model.active_action_id(), Some(&ActionId::new("action.settings")));
        assert!(!transition.model.matches()[0].positions().is_empty());
    }

    #[test]
    fn fuzzy_and_exact_queries_use_shared_scoring() {
        let exact = reduce_command_palette(&model(), CommandPaletteMessage::ReplaceQuery("Settings".to_string()));
        assert_eq!(exact.model.matches().len(), 1);
        assert_eq!(exact.model.active_action_id(), Some(&ActionId::new("action.settings")));

        let fuzzy = reduce_command_palette(&model(), CommandPaletteMessage::ReplaceQuery("stg".to_string()));
        assert_eq!(fuzzy.model.matches().len(), 1);
        assert_eq!(fuzzy.model.active_action_id(), Some(&ActionId::new("action.settings")));
    }

    #[test]
    fn navigation_wraps_in_both_directions() {
        let original = model();
        let previous =
            reduce_command_palette(&original, CommandPaletteMessage::Move(CommandPaletteDirection::Previous));
        assert_eq!(previous.model.active_action_id(), Some(&ActionId::new("action.settings")));
        let next = reduce_command_palette(&previous.model, CommandPaletteMessage::Move(CommandPaletteDirection::Next));
        assert_eq!(next.model.active_action_id(), original.active_action_id());
    }

    #[test]
    fn disabled_activation_emits_diagnostic_and_preserves_state() {
        let original = model();
        let transition =
            reduce_command_palette(&original, CommandPaletteMessage::ActivateAction(ActionId::new("action.build")));

        assert!(!transition.accepted);
        assert_eq!(transition.model, original);
        assert_eq!(transition.diagnostics[0].code, CommandPaletteDiagnosticCode::DisabledAction);
    }

    #[test]
    fn stale_hover_is_rejected_without_state_change() {
        let original = model();
        let transition =
            reduce_command_palette(&original, CommandPaletteMessage::Hover(ActionId::new("action.missing")));

        assert!(!transition.accepted);
        assert_eq!(transition.model, original);
        assert_eq!(transition.diagnostics[0].code, CommandPaletteDiagnosticCode::UnknownAction);
    }

    #[test]
    fn empty_results_reject_movement_and_activation() {
        let filtered = reduce_command_palette(&model(), CommandPaletteMessage::ReplaceQuery("zzzz".to_string())).model;
        for message in [
            CommandPaletteMessage::Move(CommandPaletteDirection::Next),
            CommandPaletteMessage::Activate,
        ] {
            let transition = reduce_command_palette(&filtered, message);
            assert!(!transition.accepted);
            assert_eq!(transition.model, filtered);
            assert_eq!(transition.diagnostics[0].code, CommandPaletteDiagnosticCode::NoMatchingAction);
        }
    }

    #[test]
    fn candidate_replacement_repairs_a_stale_active_action() {
        let active_build =
            reduce_command_palette(&model(), CommandPaletteMessage::Hover(ActionId::new("action.build"))).model;
        let transition = reduce_command_palette(
            &active_build,
            CommandPaletteMessage::ReplaceCandidates(vec![CommandPaletteCandidate::enabled(
                "action.new",
                "New action",
            )]),
        );

        assert!(transition.accepted);
        assert_eq!(transition.model.active_action_id(), Some(&ActionId::new("action.new")));
    }

    #[test]
    fn candidate_reordering_preserves_active_identity() {
        let active_settings =
            reduce_command_palette(&model(), CommandPaletteMessage::Hover(ActionId::new("action.settings"))).model;
        let transition = reduce_command_palette(
            &active_settings,
            CommandPaletteMessage::ReplaceCandidates(vec![
                CommandPaletteCandidate::enabled("action.settings", "Settings"),
                CommandPaletteCandidate::enabled("action.open", "Open file"),
            ]),
        );

        assert!(transition.accepted);
        assert_eq!(transition.model.active_action_id(), Some(&ActionId::new("action.settings")));
    }

    #[test]
    fn candidate_disablement_preserves_identity_but_blocks_activation() {
        let active_settings =
            reduce_command_palette(&model(), CommandPaletteMessage::Hover(ActionId::new("action.settings"))).model;
        let changed = reduce_command_palette(
            &active_settings,
            CommandPaletteMessage::ReplaceCandidates(vec![
                CommandPaletteCandidate::enabled("action.open", "Open file"),
                CommandPaletteCandidate::disabled("action.settings", "Settings", "policy"),
            ]),
        )
        .model;
        let activation = reduce_command_palette(&changed, CommandPaletteMessage::Activate);

        assert_eq!(changed.active_action_id(), Some(&ActionId::new("action.settings")));
        assert!(!activation.accepted);
        assert_eq!(activation.model, changed);
        assert_eq!(activation.diagnostics[0].code, CommandPaletteDiagnosticCode::DisabledAction);
    }

    #[test]
    fn malformed_candidate_replacement_is_rejected() {
        let original = model();
        let transition = reduce_command_palette(
            &original,
            CommandPaletteMessage::ReplaceCandidates(vec![CommandPaletteCandidate::enabled("", "Missing ID")]),
        );

        assert!(!transition.accepted);
        assert_eq!(transition.model, original);
        assert_eq!(transition.diagnostics[0].code, CommandPaletteDiagnosticCode::EmptyActionId);
    }

    #[test]
    fn build_omits_duplicate_ids_with_stable_diagnostic() {
        let build = CommandPaletteModel::build(
            vec![
                CommandPaletteCandidate::enabled("same", "First"),
                CommandPaletteCandidate::enabled("same", "Second"),
            ],
            CommandPaletteNavigationPolicy::Wrap,
            false,
        );

        assert_eq!(build.model.candidates().len(), 1);
        assert_eq!(build.diagnostics[0].code, CommandPaletteDiagnosticCode::DuplicateActionId);
    }

    #[test]
    fn open_resets_query_selection_and_cancellation() {
        let queried =
            reduce_command_palette(&model(), CommandPaletteMessage::ReplaceQuery("settings".to_string())).model;
        let selected = reduce_command_palette(&queried, CommandPaletteMessage::Activate).model;
        let cancelled = reduce_command_palette(&selected, CommandPaletteMessage::Cancel).model;

        let opened = reduce_command_palette(&cancelled, CommandPaletteMessage::Open);

        assert!(opened.accepted);
        assert!(opened.model.is_open());
        assert_eq!(opened.model.query(), "");
        assert_eq!(opened.model.selected_action_id(), None);
        assert!(!opened.model.is_cancelled());
        assert_eq!(opened.model.active_action_id(), Some(&ActionId::new("action.open")));
    }

    #[test]
    fn malformed_query_is_diagnostic_and_preserves_state() {
        let original = model();
        let transition =
            reduce_command_palette(&original, CommandPaletteMessage::ReplaceQuery("bad\nquery".to_string()));

        assert!(!transition.accepted);
        assert_eq!(transition.model, original);
        assert_eq!(transition.diagnostics[0].code, CommandPaletteDiagnosticCode::MalformedQuery);
    }

    #[test]
    fn selection_and_cancel_emit_explicit_effect_requests() {
        let selected = reduce_command_palette(&model(), CommandPaletteMessage::Activate);
        assert_eq!(selected.output, Some(CommandPaletteOutput::Select(ActionId::new("action.open"))));
        assert!(!selected.model.is_open());

        let cancelled = reduce_command_palette(&model(), CommandPaletteMessage::Cancel);
        assert_eq!(cancelled.output, Some(CommandPaletteOutput::Close));
        assert!(cancelled.model.is_cancelled());
    }

    #[test]
    fn clamp_navigation_policy_accepts_and_stays_at_boundary() {
        let original = CommandPaletteModel::build(candidates(), CommandPaletteNavigationPolicy::Clamp, true).model;
        let transition =
            reduce_command_palette(&original, CommandPaletteMessage::Move(CommandPaletteDirection::Previous));

        assert!(transition.accepted);
        assert_eq!(transition.model, original);
        assert!(transition.diagnostics.is_empty());
    }

    #[test]
    fn reject_navigation_policy_preserves_boundary_state() {
        let original = CommandPaletteModel::build(candidates(), CommandPaletteNavigationPolicy::Reject, true).model;
        let transition =
            reduce_command_palette(&original, CommandPaletteMessage::Move(CommandPaletteDirection::Previous));

        assert!(!transition.accepted);
        assert_eq!(transition.model, original);
        assert_eq!(transition.diagnostics[0].code, CommandPaletteDiagnosticCode::NavigationBoundary);
    }
}
