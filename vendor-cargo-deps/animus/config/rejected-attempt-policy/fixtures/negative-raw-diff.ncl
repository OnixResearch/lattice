let schema = import "../../rejected-attempt-policy.ncl" in
{
  expected_issue = "REJECTED_ATTEMPT_RAW_DIFF",
  fixture | schema.rejected_attempt_policy_contract =
    schema.rejected_attempt_policy & { raw_diff | force = "allowed" },
}
