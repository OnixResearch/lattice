let schema = import "../../provider-retry-policy.ncl" in
let base = schema.provider_retry_policy in
let first = std.array.first base.mappings in
let changed = first & { native_class | force = "/private/status" } in
let changed_mappings = std.array.map
  (fun mapping => if mapping.native_class == first.native_class then changed else mapping)
  base.mappings
in
{
  expected_issue = "PROVIDER_RETRY_NATIVE_CLASS",
  fixture | schema.provider_retry_policy_contract =
    base & { mappings | force = changed_mappings },
}
