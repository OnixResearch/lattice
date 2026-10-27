let schema = import "../../rejected-attempt-policy.ncl" in
{
  expected_issue = "REJECTED_ATTEMPT_WEAKENED_NON_CLAIMS",
  fixture | schema.rejected_attempt_policy_contract =
    schema.rejected_attempt_policy & { non_claims | force = ["does-not-authorize-retry"] },
}
