# Typed local-operator authority only: no UCAN verification or execution receipt.
# Runtime admission additionally checks the observed clock and UTF-8 byte bounds.
let MaximumGrants = 64 in
let MaximumRoots = 64 in
let MaximumRootIdBytes = 128 in
let MaximumU64 = 18446744073709551615 in
let ProfileSchema = "lattice.standalone-effect-authority.v1" in
let PolicyRef = "blake3:56800b3cb2f0819e31cbd32fc3ee41e1a74e7dbf6495540ad8d50db5341f0a79" in

let Exact = fun expected => std.contract.from_predicate (fun value =>
  std.is_string value && value == expected
) in

let Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^blake3:[0-9a-f]{64}$" value
) in

let Unsigned64 = std.contract.from_predicate (fun value =>
  std.is_number value
  && value >= 0
  && value <= MaximumU64
  && std.number.floor value == value
) in

let Expiry = std.contract.Sequence [
  Unsigned64,
  std.contract.from_predicate (fun value => value > 0),
] in

# Profiles are evaluated on the Unix standalone host. Opening roots is a host act,
# not something this contract or the workflow may do.
let AbsolutePath = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^/" value
) in

let GrantShape = {
  node_id | Unsigned64,
  operation | String,
  resource | String,
  ability | String,
} in

let grant_is_closed = fun grant =>
  let node = std.string.from_number grant.node_id in
  (
    (grant.operation == "lattice/shell" || grant.operation == "lattice/operation")
    && grant.resource == "lattice://process/" ++ node
    && grant.ability == "process/execute"
  )
  || (
    grant.operation == "lattice/http"
    && grant.resource == "lattice://network/" ++ node
    && grant.ability == "network/request"
  )
  || (
    grant.operation == "lattice/file"
    && grant.resource == "lattice://file/" ++ node
    && std.array.elem grant.ability ["file/read", "file/write", "file/append"]
  )
in

let Grant = std.contract.Sequence [
  GrantShape,
  std.contract.from_predicate grant_is_closed,
] in

let ProfileShape = {
  schema | Exact ProfileSchema,
  workflow_revision | Identity,
  machine_identity | Identity,
  policy_ref | Exact PolicyRef,
  expires_at_unix_ms | Expiry,
  grants | Array Grant,
  roots | { _ | AbsolutePath },
} in

let profile_is_bounded = fun profile =>
  let root_ids = std.record.fields profile.roots in
  std.array.length profile.grants > 0
  && std.array.length profile.grants <= MaximumGrants
  && std.array.all (fun grant =>
    std.array.length (
      std.array.filter (fun other => other.node_id == grant.node_id) profile.grants
    ) == 1
  ) profile.grants
  && std.array.length root_ids <= MaximumRoots
  # Nickel measures grapheme clusters; Rust enforces the stricter UTF-8 byte
  # bound at authoritative admission, including imported evaluated root IDs.
  && std.array.all (fun id =>
    std.string.length id > 0 && std.string.length id <= MaximumRootIdBytes
  ) root_ids
in

{
  StandaloneEffectAuthorityProfile = std.contract.Sequence [
    ProfileShape,
    std.contract.from_predicate profile_is_bounded,
  ],
}
