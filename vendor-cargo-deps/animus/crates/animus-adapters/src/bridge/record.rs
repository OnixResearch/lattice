use super::COMMAND_FIELD;
use super::Collection;
use super::ENDPOINT_FIELD;
use super::HashInput;
use super::ID_FIELD;
use super::InteropError;
use super::InteropOperationKind;
use super::MIN_COUNT;
use super::REDACTION_FIELD;
use super::SESSION_FIELD;
use super::STATUS_FIELD;
use super::Text;
use super::hash_parts;
use super::required_text;
use super::text_collection;
use super::validate_hash;

/// Untrusted safe interop receipt metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InteropReceiptDraft<'a> {
    /// Operation kind label.
    pub operation: &'a str,
    /// Status label.
    pub status: &'a str,
    /// Bridge identity hash.
    pub bridge_hash: &'a str,
    /// Command label hash.
    pub command_hash: Option<&'a str>,
    /// Endpoint label hash.
    pub endpoint_hash: Option<&'a str>,
    /// External session hash.
    pub external_session_hash: Option<&'a str>,
    /// Environment key count.
    pub env_key_count: u32,
    /// Argument count.
    pub argument_count: u32,
    /// Redaction markers.
    pub redactions: &'a [&'a str],
}

/// Safe interop receipt with hashes and bounded counts only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteropReceipt {
    operation: InteropOperationKind,
    status: Text,
    identity_hash: Text,
    env_key_count: u32,
    argument_count: u32,
    redactions: Collection<Text>,
}

impl InteropReceipt {
    /// Builds a safe interop receipt.
    ///
    /// # Errors
    ///
    /// Returns [`InteropError`] when labels are malformed, hashes are invalid, or
    /// redactions are empty.
    pub fn new(draft: &InteropReceiptDraft<'_>) -> Result<Self, InteropError> {
        let operation = InteropOperationKind::parse(draft.operation)?;
        let status = required_text(STATUS_FIELD, draft.status)?;
        validate_hash(HashInput {
            field: ID_FIELD.as_str(),
            value: draft.bridge_hash,
        })?;
        if let Some(command_hash) = draft.command_hash {
            validate_hash(HashInput {
                field: COMMAND_FIELD.as_str(),
                value: command_hash,
            })?;
        }
        if let Some(endpoint_hash) = draft.endpoint_hash {
            validate_hash(HashInput {
                field: ENDPOINT_FIELD.as_str(),
                value: endpoint_hash,
            })?;
        }
        if let Some(external_session_hash) = draft.external_session_hash {
            validate_hash(HashInput {
                field: SESSION_FIELD.as_str(),
                value: external_session_hash,
            })?;
        }
        let redactions = text_collection(REDACTION_FIELD, draft.redactions, true)?;
        let env_count = draft.env_key_count.saturating_add(MIN_COUNT);
        let arg_count = draft.argument_count.saturating_add(MIN_COUNT);
        let identity_hash = hash_parts(&[
            operation.as_str(),
            status.as_str(),
            draft.bridge_hash,
            &alloc::format!("{env_count}"),
            &alloc::format!("{arg_count}"),
        ]);
        Ok(Self {
            operation,
            status,
            identity_hash,
            env_key_count: env_count,
            argument_count: arg_count,
            redactions,
        })
    }

    /// Returns operation kind.
    #[must_use]
    pub const fn operation(&self) -> InteropOperationKind {
        self.operation
    }

    /// Returns status label.
    #[must_use]
    pub fn status(&self) -> &str {
        self.status.as_str()
    }

    /// Returns receipt identity hash.
    #[must_use]
    pub fn identity_hash(&self) -> &str {
        self.identity_hash.as_str()
    }

    /// Returns environment key count.
    #[must_use]
    pub const fn env_key_count(&self) -> u32 {
        self.env_key_count
    }

    /// Returns argument count.
    #[must_use]
    pub const fn argument_count(&self) -> u32 {
        self.argument_count
    }

    /// Returns redaction markers.
    #[must_use]
    pub fn redactions(&self) -> &[Text] {
        self.redactions.as_slice()
    }
}

/// Validates rendered receipt text for raw interop material.
///
/// # Errors
///
/// Returns [`InteropError::ReceiptLeak`] when raw command, endpoint, argument, or
/// credential-looking material appears.
pub fn validate_interop_receipt_text(text: &str) -> Result<(), InteropError> {
    let lowered = text.to_ascii_lowercase();
    if contains_receipt_leak_marker(lowered.as_str()) {
        return Err(InteropError::ReceiptLeak);
    }
    Ok(())
}

fn contains_receipt_leak_marker(lowered: &str) -> bool {
    if lowered.contains("http://") {
        return true;
    }
    if lowered.contains("https://") {
        return true;
    }
    if lowered.contains("api_key") {
        return true;
    }
    if lowered.contains("args=") {
        return true;
    }
    lowered.contains("session=")
}
