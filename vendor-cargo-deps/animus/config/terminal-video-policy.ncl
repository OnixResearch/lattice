# Typed policy for bounded terminal-video scheduling and host execution.
let contracts = import "../policy/contracts/primitives.ncl" in
let max_frame_bytes_value = 8388608 in
let max_buffered_bytes_value = 16777216 in
let max_queue_frames_value = 2 in
let max_frame_dimension_value = 8192 in
let max_frame_pixels_value = 16777216 in
let max_frames_value = 3600 in
let max_frame_rate_millihertz_value = 60000 in
let max_duration_ticks_value = 600000000 in
let max_dropped_frames_value = 300 in
let max_actions_value = 8 in
let max_lag_ticks_value = 100000 in
let max_frame_deadline_ticks_value = 2000000 in
let max_cleanup_obligations_value = 3 in
let BackendRow = {
  mode | String,
  protocol | String,
  frame_transfer | Bool,
  animation_control | Bool,
  placement | Bool,
  deletion | Bool,
  static_replacement | Bool,
  upstream_frame_adapter | String,
  lifecycle_adapter | String,
} in
let Policy = {
  schema | contracts.exact "TERMINAL_VIDEO_POLICY_SCHEMA" "animus.terminal-video-policy.v1",
  policy_id | contracts.safe_identifier "TERMINAL_VIDEO_POLICY_ID",
  rats_revision | contracts.exact "TERMINAL_VIDEO_RATS_REVISION" "3b498c3d7c248c335d3e7faf12f133a18c87d632",
  max_frame_bytes | contracts.positive_integer "TERMINAL_VIDEO_FRAME_BYTES" max_frame_bytes_value,
  max_buffered_bytes | contracts.positive_integer "TERMINAL_VIDEO_BUFFER_BYTES" max_buffered_bytes_value,
  max_queue_frames | contracts.positive_integer "TERMINAL_VIDEO_QUEUE_FRAMES" max_queue_frames_value,
  max_frame_width | contracts.positive_integer "TERMINAL_VIDEO_FRAME_WIDTH" max_frame_dimension_value,
  max_frame_height | contracts.positive_integer "TERMINAL_VIDEO_FRAME_HEIGHT" max_frame_dimension_value,
  max_frame_pixels | contracts.positive_integer "TERMINAL_VIDEO_FRAME_PIXELS" max_frame_pixels_value,
  max_frames | contracts.positive_integer "TERMINAL_VIDEO_FRAMES" max_frames_value,
  max_frame_rate_millihertz | contracts.positive_integer "TERMINAL_VIDEO_RATE" max_frame_rate_millihertz_value,
  max_duration_ticks | contracts.positive_integer "TERMINAL_VIDEO_DURATION" max_duration_ticks_value,
  max_dropped_frames | contracts.positive_integer "TERMINAL_VIDEO_DROPS" max_dropped_frames_value,
  max_actions | contracts.positive_integer "TERMINAL_VIDEO_ACTIONS" max_actions_value,
  max_lag_ticks | contracts.positive_integer "TERMINAL_VIDEO_LAG" max_lag_ticks_value,
  max_frame_deadline_ticks | contracts.positive_integer "TERMINAL_VIDEO_DEADLINE" max_frame_deadline_ticks_value,
  max_cleanup_obligations | contracts.positive_integer "TERMINAL_VIDEO_CLEANUP" max_cleanup_obligations_value,
  allow_frame_drop | Bool,
  allow_static_replacement | Bool,
  allow_thumbnail_fallback | Bool,
  allow_text_fallback | Bool,
  require_injected_frame_source | Bool,
  require_injected_terminal_backend | Bool,
  require_explicit_monotonic_ticks | Bool,
  backend_rows | Array BackendRow,
  receipt_redaction_profile | String,
  non_claims | Array String,
} in
let CheckedPolicy = contracts.with_validator Policy (fun value =>
  if value.max_buffered_bytes < value.max_frame_bytes then
    'Error { message = "TERMINAL_VIDEO_BUFFER_BOUND" }
  else if value.max_frame_pixels < value.max_frame_width then
    'Error { message = "TERMINAL_VIDEO_PIXEL_BOUND" }
  else if value.max_cleanup_obligations > value.max_actions then
    'Error { message = "TERMINAL_VIDEO_CLEANUP_ACTION_BOUND" }
  else if !value.require_injected_frame_source || !value.require_injected_terminal_backend then
    'Error { message = "TERMINAL_VIDEO_INJECTED_SERVICES_REQUIRED" }
  else if !value.require_explicit_monotonic_ticks then
    'Error { message = "TERMINAL_VIDEO_EXPLICIT_TICKS_REQUIRED" }
  else if std.array.length value.backend_rows != 4 then
    'Error { message = "TERMINAL_VIDEO_BACKEND_ROW_COUNT" }
  else if std.array.length value.non_claims != 10 then
    'Error { message = "TERMINAL_VIDEO_NON_CLAIMS_COUNT" }
  else
    'Ok
) in
{
  terminal_video_policy_contract = CheckedPolicy,
  terminal_video_policy | CheckedPolicy = {
    schema = "animus.terminal-video-policy.v1",
    policy_id = "animus.terminal-video.default.v1",
    rats_revision = "3b498c3d7c248c335d3e7faf12f133a18c87d632",
    max_frame_bytes = max_frame_bytes_value,
    max_buffered_bytes = max_buffered_bytes_value,
    max_queue_frames = max_queue_frames_value,
    max_frame_width = max_frame_dimension_value,
    max_frame_height = max_frame_dimension_value,
    max_frame_pixels = max_frame_pixels_value,
    max_frames = max_frames_value,
    max_frame_rate_millihertz = max_frame_rate_millihertz_value,
    max_duration_ticks = max_duration_ticks_value,
    max_dropped_frames = max_dropped_frames_value,
    max_actions = max_actions_value,
    max_lag_ticks = max_lag_ticks_value,
    max_frame_deadline_ticks = max_frame_deadline_ticks_value,
    max_cleanup_obligations = max_cleanup_obligations_value,
    allow_frame_drop = true,
    allow_static_replacement = true,
    allow_thumbnail_fallback = true,
    allow_text_fallback = true,
    require_injected_frame_source = true,
    require_injected_terminal_backend = true,
    require_explicit_monotonic_ticks = true,
    backend_rows = [
      {
        mode = "kitty-animation",
        protocol = "kitty",
        frame_transfer = true,
        animation_control = true,
        placement = true,
        deletion = true,
        static_replacement = true,
        upstream_frame_adapter = "rat-image-static-frame-transfer",
        lifecycle_adapter = "host-admitted-kitty-animation-controls",
      },
      {
        mode = "static-replacement",
        protocol = "kitty",
        frame_transfer = true,
        animation_control = false,
        placement = true,
        deletion = true,
        static_replacement = true,
        upstream_frame_adapter = "rat-image-static-frame-transfer",
        lifecycle_adapter = "host-admitted-static-lifecycle",
      },
      {
        mode = "thumbnail-fallback",
        protocol = "text",
        frame_transfer = false,
        animation_control = false,
        placement = false,
        deletion = false,
        static_replacement = false,
        upstream_frame_adapter = "none",
        lifecycle_adapter = "injected-thumbnail-source",
      },
      {
        mode = "text-fallback",
        protocol = "text",
        frame_transfer = false,
        animation_control = false,
        placement = false,
        deletion = false,
        static_replacement = false,
        upstream_frame_adapter = "none",
        lifecycle_adapter = "bounded-visible-text",
      },
    ],
    receipt_redaction_profile = "metadata-only-v1",
    non_claims = [
      "not-decoder-safety",
      "not-media-authenticity",
      "not-exact-timing",
      "not-smooth-playback",
      "not-frame-completeness",
      "not-color-fidelity",
      "not-audio-or-av-synchronization",
      "not-terminal-correctness",
      "not-production-readiness",
      "not-release-eligibility",
    ],
  },
}
