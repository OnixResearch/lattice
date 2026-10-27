# Typed policy for the optional Prime Agent schedule reliability cohort.
let contracts = import "../policy/contracts/primitives.ncl" in
let minimum_interval_ticks_value = 1 in
let maximum_interval_ticks_value = 86400 in
let maximum_jobs_per_session_value = 64 in
let maximum_missed_count_value = 1000000 in
let Profile = {
  schema | contracts.exact "SCHEDULE_SCHEMA" "animus.schedule-reliability.v1",
  cohort | contracts.exact "SCHEDULE_COHORT" "prime-agent-schedule-reliability",
  license | contracts.exact "SCHEDULE_LICENSE" "MIT",
  source_revision | contracts.safe_identifier "SCHEDULE_SOURCE_REVISION",
  minimum_interval_ticks | contracts.positive_integer "SCHEDULE_MIN_INTERVAL" minimum_interval_ticks_value,
  maximum_interval_ticks | contracts.positive_integer "SCHEDULE_MAX_INTERVAL" maximum_interval_ticks_value,
  maximum_jobs_per_session | contracts.positive_integer "SCHEDULE_MAX_JOBS" maximum_jobs_per_session_value,
  maximum_missed_count | contracts.positive_integer "SCHEDULE_MAX_MISSED" maximum_missed_count_value,
  surfaces | Array String,
  catch_up_rules | { _ | String },
  receipt_fields | Array String,
  non_claims | Array String,
  claim_before_delivery | Bool,
  one_user_heartbeat | Bool,
  session_scoped | Bool,
} in
let ProfileWithRules = contracts.with_validator Profile (fun value =>
  if std.array.length value.surfaces != 3 then
    'Error { message = "SCHEDULE_SURFACE_COUNT" }
  else if value.minimum_interval_ticks > value.maximum_interval_ticks then
    'Error { message = "SCHEDULE_INTERVAL_ORDER" }
  else if std.array.length value.receipt_fields != 8 then
    'Error { message = "SCHEDULE_RECEIPT_FIELDS" }
  else if std.array.length value.non_claims != 5 then
    'Error { message = "SCHEDULE_NON_CLAIMS" }
  else if !std.array.elem "not-exactly-once-execution" value.non_claims then
    'Error { message = "SCHEDULE_EXACTLY_ONCE_NON_CLAIM" }
  else if !std.array.elem "not-production-or-release-readiness" value.non_claims then
    'Error { message = "SCHEDULE_RELEASE_NON_CLAIM" }
  else if !value.claim_before_delivery || !value.one_user_heartbeat || !value.session_scoped then
    'Error { message = "SCHEDULE_BOUNDARY_RULES" }
  else
    'Ok
) in
{
  schedule_reliability_policy_contract = ProfileWithRules,
  schedule_reliability_policy | ProfileWithRules = {
    schema = "animus.schedule-reliability.v1",
    cohort = "prime-agent-schedule-reliability",
    license = "MIT",
    source_revision = "c5991bc853d27754aed345c13ff4d2e05c40f1f4",
    minimum_interval_ticks = minimum_interval_ticks_value,
    maximum_interval_ticks = maximum_interval_ticks_value,
    maximum_jobs_per_session = maximum_jobs_per_session_value,
    maximum_missed_count = maximum_missed_count_value,
    surfaces = ["user_heartbeat", "agent_recurring", "scheduled"],
    catch_up_rules = {
      user_heartbeat = "fire_once",
      agent_recurring = "skip",
      scheduled = "pause",
    },
    receipt_fields = [
      "policy_identity",
      "state_identity",
      "job_identity",
      "surface",
      "decision_class",
      "clock_identity",
      "missed_count",
      "non_claims",
    ],
    non_claims = [
      "not-exactly-once-execution",
      "not-real-time",
      "not-timer-accuracy",
      "not-outcome-correctness",
      "not-production-or-release-readiness",
    ],
    claim_before_delivery = true,
    one_user_heartbeat = true,
    session_scoped = true,
  },
}
