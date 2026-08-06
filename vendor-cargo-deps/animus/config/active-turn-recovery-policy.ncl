# Typed active-turn recovery policy source of truth.
# r[impl agent_harness_runtime.active_turn_recovery.policy]
let contracts = import "../policy/contracts/primitives.ncl" in

let schema_identity = "animus.active-turn-recovery.v1" in
let policy_id_value = "durable-active-turn" in
let maximum_entry_limit = 4096 in
let maximum_entry_bytes = 1048576 in
let maximum_aggregate_bytes = 67108864 in
let maximum_steering_limit = 256 in
let required_receipt_fields = [
  "policy_hash",
  "turn_hash",
  "checkpoint_hash",
  "plan_hash",
  "generation",
  "action",
  "terminal",
] in
let required_non_claims = [
  "store_correctness",
  "power_loss_durability",
  "provider_acceptance",
  "exactly_once_effects",
  "task_completion",
  "production_readiness",
] in
let PolicyShape = {
  schema | contracts.exact "ACTIVE_TURN_SCHEMA" schema_identity,
  policy_id | contracts.safe_identifier "ACTIVE_TURN_POLICY_ID",
  max_entries | contracts.integer_between "ACTIVE_TURN_ENTRY_BOUND" 1 maximum_entry_limit,
  max_entry_bytes | contracts.integer_between "ACTIVE_TURN_ENTRY_BYTE_BOUND" 1 maximum_entry_bytes,
  max_aggregate_bytes | contracts.integer_between "ACTIVE_TURN_AGGREGATE_BOUND" 1 maximum_aggregate_bytes,
  max_steering_entries | contracts.integer_between "ACTIVE_TURN_STEERING_BOUND" 1 maximum_steering_limit,
  corruption_posture | contracts.enum "ACTIVE_TURN_CORRUPTION_POSTURE" ["fail_closed"],
  replacement_posture | contracts.enum "ACTIVE_TURN_REPLACEMENT_POSTURE" ["compare_and_swap_atomic"],
  transaction_evidence_required | contracts.exact "ACTIVE_TURN_TRANSACTION_EVIDENCE" true,
  retention_class | contracts.enum "ACTIVE_TURN_RETENTION_CLASS" ["host_managed"],
  receipt_fields | Array String,
  non_claims | Array String,
  policy_hash | contracts.lowercase_blake3 "ACTIVE_TURN_POLICY_HASH",
} in
let _whole_policy_validator = fun value =>
  if value.max_entry_bytes > value.max_aggregate_bytes then
    'Error { message = "ACTIVE_TURN_AGGREGATE_BOUND" }
  else if value.receipt_fields != required_receipt_fields then
    'Error { message = "ACTIVE_TURN_RECEIPT_FIELDS" }
  else if value.non_claims != required_non_claims then
    'Error { message = "ACTIVE_TURN_NON_CLAIMS" }
  else
    'Ok
in
let ActiveTurnRecoveryPolicy = contracts.with_validator PolicyShape _whole_policy_validator in

{
  active_turn_recovery_policy_contract = ActiveTurnRecoveryPolicy,
  active_turn_recovery_policy | ActiveTurnRecoveryPolicy = {
    schema = schema_identity,
    policy_id = policy_id_value,
    max_entries = 1024,
    max_entry_bytes = 65536,
    max_aggregate_bytes = 8388608,
    max_steering_entries = 64,
    corruption_posture = "fail_closed",
    replacement_posture = "compare_and_swap_atomic",
    transaction_evidence_required = true,
    retention_class = "host_managed",
    receipt_fields = required_receipt_fields,
    non_claims = required_non_claims,
    policy_hash = "4c39f74256c4cd5ec5a095095ff64c1d71864a8cfc650bd89a66674867e2205d",
  },
}
