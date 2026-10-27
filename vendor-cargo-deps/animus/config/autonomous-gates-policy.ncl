# Typed policy for the optional autonomous-gates contract family.
# r[impl agent_harness_runtime.autonomous_gates.profile]
let contracts = import "../policy/contracts/primitives.ncl" in
let maximum_bound = 100000000 in
let Profile = {
  schema | contracts.exact "AUTONOMOUS_GATES_SCHEMA" "animus.autonomous-gates.v1",
  policy_id | contracts.safe_identifier "AUTONOMOUS_GATES_POLICY_ID",
  limits = {
    max_continuations | contracts.positive_integer "AUTONOMOUS_GATES_MAX_CONTINUATIONS" maximum_bound,
    max_turns | contracts.positive_integer "AUTONOMOUS_GATES_MAX_TURNS" maximum_bound,
    max_tokens | contracts.positive_integer "AUTONOMOUS_GATES_MAX_TOKENS" maximum_bound,
    max_wall_clock_milliseconds | contracts.positive_integer "AUTONOMOUS_GATES_MAX_WALL_CLOCK" maximum_bound,
  },
  max_gate_output_bytes | contracts.positive_integer "AUTONOMOUS_GATES_MAX_OUTPUT" maximum_bound,
  gate_classes | Array String,
  receipt_fields | Array String,
  non_claims | Array String,
  policy_hash | String,
} in
let ProfileWithRules = contracts.with_validator Profile (fun value =>
  if std.array.length value.gate_classes == 0 then
    'Error { message = "AUTONOMOUS_GATES_GATE_CLASSES" }
  else if std.array.length value.receipt_fields == 0 then
    'Error { message = "AUTONOMOUS_GATES_RECEIPT_FIELDS" }
  else if std.array.length value.non_claims < 8 then
    'Error { message = "AUTONOMOUS_GATES_NON_CLAIMS" }
  else if std.string.length value.policy_hash != 64 then
    'Error { message = "AUTONOMOUS_GATES_POLICY_HASH" }
  else
    'Ok
) in
{
  autonomous_gates_policy_contract = ProfileWithRules,
  autonomous_gates_policy | ProfileWithRules = {
    schema = "animus.autonomous-gates.v1",
    policy_id = "autonomous-default",
    limits = {
      max_continuations = 8,
      max_turns = 32,
      max_tokens = 100000,
      max_wall_clock_milliseconds = 3600000,
    },
    max_gate_output_bytes = 4096,
    gate_classes = ["check", "test", "lint"],
    receipt_fields = [
      "policy_identity",
      "gate_identity",
      "verified_scope",
      "outcome_class",
      "fingerprint_identity",
      "limit_class",
    ],
    non_claims = [
      "not-task-success",
      "not-objective-completion",
      "not-verification-outside-gate-scope",
      "not-change-detection-outside-admitted-file-set",
      "not-gate-sandboxing",
      "not-production-readiness",
      "not-release-eligibility",
      "not-authority-widening",
    ],
    policy_hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  },
}
