const VIDEO_HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const VIDEO_LABEL: &str = "demo-video-redacted";
const VIDEO_BYTES: u64 = 10_240;
const DURATION_MS: u64 = 12_000;
const WIDTH_PX: u32 = 1_920;
const HEIGHT_PX: u32 = 1_080;
const FRAME_RATE_MILLIHZ: u32 = 30_000;
const MIXED_PART_COUNT: usize = 3;
const VIDEO_PART_INDEX: usize = 1;
const ONE_VIDEO: u64 = 1;
const ONE_VIDEO_OUTCOME: usize = 1;
const FIRST_OUTCOME_INDEX: usize = 0;
const PROVIDER_PROFILE: &str = "google-gemini-v1beta-files-fixture";
const MODEL_PROFILE: &str = "gemini-video-fixture-model";
const SAFE_HANDLE: &str = "provider-file-video-redacted";

struct StaticVideoService;

impl super::super::VideoPayloadService for StaticVideoService {
    fn resolve_video_payload(
        &self,
        _video: &super::super::VideoPart,
    ) -> Result<super::super::VideoPayloadHandle, super::super::VideoPayloadError> {
        super::super::VideoPayloadHandle::from_draft(super::super::VideoPayloadHandleDraft {
            kind: super::super::VideoPayloadHandleKind::ProviderFileId,
            safe_handle: SAFE_HANDLE,
            content_hash: VIDEO_HASH,
            media_type: "video/mp4",
            expired: false,
        })
        .map_err(|_error| super::super::VideoPayloadError::InvalidHandle {
            field: "video_payload_handle",
            reason: super::super::UnsafeContentReason::ProviderPayload,
        })
    }
}

fn valid_video_draft() -> super::super::VideoPartDraft<'static> {
    super::super::VideoPartDraft {
        safe_label: VIDEO_LABEL,
        media_type: "video/mp4",
        byte_count: VIDEO_BYTES,
        content_hash: VIDEO_HASH,
        source_trust: super::super::VideoSourceTrust::UserProvided,
        metadata: Some(super::super::VideoMetadataDraft {
            duration_ms: Some(DURATION_MS),
            width_px: Some(WIDTH_PX),
            height_px: Some(HEIGHT_PX),
            frame_rate_millihz: Some(FRAME_RATE_MILLIHZ),
            codec: Some("h264"),
            container: Some("mp4"),
            has_audio: Some(true),
            detail: Some(super::super::VideoDetailPreference::Auto),
        }),
        descriptor_hints: &["host-admitted-handle"],
    }
}

fn video_assembly() -> super::super::PromptAssembly {
    super::super::assemble_prompt_parts(
        &[
            super::super::ContentPartDraft::Text { text: "inspect" },
            super::super::ContentPartDraft::Video(valid_video_draft()),
            super::super::ContentPartDraft::Text { text: "summarize" },
        ],
        super::super::PromptAssemblyPolicy::default(),
    )
    .expect("video assembly should validate")
}

fn video_capabilities() -> super::super::ModelInputCapabilities {
    super::super::ModelInputCapabilities::from_draft(super::super::ModelInputCapabilitiesDraft {
        model_label: MODEL_PROFILE,
        supports_text: true,
        supports_images: false,
        image_adapter_available: false,
        video: super::super::VideoInputCapabilitiesDraft {
            supported: true,
            service_available: true,
            projection_available: true,
            media_types: &[super::super::VideoMediaType::Mp4],
            byte_limit: super::super::DEFAULT_PROMPT_VIDEO_BYTE_BUDGET,
            duration_limit_ms: super::super::MAX_VIDEO_DURATION_MS,
        },
    })
    .expect("video capabilities should validate")
}

fn provider_policy() -> super::super::ProviderSerializationPolicy {
    super::super::ProviderSerializationPolicy::from_draft(
        super::super::ProviderSerializationPolicyDraft {
            provider_label: PROVIDER_PROFILE,
            mode: super::super::ProviderSerializationMode::NativeFixture,
            video_projection: Some(super::super::ProviderVideoProjection {
                declared_metadata: super::super::ProviderProjectionSupport::Supported,
                audio_marker: super::super::ProviderProjectionSupport::Supported,
                detail: super::super::ProviderProjectionSupport::Supported,
                ordering: super::super::ProviderProjectionSupport::Supported,
            }),
        },
    )
    .expect("provider policy should validate")
}

#[test]
fn video_descriptor_and_mixed_assembly_preserve_safe_ordered_intent() {
    // r[verify agent_harness_runtime.video_payloads.typed_parts.valid]
    // r[verify agent_harness_runtime.video_payloads.assembly.ordered]
    let video = super::super::VideoPart::from_draft(valid_video_draft())
        .expect("video descriptor should validate");
    assert_eq!(video.media_type(), super::super::VideoMediaType::Mp4);
    assert_eq!(video.byte_count(), VIDEO_BYTES);
    assert_eq!(
        video
            .metadata()
            .and_then(super::super::VideoMetadata::duration_ms),
        Some(DURATION_MS)
    );

    let assembly = video_assembly();
    assert_eq!(assembly.parts().len(), MIXED_PART_COUNT);
    assert!(matches!(
        &assembly.parts()[VIDEO_PART_INDEX],
        super::super::ContentPart::Video(_video)
    ));
    assert_eq!(assembly.video_count(), ONE_VIDEO);
    assert_eq!(assembly.video_outcomes().len(), ONE_VIDEO_OUTCOME);
    assert_eq!(
        assembly.video_outcomes()[FIRST_OUTCOME_INDEX].status(),
        super::super::VideoOutcomeStatus::Included
    );
}

#[test]
fn exact_video_capability_and_injected_service_produce_native_fixture() {
    // r[verify agent_harness_runtime.video_payloads.capabilities.supported]
    // r[verify agent_harness_runtime.video_payloads.payload_service.fixture]
    let assembly = video_assembly();
    let dispatch = super::super::prepare_dispatch(&assembly, &video_capabilities())
        .expect("exact video capability should dispatch");
    assert_eq!(dispatch.video_count(), ONE_VIDEO);
    assert_eq!(
        dispatch.video_outcome(),
        super::super::VideoCapabilityOutcome::Supported
    );

    let fixture = super::super::serialize_provider_native_video_fixture(
        &assembly,
        &provider_policy(),
        None,
        Some(&StaticVideoService),
    )
    .expect("injected fixture service should resolve safe handle metadata");
    assert_eq!(fixture.provider_label(), PROVIDER_PROFILE);
    assert!(matches!(
        fixture.parts()[VIDEO_PART_INDEX],
        super::super::ProviderNativePart::Video { .. }
    ));
}

#[test]
fn metadata_only_video_receipt_is_deterministic() {
    // r[verify agent_harness_runtime.video_payloads.evidence.safe]
    let draft = super::super::VideoReceiptDraft {
        status: super::super::VideoOutcomeStatus::Included,
        safe_label: VIDEO_LABEL,
        media_type: "video/mp4",
        byte_count: VIDEO_BYTES,
        content_hash: VIDEO_HASH,
        duration_ms: Some(DURATION_MS),
        width_px: Some(WIDTH_PX),
        height_px: Some(HEIGHT_PX),
        frame_rate_millihz: Some(FRAME_RATE_MILLIHZ),
        service_class: "injected_fixture_service",
        provider_profile: PROVIDER_PROFILE,
        redaction_marker: "metadata_only",
        loss_classes: &[super::super::VideoLossClass::None],
        non_claims: &["no_live_provider_acceptance", "no_media_authenticity"],
        metadata_fragments: &["ordered_part:1", "audio_presence:declared"],
    };
    let first = super::super::VideoReceipt::validate(&draft).expect("safe receipt should validate");
    let second = super::super::VideoReceipt::validate(&draft).expect("safe receipt should repeat");
    assert_eq!(first.modality(), "video");
    assert_eq!(first.receipt_hash(), second.receipt_hash());
    assert_eq!(first.content_hash(), VIDEO_HASH);
}
