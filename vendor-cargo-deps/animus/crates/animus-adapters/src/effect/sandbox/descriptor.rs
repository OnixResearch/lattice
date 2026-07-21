/// Normalized portable sandbox descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxDescriptor {
    descriptor_id: alloc::string::String,
    filesystem: SandboxFilesystemMode,
    writable_roots: alloc::vec::Vec<alloc::string::String>,
    workdir: SandboxWorkdirMode,
    cow: SandboxCowDisposition,
    process: SandboxProcessProfile,
    environment: SandboxEnvironmentPolicy,
    network: SandboxNetworkPolicy,
    http: SandboxHttpPolicy,
    determinism: SandboxDeterminismPolicy,
    enforcement: SandboxEnforcementRequirement,
    descriptor_hash: super::SafeHash,
}

impl SandboxDescriptor {
    /// Validates and normalizes a portable sandbox descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when required fields are empty,
    /// unsupported, malformed, duplicated, or mutually conflicting.
    pub fn new(draft: SandboxDescriptorDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        let descriptor_id =
            crate::validation::normalized_adapter_field(DESCRIPTOR_ID_FIELD, draft.descriptor_id)?;
        let filesystem = SandboxFilesystemMode::parse(draft.filesystem_mode)?;
        let writable_roots = required_unique_fields(draft.writable_roots)?;
        let workdir = SandboxWorkdirMode::parse(draft.workdir_mode)?;
        let cow = SandboxCowDisposition::parse(draft.cow_disposition)?;
        let process = SandboxProcessProfile::parse(draft.process_profile)?;
        let environment = SandboxEnvironmentPolicy::parse(draft.environment_policy)?;
        let network = SandboxNetworkPolicy::parse(draft.network_policy)?;
        let http = SandboxHttpPolicy::parse(draft.http_policy)?;
        let determinism = SandboxDeterminismPolicy::parse(draft.determinism_policy)?;
        let enforcement = SandboxEnforcementRequirement::parse(draft.enforcement_requirement)?;
        reject_descriptor_conflicts(DescriptorConflictInput {
            workdir,
            cow,
            network,
            http,
        })?;
        let descriptor_hash_input = DescriptorHashInput {
            descriptor_id: descriptor_id.as_str(),
            filesystem,
            writable_roots: writable_roots.as_slice(),
            workdir,
            cow,
            process,
            environment,
            network,
            http,
            determinism,
            enforcement,
        };
        let descriptor_hash = descriptor_hash(&descriptor_hash_input);
        Ok(Self {
            descriptor_id,
            filesystem,
            writable_roots,
            workdir,
            cow,
            process,
            environment,
            network,
            http,
            determinism,
            enforcement,
            descriptor_hash,
        })
    }

    /// Returns the descriptor identity.
    #[must_use]
    pub fn descriptor_id(&self) -> &str {
        self.descriptor_id.as_str()
    }

    /// Returns the filesystem mode.
    #[must_use]
    pub const fn filesystem_mode(&self) -> SandboxFilesystemMode {
        self.filesystem
    }

    /// Returns writable root labels.
    #[must_use]
    pub fn writable_roots(&self) -> &[alloc::string::String] {
        self.writable_roots.as_slice()
    }

    /// Returns the workdir mode.
    #[must_use]
    pub const fn workdir_mode(&self) -> SandboxWorkdirMode {
        self.workdir
    }

    /// Returns the COW disposition.
    #[must_use]
    pub const fn cow_disposition(&self) -> SandboxCowDisposition {
        self.cow
    }

    /// Returns the process profile.
    #[must_use]
    pub const fn process_profile(&self) -> SandboxProcessProfile {
        self.process
    }

    /// Returns the environment policy.
    #[must_use]
    pub const fn environment_policy(&self) -> SandboxEnvironmentPolicy {
        self.environment
    }

    /// Returns the network policy.
    #[must_use]
    pub const fn network_policy(&self) -> SandboxNetworkPolicy {
        self.network
    }

    /// Returns the HTTP policy.
    #[must_use]
    pub const fn http_policy(&self) -> SandboxHttpPolicy {
        self.http
    }

    /// Returns the determinism policy.
    #[must_use]
    pub const fn determinism_policy(&self) -> SandboxDeterminismPolicy {
        self.determinism
    }

    /// Returns the enforcement requirement.
    #[must_use]
    pub const fn enforcement_requirement(&self) -> SandboxEnforcementRequirement {
        self.enforcement
    }

    /// Returns the BLAKE3 hash of normalized descriptor intent.
    #[must_use]
    pub const fn descriptor_hash(&self) -> &super::SafeHash {
        &self.descriptor_hash
    }

    /// Returns policy constraint classes required by this descriptor.
    #[must_use]
    pub fn constraint_classes(&self) -> alloc::vec::Vec<SandboxConstraintClass> {
        let mut classes = alloc::vec![
            SandboxConstraintClass::Filesystem,
            SandboxConstraintClass::Process,
            SandboxConstraintClass::Environment,
        ];
        if self.workdir == SandboxWorkdirMode::CowStaged
            || self.cow != SandboxCowDisposition::Disabled
        {
            classes.push(SandboxConstraintClass::WorkdirCow);
        }
        if self.network != SandboxNetworkPolicy::DenyAll {
            classes.push(SandboxConstraintClass::Network);
        }
        if self.http != SandboxHttpPolicy::None {
            classes.push(SandboxConstraintClass::Http);
        }
        if self.determinism != SandboxDeterminismPolicy::Host {
            classes.push(SandboxConstraintClass::Determinism);
        }
        classes
    }

    /// Returns platform capabilities required by this descriptor.
    #[must_use]
    pub fn required_platform_capabilities(&self) -> alloc::vec::Vec<SandboxPlatformCapability> {
        let mut capabilities = alloc::vec![
            SandboxPlatformCapability::LandlockFilesystem,
            SandboxPlatformCapability::SeccompBpf,
        ];
        if self.workdir == SandboxWorkdirMode::CowStaged
            || self.cow != SandboxCowDisposition::Disabled
        {
            capabilities.push(SandboxPlatformCapability::CowStaging);
        }
        if self.process == SandboxProcessProfile::ResourceLimited {
            capabilities.push(SandboxPlatformCapability::ResourceLimits);
        }
        if self.network != SandboxNetworkPolicy::DenyAll {
            capabilities.push(SandboxPlatformCapability::NetworkAcl);
        }
        if self.http != SandboxHttpPolicy::None {
            capabilities.push(SandboxPlatformCapability::HttpAcl);
        }
        if self.determinism != SandboxDeterminismPolicy::Host {
            capabilities.push(SandboxPlatformCapability::DeterminismControls);
        }
        capabilities
    }
}

/// Borrowed portable sandbox descriptor input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SandboxDescriptorDraft<'a> {
    /// Stable descriptor identity.
    pub descriptor_id: &'a str,
    /// Filesystem mode label.
    pub filesystem_mode: &'a str,
    /// Safe writable root labels.
    pub writable_roots: &'a [&'a str],
    /// Workdir mode label.
    pub workdir_mode: &'a str,
    /// COW disposition label.
    pub cow_disposition: &'a str,
    /// Process profile label.
    pub process_profile: &'a str,
    /// Environment policy label.
    pub environment_policy: &'a str,
    /// Network policy label.
    pub network_policy: &'a str,
    /// HTTP policy label.
    pub http_policy: &'a str,
    /// Determinism policy label.
    pub determinism_policy: &'a str,
    /// Enforcement requirement label.
    pub enforcement_requirement: &'a str,
}
