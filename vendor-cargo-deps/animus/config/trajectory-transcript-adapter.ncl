# Typed, non-default @letta-ai/trajectory transcript adapter profile.
let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && value != "") in
let Blake3Hex = std.contract.from_predicate (fun value => std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value) in
let GitRevision = std.contract.from_predicate (fun value => std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value) in
let Positive = std.contract.from_predicate (fun value => std.is_number value && value > 0) in
let Source = [| 'claude_code, 'codex, 'hermes, 'letta_code, 'openclaw, 'openhands, 'pi |] in
let Disposition = [| 'accepted_cleanup, 'lossy, 'rejected |] in
let Profile = {
  schema | NonEmpty,
  enabled_by_default | Bool,
  cohort | {
    package | NonEmpty,
    version | NonEmpty,
    revision | GitRevision,
    trajectory_schema | NonEmpty,
    trajectory_schema_blake3 | Blake3Hex,
    helper_blake3 | Blake3Hex,
    license | NonEmpty,
    sources | Array Source,
  },
  bounds | {
    max_transcript_bytes | Positive,
    max_output_bytes | Positive,
    max_records | Positive,
    max_content_bytes | Positive,
    max_tool_calls | Positive,
    max_tool_argument_bytes | Positive,
    max_tool_result_bytes | Positive,
    max_diagnostics | Positive,
  },
  diagnostics | Array { code | NonEmpty, disposition | Disposition },
  content | {
    reasoning | NonEmpty,
    tool_arguments | NonEmpty,
    tool_results | NonEmpty,
    path_metadata | NonEmpty,
  },
  projection | NonEmpty,
  helper_protocol | NonEmpty,
  timeout_class | NonEmpty,
  memory_class | NonEmpty,
  receipt_schema | NonEmpty,
  non_claims | Array NonEmpty,
} in
{
  trajectory_transcript_adapter | Profile = {
    schema = "animus.trajectory-transcript-adapter.v1",
    enabled_by_default = false,
    cohort = {
      package = "@letta-ai/trajectory",
      version = "0.2.0",
      revision = "a665556089d6a3c59f5806ee109b4d36f6c2ac83",
      trajectory_schema = "https://letta.ai/schemas/trajectory/v1.json",
      trajectory_schema_blake3 = "4193e80e761f1aaa9e59545f356de02d2aa33443b4430fae4d1d82e29bca1212",
      helper_blake3 = "1ef86c7687ff8f7354cb08e515e391e66f4e894f892aa807597d4ce399f5de20",
      license = "Apache-2.0",
      sources = ['claude_code, 'codex, 'hermes, 'letta_code, 'openclaw, 'openhands, 'pi],
    },
    bounds = {
      max_transcript_bytes = 1048576,
      max_output_bytes = 2097152,
      max_records = 4096,
      max_content_bytes = 262144,
      max_tool_calls = 1024,
      max_tool_argument_bytes = 65536,
      max_tool_result_bytes = 262144,
      max_diagnostics = 1024,
    },
    diagnostics = [
      { code = "invalid_json_line", disposition = 'lossy },
      { code = "non_object_json_line", disposition = 'lossy },
      { code = "injected_context_dropped", disposition = 'accepted_cleanup },
      { code = "noise_record_dropped", disposition = 'accepted_cleanup },
      { code = "sidechain_record_dropped", disposition = 'lossy },
      { code = "tool_call_id_synthesized", disposition = 'lossy },
      { code = "duplicate_tool_call_id", disposition = 'lossy },
      { code = "orphan_tool_result", disposition = 'lossy },
      { code = "duplicate_tool_result", disposition = 'lossy },
      { code = "unknown_tool_name", disposition = 'lossy },
      { code = "tool_arguments_reshaped", disposition = 'lossy },
      { code = "tool_arguments_truncated", disposition = 'lossy },
      { code = "tool_result_truncated", disposition = 'lossy },
      { code = "timestamps_synthesized", disposition = 'accepted_cleanup },
      { code = "timestamps_interpolated", disposition = 'accepted_cleanup },
    ],
    content = {
      reasoning = "omit",
      tool_arguments = "omit",
      tool_results = "omit",
      path_metadata = "omit",
    },
    projection = "ledger-and-batch-drafts",
    helper_protocol = "trajectory-bridge-v1",
    timeout_class = "trajectory-short",
    memory_class = "trajectory-bounded",
    receipt_schema = "animus.trajectory-transcript-receipt.v1",
    non_claims = [
      "source-authenticity", "transcript-completeness", "timestamp-truth", "tool-execution-truth",
      "semantic-equivalence", "privacy", "model-correctness", "dataset-export-fitness", "release-eligibility"
    ],
  },
}
