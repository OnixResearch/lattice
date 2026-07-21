let schema = import "../../provider-retry-policy.ncl" in
{
  expected_issue = "PROVIDER_RETRY_POLICY_HASH",
  fixture | schema.provider_retry_policy_contract =
    schema.provider_retry_policy & { policy_hash | force = "BAD" },
}
