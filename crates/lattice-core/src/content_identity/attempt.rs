//! The implementation-attempt worker profile revision identity.

pub const WORKER_PROFILE_REVISION_DOMAIN: &str = "lattice-worker-profile-revision-v2";

/// One reviewed immutable runtime artifact of a worker profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeArtifactInput<'a> {
    pub path: &'a str,
    pub blake3: &'a str,
}

/// Profile-level inputs of one worker profile revision.
///
/// Per-attempt values are deliberately absent: the workspace root, base
/// snapshot, task prompt, broker expiry, envelope and execution binding. The
/// attempt binds them through its envelope and exact request bytes instead.
/// The reviewed harness template and the request schema are the owner's
/// effective code constants, passed in by reference rather than restated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkerProfileRevisionInput<'a> {
    /// The request schema the profile is exported under (compatibility).
    pub request_schema: &'a str,
    /// The fixed harness launch prefix, in argv order.
    pub harness_launch: &'a [&'a str],
    /// The reviewed harness print options, in argv order.
    pub harness_print_options: &'a [&'a str],
    pub program: &'a str,
    pub sandbox_program: &'a str,
    pub harness_revision: &'a str,
    pub harness_executable_blake3: &'a str,
    pub harness_model: &'a str,
    pub broker_schema: &'a str,
    pub broker_endpoint: &'a str,
    pub broker_model: &'a str,
    pub credential_provider: Option<&'a str>,
    pub credential_key: Option<&'a str>,
    pub credential_field: Option<&'a str>,
    pub relay: RuntimeArtifactInput<'a>,
    pub shell: RuntimeArtifactInput<'a>,
    pub sandbox_blake3: &'a str,
    pub max_requests: u64,
    pub read_paths: &'a [&'a str],
    pub write_paths: &'a [&'a str],
    pub output_limit_bytes: u64,
    pub allow_unchanged: bool,
    /// Canonical JSON of the compute requirement.
    pub compute: &'a str,
    /// Canonical JSON of the capability declaration; `None` when undeclared.
    pub capability: Option<&'a str>,
}

/// Build the `lattice-worker-profile-revision-v2` identity, spelled as bare
/// lowercase hex like every earlier profile revision.
// r[impl lattice.implementation_attempt.consumer.scenario.profile_revision]
// r[impl lattice.agent_revisions.framing]
pub fn worker_profile_revision(
    input: WorkerProfileRevisionInput<'_>,
) -> Result<String, content_identity_core::IdentityError> {
    let mut hasher =
        content_identity_core::IdentityHasher::new(content_identity_core::Domain::new(WORKER_PROFILE_REVISION_DOMAIN)?);
    hasher
        .update_tagged_str(b"request-schema", input.request_schema)?
        .update_ordered_strs(b"harness-launch", input.harness_launch)?
        .update_ordered_strs(b"harness-print-options", input.harness_print_options)?
        .update_tagged_str(b"program", input.program)?
        .update_tagged_str(b"sandbox-program", input.sandbox_program)?
        .update_tagged_str(b"harness-revision", input.harness_revision)?
        .update_tagged_str(b"harness-executable", input.harness_executable_blake3)?
        .update_tagged_str(b"harness-model", input.harness_model)?
        .update_tagged_str(b"broker-schema", input.broker_schema)?
        .update_tagged_str(b"broker-endpoint", input.broker_endpoint)?
        .update_tagged_str(b"broker-model", input.broker_model)?
        .update_optional_str(b"credential-provider", input.credential_provider)?
        .update_optional_str(b"credential-key", input.credential_key)?
        .update_optional_str(b"credential-field", input.credential_field)?
        .update_tagged_str(b"relay-path", input.relay.path)?
        .update_tagged_str(b"relay-blake3", input.relay.blake3)?
        .update_tagged_str(b"shell-path", input.shell.path)?
        .update_tagged_str(b"shell-blake3", input.shell.blake3)?
        .update_tagged_str(b"sandbox-blake3", input.sandbox_blake3)?
        .update_tagged_u64_le(b"max-requests", input.max_requests)
        .update_ordered_strs(b"read-paths", input.read_paths)?
        .update_ordered_strs(b"write-paths", input.write_paths)?
        .update_tagged_u64_le(b"output-limit-bytes", input.output_limit_bytes)
        .update_tagged_bool(b"allow-unchanged", input.allow_unchanged)
        .update_tagged_str(b"compute", input.compute)?
        .update_optional_str(b"capability", input.capability)?;
    let revision = hasher.finish().to_hex();
    debug_assert_eq!(revision.len(), 64, "a profile revision is one BLAKE3 digest");
    debug_assert!(
        revision.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "a profile revision is bare lowercase hex"
    );
    Ok(revision)
}
