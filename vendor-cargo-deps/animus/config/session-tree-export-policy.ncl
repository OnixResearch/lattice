# Typed redacted session-tree export policy source of truth.
# r[impl agent_harness_runtime.session_tree_export.policy]
let contracts = import "../policy/contracts/primitives.ncl" in

let schema_identity = "animus.session-tree-export-policy.v1" in
let policy_id_value = "redacted-session-tree" in
let max_entry_limit = 4096 in
let max_depth_limit = 128 in
let max_summary_limit_chars = 512 in
let max_artifact_limit_bytes = 8388608 in
let safe_entry_kinds = [
  "user_omitted",
  "assistant_omitted",
  "tool_status",
  "compaction",
  "repair",
  "branch_summary",
  "system_status",
] in
let safe_formats = ["html", "safe_lines"] in
let safe_receipt_fields = [
  "source_identity",
  "projection_identity",
  "artifact_identity",
  "counts",
  "format",
  "byte_count",
  "redaction",
  "write_status",
  "non_claim",
] in
let PolicyShape = {
  schema | contracts.exact "SESSION_TREE_EXPORT_SCHEMA" schema_identity,
  policy_id | contracts.safe_identifier "SESSION_TREE_EXPORT_POLICY_ID",
  max_entries | contracts.integer_between "SESSION_TREE_EXPORT_ENTRY_BOUND" 1 max_entry_limit,
  max_depth | contracts.integer_between "SESSION_TREE_EXPORT_DEPTH_BOUND" 1 max_depth_limit,
  max_summary_chars | contracts.integer_between "SESSION_TREE_EXPORT_SUMMARY_BOUND" 1 max_summary_limit_chars,
  max_html_bytes | contracts.integer_between "SESSION_TREE_EXPORT_HTML_BOUND" 1 max_artifact_limit_bytes,
  max_safe_lines_bytes | contracts.integer_between "SESSION_TREE_EXPORT_LINES_BOUND" 1 max_artifact_limit_bytes,
  allowed_entry_kinds | Array (contracts.enum "SESSION_TREE_EXPORT_ENTRY_KIND" safe_entry_kinds),
  formats | Array (contracts.enum "SESSION_TREE_EXPORT_FORMAT" safe_formats),
  branch_visibility | contracts.enum "SESSION_TREE_EXPORT_BRANCH_VISIBILITY" ["all_redacted"],
  tool_metadata_posture | contracts.enum "SESSION_TREE_EXPORT_TOOL_METADATA" ["status_only"],
  custom_entry_posture | contracts.enum "SESSION_TREE_EXPORT_CUSTOM_ENTRY" ["reject"],
  receipt_fields | Array (contracts.enum "SESSION_TREE_EXPORT_RECEIPT_FIELD" safe_receipt_fields),
  content_posture | contracts.enum "SESSION_TREE_EXPORT_CONTENT_POSTURE" ["metadata_only"],
  path_posture | contracts.enum "SESSION_TREE_EXPORT_PATH_POSTURE" ["omit"],
  malformed_input | contracts.enum "SESSION_TREE_EXPORT_MALFORMED_INPUT" ["reject"],
  scripts | contracts.enum "SESSION_TREE_EXPORT_SCRIPT_POSTURE" ["forbidden"],
  remote_resources | contracts.enum "SESSION_TREE_EXPORT_REMOTE_RESOURCE_POSTURE" ["forbidden"],
  policy_hash | contracts.lowercase_blake3 "SESSION_TREE_EXPORT_POLICY_HASH",
} in
let _whole_policy_validator = fun value =>
  if std.array.length value.allowed_entry_kinds == 0 then
    'Error { message = "SESSION_TREE_EXPORT_EMPTY_ENTRY_KINDS" }
  else if !contracts.unique_by (fun kind => kind) value.allowed_entry_kinds then
    'Error { message = "SESSION_TREE_EXPORT_DUPLICATE_ENTRY_KIND" }
  else if std.array.length value.formats == 0 then
    'Error { message = "SESSION_TREE_EXPORT_EMPTY_FORMATS" }
  else if !contracts.unique_by (fun format => format) value.formats then
    'Error { message = "SESSION_TREE_EXPORT_DUPLICATE_FORMAT" }
  else if value.receipt_fields != safe_receipt_fields then
    'Error { message = "SESSION_TREE_EXPORT_RECEIPT_FIELDS" }
  else
    'Ok
in
let SessionTreeExportPolicy = contracts.with_validator PolicyShape _whole_policy_validator in

{
  session_tree_export_policy_contract = SessionTreeExportPolicy,
  session_tree_export_policy | SessionTreeExportPolicy = {
    schema = schema_identity,
    policy_id = policy_id_value,
    max_entries = 256,
    max_depth = 32,
    max_summary_chars = 160,
    max_html_bytes = 1048576,
    max_safe_lines_bytes = 1048576,
    allowed_entry_kinds = safe_entry_kinds,
    formats = safe_formats,
    branch_visibility = "all_redacted",
    tool_metadata_posture = "status_only",
    custom_entry_posture = "reject",
    receipt_fields = safe_receipt_fields,
    content_posture = "metadata_only",
    path_posture = "omit",
    malformed_input = "reject",
    scripts = "forbidden",
    remote_resources = "forbidden",
    policy_hash = "828fed5de19fe77a8e7a7050778113872d686343cd2a997070cf5e30930284db",
  },
}
