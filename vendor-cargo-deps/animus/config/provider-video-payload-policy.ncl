# Typed provider-neutral video payload policy.
let contracts = import "../policy/contracts/primitives.ncl" in
let max_descriptor_bytes_value = 536870912 in
let max_prompt_video_bytes_value = 104857600 in
let max_aggregate_media_bytes_value = 134217728 in
let max_duration_ms_value = 1200000 in
let max_dimension_px_value = 32768 in
let max_frame_rate_millihz_value = 120000 in
let max_video_count_value = 4 in
let max_provider_handle_chars_value = 256 in
let max_receipt_fragments_value = 16 in
let Policy = {
  schema | contracts.exact "VIDEO_POLICY_SCHEMA" "animus.provider-video-payload-policy.v1",
  policy_id | contracts.safe_identifier "VIDEO_POLICY_ID",
  max_descriptor_bytes | contracts.positive_integer "VIDEO_DESCRIPTOR_BYTES" max_descriptor_bytes_value,
  max_prompt_video_bytes | contracts.positive_integer "VIDEO_PROMPT_BYTES" max_prompt_video_bytes_value,
  max_aggregate_media_bytes | contracts.positive_integer "VIDEO_AGGREGATE_BYTES" max_aggregate_media_bytes_value,
  max_duration_ms | contracts.positive_integer "VIDEO_DURATION" max_duration_ms_value,
  max_dimension_px | contracts.positive_integer "VIDEO_DIMENSION" max_dimension_px_value,
  max_frame_rate_millihz | contracts.positive_integer "VIDEO_FRAME_RATE" max_frame_rate_millihz_value,
  max_video_count | contracts.positive_integer "VIDEO_COUNT" max_video_count_value,
  max_provider_handle_chars | contracts.positive_integer "VIDEO_HANDLE" max_provider_handle_chars_value,
  max_receipt_fragments | contracts.positive_integer "VIDEO_RECEIPT" max_receipt_fragments_value,
  admitted_media_types | Array String,
  exact_provider_profiles | Array String,
  generic_json_video_allowed | Bool,
  require_injected_payload_service | Bool,
  allow_optional_omission | Bool,
  receipt_redaction_profile | String,
  non_claims | Array String,
} in
let CheckedPolicy = contracts.with_validator Policy (fun value =>
  if std.array.length value.admitted_media_types != 4 then
    'Error { message = "VIDEO_MEDIA_TYPE_COUNT" }
  else if std.array.length value.exact_provider_profiles != 1 then
    'Error { message = "VIDEO_PROVIDER_PROFILE_COUNT" }
  else if value.generic_json_video_allowed then
    'Error { message = "VIDEO_GENERIC_JSON_FORBIDDEN" }
  else if !value.require_injected_payload_service then
    'Error { message = "VIDEO_SERVICE_REQUIRED" }
  else if value.allow_optional_omission then
    'Error { message = "VIDEO_DEFAULT_OMISSION_FORBIDDEN" }
  else if std.array.length value.non_claims != 8 then
    'Error { message = "VIDEO_NON_CLAIMS_COUNT" }
  else
    'Ok
) in
{
  provider_video_payload_policy_contract = CheckedPolicy,
  provider_video_payload_policy | CheckedPolicy = {
    schema = "animus.provider-video-payload-policy.v1",
    policy_id = "animus.provider-video-payload.default.v1",
    max_descriptor_bytes = max_descriptor_bytes_value,
    max_prompt_video_bytes = max_prompt_video_bytes_value,
    max_aggregate_media_bytes = max_aggregate_media_bytes_value,
    max_duration_ms = max_duration_ms_value,
    max_dimension_px = max_dimension_px_value,
    max_frame_rate_millihz = max_frame_rate_millihz_value,
    max_video_count = max_video_count_value,
    max_provider_handle_chars = max_provider_handle_chars_value,
    max_receipt_fragments = max_receipt_fragments_value,
    admitted_media_types = ["video/mp4", "video/webm", "video/quicktime", "video/mpeg"],
    exact_provider_profiles = ["google-gemini-v1beta-files-fixture"],
    generic_json_video_allowed = false,
    require_injected_payload_service = true,
    allow_optional_omission = false,
    receipt_redaction_profile = "metadata-only-v1",
    non_claims = [
      "not-media-authenticity",
      "not-codec-safety",
      "not-successful-decoding",
      "not-upload-completion",
      "not-provider-acceptance",
      "not-semantic-understanding",
      "not-production-readiness",
      "not-release-eligibility",
    ],
  },
}
