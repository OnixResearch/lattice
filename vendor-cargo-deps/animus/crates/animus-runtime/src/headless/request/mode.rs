/// Noninteractive behavior for prompt or permission decisions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoninteractiveBehavior {
    /// Fail closed with a typed prompt-required or input-required completion.
    FailClosed,
    /// Use preauthorized host decisions supplied with the request.
    Preauthorized,
}

impl NoninteractiveBehavior {
    pub(super) fn parse(value: &str) -> Option<Self> {
        match value {
            super::super::NONINTERACTIVE_FAIL_CLOSED => Some(Self::FailClosed),
            super::super::NONINTERACTIVE_PREAUTHORIZED => Some(Self::Preauthorized),
            _unsupported => None,
        }
    }

    /// Returns the stable behavior label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FailClosed => super::super::NONINTERACTIVE_FAIL_CLOSED,
            Self::Preauthorized => super::super::NONINTERACTIVE_PREAUTHORIZED,
        }
    }
}

/// Runtime sandbox profile requested by automation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxProfile {
    /// Read-only workspace access.
    ReadOnlyWorkspace,
    /// Workspace write access.
    WorkspaceWrite,
    /// Full host access; must be guarded by policy before use.
    FullAccess,
}

impl SandboxProfile {
    pub(super) fn parse(value: &str) -> Option<Self> {
        match value {
            super::super::SANDBOX_READ_ONLY => Some(Self::ReadOnlyWorkspace),
            super::super::SANDBOX_WORKSPACE_WRITE => Some(Self::WorkspaceWrite),
            super::super::SANDBOX_FULL_ACCESS => Some(Self::FullAccess),
            _unsupported => None,
        }
    }

    /// Returns the stable sandbox label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnlyWorkspace => super::super::SANDBOX_READ_ONLY,
            Self::WorkspaceWrite => super::super::SANDBOX_WORKSPACE_WRITE,
            Self::FullAccess => super::super::SANDBOX_FULL_ACCESS,
        }
    }
}

/// Final-output preference for non-interactive automation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutputPreference {
    /// Emit plain text from the terminal response item.
    Text,
    /// Validate final response against a schema identity before success.
    Structured {
        /// Schema identity supplied by the shell/controller.
        schema_id: super::super::Text,
    },
}

impl OutputPreference {
    pub(super) fn from_draft(
        output_mode: &str,
        structured_schema_id: Option<&str>,
    ) -> Result<Self, super::super::AutomationError> {
        match output_mode {
            super::super::OUTPUT_TEXT => Ok(Self::Text),
            super::super::OUTPUT_STRUCTURED => Ok(Self::Structured {
                schema_id: super::super::required_optional_text(super::super::OptionalTextField {
                    field: super::super::SCHEMA_ID_FIELD,
                    value: structured_schema_id,
                })?,
            }),
            _unsupported => Err(super::super::unsupported(super::super::TextField {
                field: super::super::OUTPUT_MODE_FIELD,
                value: output_mode,
            })),
        }
    }

    /// Returns the stable output mode label.
    #[must_use]
    pub const fn mode_label(&self) -> &'static str {
        match self {
            Self::Text => super::super::OUTPUT_TEXT,
            Self::Structured { .. } => super::super::OUTPUT_STRUCTURED,
        }
    }

    /// Returns the schema identity for structured output.
    #[must_use]
    pub fn schema_id(&self) -> Option<&str> {
        match self {
            Self::Text => None,
            Self::Structured { schema_id } => Some(schema_id.as_str()),
        }
    }
}
