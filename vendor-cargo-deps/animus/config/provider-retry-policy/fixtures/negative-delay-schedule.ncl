let schema = import "../../provider-retry-policy.ncl" in
{
  expected_issue = "PROVIDER_RETRY_DELAY_SCHEDULE",
  fixture | schema.provider_retry_policy_contract =
    schema.provider_retry_policy & { delay_schedule_ms | force = [0] },
}
