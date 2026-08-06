//! Pure renderer identity, lifecycle, scope, and evidence policy.

#[path = "taxonomy/checks.rs"]
mod checks;
#[path = "taxonomy/data.rs"]
mod data;
#[path = "taxonomy/model.rs"]
mod model;

pub use checks::RendererPolicyIssue;
pub use checks::RendererPolicyIssueCode;
pub use checks::RendererPolicyReport;
pub use checks::validate_renderer_registry;
pub use checks::validate_renderer_scope;
pub use checks::validate_scoped_renderer_cells;
pub use data::ADAPTER_ICED_COMPILE;
pub use data::ADAPTER_ICED_NATIVE_SMOKE;
pub use data::ADAPTER_ICED_REPLAY;
pub use data::ADAPTER_ICED_SEMANTIC;
pub use data::ADAPTER_METEORITE_DOM;
pub use data::ADAPTER_SUBWAYRAT_TYPST;
pub use data::BuiltInRendererScope;
pub use data::FAMILY_ICED;
pub use data::FAMILY_METEORITE;
pub use data::FAMILY_SUBWAYRAT;
pub use data::TARGET_ICED_NATIVE;
pub use data::TARGET_METEORITE_DOM;
pub use data::TARGET_METEORITE_EMBEDDED_RATZILLA;
pub use data::TARGET_METEORITE_NATIVE_DOM;
pub use data::TARGET_SUBWAYRAT_RATZILLA;
pub use data::TARGET_SUBWAYRAT_TERMINAL;
pub use data::built_in_renderer_registry;
pub use data::built_in_renderer_scope;
pub use model::EvidenceAdapterDescriptor;
pub use model::EvidenceAdapterId;
pub use model::EvidenceRole;
pub use model::RendererCellStatus;
pub use model::RendererFamilyDescriptor;
pub use model::RendererFamilyId;
pub use model::RendererLifecycle;
pub use model::RendererRegistry;
pub use model::RendererRequirementScope;
pub use model::RendererScopeCompletion;
pub use model::RendererScopeId;
pub use model::RendererSubject;
pub use model::RendererSubjectKind;
pub use model::RendererTargetDescriptor;
pub use model::RendererTargetId;
pub use model::ScopedRendererCell;

#[cfg(test)]
#[path = "taxonomy/tests.rs"]
mod tests;
