let schema = import "../../provider-retry-policy.ncl" in
let base = schema.provider_retry_policy in
let first = std.array.first base.mappings in
let changed = first & { canonical_class | force = "maybe_retry" } in
let changed_mappings = std.array.map
  (fun mapping => if mapping.native_class == first.native_class then changed else mapping)
  base.mappings
in
{
  expected_issue = "PROVIDER_RETRY_CANONICAL_CLASS",
  fixture | schema.provider_retry_policy_contract =
    base & { mappings | force = changed_mappings },
}
