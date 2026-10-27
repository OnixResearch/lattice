//! OIDC identity verification and bounded UCAN exchange.
//!
//! The core verifies one already-acquired token against explicit policy, public
//! verification material, and caller-supplied time. The shell owns HTTPS,
//! cache state, replay mutation, and UCAN issuance.
//!

// r[impl ucan_token_flow.oidc.versioned_policy]
// r[impl ucan_token_flow.oidc.pure_verification]
// r[impl ucan_token_flow.oidc.attenuated_ucan]
// r[impl ucan_token_flow.oidc.bounded_jwks]
// r[impl ucan_token_flow.oidc.replay_boundary]
// r[impl ucan_token_flow.oidc.redacted_evidence]

mod core;
mod issue;
mod model;
mod network;
mod receipt;
mod shell;

pub use core::{inspect_token_key_id, verify_and_plan};
pub use issue::{
    IssueFailure, IssuerAdmission, issue_plan, signer_verification_digest,
    validate_issuer_admission,
};
pub use model::{
    CachePolicy, CapabilityPolicy, ExchangeRequest, IssuancePlan, MantleOperation, NetworkPolicy,
    OidcPolicy, PolicyError, PublicJwk, RejectReason, ReplayKey, ReplayPolicy, TimePolicy,
    VerifiedSubject, parse_policy,
};
pub use network::{
    BoundedHttpDocumentPort, DocumentFailure, DocumentKind, DocumentPort, DocumentResponse,
    JwksCache, KeyLoadFailure, load_verification_key,
};
pub use receipt::{
    EXCHANGE_RECEIPT_SCHEMA, ExchangeOutcome, ExchangeReceipt, receipt_for_failure,
    receipt_for_success, validate_receipt_redaction,
};
pub use shell::{
    ExchangeFailure, ExchangeFailureClass, ExchangeResult, ReplayCommit, ReplayFailure, ReplayPort,
    exchange, exchange_to_recipient,
};

/// Reviewed `jsonwebtoken` release.
pub const JSONWEBTOKEN_VERSION: &str = "11.0.0";
/// Immutable upstream object for the reviewed `jsonwebtoken` release.
pub const JSONWEBTOKEN_REVISION: &str = "900010fb1be550ec2cfa8c32038eb154f9ef56d7";
/// Reviewed Bounded HTTP Radicle repository identifier.
pub const BOUNDED_HTTP_RID: &str = "rad:z4ARxEt93T6gJTrTZpUyKHMAemPiv";
/// Immutable reviewed Bounded HTTP implementation object.
pub const BOUNDED_HTTP_REVISION: &str = "5abc2b93e0f288aba251a0750ce2d716eb9757fc";
