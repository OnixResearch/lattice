let schema = import "../../provider-retry-policy.ncl" in
let base = schema.provider_retry_policy in
{
  expected_issue = "PROVIDER_RETRY_DUPLICATE_MAPPING",
  fixture | schema.provider_retry_policy_contract =
    base & { mappings | force = base.mappings @ [std.array.first base.mappings] },
}
