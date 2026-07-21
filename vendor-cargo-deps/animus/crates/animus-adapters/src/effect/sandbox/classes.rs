/// Optional HTTP ACL policy intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxHttpPolicy {
    /// No HTTP inspection or ACL is requested.
    None,
    /// HTTP requests are constrained by method, host, and path labels.
    MethodHostPathAllowlist,
}

impl SandboxHttpPolicy {
    /// Parses an HTTP policy label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        parse_value(HTTP_FIELD, value, |normalized| match normalized {
            HTTP_NONE => Some(Self::None),
            HTTP_METHOD_HOST_PATH => Some(Self::MethodHostPathAllowlist),
            _ => None,
        })
    }

    /// Returns the stable HTTP policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => HTTP_NONE,
            Self::MethodHostPathAllowlist => HTTP_METHOD_HOST_PATH,
        }
    }
}

/// Determinism controls requested for sandboxed execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxDeterminismPolicy {
    /// Host time/randomness may be visible.
    Host,
    /// Stable time/randomness controls are required.
    StableTimeRandomness,
}

impl SandboxDeterminismPolicy {
    /// Parses a determinism policy label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        parse_value(DETERMINISM_FIELD, value, |normalized| match normalized {
            DETERMINISM_HOST => Some(Self::Host),
            DETERMINISM_STABLE => Some(Self::StableTimeRandomness),
            _ => None,
        })
    }

    /// Returns the stable determinism policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Host => DETERMINISM_HOST,
            Self::StableTimeRandomness => DETERMINISM_STABLE,
        }
    }
}

/// Sandbox enforcement requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxEnforcementRequirement {
    /// Host dispatch must fail closed if constraints cannot be fully enforced.
    Required,
    /// Host dispatch may continue with explicit partial-enforcement evidence.
    BestEffort,
}

impl SandboxEnforcementRequirement {
    /// Parses an enforcement requirement label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        parse_value(ENFORCEMENT_FIELD, value, |normalized| match normalized {
            ENFORCEMENT_REQUIRED => Some(Self::Required),
            ENFORCEMENT_BEST_EFFORT => Some(Self::BestEffort),
            _ => None,
        })
    }

    /// Returns the stable enforcement requirement label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Required => ENFORCEMENT_REQUIRED,
            Self::BestEffort => ENFORCEMENT_BEST_EFFORT,
        }
    }
}

/// Constraint class used by policy fixtures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxConstraintClass {
    /// Filesystem read/write limits.
    Filesystem,
    /// Workdir and COW staging limits.
    WorkdirCow,
    /// Process/syscall/resource limits.
    Process,
    /// Environment visibility limits.
    Environment,
    /// Network ACL limits.
    Network,
    /// HTTP method/host/path ACL limits.
    Http,
    /// Determinism controls.
    Determinism,
}

impl SandboxConstraintClass {
    /// Parses a constraint class label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        parse_value(
            CONSTRAINT_CLASS_FIELD,
            value,
            |normalized| match normalized {
                CLASS_FILESYSTEM => Some(Self::Filesystem),
                CLASS_WORKDIR_COW => Some(Self::WorkdirCow),
                CLASS_PROCESS => Some(Self::Process),
                CLASS_ENVIRONMENT => Some(Self::Environment),
                CLASS_NETWORK => Some(Self::Network),
                CLASS_HTTP => Some(Self::Http),
                CLASS_DETERMINISM => Some(Self::Determinism),
                _ => None,
            },
        )
    }

    /// Returns the stable constraint class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Filesystem => CLASS_FILESYSTEM,
            Self::WorkdirCow => CLASS_WORKDIR_COW,
            Self::Process => CLASS_PROCESS,
            Self::Environment => CLASS_ENVIRONMENT,
            Self::Network => CLASS_NETWORK,
            Self::Http => CLASS_HTTP,
            Self::Determinism => CLASS_DETERMINISM,
        }
    }
}

/// Platform capability label for enforcing normalized descriptors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxPlatformCapability {
    /// Landlock-style filesystem constraints.
    LandlockFilesystem,
    /// seccomp-bpf syscall filtering.
    SeccompBpf,
    /// seccomp user-notification supervision.
    SeccompNotify,
    /// COW workdir staging.
    CowStaging,
    /// Resource-limit supervision.
    ResourceLimits,
    /// Network ACL enforcement.
    NetworkAcl,
    /// HTTP method/host/path ACL enforcement.
    HttpAcl,
    /// Deterministic time/randomness controls.
    DeterminismControls,
}

impl SandboxPlatformCapability {
    /// Parses a platform capability label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        parse_value(
            PLATFORM_CAPABILITY_FIELD,
            value,
            |normalized| match normalized {
                CAP_LANDLOCK_FILESYSTEM => Some(Self::LandlockFilesystem),
                CAP_SECCOMP_BPF => Some(Self::SeccompBpf),
                CAP_SECCOMP_NOTIFY => Some(Self::SeccompNotify),
                CAP_COW_STAGING => Some(Self::CowStaging),
                CAP_RESOURCE_LIMITS => Some(Self::ResourceLimits),
                CAP_NETWORK_ACL => Some(Self::NetworkAcl),
                CAP_HTTP_ACL => Some(Self::HttpAcl),
                CAP_DETERMINISM => Some(Self::DeterminismControls),
                _ => None,
            },
        )
    }

    /// Returns the stable platform capability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LandlockFilesystem => CAP_LANDLOCK_FILESYSTEM,
            Self::SeccompBpf => CAP_SECCOMP_BPF,
            Self::SeccompNotify => CAP_SECCOMP_NOTIFY,
            Self::CowStaging => CAP_COW_STAGING,
            Self::ResourceLimits => CAP_RESOURCE_LIMITS,
            Self::NetworkAcl => CAP_NETWORK_ACL,
            Self::HttpAcl => CAP_HTTP_ACL,
            Self::DeterminismControls => CAP_DETERMINISM,
        }
    }
}
