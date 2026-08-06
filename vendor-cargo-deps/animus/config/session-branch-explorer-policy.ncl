# Redaction-first session branch explorer policy.
# r[impl agent_harness_runtime.session_branch_explorer.projection]
let contracts = import "../policy/contracts/primitives.ncl" in
let schema_identity = "animus.session-branch-explorer.policy.v1" in
let maximum_nodes = 16384 in
let maximum_depth = 1024 in
let maximum_label_bytes = 256 in
let maximum_labels_per_node = 32 in
let maximum_ordered_children = 4096 in
let maximum_comparison_suffix = 4096 in
let maximum_safe_references = 256 in
let required_non_claims = [
  "transcript_truth",
  "effect_rollback",
  "world_isolation",
  "semantic_quality",
  "ui_delivery",
  "accessibility",
  "production_readiness",
  "release_eligibility",
] in
let Shape = {
  schema | contracts.exact "BRANCH_EXPLORER_SCHEMA" schema_identity,
  max_nodes | contracts.integer_between "BRANCH_EXPLORER_NODE_BOUND" 1 maximum_nodes,
  max_depth | contracts.integer_between "BRANCH_EXPLORER_DEPTH_BOUND" 1 maximum_depth,
  max_label_bytes | contracts.integer_between "BRANCH_EXPLORER_LABEL_BOUND" 1 maximum_label_bytes,
  max_labels_per_node | contracts.integer_between "BRANCH_EXPLORER_LABEL_COUNT" 1 maximum_labels_per_node,
  max_ordered_children | contracts.integer_between "BRANCH_EXPLORER_ORDER_BOUND" 1 maximum_ordered_children,
  max_comparison_suffix | contracts.integer_between "BRANCH_EXPLORER_COMPARISON_BOUND" 1 maximum_comparison_suffix,
  max_safe_references | contracts.integer_between "BRANCH_EXPLORER_REFERENCE_BOUND" 1 maximum_safe_references,
  redaction_first | Bool,
  one_replay_parent | Bool,
  annotations_are_sidecars | Bool,
  external_state_required | Bool,
  destructive_actions_allowed | Bool,
  receipt_metadata_only | Bool,
  non_claims | Array String,
} in
let validator = fun value =>
  if !value.redaction_first then 'Error { message = "BRANCH_EXPLORER_REDACTION" }
  else if !value.one_replay_parent then 'Error { message = "BRANCH_EXPLORER_PARENT" }
  else if !value.annotations_are_sidecars then 'Error { message = "BRANCH_EXPLORER_ANNOTATIONS" }
  else if !value.external_state_required then 'Error { message = "BRANCH_EXPLORER_EXTERNAL_STATE" }
  else if value.destructive_actions_allowed then 'Error { message = "BRANCH_EXPLORER_DESTRUCTIVE" }
  else if !value.receipt_metadata_only then 'Error { message = "BRANCH_EXPLORER_RECEIPT" }
  else if value.non_claims != required_non_claims then 'Error { message = "BRANCH_EXPLORER_NON_CLAIMS" }
  else 'Ok
in
let Policy = contracts.with_validator Shape validator in
{
  session_branch_explorer_policy_contract = Policy,
  session_branch_explorer_policy | Policy = {
    schema = schema_identity,
    max_nodes = 2048,
    max_depth = 128,
    max_label_bytes = 96,
    max_labels_per_node = 8,
    max_ordered_children = 128,
    max_comparison_suffix = 256,
    max_safe_references = 64,
    redaction_first = true,
    one_replay_parent = true,
    annotations_are_sidecars = true,
    external_state_required = true,
    destructive_actions_allowed = false,
    receipt_metadata_only = true,
    non_claims = required_non_claims,
  },
}
