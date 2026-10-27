# Typed rejected-attempt context policy.
# r[impl agent_harness_runtime.rejected_attempt.policy]
let contracts = import "../policy/contracts/primitives.ncl" in

let schema_identity = "animus.rejected-attempt-policy.v1" in
let maximum_summary_bytes = 16384 in
let maximum_diff_bytes = 1048576 in
let accepted_profiles = ["cairn_revision_handoff", "policy_declared_equivalent"] in
let accepted_providers = ["jujutsu", "git", "policy_declared"] in
let required_non_claims = [
  "does-not-grant-lifecycle-authority",
  "does-not-authorize-retry",
  "does-not-create-a-run",
  "does-not-mutate-vcs",
  "does-not-prove-later-correction",
] in
let safe_receipt_fields = [
  "receipt_id",
  "task_id",
  "run_id",
  "attempt_id",
  "revision_id",
  "handoff_id",
  "summary_id",
  "raw_diff_disposition",
  "claims",
  "non_claims",
] in
let PolicyShape = {
  schema | contracts.exact "REJECTED_ATTEMPT_SCHEMA" schema_identity,
  policy_id | contracts.safe_identifier "REJECTED_ATTEMPT_POLICY_ID",
  accepted_review_profiles | Array (contracts.enum "REJECTED_ATTEMPT_REVIEW_PROFILE" accepted_profiles),
  accepted_vcs_providers | Array (contracts.enum "REJECTED_ATTEMPT_VCS_PROVIDER" accepted_providers),
  freshness | contracts.enum "REJECTED_ATTEMPT_FRESHNESS" ["exact_current_identity"],
  redaction | contracts.enum "REJECTED_ATTEMPT_REDACTION" ["required_before_projection"],
  max_summary_bytes | contracts.integer_between "REJECTED_ATTEMPT_SUMMARY_BOUND" 1 maximum_summary_bytes,
  raw_diff | contracts.enum "REJECTED_ATTEMPT_RAW_DIFF" ["deny_by_default"],
  max_raw_diff_bytes | contracts.integer_between "REJECTED_ATTEMPT_DIFF_BOUND" 1 maximum_diff_bytes,
  receipt_fields | Array (contracts.enum "REJECTED_ATTEMPT_RECEIPT_FIELD" safe_receipt_fields),
  non_claims | Array (contracts.enum "REJECTED_ATTEMPT_NON_CLAIM" required_non_claims),
} in
let _whole_policy_validator = fun value =>
  if std.array.length value.accepted_review_profiles == 0 then
    'Error { message = "REJECTED_ATTEMPT_EMPTY_REVIEW_PROFILES" }
  else if !contracts.unique_by (fun profile => profile) value.accepted_review_profiles then
    'Error { message = "REJECTED_ATTEMPT_DUPLICATE_REVIEW_PROFILE" }
  else if std.array.length value.accepted_vcs_providers == 0 then
    'Error { message = "REJECTED_ATTEMPT_EMPTY_VCS_PROVIDERS" }
  else if !contracts.unique_by (fun provider => provider) value.accepted_vcs_providers then
    'Error { message = "REJECTED_ATTEMPT_DUPLICATE_VCS_PROVIDER" }
  else if value.receipt_fields != safe_receipt_fields then
    'Error { message = "REJECTED_ATTEMPT_UNSAFE_RECEIPT_FIELDS" }
  else if value.non_claims != required_non_claims then
    'Error { message = "REJECTED_ATTEMPT_WEAKENED_NON_CLAIMS" }
  else
    'Ok
in
let RejectedAttemptPolicy = contracts.with_validator PolicyShape _whole_policy_validator in

{
  rejected_attempt_policy_contract = RejectedAttemptPolicy,
  rejected_attempt_policy | RejectedAttemptPolicy = {
    schema = schema_identity,
    policy_id = "rejected-attempt-default",
    accepted_review_profiles = accepted_profiles,
    accepted_vcs_providers = accepted_providers,
    freshness = "exact_current_identity",
    redaction = "required_before_projection",
    max_summary_bytes = 4096,
    raw_diff = "deny_by_default",
    max_raw_diff_bytes = 65536,
    receipt_fields = safe_receipt_fields,
    non_claims = required_non_claims,
  },
}
