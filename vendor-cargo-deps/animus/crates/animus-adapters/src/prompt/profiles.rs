//! Profile descriptors, directive parsing, mode binding, chain plans, and receipts.

mod authority;
mod compose;
mod continuity;
mod directives;
mod evidence;
mod support;
#[cfg(test)]
mod tests;
mod types;

pub use authority::Binding;
pub use authority::BindingRequest;
pub use authority::BindingStatus;
pub use authority::CapabilityBinding;
pub use authority::bind_mode;
pub use compose::Assembly;
pub use compose::AssemblyDiagnostic;
pub use compose::AssemblyPolicy;
pub use compose::OmittedReason;
pub use compose::Segment;
pub use compose::SourceAdmission;
pub use compose::assemble;
pub use continuity::ChainDecision;
pub use continuity::ChainPlan;
pub use continuity::ChainRequest;
pub use directives::Diagnostic;
pub use directives::Issue;
pub use directives::Report;
pub use directives::parse;
pub use evidence::Receipt;
pub use evidence::ReceiptRedaction;
pub use types::Capability;
pub use types::Descriptor;
pub use types::DescriptorDraft;
pub use types::Error;
pub use types::InclusionPolicy;
pub use types::Mode;
pub use types::SourceTrust;
pub use types::TrustScope;
