# Typed cross-stack bounded-execution feasibility profile.
# r[impl agent_harness_runtime.bounded_execution_evaluation.capability_states]
# r[impl agent_harness_runtime.bounded_execution_evaluation.outcomes]
# r[impl agent_harness_runtime.bounded_execution_evaluation.non_claims]
let contracts = import "../../policy/contracts/primitives.ncl" in
let schema_id = "animus.bounded-execution-evaluation.v1" in
let consumer_ids = ["animus", "octet", "nickel_export", "mantle"] in
let states = ["requested", "configured", "enforced", "observed", "unavailable", "unsupported", "not_applicable"] in
let dispositions = ["lossless", "consumer_extension", "mechanism_fact", "retained_authority", "interoperability", "blocker"] in
let semantic_classes = ["environment", "filesystem_root", "network", "timeout", "output", "cancellation", "signal", "child_cleanup"] in
let outcome_classes = ["success", "nonzero_exit", "launch_denial", "missing_executable", "policy_rejection", "timeout", "cancellation", "signal_termination", "output_overflow", "cleanup_failure", "unsupported_mechanism"] in
let required_non_claim_labels = [
  "sandbox_equivalence",
  "hermeticity",
  "network_denial",
  "filesystem_isolation",
  "executor_correctness",
  "proof_soundness",
  "build_correctness",
  "future_determinism",
  "release_eligibility",
] in
let Consumer = {
  id | contracts.enum "BOUNDED_EXEC_CONSUMER" consumer_ids,
  revision | String,
  source_blake3 | String,
  license | String,
  owner | String,
} in
let Mapping = {
  consumer | contracts.enum "BOUNDED_EXEC_MAPPING_CONSUMER" consumer_ids,
  semantic_class | contracts.enum "BOUNDED_EXEC_SEMANTIC_CLASS" semantic_classes,
  state | contracts.enum "BOUNDED_EXEC_CAPABILITY_STATE" states,
  mechanism | String,
  disposition | contracts.enum "BOUNDED_EXEC_MAPPING_DISPOSITION" dispositions,
  observation | String,
} in
let Outcome = {
  consumer | contracts.enum "BOUNDED_EXEC_OUTCOME_CONSUMER" consumer_ids,
  outcome | contracts.enum "BOUNDED_EXEC_OUTCOME_CLASS" outcome_classes,
  disposition | contracts.enum "BOUNDED_EXEC_OUTCOME_DISPOSITION" dispositions,
  native_detail | String,
} in
let ProfileShape = {
  schema | contracts.exact "BOUNDED_EXEC_SCHEMA" schema_id,
  consumers | Array Consumer,
  mappings | Array Mapping,
  outcomes | Array Outcome,
  required_non_claims | Array String,
  minimum_independent_consumers | contracts.integer_between "BOUNDED_EXEC_MIN_CONSUMERS" 2 8,
  independently_useful_release | Bool,
  migration_strategy_ready | Bool,
  dependency_cycle | Bool,
  next_action | String,
} in
let _validator = fun profile =>
  if std.array.length profile.consumers != std.array.length consumer_ids then
    'Error { message = "BOUNDED_EXEC_CONSUMER_COVERAGE" }
  else if !contracts.unique_by (fun consumer => consumer.id) profile.consumers then
    'Error { message = "BOUNDED_EXEC_DUPLICATE_CONSUMER" }
  else if !contracts.unique_by (fun mapping => mapping.consumer ++ ":" ++ mapping.semantic_class) profile.mappings then
    'Error { message = "BOUNDED_EXEC_DUPLICATE_MAPPING" }
  else if std.array.length profile.required_non_claims != std.array.length required_non_claim_labels then
    'Error { message = "BOUNDED_EXEC_NON_CLAIM_COVERAGE" }
  else
    'Ok
in
let Profile = contracts.with_validator ProfileShape _validator in
let consumer = fun consumer_id consumer_revision consumer_source consumer_license consumer_owner => { id = consumer_id, revision = consumer_revision, source_blake3 = consumer_source, license = consumer_license, owner = consumer_owner } in
let mapping = fun mapping_consumer mapping_class mapping_state mapping_mechanism mapping_disposition mapping_observation => { consumer = mapping_consumer, semantic_class = mapping_class, state = mapping_state, mechanism = mapping_mechanism, disposition = mapping_disposition, observation = mapping_observation } in
let outcome = fun outcome_consumer outcome_class outcome_disposition outcome_detail => { consumer = outcome_consumer, outcome = outcome_class, disposition = outcome_disposition, native_detail = outcome_detail } in
let consumer_records = [
  consumer "animus" "0c1f6dce0ed11b4b848df0b4f56cac49bd8c19ab" "196c9a78574536f5012995a386f6fbef87fd95ce3ea9b5a73dfdaa90384345b3" "MIT OR Apache-2.0" "agent-harness policy and host adapters",
  consumer "octet" "d87153a1bbfe4c3469b2dee6fb5512eafa812d88" "388f36d5f7aebeef6d46a7f897b83d7a8b9393981a8712df366b8426ee770e59" "AGPL-3.0-or-later" "proof selection and proof-runner evidence",
  consumer "nickel_export" "0de12cd4400cf0df6f4f66b235d5dba9ac0b2870" "51f81fcb325517b62f644ce81577bf76d4f6cda72c75fe1bedc80612f6f38a17" "MPL-2.0 core; AGPL-3.0-or-later shell" "evaluator capture, freshness, and publication",
  consumer "mantle" "3854efbd7ed144ecafabd694a3913c5470cb7180" "46d228d83f855fe9e3f8d3ea8bf434c8a42dabd0bc52fbcaf1f65a7edf9314e8" "AGPL-3.0-or-later" "build sandbox, store, scheduler, and release policy",
] in
let mapping_records = [
  mapping "animus" "environment" "requested" "backend-neutral allowlist plan" "retained_authority" "host constructs environment",
  mapping "animus" "filesystem_root" "requested" "profile and sandbox descriptor" "retained_authority" "host resolves roots",
  mapping "animus" "network" "requested" "portable descriptor" "retained_authority" "host selects enforcement",
  mapping "animus" "timeout" "configured" "resource profile" "consumer_extension" "executor owns clock",
  mapping "animus" "output" "configured" "resource profile" "consumer_extension" "executor owns capture",
  mapping "animus" "cancellation" "requested" "host service contract" "retained_authority" "host owns cancellation",
  mapping "animus" "signal" "unsupported" "no shared signal observation" "blocker" "portable plan has no signal result",
  mapping "animus" "child_cleanup" "requested" "host cleanup contract" "retained_authority" "plan is not cleanup evidence",
  mapping "octet" "environment" "enforced" "cleared allowlisted child environment" "mechanism_fact" "proof runner shell",
  mapping "octet" "filesystem_root" "enforced" "fresh work root and read-only store" "mechanism_fact" "proof runner shell",
  mapping "octet" "network" "enforced" "denied network namespace" "mechanism_fact" "packaged bubblewrap provider",
  mapping "octet" "timeout" "observed" "kill after monotonic deadline" "lossless" "timed_out observation",
  mapping "octet" "output" "observed" "bounded stdout and stderr prefixes" "lossless" "truncation observation",
  mapping "octet" "cancellation" "unsupported" "no general cancellation result" "blocker" "proof rail exposes timeout only",
  mapping "octet" "signal" "observed" "native exit status detail" "consumer_extension" "proof shell detail",
  mapping "octet" "child_cleanup" "observed" "kill and reap diagnostics" "consumer_extension" "cleanup errors remain proof diagnostics",
  mapping "nickel_export" "environment" "enforced" "environment cleared for captured evaluator" "mechanism_fact" "snapshot-only claim",
  mapping "nickel_export" "filesystem_root" "observed" "private declared-file snapshot" "blocker" "full import closure is not confined",
  mapping "nickel_export" "network" "unsupported" "no network sandbox" "blocker" "network denial is not claimed",
  mapping "nickel_export" "timeout" "observed" "kill and reap after evaluator deadline" "lossless" "staged timeout failure",
  mapping "nickel_export" "output" "enforced" "bounded stdout and stderr readers" "lossless" "oversize suppresses receipt",
  mapping "nickel_export" "cancellation" "unsupported" "no cancellation vocabulary" "blocker" "only timeout supervision exists",
  mapping "nickel_export" "signal" "unsupported" "nonzero evaluator failure stage" "blocker" "signal cause is not canonical",
  mapping "nickel_export" "child_cleanup" "observed" "timeout kill and wait" "consumer_extension" "cleanup failure is not a canonical outcome",
  mapping "mantle" "environment" "enforced" "strict or practical build envelope" "mechanism_fact" "mode changes admission",
  mapping "mantle" "filesystem_root" "enforced" "bubblewrap inputs, outputs, and store closure" "mechanism_fact" "build sandbox semantics",
  mapping "mantle" "network" "enforced" "build request constraint and profile policy" "mechanism_fact" "fetchers are a distinct network lane",
  mapping "mantle" "timeout" "unavailable" "scheduler and builder specific" "blocker" "no reviewed common terminal timeout mapping",
  mapping "mantle" "output" "observed" "store output and build result" "consumer_extension" "not stdout-prefix semantics",
  mapping "mantle" "cancellation" "unavailable" "worker and scheduler lifecycle" "blocker" "no reviewed common cancellation mapping",
  mapping "mantle" "signal" "observed" "builder-native failure detail" "consumer_extension" "build result owns meaning",
  mapping "mantle" "child_cleanup" "configured" "sandbox and worker teardown" "consumer_extension" "store and goal cleanup remain product-owned",
] in
let outcome_records = [
  outcome "animus" "success" "consumer_extension" "plan admitted, not executed",
  outcome "octet" "success" "lossless" "proof stage exited successfully",
  outcome "nickel_export" "success" "lossless" "evaluator output admitted",
  outcome "mantle" "success" "consumer_extension" "build output persisted",
  outcome "octet" "timeout" "lossless" "timed_out observation",
  outcome "nickel_export" "timeout" "lossless" "evaluator-timeout stage",
  outcome "mantle" "timeout" "blocker" "no reviewed common terminal mapping",
  outcome "nickel_export" "cleanup_failure" "blocker" "not exposed as canonical outcome",
  outcome "octet" "output_overflow" "lossless" "bounded log truncation",
  outcome "nickel_export" "output_overflow" "lossless" "stream bound failure",
  outcome "mantle" "output_overflow" "consumer_extension" "artifact/store semantics differ",
] in
let evaluation_profile | Profile = {
  schema = schema_id,
  consumers = consumer_records,
  mappings = mapping_records,
  outcomes = outcome_records,
  required_non_claims = required_non_claim_labels,
  minimum_independent_consumers = 2,
  independently_useful_release = false,
  migration_strategy_ready = false,
  dependency_cycle = false,
  next_action = "dual-run an outcome-only observation mapping in Octet and Nickel Export without sandbox capability claims",
} in
let consumer_line = fun value => "consumer|%{value.id}|%{value.revision}|%{value.source_blake3}|%{value.license}|%{value.owner}" in
let mapping_line = fun value => "mapping|%{value.consumer}|%{value.semantic_class}|%{value.state}|%{value.mechanism}|%{value.disposition}|%{value.observation}" in
let outcome_line = fun value => "outcome|%{value.consumer}|%{value.outcome}|%{value.disposition}|%{value.native_detail}" in
{
  profile = evaluation_profile,
  projection_lines = std.string.join "\n" (
    ["schema|%{evaluation_profile.schema}"]
    @ std.array.map consumer_line evaluation_profile.consumers
    @ std.array.map mapping_line evaluation_profile.mappings
    @ std.array.map outcome_line evaluation_profile.outcomes
    @ [
      "criteria|%{evaluation_profile.minimum_independent_consumers}|%{evaluation_profile.independently_useful_release}|%{evaluation_profile.migration_strategy_ready}|%{evaluation_profile.dependency_cycle}",
      "nonclaims|%{std.string.join "," evaluation_profile.required_non_claims}",
      "next_action|%{evaluation_profile.next_action}",
    ]
  ),
}
