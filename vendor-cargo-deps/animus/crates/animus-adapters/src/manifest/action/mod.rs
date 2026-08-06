type Collection<T> = super::Collection<T>;
type Text = super::Text;

mod access;
pub(in crate::manifest) mod check;
mod consts;
mod context;
mod plan;
mod types;

pub use types::PluginCompatibility;
pub use types::PluginCompatibilityDraft;
pub use types::PluginDiagnostic;
pub use types::PluginDiagnosticKind;
pub use types::PluginDiagnosticSeverity;
pub use types::PluginHostCompatibilityDraft;
pub use types::PluginInvocationContext;
pub use types::PluginInvocationContextDraft;
pub use types::PluginInvocationContextLeak;
pub use types::PluginRegistryOperationDraft;
pub use types::PluginRegistryOperationKind;
pub use types::PluginRegistryPlan;
pub use types::PluginRegistryPlanStatus;
pub use types::PluginRole;
pub use types::PluginRoleDraft;
pub use types::PluginRoleKind;
