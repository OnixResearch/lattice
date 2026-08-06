# Typed policy and parity source for the optional Lean 4 loop fabric.
# r[impl agent_harness_runtime.lean4_loop.profile]
let contracts = import "../policy/contracts/primitives.ncl" in

let protocol_identity = "animus.lean-loop.protocol.v1" in
let envelope_schema_identity = "animus.lean-loop.envelope.v1" in
let toolchain_cohort = "lean4-4.30.0" in
let package_identity = "animus-lean-0.1.0" in
let source_module_name = "AnimusLean.Core" in
let reviewed_proof_source_identity = "d8e0ffd0ae32507bb7051c5b77e162f76484debde12a32d09f40616fed7c0c54" in
let maximum_bound_value = 1000000000 in
let allowed_decision_classes = ["model", "tool", "respond", "stop"] in
let redaction_marker =
  "raw-turn-model-tool-provider-source-path-environment-credential-stderr-and-hidden-material-omitted"
in
let Profile = {
  schema | contracts.exact "LEAN_LOOP_PROFILE_SCHEMA" "animus.lean-loop.profile.v1",
  protocol = {
    identity | contracts.exact "LEAN_LOOP_PROTOCOL" protocol_identity,
    envelope_schema | contracts.exact "LEAN_LOOP_SCHEMA" envelope_schema_identity,
  },
  toolchain = {
    cohort | contracts.exact "LEAN_LOOP_TOOLCHAIN" toolchain_cohort,
    package | contracts.exact "LEAN_LOOP_PACKAGE" package_identity,
    source_module | contracts.exact "LEAN_LOOP_SOURCE_MODULE" source_module_name,
  },
  allowed_decisions | Array (contracts.enum "LEAN_LOOP_DECISION" allowed_decision_classes),
  bounds = {
    frame_bytes | contracts.positive_integer "LEAN_LOOP_FRAME_BYTES" maximum_bound_value,
    text_bytes | contracts.positive_integer "LEAN_LOOP_TEXT_BYTES" maximum_bound_value,
    sequence_limit | contracts.positive_integer "LEAN_LOOP_SEQUENCE_LIMIT" maximum_bound_value,
    diagnostic_bytes | contracts.positive_integer "LEAN_LOOP_DIAGNOSTIC_BYTES" maximum_bound_value,
    startup_poll_count | contracts.positive_integer "LEAN_LOOP_STARTUP_POLLS" maximum_bound_value,
    response_poll_count | contracts.positive_integer "LEAN_LOOP_RESPONSE_POLLS" maximum_bound_value,
    shutdown_poll_count | contracts.positive_integer "LEAN_LOOP_SHUTDOWN_POLLS" maximum_bound_value,
    poll_interval_millis | contracts.positive_integer "LEAN_LOOP_POLL_MILLIS" maximum_bound_value,
    cleanup_poll_count | contracts.positive_integer "LEAN_LOOP_CLEANUP_POLLS" maximum_bound_value,
    argument_count | contracts.positive_integer "LEAN_LOOP_ARGUMENT_COUNT" maximum_bound_value,
    target_count | contracts.positive_integer "LEAN_LOOP_TARGET_COUNT" maximum_bound_value,
    capability_count | contracts.positive_integer "LEAN_LOOP_CAPABILITY_COUNT" maximum_bound_value,
  },
  launch = {
    executable_selection | contracts.exact "LEAN_LOOP_EXECUTABLE_SELECTION" "explicit-absolute-path",
    path_discovery | Bool,
    inherited_environment | Bool,
    environment_allowlist | Array String,
    working_directory | contracts.exact "LEAN_LOOP_WORKING_DIRECTORY" "explicit-absolute-path",
    sandbox_posture | contracts.enum "LEAN_LOOP_SANDBOX" ["required", "fixture-only"],
    fallback_policy | contracts.exact "LEAN_LOOP_FALLBACK" "deny",
    restart_policy | contracts.exact "LEAN_LOOP_RESTART" "deny-after-activation",
  },
  identity = {
    algorithm | contracts.exact "LEAN_LOOP_HASH_ALGORITHM" "blake3",
    profile_domain | contracts.exact "LEAN_LOOP_PROFILE_DOMAIN" "animus.lean-loop.profile.v1",
    artifact_domain | contracts.exact "LEAN_LOOP_ARTIFACT_DOMAIN" "animus.lean-loop.artifact.v1",
    proof_source_domain | contracts.exact "LEAN_LOOP_PROOF_SOURCE_DOMAIN" "animus.lean-loop.proof-source.v1",
    proof_source_identity | contracts.exact "LEAN_LOOP_PROOF_SOURCE_IDENTITY" reviewed_proof_source_identity,
    receipt_domain | contracts.exact "LEAN_LOOP_RECEIPT_DOMAIN" "animus.lean-loop.receipt.v1",
  },
  redaction = {
    marker | contracts.exact "LEAN_LOOP_REDACTION" redaction_marker,
    raw_payloads | Bool,
    authoritative_completion_claim | Bool,
  },
  non_claims | Array String,
} in
let validate = fun value =>
  if std.array.length value.allowed_decisions != std.array.length allowed_decision_classes then
    'Error { message = "LEAN_LOOP_DECISION_SET" }
  else if value.bounds.text_bytes > value.bounds.frame_bytes then
    'Error { message = "LEAN_LOOP_BOUND_ORDER" }
  else if value.launch.path_discovery || value.launch.inherited_environment then
    'Error { message = "LEAN_LOOP_AMBIENT_AUTHORITY" }
  else if std.array.length value.launch.environment_allowlist != 0 then
    'Error { message = "LEAN_LOOP_ENVIRONMENT_ALLOWLIST" }
  else if value.redaction.raw_payloads || value.redaction.authoritative_completion_claim then
    'Error { message = "LEAN_LOOP_REDACTION_WEAKENED" }
  else if std.array.length value.non_claims < 6 then
    'Error { message = "LEAN_LOOP_NON_CLAIMS_WEAKENED" }
  else
    'Ok
in
let LeanLoopProfile = contracts.with_validator Profile validate in
{
  lean_loop_profile_contract = LeanLoopProfile,
  lean_loop_profile | LeanLoopProfile = {
    schema = "animus.lean-loop.profile.v1",
    protocol = { identity = protocol_identity, envelope_schema = envelope_schema_identity },
    toolchain = {
      cohort = toolchain_cohort,
      package = package_identity,
      source_module = source_module_name,
    },
    allowed_decisions = allowed_decision_classes,
    bounds = {
      frame_bytes = 65536,
      text_bytes = 8192,
      sequence_limit = 4096,
      diagnostic_bytes = 4096,
      startup_poll_count = 100,
      response_poll_count = 500,
      shutdown_poll_count = 100,
      poll_interval_millis = 10,
      cleanup_poll_count = 100,
      argument_count = 32,
      target_count = 32,
      capability_count = 32,
    },
    launch = {
      executable_selection = "explicit-absolute-path",
      path_discovery = false,
      inherited_environment = false,
      environment_allowlist = [],
      working_directory = "explicit-absolute-path",
      sandbox_posture = "fixture-only",
      fallback_policy = "deny",
      restart_policy = "deny-after-activation",
    },
    identity = {
      algorithm = "blake3",
      profile_domain = "animus.lean-loop.profile.v1",
      artifact_domain = "animus.lean-loop.artifact.v1",
      proof_source_domain = "animus.lean-loop.proof-source.v1",
      proof_source_identity = reviewed_proof_source_identity,
      receipt_domain = "animus.lean-loop.receipt.v1",
    },
    redaction = {
      marker = redaction_marker,
      raw_payloads = false,
      authoritative_completion_claim = false,
    },
    non_claims = [
      "not-arbitrary-policy-correctness",
      "not-lean-compiler-or-runtime-soundness",
      "not-child-process-confinement",
      "not-atomic-executable-path-pinning",
      "not-provider-or-tool-correctness",
      "not-theorem-to-rust-equivalence",
      "not-authoritative-completion",
      "not-whole-agent-correctness",
      "not-production-readiness",
    ],
  },
}
