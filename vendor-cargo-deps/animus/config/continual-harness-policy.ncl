# Typed Nickel policy for the optional continual-harness refinement family.
let contracts = import "../policy/contracts/primitives.ncl" in
let maximum_bound = 100000000 in
let Profile = {
  schema | contracts.exact "CONTINUAL_HARNESS_SCHEMA" "animus.continual-harness.v1",
  cohort | contracts.exact "CONTINUAL_HARNESS_COHORT" "prime-agent-continual-harness",
  license | contracts.exact "CONTINUAL_HARNESS_LICENSE" "MIT",
  source_revision | contracts.safe_identifier "CONTINUAL_HARNESS_SOURCE_REVISION",
  max_content_bytes | contracts.positive_integer "CONTINUAL_HARNESS_MAX_CONTENT" maximum_bound,
  max_state_entries | contracts.positive_integer "CONTINUAL_HARNESS_MAX_ENTRIES" maximum_bound,
  max_state_bytes | contracts.positive_integer "CONTINUAL_HARNESS_MAX_STATE_BYTES" maximum_bound,
  max_snapshots | contracts.positive_integer "CONTINUAL_HARNESS_MAX_SNAPSHOTS" maximum_bound,
  max_snapshot_bytes | contracts.positive_integer "CONTINUAL_HARNESS_MAX_SNAPSHOT_BYTES" maximum_bound,
  target_classes | Array String,
  receipt_fields | Array String,
  non_claims | Array String,
  evidence_required | Bool,
  conflict_detection | Bool,
  promotion_requires_host_action | Bool,
} in
let ProfileWithRules = contracts.with_validator Profile (fun value =>
  if std.array.length value.target_classes != 4 then
    'Error { message = "CONTINUAL_HARNESS_TARGET_CLASSES" }
  else if std.array.length value.receipt_fields < 6 then
    'Error { message = "CONTINUAL_HARNESS_RECEIPT_FIELDS" }
  else if std.array.length value.non_claims < 8 then
    'Error { message = "CONTINUAL_HARNESS_NON_CLAIMS" }
  else if !value.evidence_required then
    'Error { message = "CONTINUAL_HARNESS_EVIDENCE_REQUIRED" }
  else if !value.conflict_detection then
    'Error { message = "CONTINUAL_HARNESS_CONFLICT_DETECTION" }
  else if !value.promotion_requires_host_action then
    'Error { message = "CONTINUAL_HARNESS_PROMOTION_AUTHORITY" }
  else
    'Ok
) in
{
  continual_harness_policy_contract = ProfileWithRules,
  continual_harness_policy | ProfileWithRules = {
    schema = "animus.continual-harness.v1",
    cohort = "prime-agent-continual-harness",
    license = "MIT",
    source_revision = "c5991bc853d27754aed345c13ff4d2e05c40f1f4",
    max_content_bytes = 4096,
    max_state_entries = 64,
    max_state_bytes = 65536,
    max_snapshots = 16,
    max_snapshot_bytes = 65536,
    target_classes = [
      "supplemental_prompt",
      "memory",
      "skill_description",
      "subagent_specification",
    ],
    receipt_fields = [
      "policy_identity",
      "cohort",
      "target_class",
      "operation",
      "status",
      "snapshot_identity",
      "state_identity",
      "item_count",
      "content_bytes",
    ],
    non_claims = [
      "not-lesson-correctness",
      "not-model-improvement",
      "not-generalization",
      "not-base-prompt-mutation",
      "not-automatic-promotion",
      "not-cross-session-consistency",
      "not-production-readiness",
      "not-release-eligibility",
    ],
    evidence_required = true,
    conflict_detection = true,
    promotion_requires_host_action = true,
  },
}
