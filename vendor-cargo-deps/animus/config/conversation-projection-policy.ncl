# Canonical conversation projection policy.
# r[impl agent_harness_runtime.conversation_view_projection.boundary]
let contracts = import "../policy/contracts/primitives.ncl" in
let schema_identity = "animus.conversation-projection.v1" in
let maximum_entries = 4096 in
let maximum_parts = 256 in
let required_receipt_fields = ["source", "stream", "cursor", "counts", "non_claims"] in
let required_non_claims = ["transcript_truth", "ui_delivery", "cache_durability", "renderer_correctness", "task_completion", "production_readiness"] in
let Shape = {
  schema | contracts.exact "CONVERSATION_PROJECTION_SCHEMA" schema_identity,
  policy_id | contracts.safe_identifier "CONVERSATION_PROJECTION_POLICY_ID",
  max_entries | contracts.integer_between "CONVERSATION_PROJECTION_ENTRY_BOUND" 1 maximum_entries,
  max_parts_per_item | contracts.integer_between "CONVERSATION_PROJECTION_PART_BOUND" 1 maximum_parts,
  reasoning_posture | contracts.enum "CONVERSATION_PROJECTION_REASONING" ["hidden"],
  tool_posture | contracts.enum "CONVERSATION_PROJECTION_TOOL" ["hash_only"],
  cursor_posture | contracts.enum "CONVERSATION_PROJECTION_CURSOR" ["revision_fenced"],
  cache_posture | contracts.enum "CONVERSATION_PROJECTION_CACHE" ["disposable"],
  receipt_fields | Array String,
  non_claims | Array String,
  policy_hash | contracts.lowercase_blake3 "CONVERSATION_PROJECTION_POLICY_HASH",
} in
let validator = fun value =>
  if value.receipt_fields != required_receipt_fields then 'Error { message = "CONVERSATION_PROJECTION_RECEIPTS" }
  else if value.non_claims != required_non_claims then 'Error { message = "CONVERSATION_PROJECTION_NON_CLAIMS" }
  else 'Ok
in
let Policy = contracts.with_validator Shape validator in
{
  conversation_projection_policy_contract = Policy,
  conversation_projection_policy | Policy = {
    schema = schema_identity,
    policy_id = "canonical-conversation",
    max_entries = 1024,
    max_parts_per_item = 64,
    reasoning_posture = "hidden",
    tool_posture = "hash_only",
    cursor_posture = "revision_fenced",
    cache_posture = "disposable",
    receipt_fields = required_receipt_fields,
    non_claims = required_non_claims,
    policy_hash = "ad8f2abe166c397804648b2358f32139131f19921054443e2356e6793ac58795",
  },
}
