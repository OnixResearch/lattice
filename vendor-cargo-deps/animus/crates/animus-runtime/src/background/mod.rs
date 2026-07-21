//! Pure background process-job profile contracts.
//!
//! Hosts own manifest discovery, process spawning, pueue/systemd integration,
//! storage, notifications, filesystems, clocks, and UI. This module resolves
//! already-loaded profile data into backend-neutral requests.

mod checks;
mod core;
mod data;
mod digest;
mod envelope;
mod errors;

pub use core::resolve_process_job_profile;
pub use data::{
    NamedProcessJobProfile, ProcessJobBackend, ProcessJobEnvironmentVariable,
    ProcessJobMetadataField, ProcessJobNotificationPolicy, ProcessJobProfile,
    ProcessJobProfileManifest, ProcessJobProfilePolicy, ProcessJobProfileResolveRequest,
    ProcessJobProfileSource, ProcessJobProfileSourcePrecedence, ProcessJobResourceLimits,
    ResolvedProcessJobProfile, StartProcessJobRequest,
};
pub use envelope::ProcessJobProfileIdentity;
pub use errors::{ProcessJobProfileError, ProcessJobProfileErrorKind};

pub(crate) const PROCESS_JOB_PROFILE_SCHEMA: &str = "animus.process-job-profile.v1";
pub(crate) const PROCESS_JOB_PROFILE_SCHEMA_VERSION: u16 = 1;
pub(crate) const DEFAULT_POLICY_SOURCE: &str = "default";
pub(crate) const INLINE_SOURCE_LABEL: &str = "inline";
pub(crate) const IDENTITY_PROFILE_NAME_FIELD: &str = "profile_name";
pub(crate) const IDENTITY_SCHEMA_VERSION_FIELD: &str = "identity.profile.schema_version";
pub(crate) const IDENTITY_SOURCE_FIELD: &str = "identity.profile.source";
pub(crate) const IDENTITY_POLICY_HASH_FIELD: &str = "identity.profile.policy_hash";
pub(crate) const IDENTITY_PROFILE_HASH_FIELD: &str = "identity.profile.profile_hash";
pub(crate) const IDENTITY_REQUEST_HASH_FIELD: &str = "identity.profile.request_hash";
pub(crate) const PROFILE_FIELD: &str = "profile";
pub(crate) const SOURCE_FIELD: &str = "source";
pub(crate) const POLICY_FIELD: &str = "policy";
pub(crate) const COMMAND_FIELD: &str = "command";
pub(crate) const ARGS_FIELD: &str = "args";
pub(crate) const ENV_FIELD: &str = "env";
pub(crate) const CWD_FIELD: &str = "cwd";
pub(crate) const WRITABLE_PATH_FIELD: &str = "writable_paths";
pub(crate) const BACKEND_FIELD: &str = "backend";
pub(crate) const METADATA_FIELD: &str = "metadata";
pub(crate) const RESOURCE_FIELD: &str = "resources";
pub(crate) const SOURCE_LABEL_FIELD: &str = "source_label";
pub(crate) const HASH_HEX_LENGTH: usize = 64;
pub(crate) const PRECEDENCE_GLOBAL_RANK: u8 = 1;
pub(crate) const PRECEDENCE_WORKSPACE_RANK: u8 = 2;
pub(crate) const PRECEDENCE_EXPLICIT_RANK: u8 = 3;
pub(crate) const HASH_PART_SEPARATOR: char = '|';
pub(crate) const HASH_LENGTH_SEPARATOR: char = ':';
pub(crate) const PATH_SEPARATOR: char = '/';
pub(crate) const WINDOWS_PATH_SEPARATOR: char = '\\';
pub(crate) const PARENT_PATH_SEGMENT: &str = "..";
pub(crate) const REMOTE_SCHEME_MARKER: &str = "://";
pub(crate) const ENV_HASH_PREFIX: &str = "env:";
pub(crate) const METADATA_IDENTITY_PREFIX: &str = "identity.";
pub(crate) const METADATA_HASH_PREFIX: &str = "metadata:";
pub(crate) const SOURCE_HASH_PREFIX: &str = "source:";
pub(crate) const POLICY_HASH_PREFIX: &str = "policy:";
pub(crate) const PROFILE_HASH_PREFIX: &str = "profile:";
pub(crate) const REQUEST_HASH_PREFIX: &str = "request:";
pub(crate) const EMPTY_HASH_MARKER: &str = "empty";
pub(crate) const SECRET_KEY_MARKERS: &[&str] = &[
    "TOKEN",
    "PASSWORD",
    "SECRET",
    "PRIVATE_KEY",
    "API_KEY",
    "ACCESS_KEY",
    "CREDENTIAL",
    "AUTHORIZATION",
];
pub(crate) const IDENTITY_LEAK_MARKERS: &[&str] = &[
    "authorization:",
    "api_key",
    "password",
    "secret=",
    "token=",
    "/home/",
    REMOTE_SCHEME_MARKER,
];

#[cfg(test)]
mod tests;
