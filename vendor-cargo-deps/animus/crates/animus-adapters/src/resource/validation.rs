use super::{
    CONTENT_IDENTITY_FIELD, CONTEXT_GUIDANCE_KIND_LABEL, DESCRIPTION_FIELD, Descriptor, Draft,
    Error, GLOBAL_SCOPE_LABEL, HOST_RESOURCE_KIND_LABEL, HOST_SCOPE_LABEL, KIND_FIELD, Kind,
    MAX_DESCRIPTION_CHARS, NAME_FIELD, NAME_SEPARATOR_DASH, NAME_SEPARATOR_UNDERSCORE,
    PROJECT_SCOPE_LABEL, PROMPT_TEMPLATE_KIND_LABEL, PROVENANCE_FIELD, SCOPE_FIELD,
    SESSION_SCOPE_LABEL, SKILL_KIND_LABEL, Scope, TRUST_FIELD, TRUSTED_LABEL, Text, TrustStatus,
    UNSAFE_PROVENANCE_MARKERS, UNTRUSTED_LABEL,
};

impl Descriptor {
    /// Validates host-supplied resource fields as typed data only.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when a required field is missing, malformed,
    /// unsupported, unsafe, or not a safe BLAKE3 content identity.
    pub fn validate(draft: &Draft<'_>) -> Result<Self, Error> {
        let kind = Kind::parse(draft.kind)?;
        let name = required_name(NAME_FIELD, draft.name)?;
        let description = required_description(draft.description)?;
        let scope = Scope::parse(draft.scope)?;
        let provenance = required_provenance(draft.provenance)?;
        let trust_status = TrustStatus::parse(draft.trust_status)?;
        let content_identity = required_content_identity(draft.content_identity)?;

        assert!(!name.is_empty(), "validated descriptor has a name");
        assert!(
            description.chars().count() <= MAX_DESCRIPTION_CHARS,
            "validated resource summary stays bounded"
        );
        assert!(
            !provenance.is_empty(),
            "validated descriptor has safe provenance"
        );
        assert_eq!(
            content_identity.as_str().len(),
            crate::artifact::BLAKE3_HASH_HEX_CHARS,
            "validated content identity is a full BLAKE3 hex string"
        );

        Ok(Self {
            kind,
            name,
            description,
            scope,
            provenance,
            trust_status,
            content_identity,
        })
    }
}

impl Kind {
    fn parse(value: Option<&str>) -> Result<Self, Error> {
        let normalized = required_text(KIND_FIELD, value)?;
        match normalized.as_str() {
            SKILL_KIND_LABEL => Ok(Self::Skill),
            PROMPT_TEMPLATE_KIND_LABEL => Ok(Self::PromptTemplate),
            CONTEXT_GUIDANCE_KIND_LABEL => Ok(Self::ContextGuidance),
            HOST_RESOURCE_KIND_LABEL => Ok(Self::HostResource),
            unknown => Err(Error::UnsupportedValue {
                field: KIND_FIELD.as_str(),
                value: Text::from(unknown),
            }),
        }
    }
}

impl Scope {
    fn parse(value: Option<&str>) -> Result<Self, Error> {
        let normalized = required_text(SCOPE_FIELD, value)?;
        match normalized.as_str() {
            GLOBAL_SCOPE_LABEL => Ok(Self::Global),
            PROJECT_SCOPE_LABEL => Ok(Self::Project),
            SESSION_SCOPE_LABEL => Ok(Self::Session),
            HOST_SCOPE_LABEL => Ok(Self::Host),
            unknown => Err(Error::UnsupportedValue {
                field: SCOPE_FIELD.as_str(),
                value: Text::from(unknown),
            }),
        }
    }
}

impl TrustStatus {
    fn parse(value: Option<&str>) -> Result<Self, Error> {
        let normalized = required_text(TRUST_FIELD, value)?;
        match normalized.as_str() {
            TRUSTED_LABEL => Ok(Self::Trusted),
            UNTRUSTED_LABEL => Ok(Self::Untrusted),
            unknown => Err(Error::UnsupportedValue {
                field: TRUST_FIELD.as_str(),
                value: Text::from(unknown),
            }),
        }
    }
}

fn required_description(value: Option<&str>) -> Result<Text, Error> {
    let description = required_text(DESCRIPTION_FIELD, value)?;
    let actual_chars = description.chars().count();
    if actual_chars > MAX_DESCRIPTION_CHARS {
        return Err(Error::DescriptionTooLong {
            max_chars: MAX_DESCRIPTION_CHARS,
            actual_chars,
        });
    }
    Ok(description)
}

fn required_name(field: crate::validation::FieldName, value: Option<&str>) -> Result<Text, Error> {
    let name = required_text(field, value)?;
    validate_name(field, name.as_str())?;
    Ok(name)
}

fn required_provenance(value: Option<&str>) -> Result<Text, Error> {
    let provenance = required_text(PROVENANCE_FIELD, value)?;
    if !valid_provenance(provenance.as_str()) {
        return Err(Error::MalformedProvenance { value: provenance });
    }
    Ok(provenance)
}

pub(super) fn required_text(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Text, Error> {
    match value {
        Some(raw) => crate::validation::normalized_adapter_field(field, raw).map_err(Into::into),
        None => Err(Error::MissingField {
            field: field.as_str(),
        }),
    }
}

fn required_content_identity(value: Option<&str>) -> Result<crate::artifact::Hash, Error> {
    let identity = required_text(CONTENT_IDENTITY_FIELD, value)?;
    crate::artifact::Hash::new(identity.as_str()).map_err(|error| content_identity_error(&error))
}

fn content_identity_error(error: &crate::artifact::Error) -> Error {
    match error {
        crate::artifact::Error::InvalidHash { reason, .. } => Error::InvalidContentIdentity {
            field: CONTENT_IDENTITY_FIELD.as_str(),
            reason,
        },
        crate::artifact::Error::PayloadTooLong { .. }
        | crate::artifact::Error::PayloadContainsNul { .. } => Error::InvalidContentIdentity {
            field: CONTENT_IDENTITY_FIELD.as_str(),
            reason: "resource content identity must be lowercase BLAKE3 hex",
        },
    }
}

fn validate_name(field: crate::validation::FieldName, value: &str) -> Result<(), Error> {
    if valid_name(value) {
        Ok(())
    } else {
        Err(Error::MalformedName {
            field: field.as_str(),
            value: Text::from(value),
        })
    }
}

fn valid_name(value: &str) -> bool {
    assert!(!value.is_empty(), "name candidate is non-empty");
    assert!(
        value.chars().count() <= crate::validation::MAX_ADAPTER_FIELD_CHARS,
        "name candidate fits the adapter field bound"
    );
    let mut is_previous_separator = false;
    let mut has_visible_character = false;
    for character in value.chars() {
        let is_name_separator =
            character == NAME_SEPARATOR_DASH || character == NAME_SEPARATOR_UNDERSCORE;
        if is_name_separator {
            if !has_visible_character || is_previous_separator {
                return false;
            }
            is_previous_separator = true;
            continue;
        }
        if !(character.is_ascii_lowercase() || character.is_ascii_digit()) {
            return false;
        }
        is_previous_separator = false;
        has_visible_character = true;
    }
    has_visible_character && !is_previous_separator
}

fn valid_provenance(value: &str) -> bool {
    assert!(!value.is_empty(), "provenance candidate is non-empty");
    if value.starts_with('/') {
        return false;
    }
    !UNSAFE_PROVENANCE_MARKERS
        .iter()
        .any(|marker| value.contains(marker))
}
