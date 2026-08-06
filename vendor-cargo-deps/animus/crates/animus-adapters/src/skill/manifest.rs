/// Borrowed host-supplied skill manifest fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    /// Stable user-facing skill name.
    pub name: Option<&'a str>,
    /// Safe human-facing summary shown before full skill loading.
    pub description: Option<&'a str>,
    /// Descriptor contract version in `major.minor.patch` form.
    pub version: Option<&'a str>,
    /// Host-provided category metadata; it is not an authorization class.
    pub category: Option<&'a str>,
    /// Optional host-provided tag labels.
    pub tags: &'a [&'a str],
    /// Optional related skill names declared by this descriptor.
    pub related_skills: &'a [&'a str],
    /// Safe provenance label suitable for model-visible summaries.
    pub provenance: Option<&'a str>,
    /// Host trust status for the manifest source.
    pub trust_status: Option<&'a str>,
    /// Lowercase BLAKE3 identity for the full skill body supplied by the host.
    pub content_identity: Option<&'a str>,
}

/// Validated skill manifest descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Descriptor {
    pub(super) name: super::Text,
    pub(super) description: super::Text,
    pub(super) version: super::Text,
    pub(super) category: super::Text,
    pub(super) tags: super::Collection<super::Text>,
    pub(super) related_skills: super::Collection<super::Text>,
    pub(super) provenance: super::Text,
    pub(super) trust_status: TrustStatus,
    pub(super) content_identity: crate::artifact::Hash,
}

impl Descriptor {
    /// Validates host-supplied skill manifest fields as typed data only.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when a required field is missing, malformed,
    /// unsupported, duplicated, too large, or not a safe BLAKE3 content
    /// identity.
    pub fn validate(draft: &Draft<'_>) -> Result<Self, Error> {
        super::validation::validate(draft)
    }

    /// Returns the validated skill name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the safe human-facing summary.
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns the manifest contract version.
    #[must_use]
    pub fn version(&self) -> &str {
        self.version.as_str()
    }

    /// Returns the host-provided category metadata.
    #[must_use]
    pub fn category(&self) -> &str {
        self.category.as_str()
    }

    /// Returns sorted unique tag labels.
    #[must_use]
    pub fn tags(&self) -> &[super::Text] {
        self.tags.as_slice()
    }

    /// Returns sorted unique related skill names.
    #[must_use]
    pub fn related_skills(&self) -> &[super::Text] {
        self.related_skills.as_slice()
    }

    /// Returns the safe provenance label.
    #[must_use]
    pub fn provenance(&self) -> &str {
        self.provenance.as_str()
    }

    /// Returns the host trust status.
    #[must_use]
    pub const fn trust_status(&self) -> TrustStatus {
        self.trust_status
    }

    /// Returns the safe content identity for the full body.
    #[must_use]
    pub fn content_identity(&self) -> &crate::artifact::Hash {
        &self.content_identity
    }

    pub(super) fn resource_summary(&self) -> super::Summary {
        super::Summary {
            name: self.name.clone(),
            description: self.description.clone(),
            category: self.category.clone(),
            tags: self.tags.clone(),
            provenance: self.provenance.clone(),
        }
    }
}

/// Host trust status for a skill manifest source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustStatus {
    /// Host has marked the skill source trusted for explicit full-body loading.
    Trusted,
    /// Host has not admitted the skill source for full-body loading.
    Untrusted,
}

impl TrustStatus {
    /// Returns the stable manifest label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => super::TRUSTED_LABEL,
            Self::Untrusted => super::UNTRUSTED_LABEL,
        }
    }

    pub(super) fn parse(value: Option<&str>) -> Result<Self, Error> {
        let normalized = super::validation::required_text(super::TRUST_FIELD, value)?;
        match normalized.as_str() {
            super::TRUSTED_LABEL => Ok(Self::Trusted),
            super::UNTRUSTED_LABEL => Ok(Self::Untrusted),
            unknown => Err(Error::UnsupportedValue {
                field: super::TRUST_FIELD.as_str(),
                value: super::Text::from(unknown),
            }),
        }
    }
}

/// Skill manifest validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A required manifest field was absent.
    MissingField {
        /// Missing field name.
        field: &'static str,
    },
    /// A field failed adapter-boundary validation.
    Contract(crate::AdapterContractError),
    /// A name-like field used an unsupported character or separator shape.
    MalformedName {
        /// Field containing the malformed name.
        field: &'static str,
        /// Malformed normalized value.
        value: super::Text,
    },
    /// A description exceeded the safe summary bound.
    DescriptionTooLong {
        /// Maximum accepted description characters.
        max_chars: usize,
        /// Actual description characters.
        actual_chars: usize,
    },
    /// Version did not use the supported semantic shape.
    UnsupportedVersion {
        /// Unsupported normalized version value.
        value: super::Text,
    },
    /// A tag label was duplicated after normalization.
    DuplicateTag {
        /// Duplicated tag value.
        tag: super::Text,
    },
    /// A collection exceeded the bounded item limit.
    TooManyItems {
        /// Field containing too many items.
        field: &'static str,
        /// Maximum supported items.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// Content identity failed lowercase BLAKE3 validation.
    InvalidContentIdentity {
        /// Field containing the unsafe identity.
        field: &'static str,
        /// Human-readable validation reason.
        reason: &'static str,
    },
    /// A manifest label used an unsupported value.
    UnsupportedValue {
        /// Field containing the unsupported value.
        field: &'static str,
        /// Unsupported normalized value.
        value: super::Text,
    },
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingField { field } => write!(formatter, "{field} is required"),
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::MalformedName { field, value } => {
                write!(formatter, "{field} has malformed skill name {value}")
            }
            Self::DescriptionTooLong {
                max_chars,
                actual_chars,
            } => write!(formatter, "skill description supports at most {max_chars} characters; got {actual_chars}"),
            Self::UnsupportedVersion { value } => {
                write!(formatter, "skill_version has unsupported version {value}")
            }
            Self::DuplicateTag { tag } => write!(formatter, "skill tag {tag} is duplicated"),
            Self::TooManyItems {
                field,
                max_items,
                actual_items,
            } => write!(formatter, "{field} supports at most {max_items} items; got {actual_items}"),
            Self::InvalidContentIdentity { field, reason } => {
                write!(formatter, "{field} is invalid: {reason}")
            }
            Self::UnsupportedValue { field, value } => {
                write!(formatter, "{field} has unsupported skill value {value}")
            }
        }
    }
}

impl From<crate::AdapterContractError> for Error {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}
