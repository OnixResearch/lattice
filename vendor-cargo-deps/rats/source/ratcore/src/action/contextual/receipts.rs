/// One deterministic multi-surface action replay step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRouteReplayStep {
    /// Requested shared action.
    pub action_id: crate::nexus_core::ActionId,
    /// Surface that normalized activation.
    pub surface: super::model::ActionActivationSurface,
    /// Renderer adapter provenance.
    pub renderer: super::model::ActionRendererAdapter,
    /// Generation observed by the surface.
    pub observed_generation: u64,
    /// Optional explicit active chain.
    pub active_chain: Option<Vec<super::ActionContextId>>,
}

impl ActionRouteReplayStep {
    /// Creates a replay step using the accepted active chain.
    #[must_use]
    pub fn new(
        action_id: impl Into<crate::nexus_core::ActionId>,
        surface: super::model::ActionActivationSurface,
        renderer: super::model::ActionRendererAdapter,
        observed_generation: u64,
    ) -> Self {
        Self {
            action_id: action_id.into(),
            surface,
            renderer,
            observed_generation,
            active_chain: None,
        }
    }

    /// Supplies an explicit focused-to-root chain.
    #[must_use]
    pub fn with_active_chain(mut self, active_chain: Vec<super::ActionContextId>) -> Self {
        self.active_chain = Some(active_chain);
        self
    }
}

/// Replay receipt produced by the production invocation path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRouteReplayReceipt {
    /// Stable sequence number.
    pub sequence: usize,
    /// Full generation-bound production receipt.
    pub invocation: super::model::ActionInvocationReceipt,
}

/// Deterministic contextual route replay artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRouteReplay {
    /// Routing generation against which replay ran.
    pub routing_generation: u64,
    /// Ordered production invocation receipts.
    pub receipts: Vec<ActionRouteReplayReceipt>,
}

/// Replays normalized activations through [`super::resolution::invoke_action`].
#[must_use]
pub fn replay_action_routes(
    snapshot: &super::model::ActionRoutingSnapshot,
    steps: &[ActionRouteReplayStep],
) -> ActionRouteReplay {
    debug_assert!(snapshot.generation > 0);
    debug_assert!(snapshot.active_chain.len() <= snapshot.contexts.len());
    let receipts = steps
        .iter()
        .enumerate()
        .map(|(sequence, step)| {
            let mut invocation = super::model::ActionInvocation::new(
                step.action_id.clone(),
                step.observed_generation,
                step.surface,
                step.renderer,
            );
            invocation.active_chain.clone_from(&step.active_chain);
            ActionRouteReplayReceipt {
                sequence,
                invocation: super::resolution::invoke_action(snapshot, &invocation),
            }
        })
        .collect();
    ActionRouteReplay {
        routing_generation: snapshot.generation,
        receipts,
    }
}
