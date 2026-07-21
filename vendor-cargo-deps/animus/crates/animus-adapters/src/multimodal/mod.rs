//! Provider-neutral multimodal model input descriptors.

mod body;
mod capability;
mod check;
mod compose;
mod descriptor;
mod evidence;
mod fault;
mod kinds;
#[cfg(test)]
mod tests;
mod transport;
mod wire;

pub use body::{ContentPart, ContentPartDraft, TextPart};
pub use capability::{
    DispatchPlan, ModelInputCapabilities, ModelInputCapabilitiesDraft, prepare_dispatch,
};
pub use compose::{
    ImageOutcome, ImageOutcomeStatus, PromptAssembly, PromptAssemblyPolicy, assemble_prompt_parts,
};
pub use descriptor::{ImageMetadata, ImageMetadataDraft, ImagePart, ImagePartDraft};
pub use evidence::{ImageReceipt, ImageReceiptDraft};
pub use fault::{Error, UnsafeContentReason};
pub use kinds::{ImageDetailPreference, ImageMediaType, ImageSourceTrust, InputModality};
pub use transport::{
    ImagePayloadError, ImagePayloadHandle, ImagePayloadHandleDraft, ImagePayloadHandleKind,
    ImagePayloadService,
};
pub use wire::{
    ProviderNativePart, ProviderNativeRequest, ProviderSerializationMode,
    ProviderSerializationPolicy, ProviderSerializationPolicyDraft,
    serialize_provider_native_fixture,
};

pub(super) const SAFE_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("image_safe_label");
pub(super) const TEXT_PART_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("multimodal_text_part");
pub(super) const MEDIA_TYPE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("image_media_type");
pub(super) const CONTENT_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("image_content_hash");
pub(super) const SOURCE_HINT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("image_descriptor_hint");
pub(super) const ADAPTER_CLASS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("image_adapter_class");
pub(super) const REDACTION_MARKER_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("image_redaction_marker");
pub(super) const PROVIDER_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("provider_label");
pub(super) const MODEL_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_label");
pub(super) const HANDLE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("image_payload_handle");
pub(super) const METADATA_FRAGMENT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("image_receipt_metadata");
pub(super) const BLAKE3_HEX_CHARS: usize = blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE);
pub(super) const MIN_DIMENSION_PIXELS: u32 = 1;
pub(super) const MAX_DIMENSION_PIXELS: u32 = 32_768;
pub(super) const RECEIPT_SCHEMA: &str = "animus-multimodal-image-receipt-v1";
pub(super) const HASH_PART_SEPARATOR: char = '\u{1f}';
pub(super) const IMAGE_MODALITY: &str = "image";

const HEX_CHARS_PER_BYTE: usize = 2;
const BYTES_PER_KIB: u64 = 1_024;
const KIB_PER_MIB: u64 = 1_024;
const DEFAULT_IMAGE_MIB_LIMIT: u64 = 20;
const DEFAULT_PROMPT_IMAGE_MIB_LIMIT: u64 = 8;

/// Largest image descriptor byte count accepted before host-side transformation is required.
pub const MAX_IMAGE_BYTE_COUNT: u64 = mib_to_bytes(DEFAULT_IMAGE_MIB_LIMIT);

/// Default image byte budget for one prompt assembly pass.
pub const DEFAULT_PROMPT_IMAGE_BYTE_BUDGET: u64 = mib_to_bytes(DEFAULT_PROMPT_IMAGE_MIB_LIMIT);

const fn mib_to_bytes(mib: u64) -> u64 {
    mib.saturating_mul(KIB_PER_MIB)
        .saturating_mul(BYTES_PER_KIB)
}
