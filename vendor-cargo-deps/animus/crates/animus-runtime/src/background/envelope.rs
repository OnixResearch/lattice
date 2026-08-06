/// Safe profile identity metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessJobProfileIdentity {
    /// Manifest schema version.
    schema_version: u16,
    /// Safe source label.
    source_label: String,
    /// Policy hash.
    policy_hash: String,
    /// Profile name.
    profile_name: String,
    /// Selected backend.
    selected_backend: super::ProcessJobBackend,
    /// BLAKE3 hash over selected profile content.
    profile_hash: String,
    /// BLAKE3 hash over the backend-neutral request.
    request_hash: String,
}

pub(crate) struct ProfileIdentityInput<'a> {
    pub(crate) profile_name: &'a str,
    pub(crate) source_label: &'a str,
    pub(crate) policy_hash: &'a str,
    pub(crate) selected_backend: super::ProcessJobBackend,
    pub(crate) profile_hash: String,
    pub(crate) request_hash: String,
}

impl ProcessJobProfileIdentity {
    pub(crate) fn new(input: ProfileIdentityInput<'_>) -> Result<Self, super::ProcessJobProfileError> {
        debug_assert!(!input.profile_name.is_empty());
        debug_assert!(!input.source_label.is_empty());
        super::checks::validate_profile_label(input.profile_name)?;
        super::checks::helpers::validate_identity_field(super::checks::IdentityTextInput {
            field: super::IDENTITY_SOURCE_FIELD,
            value: input.source_label,
            profile_name: input.profile_name,
        })?;
        super::checks::helpers::validate_hash_field(super::checks::HashInput {
            field: super::IDENTITY_POLICY_HASH_FIELD,
            value: input.policy_hash,
            profile_name: input.profile_name,
        })?;
        super::checks::helpers::validate_hash_field(super::checks::HashInput {
            field: super::IDENTITY_PROFILE_HASH_FIELD,
            value: input.profile_hash.as_str(),
            profile_name: input.profile_name,
        })?;
        super::checks::helpers::validate_hash_field(super::checks::HashInput {
            field: super::IDENTITY_REQUEST_HASH_FIELD,
            value: input.request_hash.as_str(),
            profile_name: input.profile_name,
        })?;
        let identity = Self {
            schema_version: super::PROCESS_JOB_PROFILE_SCHEMA_VERSION,
            source_label: String::from(input.source_label),
            policy_hash: String::from(input.policy_hash),
            profile_name: String::from(input.profile_name),
            selected_backend: input.selected_backend,
            profile_hash: input.profile_hash,
            request_hash: input.request_hash,
        };
        identity.validate()?;
        Ok(identity)
    }

    /// Validates that the identity envelope contains only safe metadata.
    ///
    /// # Errors
    ///
    /// Returns [`super::ProcessJobProfileError`] when identity metadata is unsafe or malformed.
    pub fn validate(&self) -> Result<(), super::ProcessJobProfileError> {
        debug_assert!(!self.profile_name.is_empty());
        debug_assert!(!self.source_label.is_empty());
        super::checks::validate_schema_version(self.profile_name.as_str(), self.schema_version)?;
        super::checks::validate_profile_label(self.profile_name.as_str())?;
        super::checks::helpers::validate_identity_field(super::checks::IdentityTextInput {
            field: super::IDENTITY_SOURCE_FIELD,
            value: self.source_label.as_str(),
            profile_name: self.profile_name.as_str(),
        })?;
        super::checks::helpers::validate_hash_field(super::checks::HashInput {
            field: super::IDENTITY_POLICY_HASH_FIELD,
            value: self.policy_hash.as_str(),
            profile_name: self.profile_name.as_str(),
        })?;
        super::checks::helpers::validate_hash_field(super::checks::HashInput {
            field: super::IDENTITY_PROFILE_HASH_FIELD,
            value: self.profile_hash.as_str(),
            profile_name: self.profile_name.as_str(),
        })?;
        super::checks::helpers::validate_hash_field(super::checks::HashInput {
            field: super::IDENTITY_REQUEST_HASH_FIELD,
            value: self.request_hash.as_str(),
            profile_name: self.profile_name.as_str(),
        })
    }

    /// Returns the manifest schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    /// Returns the safe source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns the policy hash.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns the profile name.
    #[must_use]
    pub fn profile_name(&self) -> &str {
        self.profile_name.as_str()
    }

    /// Returns the selected backend.
    #[must_use]
    pub const fn selected_backend(&self) -> super::ProcessJobBackend {
        self.selected_backend
    }

    /// Returns the profile hash.
    #[must_use]
    pub fn profile_hash(&self) -> &str {
        self.profile_hash.as_str()
    }

    /// Returns the request hash.
    #[must_use]
    pub fn request_hash(&self) -> &str {
        self.request_hash.as_str()
    }
}

#[cfg(test)]
pub(crate) fn identity_with_source(
    identity: &ProcessJobProfileIdentity,
    source_label: &str,
) -> ProcessJobProfileIdentity {
    let mut changed = identity.clone();
    changed.source_label = String::from(source_label);
    changed
}
