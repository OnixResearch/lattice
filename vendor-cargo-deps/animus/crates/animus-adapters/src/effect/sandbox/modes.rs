use alloc::string::ToString;

const DESCRIPTOR_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_descriptor_id");
const FILESYSTEM_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_filesystem_mode");
const WRITABLE_ROOT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_writable_root");
const WRITABLE_ROOTS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_writable_roots");
const WORKDIR_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_workdir_mode");
const COW_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_cow_disposition");
const PROCESS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_process_profile");
const ENVIRONMENT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_environment_policy");
const NETWORK_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_network_policy");
const HTTP_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_http_policy");
const DETERMINISM_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_determinism_policy");
const ENFORCEMENT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_enforcement_requirement");
const CONSTRAINT_CLASS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_constraint_class");
const PLATFORM_CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_platform_capability");
const POLICY_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_policy_id");
const PLATFORM_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_platform_id");

const FILESYSTEM_READ_ONLY: &str = "read_only";
const FILESYSTEM_WORKSPACE_WRITE: &str = "workspace_write";
const FILESYSTEM_DENY_ALL: &str = "deny_all";
const WORKDIR_HOST: &str = "host_workspace";
const WORKDIR_COW_STAGED: &str = "cow_staged";
const WORKDIR_TEMPDIR: &str = "tempdir";
const COW_DISABLED: &str = "disabled";
const COW_DRY_RUN: &str = "dry_run";
const COW_COMMIT: &str = "commit";
const PROCESS_BASIC: &str = "basic";
const PROCESS_RESOURCE_LIMITED: &str = "resource_limited";
const ENV_MINIMAL: &str = "minimal";
const ENV_ALLOWLISTED: &str = "allowlisted";
const NETWORK_DENY_ALL: &str = "deny_all";
const NETWORK_EGRESS_ALLOWLIST: &str = "egress_allowlist";
const NETWORK_BIND_DENY: &str = "bind_deny";
const HTTP_NONE: &str = "none";
const HTTP_METHOD_HOST_PATH: &str = "method_host_path_allowlist";
const DETERMINISM_HOST: &str = "host";
const DETERMINISM_STABLE: &str = "stable_time_randomness";
const ENFORCEMENT_REQUIRED: &str = "required";
const ENFORCEMENT_BEST_EFFORT: &str = "best_effort";
const CLASS_FILESYSTEM: &str = "filesystem";
const CLASS_WORKDIR_COW: &str = "workdir_cow";
const CLASS_PROCESS: &str = "process";
const CLASS_ENVIRONMENT: &str = "environment";
const CLASS_NETWORK: &str = "network";
const CLASS_HTTP: &str = "http";
const CLASS_DETERMINISM: &str = "determinism";
const CAP_LANDLOCK_FILESYSTEM: &str = "landlock_filesystem";
const CAP_SECCOMP_BPF: &str = "seccomp_bpf";
const CAP_SECCOMP_NOTIFY: &str = "seccomp_notify";
const CAP_COW_STAGING: &str = "cow_staging";
const CAP_RESOURCE_LIMITS: &str = "resource_limits";
const CAP_NETWORK_ACL: &str = "network_acl";
const CAP_HTTP_ACL: &str = "http_acl";
const CAP_DETERMINISM: &str = "determinism_controls";
const RECEIPT_SCHEMA: &str = "animus.sandbox-dispatch-receipt.v1";
const DESCRIPTOR_SCHEMA: &str = "animus.sandbox-descriptor.v1";
const RECEIPT_PART_SEPARATOR: char = '|';
const EMPTY_LABEL: &str = "none";

/// Portable filesystem constraint intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxFilesystemMode {
    /// Filesystem reads are allowed but writes are denied.
    ReadOnly,
    /// Writes are constrained to declared workspace roots.
    WorkspaceWrite,
    /// Filesystem access is denied by default.
    DenyAll,
}

impl SandboxFilesystemMode {
    /// Parses a filesystem mode label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        parse_value(FILESYSTEM_FIELD, value, |normalized| match normalized {
            FILESYSTEM_READ_ONLY => Some(Self::ReadOnly),
            FILESYSTEM_WORKSPACE_WRITE => Some(Self::WorkspaceWrite),
            FILESYSTEM_DENY_ALL => Some(Self::DenyAll),
            _ => None,
        })
    }

    /// Returns the stable filesystem mode label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => FILESYSTEM_READ_ONLY,
            Self::WorkspaceWrite => FILESYSTEM_WORKSPACE_WRITE,
            Self::DenyAll => FILESYSTEM_DENY_ALL,
        }
    }
}

/// Portable workdir materialization mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxWorkdirMode {
    /// Use the host workspace directly under adapter control.
    HostWorkspace,
    /// Stage writes through copy-on-write materialization.
    CowStaged,
    /// Use a host-owned temporary workspace.
    Tempdir,
}

impl SandboxWorkdirMode {
    /// Parses a workdir mode label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        parse_value(WORKDIR_FIELD, value, |normalized| match normalized {
            WORKDIR_HOST => Some(Self::HostWorkspace),
            WORKDIR_COW_STAGED => Some(Self::CowStaged),
            WORKDIR_TEMPDIR => Some(Self::Tempdir),
            _ => None,
        })
    }

    /// Returns the stable workdir mode label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HostWorkspace => WORKDIR_HOST,
            Self::CowStaged => WORKDIR_COW_STAGED,
            Self::Tempdir => WORKDIR_TEMPDIR,
        }
    }
}

/// Portable copy-on-write write disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxCowDisposition {
    /// COW handling is disabled.
    Disabled,
    /// COW writes are summarized and discarded.
    DryRun,
    /// COW writes may be committed by a host adapter after policy checks.
    Commit,
}

impl SandboxCowDisposition {
    /// Parses a COW disposition label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        parse_value(COW_FIELD, value, |normalized| match normalized {
            COW_DISABLED => Some(Self::Disabled),
            COW_DRY_RUN => Some(Self::DryRun),
            COW_COMMIT => Some(Self::Commit),
            _ => None,
        })
    }

    /// Returns the stable COW disposition label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => COW_DISABLED,
            Self::DryRun => COW_DRY_RUN,
            Self::Commit => COW_COMMIT,
        }
    }
}

/// Portable process/resource supervision profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxProcessProfile {
    /// Basic process confinement.
    Basic,
    /// Process confinement plus resource limits.
    ResourceLimited,
}

impl SandboxProcessProfile {
    /// Parses a process profile label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        parse_value(PROCESS_FIELD, value, |normalized| match normalized {
            PROCESS_BASIC => Some(Self::Basic),
            PROCESS_RESOURCE_LIMITED => Some(Self::ResourceLimited),
            _ => None,
        })
    }

    /// Returns the stable process profile label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Basic => PROCESS_BASIC,
            Self::ResourceLimited => PROCESS_RESOURCE_LIMITED,
        }
    }
}

/// Portable environment visibility policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxEnvironmentPolicy {
    /// Minimal host-provided environment only.
    Minimal,
    /// Explicit allowlist of environment labels.
    Allowlisted,
}

impl SandboxEnvironmentPolicy {
    /// Parses an environment policy label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        parse_value(ENVIRONMENT_FIELD, value, |normalized| match normalized {
            ENV_MINIMAL => Some(Self::Minimal),
            ENV_ALLOWLISTED => Some(Self::Allowlisted),
            _ => None,
        })
    }

    /// Returns the stable environment policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Minimal => ENV_MINIMAL,
            Self::Allowlisted => ENV_ALLOWLISTED,
        }
    }
}

/// Portable network policy intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxNetworkPolicy {
    /// No network access.
    DenyAll,
    /// Egress is restricted to explicit labels.
    EgressAllowlist,
    /// Bind/listen behavior is denied.
    BindDeny,
}

impl SandboxNetworkPolicy {
    /// Parses a network policy label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        parse_value(NETWORK_FIELD, value, |normalized| match normalized {
            NETWORK_DENY_ALL => Some(Self::DenyAll),
            NETWORK_EGRESS_ALLOWLIST => Some(Self::EgressAllowlist),
            NETWORK_BIND_DENY => Some(Self::BindDeny),
            _ => None,
        })
    }

    /// Returns the stable network policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DenyAll => NETWORK_DENY_ALL,
            Self::EgressAllowlist => NETWORK_EGRESS_ALLOWLIST,
            Self::BindDeny => NETWORK_BIND_DENY,
        }
    }
}
