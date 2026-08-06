/// Normalized renderer-owned activation source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionActivationSurface {
    /// Keyboard shortcut or keymap.
    Keybinding,
    /// Desktop or context menu.
    Menu,
    /// Command palette.
    CommandPalette,
    /// Toolbar control.
    Toolbar,
    /// Pointer hit target.
    Pointer,
    /// Accessibility semantic action.
    Semantic,
}

/// Renderer adapter producing a normalized activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionRendererAdapter {
    /// Subwayrat terminal or Ratzilla adapter.
    Subwayrat,
    /// Meteorite Dioxus adapter.
    Meteorite,
    /// Bounded Iced pilot adapter.
    IcedPilot,
}

/// Generation-bound contextual action invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionInvocation {
    /// Requested action.
    pub action_id: crate::nexus_core::ActionId,
    /// Generation observed by the activating surface.
    pub observed_generation: u64,
    /// Normalized command surface.
    pub surface: ActionActivationSurface,
    /// Renderer adapter provenance.
    pub renderer: ActionRendererAdapter,
    /// Optional caller-supplied focused-to-root chain.
    pub active_chain: Option<Vec<super::super::ActionContextId>>,
}

impl ActionInvocation {
    /// Creates an invocation that uses the snapshot's accepted active chain.
    #[must_use]
    pub fn new(
        action_id: impl Into<crate::nexus_core::ActionId>,
        observed_generation: u64,
        surface: ActionActivationSurface,
        renderer: ActionRendererAdapter,
    ) -> Self {
        Self {
            action_id: action_id.into(),
            observed_generation,
            surface,
            renderer,
            active_chain: None,
        }
    }

    /// Supplies an explicit focused-to-root active chain.
    #[must_use]
    pub fn with_active_chain(mut self, active_chain: Vec<super::super::ActionContextId>) -> Self {
        self.active_chain = Some(active_chain);
        self
    }
}

/// Decision recorded while probing one registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionRouteDecision {
    /// Registration was eligible.
    Eligible,
    /// Owner or registration was disabled.
    Disabled,
    /// Predicate did not match.
    PredicateMismatch,
    /// Modal policy blocked the route.
    ModalBlocked,
    /// A nearer or higher-priority registration won.
    Shadowed,
}

/// Inspectable probe decision for one registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRouteConsideration {
    /// Registration identity.
    pub registration_id: super::super::ActionHandlerRegistrationId,
    /// Owner context identity.
    pub owner_context: super::super::ActionContextId,
    /// Distance from focused context.
    pub context_distance: usize,
    /// Registration priority.
    pub priority: i16,
    /// Propagation policy.
    pub propagation: super::ActionPropagation,
    /// Probe decision.
    pub decision: ActionRouteDecision,
}

/// Accepted pure target route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRoute {
    /// Requested action identity.
    pub action_id: crate::nexus_core::ActionId,
    /// Winning registration identity mapped by the application shell.
    pub registration_id: super::super::ActionHandlerRegistrationId,
    /// Winning owner context.
    pub owner_context: super::super::ActionContextId,
    /// Snapshot generation used for resolution.
    pub routing_generation: u64,
    /// Owner lifecycle generation used for resolution.
    pub owner_generation: u64,
    /// Winning priority.
    pub priority: i16,
    /// Winning propagation policy.
    pub propagation: super::ActionPropagation,
}

/// Full deterministic route-probe provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRouteProvenance {
    /// Focused-to-root chain used for resolution.
    pub active_chain: Vec<super::super::ActionContextId>,
    /// Registration decisions in deterministic resolution order.
    pub considerations: Vec<ActionRouteConsideration>,
}

/// Accepted route or typed fail-closed diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionRouteOutcome {
    /// One unambiguous route was accepted.
    Accepted(ActionRoute),
    /// Resolution failed without executing a handler.
    Rejected(super::RoutingDiagnostic),
}

impl ActionRouteOutcome {
    /// Returns the accepted route, when one exists.
    #[must_use]
    pub const fn route(&self) -> Option<&ActionRoute> {
        match self {
            Self::Accepted(route) => Some(route),
            Self::Rejected(_) => None,
        }
    }

    /// Returns the rejection diagnostic, when one exists.
    #[must_use]
    pub const fn diagnostic(&self) -> Option<&super::RoutingDiagnostic> {
        match self {
            Self::Accepted(_) => None,
            Self::Rejected(diagnostic) => Some(diagnostic),
        }
    }
}

/// Non-executing contextual action route probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRouteProbe {
    /// Requested action.
    pub action_id: crate::nexus_core::ActionId,
    /// Snapshot generation used by the probe.
    pub routing_generation: u64,
    /// Accepted target or rejection.
    pub outcome: ActionRouteOutcome,
    /// Deterministic resolution evidence.
    pub provenance: ActionRouteProvenance,
}

/// Receipt returned before any shell-owned execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionInvocationReceipt {
    /// Normalized activation source.
    pub surface: ActionActivationSurface,
    /// Renderer adapter provenance.
    pub renderer: ActionRendererAdapter,
    /// Generation observed by the source surface.
    pub observed_generation: u64,
    /// Current generation at invocation.
    pub current_generation: u64,
    /// Revalidated route probe.
    pub probe: ActionRouteProbe,
}
