let schema = import "../../provider-retry-policy.ncl" in
{
  expected_issue = "PROVIDER_RETRY_ATTEMPT_BOUND",
  fixture | schema.provider_retry_policy_contract =
    schema.provider_retry_policy & { max_attempts | force = 0 },
}
