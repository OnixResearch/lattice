const INCLUDE_FILE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("ignored_file_label");
const INCLUDE_PROVENANCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("ignored_file_provenance");
const INCLUDE_REDACTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("ignored_file_redaction_class");
const INCLUDE_APPROVAL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("ignored_file_redaction_approval");

const REDACTION_PUBLIC: &str = "public";
const REDACTION_LOCAL_CONFIG: &str = "local_config";
const REDACTION_SECRET: &str = "secret";
const APPROVAL_APPROVED: &str = "approved";
const APPROVAL_DENIED: &str = "denied";
const STATUS_MATERIALIZED: &str = "materialized";
const STATUS_COPY_DENIED: &str = "copy_denied";
const REASON_NONE: &str = "none";
const REASON_MISSING_ALLOWLIST: &str = "missing_allowlist";
const REASON_SECRET_WITHOUT_APPROVAL: &str = "secret_without_approval";
const REASON_REDACTION_REJECTED: &str = "redaction_rejected";

/// Redaction classification for ignored or local-only file materialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IgnoredFileRedactionClass {
    /// Public local file with no secret-bearing body.
    Public,
    /// Local configuration file requiring explicit provenance.
    LocalConfig,
    /// Secret-bearing file; must not be copied without explicit redaction approval.
    Secret,
}

impl IgnoredFileRedactionClass {
    /// Parses an ignored-file redaction class.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(
            INCLUDE_REDACTION_FIELD,
            value,
            |normalized| match normalized {
                REDACTION_PUBLIC => Some(Self::Public),
                REDACTION_LOCAL_CONFIG => Some(Self::LocalConfig),
                REDACTION_SECRET => Some(Self::Secret),
                _ => None,
            },
        )
    }

    /// Returns the stable redaction-class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => REDACTION_PUBLIC,
            Self::LocalConfig => REDACTION_LOCAL_CONFIG,
            Self::Secret => REDACTION_SECRET,
        }
    }
}

/// Host admission result for copying an ignored or local-only file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IgnoredFileRedactionApproval {
    /// Host approved copying the file after redaction classification.
    Approved,
    /// Host denied copying the file.
    Denied,
}

impl IgnoredFileRedactionApproval {
    /// Parses an ignored-file redaction approval label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(
            INCLUDE_APPROVAL_FIELD,
            value,
            |normalized| match normalized {
                APPROVAL_APPROVED => Some(Self::Approved),
                APPROVAL_DENIED => Some(Self::Denied),
                _ => None,
            },
        )
    }

    /// Returns the stable approval label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Approved => APPROVAL_APPROVED,
            Self::Denied => APPROVAL_DENIED,
        }
    }
}

/// Allowlist entry for ignored or local-only file materialization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IgnoredFileAllowlistEntry {
    file_label: alloc::string::String,
    provenance: alloc::string::String,
    redaction_class: IgnoredFileRedactionClass,
    approval: IgnoredFileRedactionApproval,
}

impl IgnoredFileAllowlistEntry {
    /// Validates an ignored-file allowlist entry.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when fields are empty, malformed, or unsupported.
    pub fn new(
        draft: IgnoredFileAllowlistEntryDraft<'_>,
    ) -> Result<Self, crate::AdapterContractError> {
        let file_label =
            crate::validation::normalized_adapter_field(INCLUDE_FILE_FIELD, draft.file_label)?;
        let provenance = crate::validation::normalized_adapter_field(
            INCLUDE_PROVENANCE_FIELD,
            draft.provenance,
        )?;
        let redaction_class = IgnoredFileRedactionClass::parse(draft.redaction_class)?;
        let approval = IgnoredFileRedactionApproval::parse(draft.approval)?;
        Ok(Self {
            file_label,
            provenance,
            redaction_class,
            approval,
        })
    }

    /// Returns the safe file label.
    #[must_use]
    pub fn file_label(&self) -> &str {
        self.file_label.as_str()
    }

    /// Returns safe provenance metadata.
    #[must_use]
    pub fn provenance(&self) -> &str {
        self.provenance.as_str()
    }

    /// Returns the redaction class.
    #[must_use]
    pub const fn redaction_class(&self) -> IgnoredFileRedactionClass {
        self.redaction_class
    }

    /// Returns host redaction approval.
    #[must_use]
    pub const fn approval(&self) -> IgnoredFileRedactionApproval {
        self.approval
    }
}

/// Borrowed ignored-file allowlist input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IgnoredFileAllowlistEntryDraft<'a> {
    /// Safe file label; not a raw absolute path.
    pub file_label: &'a str,
    /// Safe provenance label for why copying is allowed.
    pub provenance: &'a str,
    /// Redaction classification label.
    pub redaction_class: &'a str,
    /// Host redaction approval label.
    pub approval: &'a str,
}

