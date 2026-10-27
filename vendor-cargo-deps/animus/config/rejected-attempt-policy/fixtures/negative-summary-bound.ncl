let schema = import "../../rejected-attempt-policy.ncl" in
{
  expected_issue = "REJECTED_ATTEMPT_SUMMARY_BOUND",
  fixture | schema.rejected_attempt_policy_contract =
    schema.rejected_attempt_policy & { max_summary_bytes | force = 0 },
}
