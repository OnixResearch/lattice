let schema = import "../../rejected-attempt-policy.ncl" in
{
  expected_issue = "OK",
  fixture | schema.rejected_attempt_policy_contract = schema.rejected_attempt_policy,
}
