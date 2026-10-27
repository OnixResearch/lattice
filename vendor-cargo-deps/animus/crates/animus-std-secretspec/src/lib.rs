//! Optional bounded `SecretSpec` host credential resolver.
//!
//! Pure admission remains in `animus-adapters`. This crate owns explicit file
//! access, the pinned `SecretSpec` SDK, bounded worker processes, and ephemeral
//! delivery. It grants no caller authority and emits metadata-only receipts.

#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
#![forbid(unsafe_code)]

mod interactive;
mod material;
mod policy;
mod protocol;
mod receipt;
mod shell;
#[doc(hidden)]
pub mod sops_guard;
#[doc(hidden)]
pub mod worker;

/// Authority-free request, admission, plan, and injected-service contracts.
pub use animus_adapters::secret_resolution;
pub use interactive::{
    InteractiveSecretFileHost, MaskedSecretInput, SecretCleanupFailure, SecretEntryAuthority,
    SecretEntryError, SecretEntryRequest, SecretEntrySessionClass, SecretPathResult,
    validate_path_only_record,
};
pub use material::{CredentialOrigin, ProviderCredentialSink, ResolvedCredential};
pub use policy::{
    BOUNDED_EXEC_COMMIT, PolicyError, ProfilePolicy, RESOLVER_IDENTITY, SECRETSPEC_COMMIT,
    SECRETSPEC_CRATE_CHECKSUM, SECRETSPEC_VERSION, SecretSpecPolicy, parse_policy_json,
};
pub use receipt::{EvidenceOmission, ResolutionCapability, ResolutionOutcome, ResolutionReceipt};
pub use shell::{
    BoundedSecretSpecResolver, ProviderInputs, ResolutionFailure, ResolutionSuccess,
    ResolverDeployment, SopsCredentialName,
};
