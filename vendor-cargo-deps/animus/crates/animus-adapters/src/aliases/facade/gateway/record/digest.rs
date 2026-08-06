pub(super) const LABEL_AUDIENCE: &str = "audience";
pub(super) const LABEL_ACTOR: &str = "actor";
const ZERO_IDENTITY_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";

pub(super) fn identity_for_label(
    input: super::render::LabelPayload<'_>,
) -> Result<crate::ArtifactHash, crate::ArtifactValidationError> {
    let payload = super::render::label_payload(input);
    identity_for_payload(payload.as_str())
}

pub(super) fn identity_for_text(
    record: &super::AdmissionReceipt,
) -> Result<crate::ArtifactHash, crate::ArtifactValidationError> {
    identity_for_payload(super::render::safe_text(record).as_str())
}

pub(super) fn zero_identity() -> Result<crate::ArtifactHash, crate::ArtifactValidationError> {
    crate::ArtifactHash::new(ZERO_IDENTITY_HEX)
}

fn identity_for_payload(payload: &str) -> Result<crate::ArtifactHash, crate::ArtifactValidationError> {
    debug_assert!(!payload.is_empty());
    let envelope = crate::ArtifactEnvelope::new(crate::ArtifactEnvelopeDraft::new(
        crate::ArtifactKind::RuntimeReceipt,
        crate::ArtifactRedactionClass::MetadataOnly,
        payload,
    ))?;
    Ok(envelope.identity().clone())
}
