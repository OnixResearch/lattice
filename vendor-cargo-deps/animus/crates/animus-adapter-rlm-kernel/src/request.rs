use alloc::string::String;

const MAX_TARGET_CHARS: u32 = 1_024;
const MAX_TOOL_NAME_CHARS: u32 = 128;
const MAX_CONTEXT_KEY_CHARS: u32 = 256;

/// Typed host operation classes available to model-written Steel code.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HostOperationClass {
    /// Read through a host-owned file adapter.
    FileRead,
    /// Edit through a host-owned file adapter.
    FileEdit,
    /// Run a command through a host-owned process adapter.
    Shell,
    /// Call a registered tool through the tool executor kit.
    Tool,
    /// Spawn a bounded child through existing subagent owners.
    Subagent,
    /// Read an admitted context value through a host resolver.
    ContextRead,
    /// Clear the kernel through an explicit host action.
    KernelClear,
}

impl HostOperationClass {
    /// Returns the stable operation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FileRead => "file_read",
            Self::FileEdit => "file_edit",
            Self::Shell => "shell",
            Self::Tool => "tool",
            Self::Subagent => "subagent",
            Self::ContextRead => "context_read",
            Self::KernelClear => "kernel_clear",
        }
    }
}

/// Typed host request returned by model-written Steel code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostRequest {
    /// Read a caller-owned target.
    FileRead {
        /// Opaque target interpreted only by the host adapter.
        target: String,
    },
    /// Edit a caller-owned target.
    FileEdit {
        /// Opaque target interpreted only by the host adapter.
        target: String,
        /// Replacement payload passed only to the host adapter.
        replacement: String,
    },
    /// Run one host-admitted command request.
    Shell {
        /// Command request text interpreted only by the host adapter.
        command: String,
    },
    /// Call one registered host tool.
    Tool {
        /// Registered tool name.
        name: String,
        /// Typed tool payload serialized for the current tool schema.
        payload: String,
    },
    /// Spawn one bounded child agent.
    Subagent(crate::ChildSpawnRequest),
    /// Read one admitted context key.
    ContextRead {
        /// Context key resolved by the host.
        key: String,
    },
    /// Explicitly clear the current kernel.
    KernelClear,
}

impl HostRequest {
    /// Builds a file-read request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when the target is empty, oversized, or contains NUL.
    pub fn file_read(target: &str) -> Result<Self, crate::Error> {
        Ok(Self::FileRead {
            target: crate::required_text("target", target, MAX_TARGET_CHARS)?,
        })
    }

    /// Builds a file-edit request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when the target or replacement is malformed.
    pub fn file_edit(target: &str, replacement: &str, max_replacement_chars: u32) -> Result<Self, crate::Error> {
        Ok(Self::FileEdit {
            target: crate::required_text("target", target, MAX_TARGET_CHARS)?,
            replacement: crate::required_text("replacement", replacement, max_replacement_chars)?,
        })
    }

    /// Builds a shell request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when the command is malformed.
    pub fn shell(command: &str, max_command_chars: u32) -> Result<Self, crate::Error> {
        Ok(Self::Shell {
            command: crate::required_text("command", command, max_command_chars)?,
        })
    }

    /// Builds a registered-tool request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when the name or payload is malformed.
    pub fn tool(name: &str, payload: &str, max_payload_chars: u32) -> Result<Self, crate::Error> {
        Ok(Self::Tool {
            name: crate::required_text("tool_name", name, MAX_TOOL_NAME_CHARS)?,
            payload: crate::required_text("tool_payload", payload, max_payload_chars)?,
        })
    }

    /// Builds a context-read request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when the key is malformed.
    pub fn context_read(key: &str) -> Result<Self, crate::Error> {
        Ok(Self::ContextRead {
            key: crate::required_text("context_key", key, MAX_CONTEXT_KEY_CHARS)?,
        })
    }

    /// Returns the request operation class.
    #[must_use]
    pub const fn operation_class(&self) -> HostOperationClass {
        match self {
            Self::FileRead { .. } => HostOperationClass::FileRead,
            Self::FileEdit { .. } => HostOperationClass::FileEdit,
            Self::Shell { .. } => HostOperationClass::Shell,
            Self::Tool { .. } => HostOperationClass::Tool,
            Self::Subagent(_) => HostOperationClass::Subagent,
            Self::ContextRead { .. } => HostOperationClass::ContextRead,
            Self::KernelClear => HostOperationClass::KernelClear,
        }
    }

    /// Returns the bounded request character count without exposing request content.
    #[must_use]
    pub fn character_count(&self) -> u32 {
        let count = match self {
            Self::FileRead { target } => target.chars().count(),
            Self::FileEdit { target, replacement } => {
                target.chars().count().saturating_add(replacement.chars().count())
            }
            Self::Shell { command } => command.chars().count(),
            Self::Tool { name, payload } => name.chars().count().saturating_add(payload.chars().count()),
            Self::Subagent(request) => request.character_count(),
            Self::ContextRead { key } => key.chars().count(),
            Self::KernelClear => 0,
        };
        crate::usize_to_u32(count)
    }
}

/// Pure admitted host-request plan before any effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostRequestAdmission {
    operation: HostOperationClass,
    request_chars: u32,
    policy_identity: String,
}

impl HostRequestAdmission {
    /// Returns the admitted operation class.
    #[must_use]
    pub const fn operation(&self) -> HostOperationClass {
        self.operation
    }

    /// Returns the bounded request character count.
    #[must_use]
    pub const fn request_chars(&self) -> u32 {
        self.request_chars
    }

    /// Returns the policy identity that admitted the request.
    #[must_use]
    pub fn policy_identity(&self) -> &str {
        self.policy_identity.as_str()
    }
}

/// Revalidates a typed host request before host execution.
///
/// # Errors
///
/// Returns [`crate::Error`] when the operation class is not admitted or the
/// request exceeds policy. No effect occurs in this function.
pub fn admit_host_request(
    policy: &crate::RlmKernelPolicy,
    request: &HostRequest,
) -> Result<HostRequestAdmission, crate::Error> {
    let operation = request.operation_class();
    if !policy.admits_operation(operation) {
        return Err(crate::Error::OperationNotAdmitted { operation });
    }
    if let HostRequest::Subagent(child) = request {
        crate::admit_child_spawn(policy, child)?;
    }
    let request_chars = request.character_count();
    crate::ensure_at_most(
        "host_request_chars",
        u64::from(request_chars),
        u64::from(policy.bounds().max_host_request_chars),
    )?;
    Ok(HostRequestAdmission {
        operation,
        request_chars,
        policy_identity: String::from(policy.identity()),
    })
}
