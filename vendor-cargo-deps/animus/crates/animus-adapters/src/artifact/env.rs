const REVIEW_SUMMARY_PREFIX: &str = "artifact";
const METADATA_ONLY_PAYLOAD_MARKER: &str = "[metadata-only payload omitted]";
const REDACTED_PAYLOAD_MARKER: &str = "[redacted payload omitted]";
const SECRET_PAYLOAD_MARKER: &str = "[secret payload omitted]";
const NUL_CHARACTER: char = '\0';
const CARRIAGE_RETURN: char = '\r';
const LINE_FEED: char = '\n';

/// Active envelope version used for Animus-owned BLAKE3 identities.
pub const VERSION: u32 = 1;

/// Largest normalized payload accepted by the no-std DTO layer.
pub const MAX_PAYLOAD_CHARS: usize = 8_192;

const _: () = assert!(VERSION > 0);
const _: () = assert!(MAX_PAYLOAD_CHARS > 0);

/// Borrowed inputs used to build a canonical envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    kind: super::Kind,
    redaction: super::RedactionClass,
    payload: &'a str,
    dependencies: &'a [super::Hash],
}

impl<'a> Draft<'a> {
    /// Builds a draft without dependency hashes.
    #[must_use]
    pub const fn new(kind: super::Kind, redaction: super::RedactionClass, payload: &'a str) -> Self {
        Self {
            kind,
            redaction,
            payload,
            dependencies: &[],
        }
    }

    /// Adds dependency hashes to the envelope draft.
    #[must_use]
    pub const fn with_dependencies(mut self, dependencies: &'a [super::Hash]) -> Self {
        self.dependencies = dependencies;
        self
    }
}

/// Canonical Animus envelope with deterministic BLAKE3 identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Envelope {
    kind: super::Kind,
    redaction: super::RedactionClass,
    payload: alloc::string::String,
    dependencies: alloc::vec::Vec<super::Hash>,
    identity: super::Hash,
}

impl Envelope {
    /// Builds a canonical envelope and computes its BLAKE3 identity.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when payload normalization rejects a NUL
    /// character or an oversized payload.
    pub fn new(draft: Draft<'_>) -> Result<Self, super::Error> {
        let payload = normalize_payload(draft.payload)?;
        let dependencies = normalized_dependencies(draft.dependencies);
        let identity = identity(draft.kind, draft.redaction, &payload, &dependencies);
        Ok(Self {
            kind: draft.kind,
            redaction: draft.redaction,
            payload,
            dependencies,
            identity,
        })
    }

    /// Returns the artifact kind.
    #[must_use]
    pub const fn kind(&self) -> super::Kind {
        self.kind
    }

    /// Returns the redaction class.
    #[must_use]
    pub const fn redaction(&self) -> super::RedactionClass {
        self.redaction
    }

    /// Returns the normalized payload retained by the envelope.
    #[must_use]
    pub fn normalized_payload(&self) -> &str {
        self.payload.as_str()
    }

    /// Returns sorted and deduplicated dependency hashes.
    #[must_use]
    pub fn dependencies(&self) -> &[super::Hash] {
        self.dependencies.as_slice()
    }

    /// Returns the deterministic BLAKE3 identity.
    #[must_use]
    pub const fn identity(&self) -> &super::Hash {
        &self.identity
    }

    /// Returns the payload view allowed for review output.
    #[must_use]
    pub fn review_payload(&self) -> &str {
        match self.redaction {
            super::RedactionClass::Public => self.payload.as_str(),
            super::RedactionClass::MetadataOnly => METADATA_ONLY_PAYLOAD_MARKER,
            super::RedactionClass::RedactedPayload => REDACTED_PAYLOAD_MARKER,
            super::RedactionClass::Secret => SECRET_PAYLOAD_MARKER,
        }
    }

    /// Renders a bounded review summary without exposing non-public payloads.
    #[must_use]
    pub fn review_summary(&self) -> alloc::string::String {
        let mut summary = alloc::string::String::new();
        summary.push_str(REVIEW_SUMMARY_PREFIX);
        summary.push(' ');
        summary.push_str(self.kind.as_str());
        summary.push(' ');
        summary.push_str(self.redaction.as_str());
        summary.push(' ');
        summary.push_str(self.identity.as_str());
        summary.push(' ');
        summary.push_str(self.review_payload());
        summary
    }
}

fn normalize_payload(value: &str) -> Result<alloc::string::String, super::Error> {
    validate_payload_length(value)?;
    copy_payload(value)
}

fn validate_payload_length(value: &str) -> Result<(), super::Error> {
    let actual_chars = value.chars().count();
    assert!(actual_chars <= value.len(), "payload char count must fit byte length");
    if actual_chars > MAX_PAYLOAD_CHARS {
        return Err(super::Error::PayloadTooLong {
            max_chars: MAX_PAYLOAD_CHARS,
            actual_chars,
        });
    }
    Ok(())
}

fn copy_payload(value: &str) -> Result<alloc::string::String, super::Error> {
    assert!(value.len() >= value.chars().count(), "payload byte length must cover character count");
    let mut normalized = alloc::string::String::with_capacity(value.len());
    assert!(normalized.capacity() >= value.len(), "payload normalization buffer must reserve enough bytes");
    let mut is_skipping_line_feed = false;
    for (byte_index, character) in value.char_indices() {
        if character == NUL_CHARACTER {
            return Err(super::Error::PayloadContainsNul { byte_index });
        }
        if is_skipping_line_feed && character == LINE_FEED {
            is_skipping_line_feed = false;
            continue;
        }
        is_skipping_line_feed = character == CARRIAGE_RETURN;
        normalized.push(if character == CARRIAGE_RETURN {
            LINE_FEED
        } else {
            character
        });
    }
    Ok(normalized)
}

fn normalized_dependencies(dependencies: &[super::Hash]) -> alloc::vec::Vec<super::Hash> {
    let mut normalized = dependencies.to_vec();
    normalized.sort();
    normalized.dedup();
    assert!(normalized.len() <= dependencies.len(), "deduplicated dependencies cannot grow");
    normalized
}

fn identity(
    kind: super::Kind,
    redaction: super::RedactionClass,
    payload: &str,
    dependencies: &[super::Hash],
) -> super::Hash {
    let canonical = super::canonical::text(kind, redaction, payload, dependencies);
    let digest = blake3::hash(canonical.as_bytes());
    super::Hash::from_bytes(digest.as_bytes())
}
