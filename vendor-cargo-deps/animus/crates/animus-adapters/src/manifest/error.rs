/// Validation errors for descriptor manifests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationFailure {
    /// A required descriptor field was absent.
    MissingField {
        /// Missing field name.
        field: &'static str,
    },
    /// A descriptor field failed adapter-boundary text validation.
    FieldRejected {
        /// Rejected field name.
        field: &'static str,
        /// Text-validation reason.
        reason: crate::AdapterContractError,
    },
    /// A descriptor field used an unknown enumerated value.
    UnknownDescriptorValue {
        /// Field name with the unknown value.
        field: &'static str,
        /// Unknown value after normalization.
        value: alloc::string::String,
    },
    /// A declared tool name appeared more than once in the same descriptor.
    DuplicateTool {
        /// Duplicate tool name.
        name: alloc::string::String,
    },
    /// A declared tool name collided with a reserved or already registered name.
    ToolNameCollision {
        /// Colliding tool name.
        name: alloc::string::String,
    },
    /// A manifest role kind was not supported by Animus.
    UnsupportedRole {
        /// Unsupported role value.
        value: alloc::string::String,
    },
    /// A manifest role ID appeared more than once in the same descriptor.
    DuplicateRole {
        /// Duplicate role ID.
        id: alloc::string::String,
    },
    /// A role, compatibility, host, or context label contained unsafe syntax.
    UnsafeRoleLabel {
        /// Rejected field name.
        field: &'static str,
        /// Rejected value.
        value: alloc::string::String,
    },
    /// A compatibility version was malformed.
    InvalidCompatibilityVersion {
        /// Rejected field name.
        field: &'static str,
        /// Rejected value.
        value: alloc::string::String,
    },
    /// Invocation context included unsafe data that must not cross the boundary.
    UnsafeInvocationContext {
        /// Rejected field name.
        field: &'static str,
        /// Leak class.
        leak: super::PluginInvocationContextLeak,
    },
    /// A runtime kind that needs a data-only command label omitted it.
    MissingLaunchCommand {
        /// Runtime kind that requires the label.
        runtime_kind: super::RuntimeKind,
    },
    /// A writable-root descriptor was absolute.
    AbsoluteWritableRoot {
        /// Rejected writable-root value.
        value: alloc::string::String,
    },
    /// A writable-root descriptor escaped the project boundary.
    PathEscape {
        /// Rejected writable-root value.
        value: alloc::string::String,
    },
    /// Manifest provenance looked like an unsafe host path.
    UnsafeProvenance {
        /// Rejected provenance value.
        value: alloc::string::String,
    },
    /// An environment allowlist entry included a value instead of a name.
    EnvironmentValueLeak {
        /// Rejected environment entry.
        entry: alloc::string::String,
    },
    /// A network claim was unsafe for the sandbox claim.
    UnsafeNetworkSandboxCombination {
        /// Network policy claim.
        network: super::NetworkPolicy,
        /// Sandbox mode claim.
        sandbox: super::SandboxMode,
    },
}

impl core::fmt::Display for ValidationFailure {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingField { field } => write!(formatter, "{field} is required"),
            Self::FieldRejected { field, reason } => {
                write!(formatter, "{field} rejected: {reason}")
            }
            Self::UnknownDescriptorValue { field, value } => {
                write!(formatter, "{field} has unknown descriptor value {value}")
            }
            Self::DuplicateTool { name } => write!(formatter, "declared tool {name} is duplicated"),
            Self::ToolNameCollision { name } => {
                write!(
                    formatter,
                    "declared tool {name} collides with an existing tool"
                )
            }
            Self::UnsupportedRole { value } => write!(
                formatter,
                "plugin manifest role kind {value} is not supported"
            ),
            Self::DuplicateRole { id } => {
                write!(formatter, "plugin manifest role {id} is duplicated")
            }
            Self::UnsafeRoleLabel { field, value } => {
                write!(formatter, "{field} must be a safe label: {value}")
            }
            Self::InvalidCompatibilityVersion { field, value } => {
                write!(
                    formatter,
                    "{field} must be a semantic version label: {value}"
                )
            }
            Self::UnsafeInvocationContext { field, leak } => write!(
                formatter,
                "{field} contains unsafe invocation context data: {}",
                leak.as_str()
            ),
            Self::MissingLaunchCommand { runtime_kind } => write!(
                formatter,
                "{} descriptors require a data-only launch command label",
                runtime_kind.as_str()
            ),
            Self::AbsoluteWritableRoot { value } => {
                write!(formatter, "writable root {value} must be project-relative")
            }
            Self::PathEscape { value } => {
                write!(
                    formatter,
                    "writable root {value} must not escape the project"
                )
            }
            Self::UnsafeProvenance { value } => {
                write!(
                    formatter,
                    "manifest provenance must be a safe label: {value}"
                )
            }
            Self::EnvironmentValueLeak { entry } => write!(
                formatter,
                "environment allowlist entry {entry} must name a variable without a value"
            ),
            Self::UnsafeNetworkSandboxCombination { network, sandbox } => write!(
                formatter,
                "network policy {} is unsafe with sandbox mode {}",
                network.as_str(),
                sandbox.as_str()
            ),
        }
    }
}
