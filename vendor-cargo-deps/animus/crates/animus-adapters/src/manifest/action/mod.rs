type Collection<T> = super::Collection<T>;
type Text = super::Text;

mod access;
pub(in crate::manifest) mod check;
mod consts;
mod context;
mod plan;
mod types;

pub use types::{
    PluginCompatibility, PluginCompatibilityDraft, PluginDiagnostic, PluginDiagnosticKind,
    PluginDiagnosticSeverity, PluginHostCompatibilityDraft, PluginInvocationContext,
    PluginInvocationContextDraft, PluginInvocationContextLeak, PluginRegistryOperationDraft,
    PluginRegistryOperationKind, PluginRegistryPlan, PluginRegistryPlanStatus, PluginRole,
    PluginRoleDraft, PluginRoleKind,
};
