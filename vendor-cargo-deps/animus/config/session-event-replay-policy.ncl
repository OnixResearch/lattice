# Typed policy for transport-neutral session event replay.
let contracts = import "../policy/contracts/primitives.ncl" in
let maximum_event_count_value = 256 in
let maximum_projected_bytes_value = 1048576 in
let Policy = {
  schema | contracts.exact "REPLAY_SCHEMA" "animus.session-event-replay.v1",
  policy_id | contracts.safe_identifier "REPLAY_POLICY_ID",
  maximum_event_count | contracts.positive_integer "REPLAY_MAX_EVENTS" maximum_event_count_value,
  maximum_projected_bytes | contracts.positive_integer "REPLAY_MAX_BYTES" maximum_projected_bytes_value,
  replayable_classes | Array String,
  omitted_classes | Array String,
  terminal_outcomes | Array String,
  redaction_profile_id | String,
  allow_bounded_rebuild | Bool,
  non_claims | Array String,
} in
let CheckedPolicy = contracts.with_validator Policy (fun value =>
  if std.array.length value.replayable_classes != 2 then
    'Error { message = "REPLAY_CLASS_COUNT" }
  else if std.array.length value.omitted_classes != 1 then
    'Error { message = "REPLAY_OMITTED_COUNT" }
  else if std.array.length value.terminal_outcomes != 3 then
    'Error { message = "REPLAY_TERMINAL_COUNT" }
  else if std.array.length value.non_claims != 4 then
    'Error { message = "REPLAY_NON_CLAIMS" }
  else if std.array.elem "live_only_delta" value.replayable_classes then
    'Error { message = "REPLAY_LIVE_DELTA" }
  else
    'Ok
) in
{
  session_event_replay_policy_contract = CheckedPolicy,
  session_event_replay_policy | CheckedPolicy = {
    schema = "animus.session-event-replay.v1",
    policy_id = "animus.session-event-replay.default.v1",
    maximum_event_count = maximum_event_count_value,
    maximum_projected_bytes = maximum_projected_bytes_value,
    replayable_classes = ["durable_semantic", "metadata_observation"],
    omitted_classes = ["live_only_delta"],
    terminal_outcomes = ["completed", "cancelled", "failed"],
    redaction_profile_id = "metadata-only-v1",
    allow_bounded_rebuild = true,
    non_claims = [
      "not-transcript-truth",
      "not-transport-delivery",
      "not-durable-storage",
      "not-conversation-continuity",
    ],
  },
}
