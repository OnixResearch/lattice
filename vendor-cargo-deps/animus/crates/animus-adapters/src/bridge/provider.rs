use super::{
    AcpPermissionMode, AcpSessionCorrelation, AcpToolPassThroughPolicy, Collection, ID_FIELD,
    InteropError, LIMITATION_FIELD, PROFILE_FIELD, Text, required_text, text_collection,
};

/// Untrusted ACP provider descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcpDescriptorDraft<'a> {
    /// External agent identity.
    pub identity: &'a str,
    /// Model or profile preference label.
    pub model_profile: &'a str,
    /// Animus permission mode label.
    pub animus_permission_mode: &'a str,
    /// External ACP permission mode label.
    pub external_permission_mode: &'a str,
    /// Session correlation policy label.
    pub session_correlation: &'a str,
    /// Known limitation labels.
    pub known_limitations: &'a [&'a str],
    /// Tool pass-through policy label.
    pub tool_pass_through_policy: &'a str,
    /// Whether host policy admits an unrestricted external mapping.
    pub host_admits_unrestricted: bool,
}

/// Validated ACP provider descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcpDescriptor {
    identity: Text,
    model_profile: Text,
    animus_permission_mode: AcpPermissionMode,
    external_permission_mode: AcpPermissionMode,
    session_correlation: AcpSessionCorrelation,
    known_limitations: Collection<Text>,
    tool_pass_through_policy: AcpToolPassThroughPolicy,
}

impl AcpDescriptor {
    /// Validates ACP provider metadata without contacting an external agent.
    ///
    /// # Errors
    ///
    /// Returns [`InteropError`] when labels are malformed, session correlation is
    /// missing, or permission mapping would downgrade guarded Animus behavior.
    pub fn validate(draft: &AcpDescriptorDraft<'_>) -> Result<Self, InteropError> {
        let identity = required_text(ID_FIELD, draft.identity)?;
        let model_profile = required_text(PROFILE_FIELD, draft.model_profile)?;
        let animus_permission_mode = AcpPermissionMode::parse(draft.animus_permission_mode)?;
        let external_permission_mode = AcpPermissionMode::parse(draft.external_permission_mode)?;
        if is_unsafe_permission_mapping(
            animus_permission_mode,
            external_permission_mode,
            draft.host_admits_unrestricted,
        ) {
            return Err(InteropError::UnsafePermissionMapping);
        }
        let session_correlation = AcpSessionCorrelation::parse(draft.session_correlation)?;
        let notes = text_collection(LIMITATION_FIELD, draft.known_limitations, false)?;
        let tool_pass_through_policy =
            AcpToolPassThroughPolicy::parse(draft.tool_pass_through_policy)?;
        Ok(Self {
            identity,
            model_profile,
            animus_permission_mode,
            external_permission_mode,
            session_correlation,
            known_limitations: notes,
            tool_pass_through_policy,
        })
    }

    /// Returns external agent identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns model/profile preference.
    #[must_use]
    pub fn model_profile(&self) -> &str {
        self.model_profile.as_str()
    }

    /// Returns Animus permission mode.
    #[must_use]
    pub const fn animus_permission_mode(&self) -> AcpPermissionMode {
        self.animus_permission_mode
    }

    /// Returns external ACP permission mode.
    #[must_use]
    pub const fn external_permission_mode(&self) -> AcpPermissionMode {
        self.external_permission_mode
    }

    /// Returns session correlation policy.
    #[must_use]
    pub const fn session_correlation(&self) -> AcpSessionCorrelation {
        self.session_correlation
    }

    /// Returns known limitation labels.
    #[must_use]
    pub fn known_limitations(&self) -> &[Text] {
        self.known_limitations.as_slice()
    }

    /// Returns tool pass-through policy.
    #[must_use]
    pub const fn tool_pass_through_policy(&self) -> AcpToolPassThroughPolicy {
        self.tool_pass_through_policy
    }
}
fn is_unsafe_permission_mapping(
    animus: AcpPermissionMode,
    external: AcpPermissionMode,
    host_admits_unrestricted: bool,
) -> bool {
    external == AcpPermissionMode::Unrestricted
        && animus != AcpPermissionMode::Unrestricted
        && !host_admits_unrestricted
}
