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

pub use authority::{Binding, BindingRequest, BindingStatus, CapabilityBinding, bind_mode};
pub use compose::{
    Assembly, AssemblyDiagnostic, AssemblyPolicy, OmittedReason, Segment, SourceAdmission, assemble,
};
pub use continuity::{ChainDecision, ChainPlan, ChainRequest};
pub use directives::{Diagnostic, Issue, Report, parse};
pub use evidence::{Receipt, ReceiptRedaction};
pub use types::{
    Capability, Descriptor, DescriptorDraft, Error, InclusionPolicy, Mode, SourceTrust, TrustScope,
};
