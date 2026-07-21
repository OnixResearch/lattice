let schema = import "../../provider-retry-policy.ncl" in
{
  expected_issue = "PROVIDER_RETRY_STREAM_POSTURE",
  fixture | schema.provider_retry_policy_contract =
    schema.provider_retry_policy & { stream_commit_policy | force = "resume_partial" },
}
