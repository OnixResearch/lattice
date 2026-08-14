//! Durable product approval gates for ordinary workflow execution.
//!
//! The pure core owns request, authority, expiry, decision, terminal, and resume
//! meaning. The shell owns clocks, persistence, Choregraph application, and
//! workflow execution.

pub mod choregraph;
pub mod cli;
pub mod core;
pub mod identity;
pub mod model;
pub mod shell;

pub use identity::ActorId;
pub use identity::ApprovalArtifactId;
pub use identity::ApprovalAuthorityInputId;
pub use identity::ApprovalAuthorityPolicyId;
pub use identity::ApprovalDecisionId;
pub use identity::ApprovalEventId;
pub use identity::ApprovalExpiryPolicyId;
pub use identity::ApprovalInputId;
pub use identity::ApprovalReceiptId;
pub use identity::ApprovalRequestId;
pub use identity::ApprovalResumeId;
pub use identity::ApprovalResumeSchemaId;
pub use identity::ApprovalStateDigest;
pub use identity::GateId;
pub use identity::ResumeBindingName;
pub use identity::ResumeOwnerId;
pub use model::*;
