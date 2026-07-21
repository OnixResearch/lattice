let schema = import "../../provider-retry-policy.ncl" in
let excessive_delay_ms = 300001 in
{
  expected_issue = "PROVIDER_RETRY_DELAY_BOUND",
  fixture | schema.provider_retry_policy_contract =
    schema.provider_retry_policy & { max_delay_ms | force = excessive_delay_ms },
}
