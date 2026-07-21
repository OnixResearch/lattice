/// Borrowed fields used to construct a safe hashed edit receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiptDraft<'a> {
    /// Pure edit plan accepted by validation.
    pub plan: &'a super::Plan,
    /// Host-reported full-content hash after mutation.
    pub after_snapshot_hash: &'a str,
    /// Resource identity redaction policy.
    pub resource_redaction: &'a str,
    /// Optional resource identity text allowed by the redaction policy.
    pub resource_identity: Option<&'a str>,
    /// Safe diagnostic labels to attach to the receipt.
    pub diagnostics: &'a [&'a str],
}

/// Safe selected-span receipt metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpanMetadata {
    start_line: u32,
    end_line: u32,
    replacement_chars: u64,
    replacement_hash: super::Text,
}

impl SpanMetadata {
    /// Returns the first selected line.
    #[must_use]
    pub const fn start_line(&self) -> u32 {
        self.start_line
    }

    /// Returns the last selected line.
    #[must_use]
    pub const fn end_line(&self) -> u32 {
        self.end_line
    }

    /// Returns the bounded replacement character count.
    #[must_use]
    pub const fn replacement_chars(&self) -> u64 {
        self.replacement_chars
    }

    /// Returns the BLAKE3 replacement operation hash.
    #[must_use]
    pub fn replacement_hash(&self) -> &str {
        self.replacement_hash.as_str()
    }
}

/// Safe receipt containing hashes, counts, redaction, and diagnostics only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    resource_class: super::Text,
    resource_redaction: super::ResourceRedaction,
    resource_identity: Option<super::Text>,
    before_snapshot_hash: super::Text,
    after_snapshot_hash: super::Text,
    plan_hash: super::Text,
    spans: super::Collection<SpanMetadata>,
    diagnostics: super::Collection<super::Text>,
    hash: super::Text,
}

impl Receipt {
    /// Builds a safe receipt for a host-applied hashed edit plan.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when the after hash is invalid, diagnostics
    /// exceed bounds, or the resource identity conflicts with redaction policy.
    pub fn new(draft: &ReceiptDraft<'_>) -> Result<Self, super::Error> {
        let after_snapshot_hash =
            super::validate_hash(super::AFTER_SNAPSHOT_HASH_FIELD, draft.after_snapshot_hash)?;
        let resource_redaction = super::ResourceRedaction::parse(draft.resource_redaction)?;
        let resource_identity =
            validate_resource_identity(resource_redaction, draft.resource_identity)?;
        let diagnostics = collect_diagnostics(draft.diagnostics)?;
        let spans = receipt_spans(draft.plan.spans());
        let receipt_hash = super::hash::receipt(super::hash::ReceiptIdentity {
            plan_hash: draft.plan.plan_hash(),
            resource_redaction,
            resource_identity: resource_identity.as_deref(),
            after_snapshot_hash: after_snapshot_hash.as_str(),
            diagnostics: diagnostics.as_slice(),
            spans: spans.as_slice(),
        });
        Ok(Self {
            resource_class: super::Text::from(draft.plan.resource_class()),
            resource_redaction,
            resource_identity,
            before_snapshot_hash: super::Text::from(draft.plan.snapshot_hash()),
            after_snapshot_hash,
            plan_hash: super::Text::from(draft.plan.plan_hash()),
            spans,
            diagnostics,
            hash: receipt_hash,
        })
    }

    /// Returns the safe resource class.
    #[must_use]
    pub fn resource_class(&self) -> &str {
        self.resource_class.as_str()
    }

    /// Returns the resource redaction mode.
    #[must_use]
    pub const fn resource_redaction(&self) -> super::ResourceRedaction {
        self.resource_redaction
    }

    /// Returns safe resource identity text, if policy allows it.
    #[must_use]
    pub fn resource_identity(&self) -> Option<&str> {
        self.resource_identity.as_deref()
    }

    /// Returns the before-edit snapshot hash.
    #[must_use]
    pub fn before_snapshot_hash(&self) -> &str {
        self.before_snapshot_hash.as_str()
    }

    /// Returns the after-edit snapshot hash.
    #[must_use]
    pub fn after_snapshot_hash(&self) -> &str {
        self.after_snapshot_hash.as_str()
    }

    /// Returns the edit plan hash.
    #[must_use]
    pub fn plan_hash(&self) -> &str {
        self.plan_hash.as_str()
    }

    /// Returns safe span metadata without raw replacement text.
    #[must_use]
    pub fn spans(&self) -> &[SpanMetadata] {
        self.spans.as_slice()
    }

    /// Returns safe diagnostic labels.
    #[must_use]
    pub fn diagnostics(&self) -> &[super::Text] {
        self.diagnostics.as_slice()
    }

    /// Returns the BLAKE3 hash of safe receipt fields.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.hash.as_str()
    }
}

fn validate_resource_identity(
    redaction: super::ResourceRedaction,
    identity: Option<&str>,
) -> Result<Option<super::Text>, super::Error> {
    match redaction {
        super::ResourceRedaction::Omitted => validate_omitted_identity(identity),
        super::ResourceRedaction::RelativePath => validate_relative_identity(identity),
        super::ResourceRedaction::ResourceHash => validate_hash_identity(identity),
    }
}

fn validate_omitted_identity(identity: Option<&str>) -> Result<Option<super::Text>, super::Error> {
    if identity.is_some_and(|value| !value.trim().is_empty()) {
        return Err(super::Error::UnexpectedResourceIdentity {
            redaction: super::ResourceRedaction::Omitted,
        });
    }
    Ok(None)
}

fn validate_relative_identity(identity: Option<&str>) -> Result<Option<super::Text>, super::Error> {
    let Some(value) = identity else {
        return Err(super::Error::MissingResourceIdentity {
            redaction: super::ResourceRedaction::RelativePath,
        });
    };
    let normalized = super::required_label(super::RESOURCE_IDENTITY_FIELD, value)?;
    if looks_private_locator(normalized.as_str()) {
        return Err(super::Error::UnsafeResourceIdentity {
            resource_identity: normalized,
        });
    }
    Ok(Some(normalized))
}

fn validate_hash_identity(identity: Option<&str>) -> Result<Option<super::Text>, super::Error> {
    let Some(value) = identity else {
        return Err(super::Error::MissingResourceIdentity {
            redaction: super::ResourceRedaction::ResourceHash,
        });
    };
    Ok(Some(super::validate_hash(
        super::RESOURCE_IDENTITY_FIELD,
        value,
    )?))
}

fn looks_private_locator(value: &str) -> bool {
    value.starts_with(super::super::ABSOLUTE_PATH_PREFIX)
        || value.starts_with(super::super::HOME_PATH_PREFIX)
        || value.contains(super::super::URI_SCHEME_MARKER)
        || value.contains(super::super::WINDOWS_SEPARATOR_MARKER)
}

fn collect_diagnostics(values: &[&str]) -> Result<super::Collection<super::Text>, super::Error> {
    super::validate_collection_count(super::CountLimit {
        field: super::DIAGNOSTICS_FIELD,
        actual_items: values.len(),
        max_items: super::MAX_RECEIPT_DIAGNOSTICS,
    })?;
    values
        .iter()
        .map(|value| super::required_label(super::RECEIPT_DIAGNOSTIC_FIELD, value))
        .collect()
}

fn receipt_spans(spans: &[super::Validated]) -> super::Collection<SpanMetadata> {
    spans
        .iter()
        .map(|span| SpanMetadata {
            start_line: span.start_line(),
            end_line: span.end_line(),
            replacement_chars: span.replacement_chars(),
            replacement_hash: super::Text::from(span.replacement_hash()),
        })
        .collect()
}
