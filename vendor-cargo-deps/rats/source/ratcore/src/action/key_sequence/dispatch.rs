use super::ActiveBindingLayers;
use super::BindingCatalogSnapshot;
use super::ResolvedActionBinding;

/// Binding provenance plus subsequent contextual route receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBindingDispatch {
    /// Resolved action/binding provenance.
    pub binding: ResolvedActionBinding,
    /// Production contextual invocation receipt; absent for stale binding data.
    pub invocation: Option<super::super::ActionInvocationReceipt>,
    /// Binding-generation rejection before contextual routing.
    pub diagnostic: Option<String>,
}

/// Passes one resolved action identity through production contextual routing.
///
/// This function stores or executes no callback. A successful receipt still
/// requires the application shell to map its accepted route into a typed
/// message/effect.
#[must_use]
pub fn dispatch_resolved_binding(
    routing: &super::super::ActionRoutingSnapshot,
    catalog: &BindingCatalogSnapshot,
    active: &ActiveBindingLayers,
    binding: &ResolvedActionBinding,
    renderer: super::super::ActionRendererAdapter,
) -> ResolvedBindingDispatch {
    if binding.catalog_generation != catalog.generation() || binding.context_generation != active.context_generation() {
        return ResolvedBindingDispatch {
            binding: binding.clone(),
            invocation: None,
            diagnostic: Some("resolved binding observes a stale catalog or context generation".to_string()),
        };
    }
    let invocation = super::super::ActionInvocation::new(
        binding.action_id.clone(),
        binding.context_generation,
        super::super::ActionActivationSurface::Keybinding,
        renderer,
    );
    ResolvedBindingDispatch {
        binding: binding.clone(),
        invocation: Some(super::super::invoke_contextual_action(routing, &invocation)),
        diagnostic: None,
    }
}
