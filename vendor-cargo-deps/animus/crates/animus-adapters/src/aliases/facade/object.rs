/// Active envelope version used for Animus-owned BLAKE3 identities.
pub const ARTIFACT_ENVELOPE_VERSION: u32 = crate::artifact::VERSION;
/// Largest normalized payload accepted by the no-std DTO layer.
pub const MAX_ARTIFACT_PAYLOAD_CHARS: usize = crate::artifact::MAX_PAYLOAD_CHARS;

pub use crate::artifact::BLAKE3_HASH_HEX_CHARS;
/// Canonical Animus envelope with deterministic BLAKE3 identity.
pub type ArtifactEnvelope = crate::artifact::Envelope;
/// Borrowed inputs used to build a canonical envelope.
pub type ArtifactEnvelopeDraft<'a> = crate::artifact::Draft<'a>;
/// Validated lowercase BLAKE3 identity.
pub type ArtifactHash = crate::artifact::Hash;
/// Artifact payload class used in canonical envelopes.
pub type ArtifactKind = crate::artifact::Kind;
/// Redaction class controlling payload display.
pub type ArtifactRedactionClass = crate::artifact::RedactionClass;
/// Errors raised while parsing or building safe artifact envelopes.
pub type ArtifactValidationError = crate::artifact::Error;
