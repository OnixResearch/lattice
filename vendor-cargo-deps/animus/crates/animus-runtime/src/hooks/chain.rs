/// Ordered host middleware chain.
pub struct RuntimeMiddlewareStack<'a> {
    hooks: Vec<&'a mut dyn super::RuntimeMiddleware>,
}

impl<'a> RuntimeMiddlewareStack<'a> {
    /// Builds an empty middleware chain.
    #[must_use]
    pub fn empty() -> Self {
        Self { hooks: Vec::new() }
    }

    /// Builds a middleware chain from hooks in deterministic execution order.
    #[must_use]
    pub fn from_hooks(hooks: Vec<&'a mut dyn super::RuntimeMiddleware>) -> Self {
        Self { hooks }
    }

    /// Returns true when no hooks are configured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }

    /// Sends an event to observe-only hooks in configured order.
    ///
    /// # Errors
    ///
    /// Returns the first observer error.
    pub fn observe_event(&mut self, event: &crate::RuntimeEvent) -> Result<(), super::MiddlewareError> {
        for hook in &mut self.hooks {
            hook.observe(event)?;
        }
        Ok(())
    }

    pub(crate) fn prepare_model_dispatch(
        &mut self,
        request: animus_adapters::ModelRequest,
    ) -> MiddlewareOutcome<animus_adapters::ModelRequest> {
        run_model(&mut self.hooks, request)
    }

    pub(crate) fn prepare_tool_dispatch(
        &mut self,
        request: animus_adapters::ToolRequest,
    ) -> MiddlewareOutcome<animus_adapters::ToolRequest> {
        run_tool(&mut self.hooks, request)
    }
}

impl Default for RuntimeMiddlewareStack<'_> {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum MiddlewareOutcome<Payload> {
    Accepted(Payload),
    Blocked {
        value: Payload,
        hook: String,
        reason: String,
    },
    Failed {
        value: Payload,
        error: super::MiddlewareError,
    },
}

fn run_model(
    hooks: &mut [&mut dyn super::RuntimeMiddleware],
    request: animus_adapters::ModelRequest,
) -> MiddlewareOutcome<animus_adapters::ModelRequest> {
    run_all(request, hooks, |hook, current| hook.before_model_dispatch(current))
}

fn run_tool(
    hooks: &mut [&mut dyn super::RuntimeMiddleware],
    request: animus_adapters::ToolRequest,
) -> MiddlewareOutcome<animus_adapters::ToolRequest> {
    run_all(request, hooks, |hook, current| hook.before_tool_dispatch(current))
}

fn run_all<Payload, Apply>(
    mut current: Payload,
    hooks: &mut [&mut dyn super::RuntimeMiddleware],
    mut apply: Apply,
) -> MiddlewareOutcome<Payload>
where
    Payload: Clone,
    Apply: FnMut(
        &mut dyn super::RuntimeMiddleware,
        Payload,
    ) -> Result<super::HookDecision<Payload>, super::MiddlewareError>,
{
    for hook in hooks {
        let hook_label = String::from(hook.label());
        debug_assert!(!hook_label.is_empty());
        match apply(*hook, current.clone()) {
            Ok(super::HookDecision::Continue { value } | super::HookDecision::Replace { value }) => {
                current = value;
            }
            Ok(super::HookDecision::Block { reason }) => {
                debug_assert!(!reason.is_empty());
                return MiddlewareOutcome::Blocked {
                    value: current,
                    hook: hook_label,
                    reason,
                };
            }
            Err(error) => {
                debug_assert!(!error.message().is_empty());
                return MiddlewareOutcome::Failed { value: current, error };
            }
        }
    }
    MiddlewareOutcome::Accepted(current)
}
