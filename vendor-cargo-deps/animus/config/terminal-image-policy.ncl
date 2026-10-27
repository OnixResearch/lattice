# Typed policy for bounded terminal-image planning and host execution.
let contracts = import "../policy/contracts/primitives.ncl" in
let max_image_bytes_value = 8388608 in
let max_prepared_bytes_value = 33554432 in
let max_capability_response_bytes_value = 256 in
let max_source_dimension_value = 8192 in
let max_source_pixels_value = 16777216 in
let max_decoded_bytes_value = 134217728 in
let max_terminal_width_value = 256 in
let max_terminal_height_value = 128 in
let max_terminal_cells_value = 32768 in
let max_chunks_value = 8192 in
let max_operations_value = 8 in
let max_lifecycle_entries_value = 64 in
let max_alt_text_chars_value = 256 in
let ProtocolRow = {
  protocol | String,
  query | Bool,
  transmit | Bool,
  place | Bool,
  replace | Bool,
  delete | Bool,
  cleanup | Bool,
  upstream_payload_adapter | String,
} in
let Policy = {
  schema | contracts.exact "TERMINAL_IMAGE_POLICY_SCHEMA" "animus.terminal-image-policy.v1",
  policy_id | contracts.safe_identifier "TERMINAL_IMAGE_POLICY_ID",
  rats_revision | contracts.exact "TERMINAL_IMAGE_RATS_REVISION" "3b498c3d7c248c335d3e7faf12f133a18c87d632",
  max_image_bytes | contracts.positive_integer "TERMINAL_IMAGE_BYTES" max_image_bytes_value,
  max_prepared_bytes | contracts.positive_integer "TERMINAL_PREPARED_BYTES" max_prepared_bytes_value,
  max_capability_response_bytes | contracts.positive_integer "TERMINAL_CAPABILITY_RESPONSE_BYTES" max_capability_response_bytes_value,
  max_source_width | contracts.positive_integer "TERMINAL_SOURCE_WIDTH" max_source_dimension_value,
  max_source_height | contracts.positive_integer "TERMINAL_SOURCE_HEIGHT" max_source_dimension_value,
  max_source_pixels | contracts.positive_integer "TERMINAL_SOURCE_PIXELS" max_source_pixels_value,
  max_decoded_bytes | contracts.positive_integer "TERMINAL_DECODED_BYTES" max_decoded_bytes_value,
  max_terminal_width | contracts.positive_integer "TERMINAL_CELL_WIDTH" max_terminal_width_value,
  max_terminal_height | contracts.positive_integer "TERMINAL_CELL_HEIGHT" max_terminal_height_value,
  max_terminal_cells | contracts.positive_integer "TERMINAL_CELLS" max_terminal_cells_value,
  max_chunks | contracts.positive_integer "TERMINAL_IMAGE_CHUNKS" max_chunks_value,
  max_operations | contracts.positive_integer "TERMINAL_IMAGE_OPERATIONS" max_operations_value,
  max_lifecycle_entries | contracts.positive_integer "TERMINAL_IMAGE_LIFECYCLE" max_lifecycle_entries_value,
  max_alt_text_chars | contracts.positive_integer "TERMINAL_IMAGE_ALT_TEXT" max_alt_text_chars_value,
  allow_text_fallback | Bool,
  require_injected_byte_resolver | Bool,
  require_injected_writer | Bool,
  protocol_rows | Array ProtocolRow,
  receipt_redaction_profile | String,
  non_claims | Array String,
} in
let CheckedPolicy = contracts.with_validator Policy (fun value =>
  if value.max_prepared_bytes < value.max_image_bytes then
    'Error { message = "TERMINAL_IMAGE_PREPARED_BOUND" }
  else if value.max_source_pixels < value.max_source_width then
    'Error { message = "TERMINAL_IMAGE_SOURCE_BOUND" }
  else if value.max_terminal_cells < value.max_terminal_width then
    'Error { message = "TERMINAL_IMAGE_CELL_BOUND" }
  else if !value.require_injected_byte_resolver || !value.require_injected_writer then
    'Error { message = "TERMINAL_IMAGE_INJECTED_SERVICES_REQUIRED" }
  else if std.array.length value.protocol_rows != 4 then
    'Error { message = "TERMINAL_IMAGE_PROTOCOL_ROW_COUNT" }
  else if std.array.length value.non_claims != 8 then
    'Error { message = "TERMINAL_IMAGE_NON_CLAIMS_COUNT" }
  else
    'Ok
) in
{
  terminal_image_policy_contract = CheckedPolicy,
  terminal_image_policy | CheckedPolicy = {
    schema = "animus.terminal-image-policy.v1",
    policy_id = "animus.terminal-image.default.v1",
    rats_revision = "3b498c3d7c248c335d3e7faf12f133a18c87d632",
    max_image_bytes = max_image_bytes_value,
    max_prepared_bytes = max_prepared_bytes_value,
    max_capability_response_bytes = max_capability_response_bytes_value,
    max_source_width = max_source_dimension_value,
    max_source_height = max_source_dimension_value,
    max_source_pixels = max_source_pixels_value,
    max_decoded_bytes = max_decoded_bytes_value,
    max_terminal_width = max_terminal_width_value,
    max_terminal_height = max_terminal_height_value,
    max_terminal_cells = max_terminal_cells_value,
    max_chunks = max_chunks_value,
    max_operations = max_operations_value,
    max_lifecycle_entries = max_lifecycle_entries_value,
    max_alt_text_chars = max_alt_text_chars_value,
    allow_text_fallback = true,
    require_injected_byte_resolver = true,
    require_injected_writer = true,
    protocol_rows = [
      {
        protocol = "kitty",
        query = true,
        transmit = true,
        place = true,
        replace = true,
        delete = true,
        cleanup = true,
        upstream_payload_adapter = "rat-image-transmit-only",
      },
      {
        protocol = "iterm2",
        query = false,
        transmit = true,
        place = false,
        replace = false,
        delete = false,
        cleanup = false,
        upstream_payload_adapter = "rat-image-transmit-only",
      },
      {
        protocol = "sixel",
        query = false,
        transmit = true,
        place = false,
        replace = false,
        delete = false,
        cleanup = false,
        upstream_payload_adapter = "rat-image-transmit-only",
      },
      {
        protocol = "text",
        query = false,
        transmit = false,
        place = false,
        replace = false,
        delete = false,
        cleanup = false,
        upstream_payload_adapter = "none",
      },
    ],
    receipt_redaction_profile = "metadata-only-v1",
    non_claims = [
      "not-live-terminal-validation",
      "not-emulator-parity",
      "not-image-decoder-correctness",
      "not-process-isolation",
      "not-video-or-audio-playback",
      "not-provider-availability",
      "not-production-readiness",
      "not-release-eligibility",
    ],
  },
}
