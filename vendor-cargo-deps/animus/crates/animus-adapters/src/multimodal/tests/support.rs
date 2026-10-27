pub(super) const VALID_HASH: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
pub(super) const OTHER_VALID_HASH: &str =
    "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
pub(super) const IMAGE_LABEL: &str = "screenshot-redacted";
pub(super) const TEXT_BEFORE: &str = "describe this";
pub(super) const TEXT_AFTER: &str = "focus on errors";
pub(super) const IMAGE_BYTE_COUNT: u64 = 4_096;
pub(super) const WIDTH_PX: u32 = 640;
pub(super) const HEIGHT_PX: u32 = 480;
pub(super) const OVER_BUDGET_BYTES: u64 = 8_192;
pub(super) const SMALL_BUDGET_BYTES: u64 = 1_024;
pub(super) const REQUEST_PART_COUNT_WITH_IMAGE: usize = 3;
pub(super) const REQUEST_PART_COUNT_TEXT_ONLY: usize = 2;
pub(super) const DISPATCH_PART_COUNT_WITH_IMAGE: u64 = 3;
pub(super) const IMAGE_COUNT_ONE: u64 = 1;
pub(super) const IMAGE_OUTCOME_COUNT_ONE: usize = 1;
pub(super) const FIRST_PART_INDEX: usize = 0;
pub(super) const IMAGE_PART_INDEX: usize = 1;
pub(super) const LAST_PART_INDEX: usize = 2;
pub(super) const FIRST_OUTCOME_INDEX: usize = 0;
pub(super) const PROVIDER_LABEL: &str = "fixture-provider";
pub(super) const MODEL_LABEL: &str = "vision-model";
pub(super) const TEXT_ONLY_MODEL_LABEL: &str = "text-only-model";
pub(super) const SAFE_HANDLE: &str = "provider-file-redacted";
pub(super) const RECEIPT_REDACTION_MARKER: &str = "hash_only";
pub(super) const EXPECTED_MODALITY: &str = "image";

pub(super) struct StaticImageService {
    pub(super) handle: super::super::ImagePayloadHandle,
}

impl super::super::ImagePayloadService for StaticImageService {
    fn resolve_image_payload(
        &self,
        _image: &super::super::ImagePart,
    ) -> Result<super::super::ImagePayloadHandle, super::super::ImagePayloadError> {
        Ok(self.handle.clone())
    }
}

pub(super) struct FailingImageService {
    pub(super) error: super::super::ImagePayloadError,
}

impl super::super::ImagePayloadService for FailingImageService {
    fn resolve_image_payload(
        &self,
        _image: &super::super::ImagePart,
    ) -> Result<super::super::ImagePayloadHandle, super::super::ImagePayloadError> {
        Err(self.error)
    }
}

pub(super) fn valid_image_draft() -> super::super::ImagePartDraft<'static> {
    super::super::ImagePartDraft {
        safe_label: IMAGE_LABEL,
        media_type: "image/png",
        byte_count: IMAGE_BYTE_COUNT,
        content_hash: VALID_HASH,
        source_trust: super::super::ImageSourceTrust::UserProvided,
        metadata: Some(super::super::ImageMetadataDraft {
            width_px: Some(WIDTH_PX),
            height_px: Some(HEIGHT_PX),
            detail: Some(super::super::ImageDetailPreference::High),
            adapter_class: Some("provider-fixture"),
        }),
        descriptor_hints: &["host-admitted"],
    }
}

pub(super) fn valid_image_part() -> super::super::ImagePart {
    super::super::ImagePart::from_draft(valid_image_draft()).expect("image draft should validate")
}

pub(super) fn valid_mixed_assembly() -> super::super::PromptAssembly {
    super::super::assemble_prompt_parts(
        &[
            super::super::ContentPartDraft::Text { text: TEXT_BEFORE },
            super::super::ContentPartDraft::Image(valid_image_draft()),
            super::super::ContentPartDraft::Text { text: TEXT_AFTER },
        ],
        super::super::PromptAssemblyPolicy::default(),
    )
    .expect("prompt should assemble")
}

pub(super) fn vision_capabilities() -> super::super::ModelInputCapabilities {
    super::super::ModelInputCapabilities::from_draft(super::super::ModelInputCapabilitiesDraft {
        model_label: MODEL_LABEL,
        supports_text: true,
        supports_images: true,
        image_adapter_available: true,
        video: super::super::VideoInputCapabilitiesDraft {
            supported: false,
            service_available: false,
            projection_available: false,
            media_types: &[],
            byte_limit: 0,
            duration_limit_ms: 0,
        },
    })
    .expect("capabilities should validate")
}

pub(super) fn text_only_capabilities() -> super::super::ModelInputCapabilities {
    super::super::ModelInputCapabilities::from_draft(super::super::ModelInputCapabilitiesDraft {
        model_label: TEXT_ONLY_MODEL_LABEL,
        supports_text: true,
        supports_images: false,
        image_adapter_available: false,
        video: super::super::VideoInputCapabilitiesDraft {
            supported: false,
            service_available: false,
            projection_available: false,
            media_types: &[],
            byte_limit: 0,
            duration_limit_ms: 0,
        },
    })
    .expect("capabilities should validate")
}

pub(super) fn native_policy() -> super::super::ProviderSerializationPolicy {
    super::super::ProviderSerializationPolicy::from_draft(
        super::super::ProviderSerializationPolicyDraft {
            provider_label: PROVIDER_LABEL,
            mode: super::super::ProviderSerializationMode::NativeFixture,
            video_projection: None,
        },
    )
    .expect("policy should validate")
}

pub(super) fn valid_handle_draft() -> super::super::ImagePayloadHandleDraft<'static> {
    super::super::ImagePayloadHandleDraft {
        kind: super::super::ImagePayloadHandleKind::ProviderFileId,
        safe_handle: SAFE_HANDLE,
        content_hash: VALID_HASH,
        media_type: "image/png",
    }
}

pub(super) fn valid_handle() -> super::super::ImagePayloadHandle {
    super::super::ImagePayloadHandle::from_draft(valid_handle_draft())
        .expect("handle should validate")
}

pub(super) fn valid_receipt_draft() -> super::super::ImageReceiptDraft<'static> {
    super::super::ImageReceiptDraft {
        status: super::super::ImageOutcomeStatus::Included,
        safe_label: IMAGE_LABEL,
        media_type: "image/png",
        byte_count: IMAGE_BYTE_COUNT,
        content_hash: VALID_HASH,
        width_px: Some(WIDTH_PX),
        height_px: Some(HEIGHT_PX),
        adapter_class: Some("provider-fixture"),
        redaction_marker: RECEIPT_REDACTION_MARKER,
        metadata_fragments: &["adapter_class:fixture", "status:included"],
    }
}

pub(super) const UNSAFE_DESCRIPTOR_FIXTURES: [(&str, super::super::UnsafeContentReason); 4] = [
    (
        "raw_bytes=[137,80,78,71]",
        super::super::UnsafeContentReason::RawBytes,
    ),
    (
        "base64:iVBORw0KGgo=",
        super::super::UnsafeContentReason::Base64Payload,
    ),
    (
        "data:image/png;base64,iVBORw0KGgo=",
        super::super::UnsafeContentReason::DataUrl,
    ),
    (
        "/home/user/private/screenshot.png",
        super::super::UnsafeContentReason::AbsolutePath,
    ),
];

pub(super) const RECEIPT_LEAK_FIXTURES: [(&str, super::super::UnsafeContentReason); 7] = [
    (
        "raw_bytes=[1,2,3]",
        super::super::UnsafeContentReason::RawBytes,
    ),
    (
        "base64:AAAA",
        super::super::UnsafeContentReason::Base64Payload,
    ),
    (
        "data:image/jpeg;base64,AAAA",
        super::super::UnsafeContentReason::DataUrl,
    ),
    (
        "/Users/person/Desktop/private.png",
        super::super::UnsafeContentReason::AbsolutePath,
    ),
    (
        "api_key=sk-secret",
        super::super::UnsafeContentReason::Credential,
    ),
    (
        "provider_payload:{raw-json}",
        super::super::UnsafeContentReason::ProviderPayload,
    ),
    (
        "unbounded_image_payload",
        super::super::UnsafeContentReason::UnboundedPayload,
    ),
];
