let schema = import "../../provider-retry-policy.ncl" in
{
  expected_issue = "PROVIDER_RETRY_EMPTY_CLASSES",
  fixture | schema.provider_retry_policy_contract =
    schema.provider_retry_policy & { retryable_classes | force = [] },
}
