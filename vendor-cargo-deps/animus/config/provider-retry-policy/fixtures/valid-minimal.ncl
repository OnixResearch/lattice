let schema = import "../../provider-retry-policy.ncl" in
{
  expected_issue = "OK",
  fixture | schema.provider_retry_policy_contract = schema.provider_retry_policy,
}
