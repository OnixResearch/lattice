mod conformance;
pub use conformance::*;

/// Stable contract identity for public model request DTOs.
pub const MODEL_REQUEST_CONTRACT_IDENTITY: &str = crate::model::MODEL_REQUEST_CONTRACT_IDENTITY;
/// Stable contract identity for public model response DTOs.
pub const MODEL_RESPONSE_CONTRACT_IDENTITY: &str = crate::model::MODEL_RESPONSE_CONTRACT_IDENTITY;
/// Stable contract identity for canonical provider-neutral model request DTOs.
pub const CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY: &str =
    crate::model::CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY;
/// Stable contract identity for canonical provider-neutral model response envelopes.
pub const CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY: &str =
    crate::model::CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY;
/// Stable contract identity for public model stream frame DTOs.
pub const MODEL_STREAM_FRAME_CONTRACT_IDENTITY: &str =
    crate::model::wire::MODEL_STREAM_FRAME_CONTRACT_IDENTITY;

/// Stable model-call correlation identifier carried by stream frames.
pub type ModelStreamCallId = crate::model::wire::parts::CallId;
/// Canonical model stream frame emitted by model adapters.
pub type ModelStreamFrame = crate::model::wire::types::Frame;
/// Borrowed model stream frame fields before validation.
pub type ModelStreamFrameDraft<'a> = crate::model::wire::types::Draft<'a>;
/// Canonical model stream frame kind.
pub type ModelStreamFrameKind = crate::model::wire::parts::Kind;
/// Model streaming validation, parsing, and assembly failure.
pub type ModelStreamError = crate::model::wire::failure::Error;
/// Strictly increasing per-call stream order.
pub type ModelStreamOrder = crate::model::wire::parts::Order;
/// Stable tool-call correlation identifier carried by tool-call deltas.
pub type ModelStreamToolCallId = crate::model::wire::parts::ToolCallId;
/// Usage counts supplied by a model stream.
pub type ModelStreamUsage = crate::model::wire::parts::Usage;
/// Default sink visibility attached to streamed text channels.
pub type ModelStreamVisibility = crate::model::wire::parts::Visibility;
/// Request sent from the harness to a model component.
pub type ModelRequest = crate::model::Request;
/// Response returned by a model component.
pub type ModelResponse = crate::model::Response;
/// Canonical provider-neutral model request.
pub type CanonicalModelRequest = crate::model::CanonicalModelRequest;
/// Borrowed canonical provider-neutral model request before validation.
pub type CanonicalModelRequestDraft<'a> = crate::model::CanonicalModelRequestDraft<'a>;
/// Canonical provider-neutral message.
pub type CanonicalModelMessage = crate::model::CanonicalModelMessage;
/// Borrowed canonical provider-neutral message before validation.
pub type CanonicalModelMessageDraft<'a> = crate::model::CanonicalModelMessageDraft<'a>;
/// Canonical provider-neutral model role.
pub type CanonicalModelRole = crate::model::CanonicalModelRole;
/// Canonical provider-neutral content part.
pub type CanonicalMessageContentPart = crate::model::CanonicalContentPart;
/// Borrowed canonical provider-neutral content part before validation.
pub type CanonicalMessageContentPartDraft<'a> = crate::model::CanonicalContentPartDraft<'a>;
/// Canonical tool declaration available to a model request.
pub type CanonicalModelToolDeclaration = crate::model::CanonicalToolDeclaration;
/// Borrowed canonical tool declaration before validation.
pub type CanonicalModelToolDeclarationDraft<'a> = crate::model::CanonicalToolDeclarationDraft<'a>;
/// Canonical model output controls.
pub type CanonicalModelOutputControls = crate::model::CanonicalOutputControls;
/// Borrowed canonical model output controls before validation.
pub type CanonicalModelOutputControlsDraft<'a> = crate::model::CanonicalOutputControlsDraft<'a>;
/// Canonical model response-format intent.
pub type CanonicalModelResponseFormatIntent = crate::model::CanonicalResponseFormatIntent;
/// Borrowed canonical model response-format intent before validation.
pub type CanonicalModelResponseFormatIntentDraft<'a> =
    crate::model::CanonicalResponseFormatIntentDraft<'a>;
/// Canonical model reasoning effort request.
pub type CanonicalModelReasoningEffort = crate::model::CanonicalReasoningEffort;
/// Canonical model reasoning request.
pub type CanonicalModelReasoningRequest = crate::model::CanonicalReasoningRequest;
/// Borrowed canonical model reasoning request before validation.
pub type CanonicalModelReasoningRequestDraft<'a> = crate::model::CanonicalReasoningRequestDraft<'a>;
/// Canonical model response envelope.
pub type CanonicalModelResponseEnvelope = crate::model::CanonicalModelResponseEnvelope;
/// Borrowed canonical model response envelope before validation.
pub type CanonicalModelResponseEnvelopeDraft<'a> =
    crate::model::CanonicalModelResponseEnvelopeDraft<'a>;
/// Canonical model response item.
pub type CanonicalModelResponseItem = crate::model::CanonicalModelResponseItem;
/// Borrowed canonical model response item before validation.
pub type CanonicalModelResponseItemDraft<'a> = crate::model::CanonicalModelResponseItemDraft<'a>;
/// Canonical response finish reason.
pub type CanonicalModelFinishReason = crate::model::CanonicalFinishReason;
/// Canonical model usage metadata.
pub type CanonicalModelUsage = crate::model::CanonicalUsage;
/// Borrowed canonical model usage metadata before validation.
pub type CanonicalModelUsageDraft = crate::model::CanonicalUsageDraft;
/// Canonical safe provenance metadata.
pub type CanonicalModelSafeMetadata = crate::model::CanonicalSafeMetadata;
/// Borrowed canonical safe provenance metadata before validation.
pub type CanonicalModelSafeMetadataDraft<'a> = crate::model::CanonicalSafeMetadataDraft<'a>;
/// Canonical model contract validation error.
pub type CanonicalModelContractError = crate::model::CanonicalModelContractError;
/// Unsafe material class rejected by canonical model contracts.
pub type CanonicalModelUnsafeTextReason = crate::model::CanonicalUnsafeTextReason;
/// Validated provider-neutral model content part.
pub type ModelContentPart = crate::multimodal::ContentPart;
/// Borrowed provider-neutral model content part before validation.
pub type ModelContentPartDraft<'a> = crate::multimodal::ContentPartDraft<'a>;
/// Validated text model content part.
pub type ModelTextPart = crate::multimodal::TextPart;
/// Validated image model content part.
pub type ModelImagePart = crate::multimodal::ImagePart;
/// Borrowed image model content part before validation.
pub type ModelImagePartDraft<'a> = crate::multimodal::ImagePartDraft<'a>;
/// Validated optional image metadata.
pub type ModelImageMetadata = crate::multimodal::ImageMetadata;
/// Borrowed optional image metadata before validation.
pub type ModelImageMetadataDraft<'a> = crate::multimodal::ImageMetadataDraft<'a>;
/// Supported model image media type.
pub type ModelImageMediaType = crate::multimodal::ImageMediaType;
/// Host-declared image detail preference.
pub type ModelImageDetailPreference = crate::multimodal::ImageDetailPreference;
/// Trust metadata attached to a model image source descriptor.
pub type ModelImageSourceTrust = crate::multimodal::ImageSourceTrust;
/// Safe image assembly outcome.
pub type ModelImageOutcome = crate::multimodal::ImageOutcome;
/// Safe image assembly outcome status.
pub type ModelImageOutcomeStatus = crate::multimodal::ImageOutcomeStatus;
/// Validated provider-neutral video part.
pub type ModelVideoPart = crate::multimodal::VideoPart;
/// Borrowed provider-neutral video part.
pub type ModelVideoPartDraft<'a> = crate::multimodal::VideoPartDraft<'a>;
/// Validated declared video metadata.
pub type ModelVideoMetadata = crate::multimodal::VideoMetadata;
/// Borrowed declared video metadata.
pub type ModelVideoMetadataDraft<'a> = crate::multimodal::VideoMetadataDraft<'a>;
/// Admitted video media type.
pub type ModelVideoMediaType = crate::multimodal::VideoMediaType;
/// Video source trust classification.
pub type ModelVideoSourceTrust = crate::multimodal::VideoSourceTrust;
/// Video detail preference.
pub type ModelVideoDetailPreference = crate::multimodal::VideoDetailPreference;
/// Ordered video assembly outcome.
pub type ModelVideoOutcome = crate::multimodal::VideoOutcome;
/// Video assembly outcome status.
pub type ModelVideoOutcomeStatus = crate::multimodal::VideoOutcomeStatus;
/// Video capability policy.
pub type ModelVideoCapabilityPolicy = crate::multimodal::VideoCapabilityPolicy;
/// Video capability result.
pub type ModelVideoCapabilityOutcome = crate::multimodal::VideoCapabilityOutcome;
/// Validated exact video input capability facts.
pub type ModelVideoInputCapabilities = crate::multimodal::VideoInputCapabilities;
/// Borrowed exact video input capability facts.
pub type ModelVideoInputCapabilitiesDraft<'a> = crate::multimodal::VideoInputCapabilitiesDraft<'a>;
/// Deterministic prompt assembly result for multimodal model content.
pub type ModelPromptAssembly = crate::multimodal::PromptAssembly;
/// Deterministic prompt assembly policy for multimodal model content.
pub type ModelPromptAssemblyPolicy = crate::multimodal::PromptAssemblyPolicy;
/// Multimodal model adapter validation error.
pub type ModelMultimodalError = crate::multimodal::Error;
/// Unsafe multimodal descriptor or receipt material class.
pub type ModelUnsafeContentReason = crate::multimodal::UnsafeContentReason;
/// Provider-neutral model input modality.
pub type ModelInputModality = crate::multimodal::InputModality;
/// Validated selected model input capability metadata.
pub type SelectedModelInputCapabilities = crate::multimodal::ModelInputCapabilities;
/// Borrowed selected model input capability metadata before validation.
pub type SelectedModelInputCapabilitiesDraft<'a> =
    crate::multimodal::ModelInputCapabilitiesDraft<'a>;
/// Multimodal dispatch plan after capability gates pass.
pub type ModelMultimodalDispatchPlan = crate::multimodal::DispatchPlan;
/// Validated provider-native serialization policy.
pub type ModelProviderSerializationPolicy = crate::multimodal::ProviderSerializationPolicy;
/// Borrowed provider-native serialization policy before validation.
pub type ModelProviderSerializationPolicyDraft<'a> =
    crate::multimodal::ProviderSerializationPolicyDraft<'a>;
/// Provider request serialization mode.
pub type ModelProviderSerializationMode = crate::multimodal::ProviderSerializationMode;
/// Exact reviewed provider video projection facts.
pub type ModelProviderVideoProjection = crate::multimodal::ProviderVideoProjection;
/// Support state for one provider video projection fact.
pub type ModelProviderProjectionSupport = crate::multimodal::ProviderProjectionSupport;
/// Provider-native request fixture.
pub type ModelProviderNativeRequest = crate::multimodal::ProviderNativeRequest;
/// Provider-native request fixture part.
pub type ModelProviderNativePart = crate::multimodal::ProviderNativePart;
/// Host-injected model image payload service.
pub type ModelImagePayloadService = dyn crate::multimodal::ImagePayloadService;
/// Host-injected model image payload service error.
pub type ModelImagePayloadError = crate::multimodal::ImagePayloadError;
/// Validated provider-native image payload handle.
pub type ModelImagePayloadHandle = crate::multimodal::ImagePayloadHandle;
/// Borrowed provider-native image payload handle before validation.
pub type ModelImagePayloadHandleDraft<'a> = crate::multimodal::ImagePayloadHandleDraft<'a>;
/// Provider-native image payload handle kind.
pub type ModelImagePayloadHandleKind = crate::multimodal::ImagePayloadHandleKind;
/// Safe image receipt metadata.
pub type ModelImageReceipt = crate::multimodal::ImageReceipt;
/// Borrowed safe image receipt metadata before validation.
pub type ModelImageReceiptDraft<'a> = crate::multimodal::ImageReceiptDraft<'a>;
/// Host-injected video payload service.
pub type ModelVideoPayloadService = dyn crate::multimodal::VideoPayloadService;
/// Video payload service error.
pub type ModelVideoPayloadError = crate::multimodal::VideoPayloadError;
/// Validated safe video payload handle.
pub type ModelVideoPayloadHandle = crate::multimodal::VideoPayloadHandle;
/// Borrowed safe video payload handle.
pub type ModelVideoPayloadHandleDraft<'a> = crate::multimodal::VideoPayloadHandleDraft<'a>;
/// Video payload handle class.
pub type ModelVideoPayloadHandleKind = crate::multimodal::VideoPayloadHandleKind;
/// Safe metadata-only video receipt.
pub type ModelVideoReceipt = crate::multimodal::VideoReceipt;
/// Borrowed safe video receipt fields.
pub type ModelVideoReceiptDraft<'a> = crate::multimodal::VideoReceiptDraft<'a>;
/// Provider video projection loss class.
pub type ModelVideoLossClass = crate::multimodal::VideoLossClass;
/// Largest admitted video byte count.
pub const MAX_MODEL_VIDEO_BYTE_COUNT: u64 = crate::multimodal::MAX_VIDEO_BYTE_COUNT;
/// Largest admitted declared video duration.
pub const MAX_MODEL_VIDEO_DURATION_MS: u64 = crate::multimodal::MAX_VIDEO_DURATION_MS;
/// Default per-video prompt budget.
pub const DEFAULT_MODEL_PROMPT_VIDEO_BYTE_BUDGET: u64 =
    crate::multimodal::DEFAULT_PROMPT_VIDEO_BYTE_BUDGET;
/// Default aggregate media prompt budget.
pub const DEFAULT_MODEL_AGGREGATE_MEDIA_BYTE_BUDGET: u64 =
    crate::multimodal::DEFAULT_AGGREGATE_MEDIA_BYTE_BUDGET;
/// Largest accepted model image byte count before host-side transformation is required.
pub const MAX_MODEL_IMAGE_BYTE_COUNT: u64 = crate::multimodal::MAX_IMAGE_BYTE_COUNT;
/// Default per-image prompt assembly budget for model image inputs.
pub const DEFAULT_MODEL_PROMPT_IMAGE_BYTE_BUDGET: u64 =
    crate::multimodal::DEFAULT_PROMPT_IMAGE_BYTE_BUDGET;
/// Assembles ordered multimodal model content and safe image outcomes.
///
/// # Errors
///
/// Returns [`ModelMultimodalError`] when any content part is malformed or no parts are supplied.
pub fn assemble_model_content_parts(
    drafts: &[ModelContentPartDraft<'_>],
    policy: ModelPromptAssemblyPolicy,
) -> Result<ModelPromptAssembly, ModelMultimodalError> {
    crate::multimodal::assemble_prompt_parts(drafts, policy)
}

/// Prepares multimodal model dispatch only when selected capabilities support all included parts.
///
/// # Errors
///
/// Returns [`ModelMultimodalError`] when a required modality or image adapter is unavailable.
pub fn prepare_model_multimodal_dispatch(
    assembly: &ModelPromptAssembly,
    capabilities: &SelectedModelInputCapabilities,
) -> Result<ModelMultimodalDispatchPlan, ModelMultimodalError> {
    crate::multimodal::prepare_dispatch(assembly, capabilities)
}

/// Prepares dispatch under an explicit required-or-optional video policy.
///
/// # Errors
///
/// Returns [`ModelMultimodalError`] when required media or its service is unavailable.
pub fn prepare_model_multimodal_dispatch_with_video_policy(
    assembly: &ModelPromptAssembly,
    capabilities: &SelectedModelInputCapabilities,
    video_policy: ModelVideoCapabilityPolicy,
) -> Result<ModelMultimodalDispatchPlan, ModelMultimodalError> {
    crate::multimodal::prepare_dispatch_with_video_policy(assembly, capabilities, video_policy)
}

/// Serializes an assembled model request into provider-native fixture parts.
///
/// # Errors
///
/// Returns [`ModelMultimodalError`] when image serialization would be lossy or lacks an injected service.
pub fn serialize_model_provider_native_fixture(
    assembly: &ModelPromptAssembly,
    policy: &ModelProviderSerializationPolicy,
    service: Option<&ModelImagePayloadService>,
) -> Result<ModelProviderNativeRequest, ModelMultimodalError> {
    crate::multimodal::serialize_provider_native_fixture(assembly, policy, service)
}

/// Serializes an exact provider-native fixture with injected image and video services.
///
/// # Errors
///
/// Returns [`ModelMultimodalError`] for missing services, unsafe handles, or visible projection loss.
pub fn serialize_model_provider_native_video_fixture(
    assembly: &ModelPromptAssembly,
    policy: &ModelProviderSerializationPolicy,
    image_service: Option<&ModelImagePayloadService>,
    video_service: Option<&ModelVideoPayloadService>,
) -> Result<ModelProviderNativeRequest, ModelMultimodalError> {
    crate::multimodal::serialize_provider_native_video_fixture(
        assembly,
        policy,
        image_service,
        video_service,
    )
}

/// Assembles assistant-visible response text from validated stream frames.
///
/// # Errors
///
/// Returns [`ModelStreamError`] when stream frames are malformed or terminally invalid.
pub fn assemble_visible_model_stream_response(
    frames: &[ModelStreamFrame],
) -> Result<ModelResponse, ModelStreamError> {
    crate::model::wire::validate::assemble(frames)
}

/// Parses raw provider-stream fixture chunks into canonical model frames.
///
/// # Errors
///
/// Returns [`ModelStreamError`] when fixture chunks cannot be parsed safely.
pub fn parse_model_stream_fixture(
    chunks: &[&str],
) -> Result<alloc::vec::Vec<ModelStreamFrame>, ModelStreamError> {
    crate::model::wire::fixture::parse(chunks)
}

/// Validates model stream ordering, correlation, and terminal status.
///
/// # Errors
///
/// Returns [`ModelStreamError`] when frame ordering, correlation, or terminal state is invalid.
pub fn validate_model_stream_frames(frames: &[ModelStreamFrame]) -> Result<(), ModelStreamError> {
    crate::model::wire::validate::frames(frames)
}

/// Projects a canonical request into provider-family literal fixture parts.
///
/// # Errors
///
/// Returns [`ProviderProjectionError`] when projection would be lossy.
pub fn project_model_provider_request_fixture(
    request: &CanonicalModelRequest,
    support: ProviderProjectionSupport,
) -> Result<ProviderRequestProjection, ProviderProjectionError> {
    crate::model::project_provider_request_fixture(request, support)
}

/// Compares a provider request projection with reviewed literal fixture data.
///
/// # Errors
///
/// Returns [`ProviderProjectionError`] when projection is lossy or mismatches expected literals.
pub fn assert_model_provider_request_fixture(
    request: &CanonicalModelRequest,
    support: ProviderProjectionSupport,
    expected: &[&str],
) -> Result<ProviderRequestProjection, ProviderProjectionError> {
    crate::model::assert_provider_request_fixture(request, support, expected)
}

/// Normalizes a provider response fixture into a canonical response envelope.
///
/// # Errors
///
/// Returns [`ProviderProjectionError`] when response semantics are malformed or leaky.
pub fn normalize_model_provider_response_fixture(
    draft: ProviderResponseFixtureDraft<'_>,
) -> Result<CanonicalModelResponseEnvelope, ProviderProjectionError> {
    crate::model::normalize_provider_response_fixture(draft)
}

/// Validates provider stream chunks through the production model-stream parser.
///
/// # Errors
///
/// Returns [`ProviderProjectionError`] when stream chunks are malformed or unexpected.
pub fn assert_model_provider_stream_fixture(
    chunks: &[&str],
    expected_kinds: &[ModelStreamFrameKind],
) -> Result<alloc::vec::Vec<ModelStreamFrame>, ProviderProjectionError> {
    crate::model::assert_provider_stream_fixture(chunks, expected_kinds)
}

/// Validates provider projection conformance index entries.
///
/// # Errors
///
/// Returns [`ProviderProjectionError`] when evidence or reasons are missing.
pub fn validate_model_provider_projection_index(
    drafts: &[ProviderProjectionIndexEntryDraft<'_>],
) -> Result<alloc::vec::Vec<ProviderProjectionIndexEntry>, ProviderProjectionError> {
    crate::model::validate_provider_projection_index(drafts)
}
