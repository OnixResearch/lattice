use super::Collection;
use super::EXTERNAL_SERVICE_FIELD;
use super::ExternalSubagentProtocol;
use super::ID_FIELD;
use super::PROFILE_FIELD;
use super::SubagentError;
use super::TOOL_FIELD;
use super::Text;
use super::optional_text;
use super::required_text;
use super::text_collection;

/// Untrusted external subagent descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExternalSubagentDescriptorDraft<'a> {
    /// Protocol label.
    pub protocol: &'a str,
    /// External agent identity label.
    pub identity: &'a str,
    /// Injected host service label.
    pub service_label: &'a str,
    /// Declared tool labels available to the external agent.
    pub declared_tools: &'a [&'a str],
    /// Optional credential reference label; never inherited ambient credentials.
    pub credential_label: Option<&'a str>,
    /// Whether the descriptor asks runtime to discover an ambient CLI.
    pub ambient_cli: bool,
    /// Whether the descriptor asks to inherit parent credentials.
    pub inherited_credentials: bool,
}

/// Validated external subagent descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalSubagentDescriptor {
    protocol: ExternalSubagentProtocol,
    identity: Text,
    service_label: Text,
    declared_tools: Collection<Text>,
    credential_label: Option<Text>,
}

impl ExternalSubagentDescriptor {
    /// Validates an external subagent descriptor as injected-adapter data only.
    ///
    /// # Errors
    ///
    /// Returns [`SubagentError`] when protocol, service, tools, or credential
    /// boundaries are malformed or ambient discovery is requested.
    pub fn validate(draft: &ExternalSubagentDescriptorDraft<'_>) -> Result<Self, SubagentError> {
        if draft.ambient_cli {
            return Err(SubagentError::AmbientExternalDiscoveryDenied);
        }
        if draft.inherited_credentials {
            return Err(SubagentError::InheritedCredentialDenied);
        }
        let protocol = ExternalSubagentProtocol::parse(draft.protocol)?;
        let identity = required_text(ID_FIELD, draft.identity)?;
        let service_label =
            required_text(EXTERNAL_SERVICE_FIELD, draft.service_label).map_err(|error| match error {
                SubagentError::Contract(crate::AdapterContractError::EmptyField { .. }) => {
                    SubagentError::MissingExternalService
                }
                other => other,
            })?;
        let declared_tools = text_collection(TOOL_FIELD, draft.declared_tools, true)?;
        let credential_label = optional_text(PROFILE_FIELD, draft.credential_label)?;
        Ok(Self {
            protocol,
            identity,
            service_label,
            declared_tools,
            credential_label,
        })
    }

    /// Returns the external protocol.
    #[must_use]
    pub const fn protocol(&self) -> ExternalSubagentProtocol {
        self.protocol
    }

    /// Returns the external identity label.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns the injected service label.
    #[must_use]
    pub fn service_label(&self) -> &str {
        self.service_label.as_str()
    }

    /// Returns declared tool labels.
    #[must_use]
    pub fn declared_tools(&self) -> &[Text] {
        self.declared_tools.as_slice()
    }

    /// Returns the optional credential reference label.
    #[must_use]
    pub fn credential_label(&self) -> Option<&str> {
        self.credential_label.as_deref()
    }
}
