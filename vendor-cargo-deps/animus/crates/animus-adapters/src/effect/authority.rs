mod evidence;
mod flow;
#[cfg(test)]
mod model;
mod outcome;
mod schema;
mod scope;

#[cfg(test)]
mod tests;

pub use evidence::{AuthorizationReceipt, validate_authorization_receipt_text};
pub use flow::{AuthorizationContext, authorize_capability};
pub use outcome::{AuthorizationIssue, AuthorizationLeak, AuthorizationStatus};
pub use schema::{CapabilityPolicyContract, CapabilityRequest, UcanGrant};
pub use scope::{
    CaveatSummary, GuardedAction, LocalCaveat, LocalCaveatDecision, LocalPolicyLayer,
    SandboxConstraint, SandboxState,
};
