# Typed policy for the optional Prime Agent messaging receipts cohort.
let contracts = import "../policy/contracts/primitives.ncl" in
let maximum_body_bytes = 4096 in
let maximum_rate_count = 16 in
let maximum_pending_count = 32 in
let Profile = {
  schema | contracts.exact "MESSAGING_SCHEMA" "animus.messaging-receipts.v1",
  cohort | contracts.exact "MESSAGING_COHORT" "prime-agent-messaging-receipts",
  license | contracts.exact "MESSAGING_LICENSE" "MIT",
  source_revision | contracts.safe_identifier "MESSAGING_SOURCE_REVISION",
  max_message_bytes | contracts.positive_integer "MESSAGING_MAX_BODY" maximum_body_bytes,
  max_rate_count | contracts.positive_integer "MESSAGING_MAX_RATE" maximum_rate_count,
  max_pending_count | contracts.positive_integer "MESSAGING_MAX_PENDING" maximum_pending_count,
  roles | Array { role | String, modes | Array String },
  receipt_fields | Array String,
  non_claims | Array String,
  roster_bound | Bool,
  host_derived_identity | Bool,
  limits_before_routing | Bool,
} in
let ProfileWithRules = contracts.with_validator Profile (fun value =>
  if std.array.length value.roles != 2 then
    'Error { message = "MESSAGING_ROLE_COUNT" }
  else if std.array.length value.receipt_fields < 7 then
    'Error { message = "MESSAGING_RECEIPT_FIELDS" }
  else if std.array.length value.non_claims < 4 then
    'Error { message = "MESSAGING_NON_CLAIMS" }
  else if !std.array.elem "not-read-or-acted" value.non_claims then
    'Error { message = "MESSAGING_DELIVERY_NON_CLAIM" }
  else if !std.array.elem "not-exactly-once-or-ordered" value.non_claims then
    'Error { message = "MESSAGING_ORDERING_NON_CLAIM" }
  else if !value.roster_bound || !value.host_derived_identity || !value.limits_before_routing then
    'Error { message = "MESSAGING_BOUNDARY_RULES" }
  else
    'Ok
) in
{
  messaging_receipts_policy_contract = ProfileWithRules,
  messaging_receipts_policy | ProfileWithRules = {
    schema = "animus.messaging-receipts.v1",
    cohort = "prime-agent-messaging-receipts",
    license = "MIT",
    source_revision = "c5991bc853d27754aed345c13ff4d2e05c40f1f4",
    max_message_bytes = maximum_body_bytes,
    max_rate_count = maximum_rate_count,
    max_pending_count = maximum_pending_count,
    roles = [
      { role = "root", modes = ["auto", "steer", "follow_up"] },
      { role = "subagent", modes = ["auto", "follow_up"] },
    ],
    receipt_fields = [
      "policy_identity",
      "roster_identity",
      "sender_identity",
      "target_count",
      "mode",
      "variant",
      "counter_identity",
    ],
    non_claims = [
      "not-read-or-acted",
      "not-transport-security",
      "not-exactly-once-or-ordered",
      "not-message-correctness-or-cooperation",
    ],
    roster_bound = true,
    host_derived_identity = true,
    limits_before_routing = true,
  },
}
