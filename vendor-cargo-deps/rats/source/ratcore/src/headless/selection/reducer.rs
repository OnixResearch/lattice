mod catalog;
mod navigation;

/// Renderer-neutral tabs interaction message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabsMessage {
    /// Move roving focus to the previous eligible trigger.
    MovePrevious,
    /// Move roving focus to the next eligible trigger.
    MoveNext,
    /// Move focus to the first eligible trigger.
    MoveFirst,
    /// Move focus to the last eligible trigger.
    MoveLast,
    /// Focus one stable trigger identity.
    FocusTab(super::model::TabId),
    /// Propose selection of the focused trigger.
    ActivateFocused,
    /// Focus and propose selection after a primary pointer activation.
    PointerActivate(super::model::TabId),
    /// Commit one caller-accepted selection proposal.
    CommitSelection(super::model::TabId),
    /// Atomically replace the catalog and deterministically repair identities.
    ReplaceCatalog(super::model::TabsCatalog),
    /// Change tab-list orientation.
    SetOrientation(super::model::TabsOrientation),
    /// Change manual or automatic activation policy.
    SetActivationPolicy(super::model::TabsActivationPolicy),
}

/// Cause attached to a caller-controlled selection proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsIntentCause {
    /// Focus moved while automatic activation was configured.
    AutomaticFocus,
    /// Manual keyboard activation targeted the focused trigger.
    KeyboardActivation,
    /// Primary pointer activation targeted a trigger.
    PointerActivation,
    /// Catalog replacement required deterministic repair.
    CatalogRepair,
}

/// Typed caller output from a tabs transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabsIntent {
    /// Ask the caller to accept a new selected identity.
    SelectionProposed {
        /// Stable proposed trigger identity.
        tab_id: super::model::TabId,
        /// Semantic proposal cause.
        cause: TabsIntentCause,
    },
}

/// Ordered reducer event used to verify callback and focus ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabsTransitionEvent {
    /// Roving focus changed before any automatic selection callback.
    FocusChanged(super::model::TabId),
    /// Caller-controlled selection was proposed.
    SelectionProposed(super::model::TabId),
    /// Caller committed accepted selected state.
    SelectionCommitted(super::model::TabId),
    /// Catalog was accepted before repaired identities were reported.
    CatalogReplaced,
    /// Orientation policy changed.
    OrientationChanged(super::model::TabsOrientation),
    /// Activation policy changed.
    ActivationPolicyChanged(super::model::TabsActivationPolicy),
}

/// State mutation classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsTransitionKind {
    /// Accepted interaction state or caller-controlled state changed.
    Accepted,
    /// Message was valid but changed no accepted state.
    Ignored,
    /// Message was rejected and state was preserved.
    Rejected,
}

/// Logical projection invalidation requested by a tabs transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsInvalidation {
    /// No renderer refresh is required.
    None,
    /// Renderer should refresh from the shared immutable projection.
    Immediate,
}

/// Complete deterministic tabs transition output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabsTransition {
    /// Accepted, ignored, or rejected classification.
    pub kind: TabsTransitionKind,
    /// Typed caller-controlled selection proposals.
    pub intents: Vec<TabsIntent>,
    /// Stable ordered focus/callback/catalog events.
    pub events: Vec<TabsTransitionEvent>,
    /// Stable rejection diagnostics.
    pub diagnostics: Vec<super::model::TabsDiagnostic>,
    /// Logical renderer invalidation.
    pub invalidation: TabsInvalidation,
}

/// Applies one renderer-neutral tabs message.
#[must_use]
pub fn reduce_tabs(state: &mut super::model::TabsState, message: TabsMessage) -> TabsTransition {
    match message {
        TabsMessage::MovePrevious => navigation::move_focus(state, navigation::MoveTarget::Previous),
        TabsMessage::MoveNext => navigation::move_focus(state, navigation::MoveTarget::Next),
        TabsMessage::MoveFirst => navigation::move_focus(state, navigation::MoveTarget::First),
        TabsMessage::MoveLast => navigation::move_focus(state, navigation::MoveTarget::Last),
        TabsMessage::FocusTab(id) => navigation::focus_tab(state, id, navigation::FocusCause::Semantic),
        TabsMessage::ActivateFocused => navigation::activate_focused(state),
        TabsMessage::PointerActivate(id) => navigation::pointer_activate(state, id),
        TabsMessage::CommitSelection(id) => navigation::commit(state, id),
        TabsMessage::ReplaceCatalog(next) => catalog::replace(state, next),
        TabsMessage::SetOrientation(orientation) => catalog::set_orientation(state, orientation),
        TabsMessage::SetActivationPolicy(policy) => catalog::set_activation(state, policy),
    }
}

pub(super) fn rejected(code: super::model::TabsDiagnosticCode, subject: &str) -> TabsTransition {
    rejected_issues(vec![super::model::validation::tabs_issue(code, subject)])
}

pub(super) fn rejected_issues(diagnostics: Vec<super::model::TabsDiagnostic>) -> TabsTransition {
    TabsTransition {
        kind: TabsTransitionKind::Rejected,
        intents: Vec::new(),
        events: Vec::new(),
        diagnostics,
        invalidation: TabsInvalidation::None,
    }
}

pub(super) fn ignored() -> TabsTransition {
    TabsTransition {
        kind: TabsTransitionKind::Ignored,
        intents: Vec::new(),
        events: Vec::new(),
        diagnostics: Vec::new(),
        invalidation: TabsInvalidation::None,
    }
}
