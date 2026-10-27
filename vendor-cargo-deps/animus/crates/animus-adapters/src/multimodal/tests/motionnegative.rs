const VIDEO_HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const OTHER_VIDEO_HASH: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const VIDEO_BYTES: u64 = 10_240;
const SMALL_VIDEO_BUDGET: u64 = 1_024;
const ONE_VIDEO_LIMIT: u64 = 1;
const TWO_VIDEO_LIMIT: u64 = 2;
const AGGREGATE_BUDGET: u64 = 12_000;
const DURATION_MS: u64 = 12_000;
const INVALID_FRAME_RATE: u32 = 120_001;
const VIDEO_WIDTH_PX: u32 = 1_280;
const VIDEO_HEIGHT_PX: u32 = 720;
const VIDEO_FRAME_RATE_MILLIHZ: u32 = 30_000;
const ZERO_DURATION: u64 = 0;
const FIRST_OUTCOME_INDEX: usize = 0;
const SECOND_OUTCOME_INDEX: usize = 1;
const TWO_OUTCOMES: usize = 2;
const PROVIDER_PROFILE: &str = "google-gemini-v1beta-files-fixture";

fn video_draft(hash: &'static str) -> super::super::VideoPartDraft<'static> {
    super::super::VideoPartDraft {
        safe_label: "safe-video",
        media_type: "video/mp4",
        byte_count: VIDEO_BYTES,
        content_hash: hash,
        source_trust: super::super::VideoSourceTrust::WorkspaceArtifact,
        metadata: Some(super::super::VideoMetadataDraft {
            duration_ms: Some(DURATION_MS),
            width_px: Some(VIDEO_WIDTH_PX),
            height_px: Some(VIDEO_HEIGHT_PX),
            frame_rate_millihz: Some(VIDEO_FRAME_RATE_MILLIHZ),
            codec: Some("h264"),
            container: Some("mp4"),
            has_audio: Some(true),
            detail: Some(super::super::VideoDetailPreference::Auto),
        }),
        descriptor_hints: &["approved-handle"],
    }
}

fn assembly() -> super::super::PromptAssembly {
    super::super::assemble_prompt_parts(
        &[super::super::ContentPartDraft::Video(video_draft(
            VIDEO_HASH,
        ))],
        super::super::PromptAssemblyPolicy::default(),
    )
    .expect("video should assemble")
}

fn provider_policy(
    mode: super::super::ProviderSerializationMode,
    projection: Option<super::super::ProviderVideoProjection>,
) -> super::super::ProviderSerializationPolicy {
    super::super::ProviderSerializationPolicy::from_draft(
        super::super::ProviderSerializationPolicyDraft {
            provider_label: PROVIDER_PROFILE,
            mode,
            video_projection: projection,
        },
    )
    .expect("policy should validate")
}

fn native_projection() -> super::super::ProviderVideoProjection {
    super::super::ProviderVideoProjection {
        declared_metadata: super::super::ProviderProjectionSupport::Supported,
        audio_marker: super::super::ProviderProjectionSupport::Supported,
        detail: super::super::ProviderProjectionSupport::Supported,
        ordering: super::super::ProviderProjectionSupport::Supported,
    }
}

struct NeverCallVideoService;

impl super::super::VideoPayloadService for NeverCallVideoService {
    fn resolve_video_payload(
        &self,
        _video: &super::super::VideoPart,
    ) -> Result<super::super::VideoPayloadHandle, super::super::VideoPayloadError> {
        Err(super::super::VideoPayloadError::LiveProviderCallRequired)
    }
}

struct FailingVideoService {
    error: super::super::VideoPayloadError,
}

impl super::super::VideoPayloadService for FailingVideoService {
    fn resolve_video_payload(
        &self,
        _video: &super::super::VideoPart,
    ) -> Result<super::super::VideoPayloadHandle, super::super::VideoPayloadError> {
        Err(self.error)
    }
}

struct StaticVideoService {
    handle: super::super::VideoPayloadHandle,
}

impl super::super::VideoPayloadService for StaticVideoService {
    fn resolve_video_payload(
        &self,
        _video: &super::super::VideoPart,
    ) -> Result<super::super::VideoPayloadHandle, super::super::VideoPayloadError> {
        Ok(self.handle.clone())
    }
}

#[test]
fn malformed_video_metadata_and_unsafe_sources_fail_closed() {
    // r[verify agent_harness_runtime.video_payloads.typed_parts.invalid]
    let media_type_error = super::super::VideoPart::from_draft(super::super::VideoPartDraft {
        media_type: "image/png",
        ..video_draft(VIDEO_HASH)
    })
    .expect_err("image support must not imply video support");
    assert!(matches!(
        media_type_error,
        super::super::Error::UnsupportedVideoMediaType { .. }
    ));

    let zero_duration_error_ms =
        super::super::VideoPart::from_draft(super::super::VideoPartDraft {
            metadata: Some(super::super::VideoMetadataDraft {
                duration_ms: Some(ZERO_DURATION),
                ..video_draft(VIDEO_HASH).metadata.expect("metadata exists")
            }),
            ..video_draft(VIDEO_HASH)
        })
        .expect_err("zero duration must fail");
    assert_eq!(
        zero_duration_error_ms,
        super::super::Error::InvalidVideoDuration {
            duration_ms: ZERO_DURATION
        }
    );

    let invalid_dimensions = super::super::VideoPart::from_draft(super::super::VideoPartDraft {
        metadata: Some(super::super::VideoMetadataDraft {
            width_px: Some(0),
            ..video_draft(VIDEO_HASH).metadata.expect("metadata exists")
        }),
        ..video_draft(VIDEO_HASH)
    })
    .expect_err("zero dimensions must fail");
    assert!(matches!(
        invalid_dimensions,
        super::super::Error::InvalidDimensions { .. }
    ));

    let invalid_rate = super::super::VideoPart::from_draft(super::super::VideoPartDraft {
        metadata: Some(super::super::VideoMetadataDraft {
            frame_rate_millihz: Some(INVALID_FRAME_RATE),
            ..video_draft(VIDEO_HASH).metadata.expect("metadata exists")
        }),
        ..video_draft(VIDEO_HASH)
    })
    .expect_err("excessive frame rate must fail");
    assert_eq!(
        invalid_rate,
        super::super::Error::InvalidVideoFrameRate {
            frame_rate_millihz: INVALID_FRAME_RATE
        }
    );

    for unsafe_hint in [
        "https://private.example/video.mp4",
        "data:video/mp4;base64,AAAA",
        "/home/user/private/video.mp4",
        "frame_body=[1,2,3]",
        "audio_body=[4,5,6]",
    ] {
        let error = super::super::VideoPart::from_draft(super::super::VideoPartDraft {
            descriptor_hints: &[unsafe_hint],
            ..video_draft(VIDEO_HASH)
        })
        .expect_err("unsafe media material must fail");
        assert!(matches!(
            error,
            super::super::Error::UnsafeDescriptor { .. }
        ));
    }
}

#[test]
fn video_budget_count_aggregate_and_duplicate_fail_visibly() {
    // r[verify agent_harness_runtime.video_payloads.assembly.over_budget]
    let local = super::super::assemble_prompt_parts(
        &[super::super::ContentPartDraft::Video(video_draft(
            VIDEO_HASH,
        ))],
        super::super::PromptAssemblyPolicy::default().with_video_limits(
            SMALL_VIDEO_BUDGET,
            ONE_VIDEO_LIMIT,
            AGGREGATE_BUDGET,
        ),
    )
    .expect("over-budget descriptor should produce outcome");
    assert_eq!(
        local.video_outcomes()[FIRST_OUTCOME_INDEX].status(),
        super::super::VideoOutcomeStatus::Rejected
    );
    assert!(local.parts().is_empty());

    let duplicate = super::super::assemble_prompt_parts(
        &[
            super::super::ContentPartDraft::Video(video_draft(VIDEO_HASH)),
            super::super::ContentPartDraft::Video(video_draft(VIDEO_HASH)),
        ],
        super::super::PromptAssemblyPolicy::default(),
    )
    .expect_err("duplicate content identity must fail");
    assert!(matches!(
        duplicate,
        super::super::Error::DuplicateVideoIdentity { .. }
    ));

    let count_overflow = super::super::assemble_prompt_parts(
        &[
            super::super::ContentPartDraft::Video(video_draft(VIDEO_HASH)),
            super::super::ContentPartDraft::Video(video_draft(OTHER_VIDEO_HASH)),
        ],
        super::super::PromptAssemblyPolicy::default().with_video_limits(
            super::super::DEFAULT_PROMPT_VIDEO_BYTE_BUDGET,
            ONE_VIDEO_LIMIT,
            super::super::DEFAULT_AGGREGATE_MEDIA_BYTE_BUDGET,
        ),
    )
    .expect("count overflow should be explicit");
    assert_eq!(count_overflow.video_outcomes().len(), TWO_OUTCOMES);
    assert_eq!(
        count_overflow.video_outcomes()[SECOND_OUTCOME_INDEX].reason(),
        Some("video_count_budget_exceeded")
    );

    let aggregate = super::super::assemble_prompt_parts(
        &[
            super::super::ContentPartDraft::Video(video_draft(VIDEO_HASH)),
            super::super::ContentPartDraft::Video(video_draft(OTHER_VIDEO_HASH)),
        ],
        super::super::PromptAssemblyPolicy::default().with_video_limits(
            super::super::DEFAULT_PROMPT_VIDEO_BYTE_BUDGET,
            TWO_VIDEO_LIMIT,
            AGGREGATE_BUDGET,
        ),
    )
    .expect("aggregate overflow should be explicit");
    assert_eq!(
        aggregate.video_outcomes()[SECOND_OUTCOME_INDEX].reason(),
        Some("aggregate_media_budget_exceeded")
    );
}

#[test]
fn unsupported_video_requires_exact_capability_or_visible_omission() {
    // r[verify agent_harness_runtime.video_payloads.capabilities.unsupported]
    let request = assembly();
    let image_only = super::super::ModelInputCapabilities::from_draft(
        super::super::ModelInputCapabilitiesDraft {
            model_label: "image-only-model",
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
        },
    )
    .expect("image-only capability should validate");
    let capability_error = super::super::prepare_dispatch(&request, &image_only)
        .expect_err("image capability must not satisfy video");
    assert!(matches!(
        capability_error,
        super::super::Error::UnsupportedModality {
            modality: super::super::InputModality::Video,
            ..
        }
    ));
    let omitted = super::super::prepare_dispatch_with_video_policy(
        &request,
        &image_only,
        super::super::VideoCapabilityPolicy::OptionalOmissionAllowed,
    )
    .expect("explicit optional policy should omit visibly");
    assert_eq!(
        omitted.video_outcome(),
        super::super::VideoCapabilityOutcome::OmittedByPolicy
    );
}

#[test]
fn missing_service_generic_conversion_and_projection_loss_fail_before_calls() {
    // r[verify agent_harness_runtime.video_payloads.payload_service.missing]
    // r[verify agent_harness_runtime.video_payloads.loss.provider]
    let request = assembly();
    let native_projection = native_projection();
    let missing = super::super::serialize_provider_native_video_fixture(
        &request,
        &provider_policy(
            super::super::ProviderSerializationMode::NativeFixture,
            Some(native_projection),
        ),
        None,
        None,
    )
    .expect_err("missing video service must fail");
    assert_eq!(missing, super::super::Error::MissingVideoService);

    let lossy = super::super::serialize_provider_native_video_fixture(
        &request,
        &provider_policy(
            super::super::ProviderSerializationMode::GenericJson,
            Some(native_projection),
        ),
        None,
        Some(&NeverCallVideoService),
    )
    .expect_err("generic video conversion must fail");
    assert_eq!(lossy, super::super::Error::LossyGenericConversion);

    let missing_metadata_projection = super::super::ProviderVideoProjection {
        declared_metadata: super::super::ProviderProjectionSupport::Unsupported,
        audio_marker: super::super::ProviderProjectionSupport::Supported,
        detail: super::super::ProviderProjectionSupport::Supported,
        ordering: super::super::ProviderProjectionSupport::Supported,
    };
    let loss = super::super::serialize_provider_native_video_fixture(
        &request,
        &provider_policy(
            super::super::ProviderSerializationMode::NativeFixture,
            Some(missing_metadata_projection),
        ),
        None,
        Some(&NeverCallVideoService),
    )
    .expect_err("unrepresentable metadata must fail before service call");
    assert_eq!(
        loss,
        super::super::Error::UnrepresentableVideoField {
            field: "declared_metadata"
        }
    );
}

#[test]
fn service_live_upload_and_mismatched_handle_failures_stay_typed() {
    // r[verify agent_harness_runtime.video_payloads.payload_service.missing]
    let request = assembly();
    let native_projection = native_projection();
    let live = super::super::serialize_provider_native_video_fixture(
        &request,
        &provider_policy(
            super::super::ProviderSerializationMode::NativeFixture,
            Some(native_projection),
        ),
        None,
        Some(&NeverCallVideoService),
    )
    .expect_err("unauthorized live fixture call must fail");
    assert_eq!(live, super::super::Error::LiveProviderCallRequired);

    let upload_service = FailingVideoService {
        error: super::super::VideoPayloadError::UploadTooLarge,
    };
    let upload_error = super::super::serialize_provider_native_video_fixture(
        &request,
        &provider_policy(
            super::super::ProviderSerializationMode::NativeFixture,
            Some(native_projection),
        ),
        None,
        Some(&upload_service),
    )
    .expect_err("oversized upload must fail");
    assert_eq!(upload_error, super::super::Error::VideoUploadTooLarge);

    let mismatched_service = StaticVideoService {
        handle: super::super::VideoPayloadHandle::from_draft(
            super::super::VideoPayloadHandleDraft {
                kind: super::super::VideoPayloadHandleKind::ProviderFileId,
                safe_handle: "mismatched-safe-handle",
                content_hash: OTHER_VIDEO_HASH,
                media_type: "video/mp4",
                expired: false,
            },
        )
        .expect("mismatched safe handle should validate alone"),
    };
    let mismatched = super::super::serialize_provider_native_video_fixture(
        &request,
        &provider_policy(
            super::super::ProviderSerializationMode::NativeFixture,
            Some(native_projection),
        ),
        None,
        Some(&mismatched_service),
    )
    .expect_err("mismatched media identity must fail");
    assert_eq!(
        mismatched,
        super::super::Error::MismatchedVideoHandle {
            field: "video_content_hash"
        }
    );
}

#[test]
fn expired_handles_and_receipt_leaks_are_rejected() {
    // r[verify agent_harness_runtime.video_payloads.evidence.leak]
    let malformed =
        super::super::VideoPayloadHandle::from_draft(super::super::VideoPayloadHandleDraft {
            kind: super::super::VideoPayloadHandleKind::ProviderFileId,
            safe_handle: "https://private.example/video.mp4",
            content_hash: VIDEO_HASH,
            media_type: "video/mp4",
            expired: false,
        })
        .expect_err("remote URL is not a safe handle label");
    assert!(matches!(
        malformed,
        super::super::Error::InvalidVideoHandle { .. }
    ));

    let expired =
        super::super::VideoPayloadHandle::from_draft(super::super::VideoPayloadHandleDraft {
            kind: super::super::VideoPayloadHandleKind::ProviderFileId,
            safe_handle: "expired-safe-handle",
            content_hash: VIDEO_HASH,
            media_type: "video/mp4",
            expired: true,
        })
        .expect_err("expired handles must fail");
    assert_eq!(expired, super::super::Error::ExpiredVideoHandle);

    for fragment in [
        "raw_bytes=[1,2,3]",
        "base64:AAAA",
        "data:video/mp4;base64,AAAA",
        "/home/user/private/video.mp4",
        "token=secret-value",
        "provider_payload={body}",
        "frame_body=[1]",
        "audio_body=[2]",
        "unbounded_metadata",
    ] {
        let error = super::super::VideoReceipt::validate(&super::super::VideoReceiptDraft {
            status: super::super::VideoOutcomeStatus::Included,
            safe_label: "safe-video",
            media_type: "video/mp4",
            byte_count: VIDEO_BYTES,
            content_hash: VIDEO_HASH,
            duration_ms: Some(DURATION_MS),
            width_px: None,
            height_px: None,
            frame_rate_millihz: None,
            service_class: "fixture_service",
            provider_profile: PROVIDER_PROFILE,
            redaction_marker: "metadata_only",
            loss_classes: &[super::super::VideoLossClass::None],
            non_claims: &["no_live_call"],
            metadata_fragments: &[fragment],
        })
        .expect_err("receipt leak must fail");
        assert!(matches!(
            error,
            super::super::Error::UnsafeDescriptor { .. }
        ));
    }
}
