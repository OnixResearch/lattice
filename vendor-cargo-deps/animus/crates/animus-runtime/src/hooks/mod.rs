#![allow(renamed_imports)]

pub(crate) mod chain;
#[allow(excessive_file_length)]
mod lifecycle;
mod turn;

pub use chain::RuntimeMiddlewareStack;
pub use lifecycle::{
    LifecycleHookCapability, LifecycleHookDefinition, LifecycleHookDefinitionDraft,
    LifecycleHookDefinitionSource, LifecycleHookEffect, LifecycleHookEffectDraft,
    LifecycleHookError, LifecycleHookErrorKind, LifecycleHookEventKind, LifecycleHookFailureClass,
    LifecycleHookInput, LifecycleHookInputDraft, LifecycleHookInputReportDraft,
    LifecycleHookOutcome, LifecycleHookOutcomeClass, LifecycleHookOutcomeDraft,
    LifecycleHookOutcomeReportDraft, LifecycleHookOutput, LifecycleHookOutputDraft,
    LifecycleHookPolicyStatus, LifecycleHookRedaction, LifecycleHookRequest,
    LifecycleHookRoutePlan, LifecycleHookTrustDecision, LifecycleHookTrustRecord,
    LifecycleHookTrustRecordDraft, LifecycleHookTrustScope, LifecycleHookTrustStatus,
    bind_lifecycle_hook_output_to_policy, evaluate_lifecycle_hook_trust,
    route_lifecycle_hook_requests,
};
pub use turn::{
    Effect as TurnHookEffect, Error as TurnHookError, ErrorKind as TurnHookErrorKind,
    Event as TurnHookEvent, Outcome as TurnHookOutcome, State as TurnHookState,
    Transition as TurnHookTransition, advance as advance_turn_hooks,
};

const DEFAULT_MIDDLEWARE_LABEL: &str = "runtime_middleware";

/// Runtime boundary guarded or observed by host-injected middleware.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HookBoundary {
    /// Observe-only event stream boundary.
    Observe,
    /// Model dispatch boundary before an adapter call.
    ModelDispatch,
    /// Tool dispatch boundary before an adapter call.
    ToolDispatch,
    /// Compaction boundary before persistence or summarization work.
    Compaction,
    /// Context replay boundary before restoring prior state.
    Replay,
    /// Session replacement boundary before switching active state.
    SessionReplacement,
}

impl HookBoundary {
    /// Returns the stable boundary label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Observe => "observe",
            Self::ModelDispatch => "model_dispatch",
            Self::ToolDispatch => "tool_dispatch",
            Self::Compaction => "compaction",
            Self::Replay => "replay",
            Self::SessionReplacement => "session_replacement",
        }
    }

    /// Returns true when hook errors must abort or deny the guarded action.
    #[must_use]
    pub const fn fails_safe(self) -> bool {
        match self {
            Self::Observe => false,
            Self::ModelDispatch
            | Self::ToolDispatch
            | Self::Compaction
            | Self::Replay
            | Self::SessionReplacement => true,
        }
    }
}

/// Runtime hook action class supported by middleware contracts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HookAction {
    /// Observe without changing the boundary payload.
    Observe,
    /// Accept a modified payload for later hooks and the adapter.
    Mutate,
    /// Deny the boundary action.
    Block,
    /// Replace the payload with a host-supplied payload.
    Replace,
}

impl HookAction {
    /// Returns the stable hook-action label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Observe => "observe",
            Self::Mutate => "mutate",
            Self::Block => "block",
            Self::Replace => "replace",
        }
    }
}

/// Host hook result for a boundary payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HookDecision<Payload> {
    /// Continue with the accepted payload, possibly after mutation.
    Continue {
        /// Payload accepted by this hook.
        value: Payload,
    },
    /// Deny the guarded action with a safe reason label/message.
    Block {
        /// Safe denial reason.
        reason: String,
    },
    /// Replace the boundary payload.
    Replace {
        /// Replacement payload accepted by this hook.
        value: Payload,
    },
}

impl<Payload> HookDecision<Payload> {
    /// Returns the action represented by this decision.
    #[must_use]
    pub const fn action(&self) -> HookAction {
        match self {
            Self::Continue { .. } => HookAction::Mutate,
            Self::Block { .. } => HookAction::Block,
            Self::Replace { .. } => HookAction::Replace,
        }
    }
}

/// Typed middleware failure raised by a host hook.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MiddlewareError {
    boundary: HookBoundary,
    hook: String,
    message: String,
}

impl MiddlewareError {
    /// Builds a typed middleware error.
    #[must_use]
    pub fn new(boundary: HookBoundary, hook: &str, message: &str) -> Self {
        debug_assert!(!hook.is_empty());
        debug_assert!(!message.is_empty());
        Self {
            boundary,
            hook: String::from(hook),
            message: String::from(message),
        }
    }

    /// Returns the boundary that failed.
    #[must_use]
    pub const fn boundary(&self) -> HookBoundary {
        self.boundary
    }

    /// Returns the middleware label that failed.
    #[must_use]
    pub fn hook(&self) -> &str {
        self.hook.as_str()
    }

    /// Returns the safe failure message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

/// Host-injected middleware contract for runtime boundary events.
pub trait RuntimeMiddleware {
    /// Returns a stable middleware label for events and failures.
    fn label(&self) -> &str {
        DEFAULT_MIDDLEWARE_LABEL
    }

    /// Observes an event without mutating runtime state.
    ///
    /// # Errors
    ///
    /// Returns [`MiddlewareError`] when the observer fails.
    #[allow(clippy::unused_self)]
    fn observe(&mut self, _event: &crate::RuntimeEvent) -> Result<(), MiddlewareError> {
        Ok(())
    }

    /// Evaluates or mutates a model dispatch request before adapter execution.
    ///
    /// # Errors
    ///
    /// Returns [`MiddlewareError`] when the guard fails. Guard failures fail
    /// safe and prevent the model adapter call.
    #[allow(clippy::unused_self)]
    fn before_model_dispatch(
        &mut self,
        request: animus_adapters::ModelRequest,
    ) -> Result<HookDecision<animus_adapters::ModelRequest>, MiddlewareError> {
        Ok(HookDecision::Continue { value: request })
    }

    /// Evaluates or mutates a tool dispatch request before adapter execution.
    ///
    /// # Errors
    ///
    /// Returns [`MiddlewareError`] when the guard fails. Guard failures fail
    /// safe and prevent the tool adapter call.
    #[allow(clippy::unused_self)]
    fn before_tool_dispatch(
        &mut self,
        request: animus_adapters::ToolRequest,
    ) -> Result<HookDecision<animus_adapters::ToolRequest>, MiddlewareError> {
        Ok(HookDecision::Continue { value: request })
    }
}
