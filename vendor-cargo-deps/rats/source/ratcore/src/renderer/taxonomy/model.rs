#[path = "model/catalog.rs"]
mod catalog;
#[path = "model/identity.rs"]
mod identity;
#[path = "model/requirements.rs"]
mod requirements;

pub use catalog::EvidenceAdapterDescriptor;
pub use catalog::EvidenceRole;
pub use catalog::RendererFamilyDescriptor;
pub use catalog::RendererRegistry;
pub use catalog::RendererTargetDescriptor;
pub use identity::EvidenceAdapterId;
pub use identity::RendererFamilyId;
pub use identity::RendererLifecycle;
pub use identity::RendererScopeId;
pub use identity::RendererSubject;
pub use identity::RendererSubjectKind;
pub use identity::RendererTargetId;
pub use requirements::RendererCellStatus;
pub use requirements::RendererRequirementScope;
pub use requirements::RendererScopeCompletion;
pub use requirements::ScopedRendererCell;
