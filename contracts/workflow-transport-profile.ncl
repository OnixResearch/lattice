# Typed operator profile for the bounded Linux workflow exchange transport.

let MaximumFrameBytes = 33554432 in
let MaximumLogBytes = 16777216 in
let MaximumConnections = 65536 in
let MaximumDeadlineMilliseconds = 300000 in
let MaximumAllowedUsers = 64 in
let MaximumUserId = 4294967295 in

let PositiveAtMost = fun maximum => std.contract.from_predicate (fun value =>
  std.is_number value && value > 0 && value <= maximum
) in

let ExactSchema = std.contract.from_predicate (fun value =>
  value == "lattice.workflow-transport-profile.v1"
) in

let NonEmptyUsers = std.contract.from_predicate (fun value =>
  if !std.is_array value then false else
  let state = std.array.fold_left
    (fun state user_id => {
      valid = state.valid
        && std.is_number user_id
        && user_id >= 0
        && user_id <= MaximumUserId
        && (state.previous == null || state.previous < user_id),
      previous = user_id,
    })
    { valid = true, previous = null }
    value in
  std.array.length value > 0
  && std.array.length value <= MaximumAllowedUsers
  && state.valid
) in

let RequiredNonClaims = std.contract.from_predicate (fun value =>
  value == [
    "transport admission does not prove workflow correctness",
    "transport peer identity does not prove peer intent",
    "transport acceptance does not prove an operation executed",
    "transport response does not prove an external effect succeeded",
    "transport checks do not establish production deployment",
  ]
) in

let TransportLimits = {
  maximum_frame_bytes | PositiveAtMost MaximumFrameBytes,
  maximum_log_bytes | PositiveAtMost MaximumLogBytes,
  maximum_connections | PositiveAtMost MaximumConnections,
  read_deadline_milliseconds | PositiveAtMost MaximumDeadlineMilliseconds,
  write_deadline_milliseconds | PositiveAtMost MaximumDeadlineMilliseconds,
} in

let PeerPolicy = {
  allowed_user_ids | NonEmptyUsers,
} in

{
  TransportProfile = {
    schema | ExactSchema,
    limits | TransportLimits,
    peers | PeerPolicy,
    non_claims | RequiredNonClaims,
  },
}
