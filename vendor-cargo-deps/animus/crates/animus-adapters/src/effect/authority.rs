mod evidence;
mod flow;
#[cfg(test)]
mod model;
mod outcome;
mod schema;
mod scope;

#[cfg(test)]
mod tests;

pub use evidence::AuthorizationReceipt;
pub use evidence::validate_authorization_receipt_text;
pub use flow::AuthorizationContext;
pub use flow::authorize_capability;
pub use outcome::AuthorizationIssue;
pub use outcome::AuthorizationLeak;
pub use outcome::AuthorizationStatus;
pub use schema::CapabilityPolicyContract;
pub use schema::CapabilityRequest;
pub use schema::UcanGrant;
pub use scope::CaveatSummary;
pub use scope::GuardedAction;
pub use scope::LocalCaveat;
pub use scope::LocalCaveatDecision;
pub use scope::LocalPolicyLayer;
pub use scope::SandboxConstraint;
pub use scope::SandboxState;
